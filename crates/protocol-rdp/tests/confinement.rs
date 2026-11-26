//! TEST-RDP-002 : confinement RDPDR aux dossiers sélectionnés (prototype J0).
//! IRP simulés (create/read/write/close) sur un partage temporaire.
use std::path::PathBuf;

use protocol_rdp::{ConfinedDriveBackend, ShareMapping, resolve_confined};

fn setup(tag: &str) -> (ConfinedDriveBackend, PathBuf, PathBuf) {
    let mut root = std::env::temp_dir();
    root.push(format!("rdpdr-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(root.join("docs")).expect("mkdir");
    std::fs::write(root.join("docs/note.txt"), b"contenu-partage").expect("write");
    let outside = root.parent().unwrap().join(format!("secret-{tag}.txt"));
    std::fs::write(&outside, b"SECRET-HORS-PARTAGE").expect("write outside");
    let backend = ConfinedDriveBackend::new(ShareMapping {
        root: root.clone(),
        share_name: "Transferts".into(),
    });
    (backend, root, outside)
}

/// TEST-RDP-001 (partie J0) : lecture/écriture dans le partage autorisé.
#[test]
fn read_write_inside_share() {
    let (mut b, root, _outside) = setup("ok");
    let fid = b
        .create("\\docs\\note.txt")
        .expect("create dans le partage");
    let data = b.read(fid, 0, 1024).expect("read");
    assert_eq!(data, b"contenu-partage");
    let n = b.write(fid, 0, b"nouveau").expect("write");
    assert_eq!(n, 7);
    // IRP_MJ_WRITE à offset 0 sans troncature : le reste du fichier suit.
    assert_eq!(
        std::fs::read(root.join("docs/note.txt")).unwrap(),
        b"nouveau-partage"
    );
    b.close(fid);
    assert!(b.read(fid, 0, 10).is_err(), "descripteur fermé");
}

/// TEST-RDP-002 : toute évasion hors du partage est refusée.
#[test]
fn escape_attempts_denied() {
    let (mut b, root, outside) = setup("esc");
    assert!(b.create("\\..\\secret-esc.txt").is_err());
    assert!(b.create("C:\\Windows\\win.ini").is_err());
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&outside, root.join("link-out")).expect("symlink");
        assert!(b.create("\\link-out").is_err(), "symlink sortant refusé");
    }
    assert!(resolve_confined(&root, "\\..\\..\\etc\\passwd").is_err());
    assert!(resolve_confined(&root, "\\docs\\note.txt").is_ok());
    assert_eq!(std::fs::read(&outside).unwrap(), b"SECRET-HORS-PARTAGE");
    let _ = std::fs::remove_file(&outside);
}

/// TEST-RDP-003 : lecture hors des bornes / descripteur inconnu refusés.
#[test]
fn invalid_handles_denied() {
    let (mut b, _root, _outside) = setup("inv");
    assert!(b.read(999, 0, 10).is_err());
    assert!(b.write(999, 0, b"x").is_err());
}
