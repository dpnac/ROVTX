//! Tests d'intégration TFTP (TEST-TFTP-001 à 004) — boucle loopback isolée.
use std::path::PathBuf;

use service_tftp::testclient as tc;
use service_tftp::{TftpConfig, TftpServer};

fn temp_root(tag: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("tftp-test-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&p).expect("mkdir racine");
    p
}

async fn start(tag: &str, allow_write: bool) -> (TftpServer, PathBuf) {
    let root = temp_root(tag);
    let cfg = TftpConfig {
        root: root.clone(),
        bind: "127.0.0.1:0".parse().unwrap(),
        allow_write,
        ..Default::default()
    };
    let srv = TftpServer::start(cfg).await.expect("démarrage serveur");
    (srv, root)
}

/// TEST-TFTP-001 : GET fichier + négociation blksize/tsize.
#[tokio::test]
async fn rrq_download_with_options() {
    let (srv, root) = start("rrq", false).await;
    let content = vec![0xABu8; 1500]; // > 1 bloc par défaut
    std::fs::write(root.join("firmware.bin"), &content).expect("écriture fixture");

    let server = srv.local_addr();
    let sock = tc::client_socket().await;
    tc::send_to(
        &sock,
        server,
        &tc::rrq("firmware.bin", &[("blksize", "1024"), ("tsize", "0")]),
    )
    .await;

    // OACK attendu avec blksize=1024 et tsize=1500 ; le TID devient la cible.
    let (p, tid) = tc::recv_from(&sock).await;
    assert_eq!(tc::op(&p), 6, "OACK attendu");
    let s = String::from_utf8_lossy(&p[2..]).to_lowercase();
    assert!(s.contains("blksize") && s.contains("1024"), "OACK: {s}");
    assert!(s.contains("tsize") && s.contains("1500"), "OACK: {s}");
    tc::send_ack(&sock, tid, 0).await;

    let mut got = Vec::new();
    loop {
        let (p, _tid_data) = tc::recv_from(&sock).await;
        assert_eq!(tc::op(&p), 3, "DATA attendu");
        let block = tc::block(&p);
        got.extend_from_slice(&p[4..]);
        tc::send_ack(&sock, tid, block).await;
        if p.len() - 4 < 1024 {
            break;
        }
    }
    assert_eq!(got, content);
    srv.shutdown().await;
}

/// TEST-TFTP-003 : PUT refusé par défaut (écriture désactivée).
#[tokio::test]
async fn wrq_denied_by_default() {
    let (srv, root) = start("wrq-deny", false).await;
    let sock = tc::client_socket().await;
    tc::send_to(&sock, srv.local_addr(), &tc::wrq("upload.bin", &[])).await;
    let (p, _tid) = tc::recv_from(&sock).await;
    assert_eq!(tc::op(&p), 5, "ERROR attendu");
    assert_eq!(tc::errcode_of(&p), 2, "access violation");
    assert!(!root.join("upload.bin").exists(), "aucun fichier créé");
    srv.shutdown().await;
}

/// TEST-TFTP-001b : PUT accepté quand l'écriture est activée explicitement.
#[tokio::test]
async fn wrq_allowed_when_enabled() {
    let (srv, root) = start("wrq-ok", true).await;
    let payload = b"image-efi-contenu".to_vec();
    let sock = tc::client_socket().await;
    tc::send_to(&sock, srv.local_addr(), &tc::wrq("boot.img", &[])).await;
    let (p, tid) = tc::recv_from(&sock).await;
    assert_eq!(tc::op(&p), 4, "ACK 0 attendu");
    tc::send_data(&sock, tid, 1, &payload).await; // bloc court = fin
    let (p, _tid2) = tc::recv_from(&sock).await;
    assert_eq!(tc::op(&p), 4);
    assert_eq!(tc::block(&p), 1);
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
    assert_eq!(
        std::fs::read(root.join("boot.img")).expect("lecture"),
        payload
    );
    srv.shutdown().await;
}

/// TEST-TFTP-002 : traversée de chemin refusée côté serveur (résolution).
#[tokio::test]
async fn path_traversal_rejected() {
    let (srv, root) = start("trav", false).await;
    std::fs::write(root.join("ok.txt"), b"ok").expect("fixture");
    let outside = root.parent().unwrap().join("secret-hors-racine.txt");
    std::fs::write(&outside, b"secret").expect("fixture hors racine");

    let sock = tc::client_socket().await;
    tc::send_to(
        &sock,
        srv.local_addr(),
        &tc::rrq("../secret-hors-racine.txt", &[]),
    )
    .await;
    let (p, _tid) = tc::recv_from(&sock).await;
    assert_eq!(tc::op(&p), 5, "ERROR attendu pour traversée");

    // Chemin absolu refusé aussi.
    let sock2 = tc::client_socket().await;
    tc::send_to(&sock2, srv.local_addr(), &tc::rrq("/etc/passwd", &[])).await;
    let (p2, _tid2) = tc::recv_from(&sock2).await;
    assert_eq!(tc::op(&p2), 5);

    // Symlink sortant refusé.
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&outside, root.join("link-out")).expect("symlink");
        let sock3 = tc::client_socket().await;
        tc::send_to(&sock3, srv.local_addr(), &tc::rrq("link-out", &[])).await;
        let (p3, _tid3) = tc::recv_from(&sock3).await;
        assert_eq!(tc::op(&p3), 5, "symlink sortant refusé");
    }
    let _ = std::fs::remove_file(&outside);
    srv.shutdown().await;
}

/// TEST-TFTP-004 : fichier absent → ERROR not found ; arrêt propre du serveur.
#[tokio::test]
async fn missing_file_and_shutdown() {
    let (srv, _root) = start("missing", false).await;
    let sock = tc::client_socket().await;
    tc::send_to(&sock, srv.local_addr(), &tc::rrq("nope.bin", &[])).await;
    let (p, _tid) = tc::recv_from(&sock).await;
    assert_eq!(tc::op(&p), 5);
    assert_eq!(tc::errcode_of(&p), 1, "not found");
    srv.shutdown().await; // retourne = arrêt propre
}
