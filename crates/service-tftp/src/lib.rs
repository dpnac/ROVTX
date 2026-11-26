//! Serveur TFTP (REQ-022, ADR-0008) — prototype J0.
//!
//! RFC 1350 + options négociées (blksize RFC 2348, timeout RFC 2349,
//! tsize RFC 2349). Racine confinée : traversée de chemin et liens
//! symboliques sortants bloqués. Écriture désactivée par défaut.
//! Arrêt immédiat : `shutdown()` coupe la boucle et les transferts.
#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use thiserror::Error;
use tokio::net::UdpSocket;
use tokio::sync::oneshot;
use tokio::task::JoinSet;

pub const DEFAULT_BLKSIZE: usize = 512;
pub const MIN_BLKSIZE: usize = 8;
pub const MAX_BLKSIZE: usize = 65464;
const TRANSFER_TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TftpError {
    #[error("chemin hors de la racine TFTP : {0}")]
    PathEscape(String),
    #[error("écriture désactivée")]
    WriteDisabled,
    #[error("limite de transferts simultanés atteinte")]
    TooManyTransfers,
    #[error("option invalide : {0}")]
    BadOption(String),
    #[error("io : {0}")]
    Io(String),
}

#[derive(Debug, Clone)]
pub struct TftpConfig {
    pub root: PathBuf,
    pub bind: SocketAddr,
    /// Écriture (WRQ) : désactivée par défaut (exigence CDC §05).
    pub allow_write: bool,
    pub max_transfers: usize,
    pub max_blksize: usize,
}

impl Default for TftpConfig {
    fn default() -> Self {
        Self {
            root: PathBuf::from("."),
            bind: "127.0.0.1:6969".parse().expect("addr statique"),
            allow_write: false,
            max_transfers: 8,
            max_blksize: 65464,
        }
    }
}

/// Codes d'erreur RFC 1350.
mod errcode {
    pub const NOT_DEFINED: u16 = 0;
    pub const NOT_FOUND: u16 = 1;
    pub const ACCESS_VIOLATION: u16 = 2;
    pub const ILLEGAL_OP: u16 = 4;
}

mod opcode {
    pub const RRQ: u16 = 1;
    pub const WRQ: u16 = 2;
    pub const DATA: u16 = 3;
    pub const ACK: u16 = 4;
    pub const ERROR: u16 = 5;
    pub const OACK: u16 = 6;
}

pub struct TftpServer {
    local_addr: SocketAddr,
    stop: Option<oneshot::Sender<()>>,
    worker: Option<tokio::task::JoinHandle<()>>,
}

