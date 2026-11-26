# TESTS_RESULTS.md — Résultats d'exécution réels

Format : RUN-XXXX | date | version/commit | OS/arch | outillage | commandes
réellement lancées | code retour | durée | totaux | résultat par test | preuve.

## RUN-0001 — 2026-11-26T21:27Z
- Version/commit : commit initial de suivi (pré-J0).
- OS/arch : Linux x86_64 (toolbox KodeOS).
- Commandes lancées :
  - `cargo --version` → code 0, sortie `cargo 1.100.0 (3acc79c0c 2026-11-05)`
  - `rustc --version` → code 0, sortie `rustc 1.100.0 (0ef1bc386 2026-11-09)`
  - `curl -sS -o /dev/null -w %{http_code} https://static.crates.io/crates/serde/serde-1.0.0.crate` → 200
  - `curl -sS -o /dev/null -w %{http_code} https://index.crates.io/config.json` → 200
  - `pkg-config --list-all | grep -Ei 'pcap|sqlite|openssl|…'` → sqlite3, openssl, xcb, wayland, fontconfig présents ; pcap absent
- Totaux tests applicatifs : 0 exécutés (aucun code encore) ; TEST-* du
  catalogue : NOT_RUN.
- Conclusion : environnement de build viable ; registre crates.io accessible.
  Aucun test fonctionnel n'a été exécuté à ce stade.

## RUN-0002 — 2026-11-26T22:57Z
- Version/commit : travail J0 (pré-commit ; commit de suivi précédent 0a9db8a).
- OS/arch : Linux x86_64 (toolbox KodeOS), rustc/cargo 1.100.0 (toolchain
  épinglée rust-toolchain.toml = 1.100.0).
- Commandes réellement lancées et résultats :
  - `cargo build --workspace` → code 0
  - `cargo test --workspace` → code 0 ; 29 passed / 0 failed / 0 ignored
    (service-dhcp 11, service-tftp 5, vault 4, terminal 4, protocol-ssh 2,
    protocol-rdp 3)
  - `cargo clippy --workspace --all-targets -- -D warnings` → code 0
  - `cargo fmt --all -- --check` → code 0
  - `cargo build --workspace --release --locked` → code 0 (log conservé :
    /tmp/build-release.log ; 0 erreur)
  - sshd OpenSSH 10.0p2 démarré sur 127.0.0.1:2222 (fixture de test SSH) :
    log « Server listening on 127.0.0.1 port 2222 »
- Détail par test : TEST-DHCP-001→005, 008→013 PASS (unitaires) ;
  TEST-DHCP-006/007 (DORA loopback, interfaces) NOT_RUN (J4) ;
  TEST-TFTP-001→004 PASS (intégration loopback) ; TEST-VAULT-001 (+b,c,d)
  PASS ; TEST-SSH-001 PASS (contre OpenSSH réel) ; TEST-SSH-002 (clé d'hôte
  inconnue refusée) PASS ; TEST-TERM-001→004 PASS (dont Ctrl+C transmis) ;
  TEST-RDP-001→003 PASS (confinement RDPDR, IRP simulés).
- Conclusion : prototypes J0 démontrés par tests réels. Non validés : DORA
  complet, RDP connexion réelle/NLA/rendu, VNC, Mosh, NFS, X11, pcap, UI egui.
