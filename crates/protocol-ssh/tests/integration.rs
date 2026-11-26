//! TEST-SSH-001 / TEST-SFTP-001 : auth par clé + exec + SFTP contre un vrai
//! serveur OpenSSH local (127.0.0.1:2222), démarré par la recette J0.
//! Prérequis : sshd actif sur 127.0.0.1:2222 avec clé client autorisée et
//! fixture known_fingerprints.txt (voir TESTS_MANUELS.md TM-00-01).
use std::path::PathBuf;

use protocol_ssh::{KnownHosts, connect_key, exec, open_sftp, sftp_download, sftp_upload};

const HOST: &str = "127.0.0.1";
const PORT: u16 = 2222;

fn key_path() -> PathBuf {
    PathBuf::from("/tmp/sshd-test/client_ed25519")
}

async fn server_up() -> bool {
    tokio::net::TcpStream::connect((HOST, PORT)).await.is_ok()
}

fn known_from_fixture() -> Option<KnownHosts> {
    let p = PathBuf::from("/tmp/sshd-test/known_fingerprints.txt");
    let content = std::fs::read_to_string(p).ok()?;
    let mut fingerprints: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for line in content.lines() {
        let mut it = line.split_whitespace();
        if let (Some(host), Some(fp)) = (it.next(), it.next()) {
            fingerprints
                .entry(host.to_string())
                .or_default()
                .push(fp.to_string());
        }
    }
    Some(KnownHosts { fingerprints })
}

#[tokio::test]
async fn ssh_key_auth_exec_and_sftp() {
    if !server_up().await {
        eprintln!("SKIPPED : sshd 127.0.0.1:2222 indisponible (recette TM-00-01)");
        return;
    }
    let known = known_from_fixture().expect("fixture known_fingerprints.txt");
    let user = std::env::var("USER").unwrap_or_else(|_| "dev".into());

    let mut handle = connect_key(HOST, PORT, &user, &key_path(), known)
        .await
        .expect("connexion SSH par clé");

    let (out, code) = exec(&mut handle, "echo PROTO-SSH-OK").await.expect("exec");
    assert_eq!(String::from_utf8_lossy(&out).trim(), "PROTO-SSH-OK");
    assert_eq!(code, Some(0));

    let mut handle2 = connect_key(
        HOST,
        PORT,
        &user,
        &key_path(),
        known_from_fixture().unwrap(),
    )
    .await
    .expect("seconde connexion");
    let sftp = open_sftp(&mut handle2).await.expect("sous-système sftp");
    let payload = b"contenu-sftp-j0-1234567890".to_vec();
    let remote = format!("/tmp/sshd-test/sftp-roundtrip-{}.bin", std::process::id());
    sftp_upload(&sftp, &remote, &payload).await.expect("upload");
    let back = sftp_download(&sftp, &remote).await.expect("download");
    assert_eq!(back, payload);
    let entries = sftp.read_dir("/tmp/sshd-test").await.expect("read_dir");
    let names: Vec<String> = entries.map(|e| e.file_name().to_string()).collect();
    let base = remote.rsplit('/').next().unwrap().to_string();
    assert!(names.contains(&base), "fichier uploadé listé : {names:?}");
    sftp.remove_file(&remote).await.expect("nettoyage");
}

#[tokio::test]
async fn unknown_host_key_rejected() {
    if !server_up().await {
        eprintln!("SKIPPED : sshd indisponible");
        return;
    }
    let known = KnownHosts::default();
    let user = std::env::var("USER").unwrap_or_else(|_| "dev".into());
    let r = connect_key(HOST, PORT, &user, &key_path(), known).await;
    assert!(r.is_err(), "clé d'hôte inconnue doit être refusée");
}