impl TftpServer {
    pub async fn start(config: TftpConfig) -> Result<Self, TftpError> {
        let sock = UdpSocket::bind(config.bind)
            .await
            .map_err(|e| TftpError::Io(e.to_string()))?;
        let local_addr = sock
            .local_addr()
            .map_err(|e| TftpError::Io(e.to_string()))?;
        let (stop_tx, stop_rx) = oneshot::channel();
        let cfg = Arc::new(config);
        let worker = tokio::spawn(accept_loop(sock, cfg, stop_rx));
        Ok(Self {
            local_addr,
            stop: Some(stop_tx),
            worker: Some(worker),
        })
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Arrêt immédiat : boucle et transferts coupés.
    pub async fn shutdown(mut self) {
        if let Some(tx) = self.stop.take() {
            let _ = tx.send(());
        }
        if let Some(w) = self.worker.take() {
            let _ = w.await;
        }
    }
}

async fn accept_loop(sock: UdpSocket, cfg: Arc<TftpConfig>, stop: oneshot::Receiver<()>) {
    let active = Arc::new(AtomicUsize::new(0));
    let mut transfers: JoinSet<()> = JoinSet::new();
    let sock = Arc::new(sock);
    let mut stop = std::pin::pin!(stop);
    let mut buf = vec![0u8; 65536];
    loop {
        tokio::select! {
            _ = &mut stop => break,
            r = sock.recv_from(&mut buf) => {
                let (n, peer) = match r { Ok(v) => v, Err(_) => continue };
                let packet = buf[..n].to_vec();
                if active.load(Ordering::Relaxed) >= cfg.max_transfers {
                    let _ = send_error_to(&sock, peer, errcode::NOT_DEFINED, "too many transfers").await;
                    continue;
                }
                active.fetch_add(1, Ordering::Relaxed);
                let cfg = cfg.clone();
                let active = active.clone();
                transfers.spawn(async move {
                    if let Err(e) = handle_request(&cfg, peer, packet).await {
                        tracing::warn!(?peer, error = %e, "transfert TFTP en échec");
                    }
                    active.fetch_sub(1, Ordering::Relaxed);
                });
            }
        }
    }
    transfers.shutdown().await;
}

async fn handle_request(
    cfg: &TftpConfig,
    peer: SocketAddr,
    packet: Vec<u8>,
) -> Result<(), TftpError> {
    if packet.len() < 4 {
        return Err(TftpError::BadOption("paquet trop court".into()));
    }
    let op = u16::from_be_bytes([packet[0], packet[1]]);
    // Socket de transfert dédiée (ports UDP de transfert, RFC 1350).
    let tsock = UdpSocket::bind("127.0.0.1:0")
        .await
        .map_err(|e| TftpError::Io(e.to_string()))?;
    tsock
        .connect(peer)
        .await
        .map_err(|e| TftpError::Io(e.to_string()))?;
    if op != opcode::RRQ && op != opcode::WRQ {
        send_error(&tsock, errcode::ILLEGAL_OP, "illegal operation").await?;
        return Err(TftpError::BadOption("premier paquet non RRQ/WRQ".into()));
    }
    let (filename, opts) = parse_request(&packet[2..])?;

    let path = match resolve_path(&cfg.root, &filename) {
        Ok(p) => p,
        Err(e) => {
            send_error(&tsock, errcode::ACCESS_VIOLATION, "path outside root").await?;
            return Err(e);
        }
    };

    if op == opcode::WRQ {
        if !cfg.allow_write {
            send_error(&tsock, errcode::ACCESS_VIOLATION, "write disabled").await?;
            return Err(TftpError::WriteDisabled);
        }
        run_wrq(&tsock, cfg, path, opts).await
    } else {
        run_rrq(&tsock, cfg, path, opts).await
    }
}

/// Confinement : pas de `..`, pas de chemin absolu, pas de symlink sortant.
pub fn resolve_path(root: &Path, requested: &str) -> Result<PathBuf, TftpError> {
    if requested.is_empty() || requested.contains('\0') {
        return Err(TftpError::PathEscape(requested.to_string()));
    }
    let rel = Path::new(requested);
    if rel.is_absolute() || requested.starts_with('/') || requested.contains(':') {
        return Err(TftpError::PathEscape(requested.to_string()));
    }
    for comp in rel.components() {
        match comp {
            Component::Normal(_) => {}
            Component::CurDir => {}
            _ => return Err(TftpError::PathEscape(requested.to_string())),
        }
    }
    let joined = root.join(rel);
    // Résolution des symlinks : le chemin réel doit rester sous la racine.
    let canon_root = root
        .canonicalize()
        .map_err(|e| TftpError::Io(e.to_string()))?;
    let probe = if joined.exists() {
        joined
            .canonicalize()
            .map_err(|e| TftpError::Io(e.to_string()))?
    } else {
        // Fichier à créer : on vérifie le parent existant le plus proche.
        let mut p = joined.clone();
        let mut suffix = Vec::new();
        while !p.exists() {
            if let Some(name) = p.file_name() {
                suffix.push(name.to_owned());
            }
            if !p.pop() {
                break;
            }
        }
        let base = p.canonicalize().map_err(|e| TftpError::Io(e.to_string()))?;
        suffix.iter().rev().fold(base, |acc, s| acc.join(s))
    };
    if !probe.starts_with(&canon_root) {
        return Err(TftpError::PathEscape(requested.to_string()));
    }
    Ok(joined)
}

fn parse_request(body: &[u8]) -> Result<(String, HashMap<String, String>), TftpError> {
    let parts: Vec<&[u8]> = body.split(|b| *b == 0).collect();
    if parts.len() < 2 {
        return Err(TftpError::BadOption("format RRQ/WRQ invalide".into()));
    }
    let filename = String::from_utf8_lossy(parts[0]).into_owned();
    // parts[1] = mode (octet/netascii) ; paires d'options ensuite.
    let mut opts = HashMap::new();
    let mut i = 2;
    while i + 1 < parts.len() {
        let k = String::from_utf8_lossy(parts[i]).to_lowercase();
        let v = String::from_utf8_lossy(parts[i + 1]).into_owned();
        opts.insert(k, v);
        i += 2;
    }
    Ok((filename, opts))
}

fn negotiate(
    opts: &HashMap<String, String>,
    cfg: &TftpConfig,
    file_size: Option<u64>,
) -> (Vec<(String, String)>, usize) {
    let mut acked = Vec::new();
    let mut blksize = DEFAULT_BLKSIZE;
    if let Some(v) = opts.get("blksize")
        && let Ok(n) = v.parse::<usize>()
    {
        let n = n.clamp(MIN_BLKSIZE, cfg.max_blksize.min(MAX_BLKSIZE));
        blksize = n;
        acked.push(("blksize".into(), n.to_string()));
    }
    if let Some(t) = opts.get("timeout").and_then(|v| v.parse::<u64>().ok()) {
        let t = t.clamp(1, 60);
        acked.push(("timeout".into(), t.to_string()));
    }
    if opts.contains_key("tsize")
        && let Some(size) = file_size
    {
        acked.push(("tsize".into(), size.to_string()));
    }
    (acked, blksize)
}

async fn run_rrq(
    sock: &UdpSocket,
    cfg: &TftpConfig,
    path: PathBuf,
    opts: HashMap<String, String>,
) -> Result<(), TftpError> {
    let data = match tokio::fs::read(&path).await {
        Ok(d) => d,
        Err(_) => {
            send_error(sock, errcode::NOT_FOUND, "file not found").await?;
            return Err(TftpError::Io("fichier illisible".into()));
        }
    };
    let (acked_opts, blksize) = negotiate(&opts, cfg, Some(data.len() as u64));
    tracing::info!(path = %path.display(), blksize, "TFTP RRQ accepté");
    let mut block: u16 = 1;
    if !acked_opts.is_empty() {
        send_oack(sock, &acked_opts).await?;
        let ack = recv_ack(sock).await?;
        if ack != 0 {
            return Err(TftpError::BadOption("ACK initial attendu = 0".into()));
        }
    }
    for chunk in data.chunks(blksize) {
        send_data(sock, block, chunk).await?;
        let ack = recv_ack(sock).await?;
        if ack != block {
            return Err(TftpError::BadOption("ACK inattendu".into()));
        }
        block = block.wrapping_add(1);
        if chunk.len() < blksize {
            return Ok(());
        }
    }
    // Contenu exactement multiple de blksize : bloc final vide.
    send_data(sock, block, &[]).await?;
    let _ = recv_ack(sock).await;
    Ok(())
}

async fn run_wrq(
    sock: &UdpSocket,
    cfg: &TftpConfig,
    path: PathBuf,
    opts: HashMap<String, String>,
) -> Result<(), TftpError> {
    let (acked_opts, blksize) = negotiate(&opts, cfg, None);
    tracing::info!(path = %path.display(), blksize, "TFTP WRQ accepté");
    if !acked_opts.is_empty() {
        send_oack(sock, &acked_opts).await?;
    } else {
        send_ack(sock, 0).await?;
    }
    let mut out: Vec<u8> = Vec::new();
    loop {
        let (block, chunk) = recv_data(sock).await?;
        out.extend_from_slice(&chunk);
        send_ack(sock, block).await?;
        if chunk.len() < blksize {
            break;
        }
    }
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| TftpError::Io(e.to_string()))?;
    }
    tokio::fs::write(&path, out)
        .await
        .map_err(|e| TftpError::Io(e.to_string()))?;
    Ok(())
}

