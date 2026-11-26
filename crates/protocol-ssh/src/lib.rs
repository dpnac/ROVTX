//! Client SSH2 / SFTP (REQ-001, REQ-012, ADR-0003) — prototype J0.
//!
//! russh (SSH2) + russh-sftp (SFTP3). Vérification stricte de la clé d'hôte
//! (known_hosts) : aucune clé inconnue n'est acceptée par défaut.
#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use russh::ChannelMsg;
use russh::client::{self, AuthResult, Handle};
use russh::keys::{
    HashAlg, PrivateKey, PrivateKeyWithHashAlg, PublicKeyOrCertificate, load_secret_key,
};
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug, Error)]
pub enum SshError {
    #[error("clé d'hôte inconnue ou différente de known_hosts : refus")]
    UnknownHostKey,
    #[error("authentification refusée")]
    AuthFailed,
    #[error("ssh : {0}")]
    Ssh(#[from] russh::Error),
    #[error("clés : {0}")]
    Keys(#[from] russh::keys::Error),
    #[error("sftp : {0}")]
    Sftp(String),
    #[error("io : {0}")]
    Io(#[from] std::io::Error),
}

/// Vérifieur known_hosts strict : la clé présentée doit exister dans la
/// carte (hôte → empreinte hexadécimale SHA-256 de la clé publique).
#[derive(Clone, Default)]
pub struct KnownHosts {
    /// Empreintes connues par hôte (plusieurs clés possibles : ed25519, rsa…).
    pub fingerprints: HashMap<String, Vec<String>>,
}

impl KnownHosts {
    /// Empreinte SHA-256 standard (`SHA256:…`, même format que ssh-keygen -lf).
    pub fn fingerprint(key: &russh::keys::PublicKey) -> String {
        key.fingerprint(HashAlg::Sha256).to_string()
    }
}

pub struct HostKeyChecker {
    pub host: String,
    pub known: KnownHosts,
}

impl client::Handler for HostKeyChecker {
    type Error = SshError;

    async fn check_server_key(
        &mut self,
        server_public_key: &PublicKeyOrCertificate,
    ) -> Result<bool, Self::Error> {
        let presented = match server_public_key {
            PublicKeyOrCertificate::PublicKey { key, .. } => KnownHosts::fingerprint(key),
            PublicKeyOrCertificate::Certificate(_) => return Ok(false), // non géré au prototype
        };
        Ok(self
            .known
            .fingerprints
            .get(&self.host)
            .is_some_and(|fps| fps.contains(&presented)))
    }
}

/// Connexion SSH2 avec authentification par clé privée (fichier).
pub async fn connect_key(
    host: &str,
    port: u16,
    user: &str,
    key_path: &Path,
    known: KnownHosts,
) -> Result<Handle<HostKeyChecker>, SshError> {
    let config = Arc::new(client::Config::default());
    let mut handle = client::connect(
        config,
        (host, port),
        HostKeyChecker {
            host: host.to_string(),
            known,
        },
    )
    .await?;
    let key: PrivateKey = load_secret_key(key_path, None)?;
    let key = PrivateKeyWithHashAlg::new(Arc::new(key), None);
    match handle.authenticate_publickey(user, key).await? {
        AuthResult::Success => Ok(handle),
        AuthResult::Failure { .. } => Err(SshError::AuthFailed),
    }
}

/// Exécute une commande et rend (stdout, code de sortie si reçu).
pub async fn exec(
    handle: &mut Handle<HostKeyChecker>,
    command: &str,
) -> Result<(Vec<u8>, Option<u32>), SshError> {
    let mut channel = handle.channel_open_session().await?;
    channel.exec(true, command).await?;
    let mut out = Vec::new();
    let mut code = None;
    // Lit jusqu'à fermeture du canal : exit-status peut suivre eof.
    while let Some(msg) = channel.wait().await {
        match msg {
            ChannelMsg::Data { data } => out.extend_from_slice(&data),
            ChannelMsg::ExitStatus { exit_status } => code = Some(exit_status),
            ChannelMsg::Close => break,
            _ => {}
        }
    }
    Ok((out, code))
}

/// Ouvre une session SFTP sur la connexion existante.
pub async fn open_sftp(
    handle: &mut Handle<HostKeyChecker>,
) -> Result<russh_sftp::client::SftpSession, SshError> {
    let channel = handle.channel_open_session().await?;
    channel.request_subsystem(true, "sftp").await?;
    let stream = channel.into_stream();
    russh_sftp::client::SftpSession::new(stream)
        .await
        .map_err(|e| SshError::Sftp(e.to_string()))
}

/// Télécharge un fichier distant vers un buffer local (preuve de transfert).
pub async fn sftp_download(
    sftp: &russh_sftp::client::SftpSession,
    remote: &str,
) -> Result<Vec<u8>, SshError> {
    let mut file = sftp
        .open(remote)
        .await
        .map_err(|e| SshError::Sftp(e.to_string()))?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).await?;
    file.close()
        .await
        .map_err(|e| SshError::Sftp(e.to_string()))?;
    Ok(buf)
}

/// Téléverse un buffer local vers un fichier distant.
pub async fn sftp_upload(
    sftp: &russh_sftp::client::SftpSession,
    remote: &str,
    data: &[u8],
) -> Result<(), SshError> {
    let mut file = sftp
        .create(remote)
        .await
        .map_err(|e| SshError::Sftp(e.to_string()))?;
    file.write_all(data).await?;
    file.close()
        .await
        .map_err(|e| SshError::Sftp(e.to_string()))?;
    Ok(())
}