// ---- encodage / décodage paquets ----

async fn send_data(sock: &UdpSocket, block: u16, chunk: &[u8]) -> Result<(), TftpError> {
    let mut p = Vec::with_capacity(4 + chunk.len());
    p.extend_from_slice(&opcode::DATA.to_be_bytes());
    p.extend_from_slice(&block.to_be_bytes());
    p.extend_from_slice(chunk);
    sock.send(&p)
        .await
        .map_err(|e| TftpError::Io(e.to_string()))?;
    Ok(())
}

async fn send_ack(sock: &UdpSocket, block: u16) -> Result<(), TftpError> {
    let mut p = Vec::with_capacity(4);
    p.extend_from_slice(&opcode::ACK.to_be_bytes());
    p.extend_from_slice(&block.to_be_bytes());
    sock.send(&p)
        .await
        .map_err(|e| TftpError::Io(e.to_string()))?;
    Ok(())
}

async fn send_oack(sock: &UdpSocket, opts: &[(String, String)]) -> Result<(), TftpError> {
    let mut p = vec![0u8; 2];
    p[0..2].copy_from_slice(&opcode::OACK.to_be_bytes());
    for (k, v) in opts {
        p.extend_from_slice(k.as_bytes());
        p.push(0);
        p.extend_from_slice(v.as_bytes());
        p.push(0);
    }
    sock.send(&p)
        .await
        .map_err(|e| TftpError::Io(e.to_string()))?;
    Ok(())
}

async fn send_error(sock: &UdpSocket, code: u16, msg: &str) -> Result<(), TftpError> {
    send_error_to(
        sock,
        sock.peer_addr().map_err(|e| TftpError::Io(e.to_string()))?,
        code,
        msg,
    )
    .await
}

async fn send_error_to(
    sock: &UdpSocket,
    peer: SocketAddr,
    code: u16,
    msg: &str,
) -> Result<(), TftpError> {
    let mut p = Vec::new();
    p.extend_from_slice(&opcode::ERROR.to_be_bytes());
    p.extend_from_slice(&code.to_be_bytes());
    p.extend_from_slice(msg.as_bytes());
    p.push(0);
    sock.send_to(&p, peer)
        .await
        .map_err(|e| TftpError::Io(e.to_string()))?;
    Ok(())
}

async fn recv_ack(sock: &UdpSocket) -> Result<u16, TftpError> {
    let p = recv_packet(sock).await?;
    if p.len() < 4 || u16::from_be_bytes([p[0], p[1]]) != opcode::ACK {
        return Err(TftpError::BadOption("ACK attendu".into()));
    }
    Ok(u16::from_be_bytes([p[2], p[3]]))
}

async fn recv_data(sock: &UdpSocket) -> Result<(u16, Vec<u8>), TftpError> {
    let p = recv_packet(sock).await?;
    if p.len() < 4 || u16::from_be_bytes([p[0], p[1]]) != opcode::DATA {
        return Err(TftpError::BadOption("DATA attendu".into()));
    }
    Ok((u16::from_be_bytes([p[2], p[3]]), p[4..].to_vec()))
}

/// Reçoit un paquet ; un ERROR reçu devient une erreur locale.
async fn recv_packet(sock: &UdpSocket) -> Result<Vec<u8>, TftpError> {
    let mut buf = vec![0u8; 65536];
    let n = tokio::time::timeout(TRANSFER_TIMEOUT, sock.recv(&mut buf))
        .await
        .map_err(|_| TftpError::Io("timeout transfert".into()))?
        .map_err(|e| TftpError::Io(e.to_string()))?;
    let p = buf[..n].to_vec();
    if p.len() >= 2 && u16::from_be_bytes([p[0], p[1]]) == opcode::ERROR {
        let msg = String::from_utf8_lossy(&p[4..])
            .trim_end_matches('\0')
            .to_string();
        return Err(TftpError::Io(format!("ERROR reçu : {msg}")));
    }
    Ok(p)
}

// ---- client de test (utilisé par les tests d'intégration) ----
#[doc(hidden)]
pub mod testclient {
    use super::*;

    pub fn rrq(filename: &str, opts: &[(&str, &str)]) -> Vec<u8> {
        request(opcode::RRQ, filename, opts)
    }
    pub fn wrq(filename: &str, opts: &[(&str, &str)]) -> Vec<u8> {
        request(opcode::WRQ, filename, opts)
    }
    fn request(op: u16, filename: &str, opts: &[(&str, &str)]) -> Vec<u8> {
        let mut p = Vec::new();
        p.extend_from_slice(&op.to_be_bytes());
        p.extend_from_slice(filename.as_bytes());
        p.push(0);
        p.extend_from_slice(b"octet");
        p.push(0);
        for (k, v) in opts {
            p.extend_from_slice(k.as_bytes());
            p.push(0);
            p.extend_from_slice(v.as_bytes());
            p.push(0);
        }
        p
    }
    /// Socket client NON connectée : comme un vrai client TFTP, elle envoie
    /// au port 69 puis suit le TID (adresse source) du premier paquet reçu.
    pub async fn client_socket() -> UdpSocket {
        UdpSocket::bind("127.0.0.1:0").await.expect("bind client")
    }
    pub async fn send_to(sock: &UdpSocket, server: SocketAddr, packet: &[u8]) {
        sock.send_to(packet, server).await.expect("send_to client");
    }
    /// Reçoit un paquet et rend l'adresse source (TID serveur).
    pub async fn recv_from(sock: &UdpSocket) -> (Vec<u8>, SocketAddr) {
        let mut b = vec![0u8; 65536];
        let (n, from) = tokio::time::timeout(Duration::from_secs(5), sock.recv_from(&mut b))
            .await
            .expect("timeout client")
            .expect("recv");
        b.truncate(n);
        (b, from)
    }
    pub fn op(p: &[u8]) -> u16 {
        u16::from_be_bytes([p[0], p[1]])
    }
    pub fn block(p: &[u8]) -> u16 {
        u16::from_be_bytes([p[2], p[3]])
    }
    pub async fn send_ack(sock: &UdpSocket, tid: SocketAddr, block: u16) {
        let mut p = Vec::new();
        p.extend_from_slice(&super::opcode::ACK.to_be_bytes());
        p.extend_from_slice(&block.to_be_bytes());
        sock.send_to(&p, tid).await.expect("ack client");
    }
    pub async fn send_data(sock: &UdpSocket, tid: SocketAddr, block: u16, chunk: &[u8]) {
        let mut p = Vec::new();
        p.extend_from_slice(&super::opcode::DATA.to_be_bytes());
        p.extend_from_slice(&block.to_be_bytes());
        p.extend_from_slice(chunk);
        sock.send_to(&p, tid).await.expect("data client");
    }
    pub fn errcode_of(p: &[u8]) -> u16 {
        u16::from_be_bytes([p[2], p[3]])
    }
}
