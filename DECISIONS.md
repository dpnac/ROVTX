# DECISIONS.md — Décisions validées, propositions, décisions en attente

Format : ADR-XXXX | statut (proposée / validée / rejetée / en attente) |
contexte | décision | conséquences | date | MSG.

## ADR-0001 — Rust 2024, egui/eframe + wgpu, Tokio — VALIDÉE (imposée par le prompt maître)
- Contexte : le prompt maître v1.2 impose Rust édition 2024, interface native
  egui/eframe + wgpu, sans Electron/WebView/React/Qt/C++ applicatif ; Tokio pour
  l'asynchrone, canaux bornés, thread graphique jamais bloqué.
- Décision : stack retenue telle quelle ; toolchain stable épinglée dans
  `rust-toolchain.toml` ; Cargo workspace ; `Cargo.lock` versionné.
- Conséquences : dépendances UI = egui, eframe, egui_dock ; tout I/O via Tokio
  derrière des canaux bornés avec annulation explicite.
- 2026-11-26 — MSG-0001.

## ADR-0002 — Organisation Cargo workspace (crates séparées) — VALIDÉE (proposition du prompt maître)
- Décision : crates `desktop, core, sessions, terminal, protocol-ssh,
  protocol-rdp, protocol-vnc, protocol-legacy, storage, vault, service-dhcp,
  service-tftp, service-other, network-tools, privileged-helper, diagnostics,
  platform` (+ `packaging/`, `tests/`, `.github/workflows/`).
- Conséquences : traits communs pour protocoles/services ; erreurs typées ;
  séparation état UI / I/O ; `unsafe` confiné et justifié dans `platform`.
- 2026-11-26 — MSG-0001.

## ADR-0003 — SSH/SFTP : russh + russh-sftp — PROPOSÉE (validation J0)
- Contexte : exigence SSH2 complète (agent, MFA, known_hosts) + SFTP graphique.
- Décision proposée : `russh` (client) et `russh-sftp` ; tunnels via russh.
- Validation requise : prototype J0 (auth par clé + KbdInt + SFTP list/transfert).
- 2026-11-26 — MSG-0001.

## ADR-0004 — RDP : IronRDP + canal RDPDR confiné — PROPOSÉE (validation J0, priorité prototype)
- Contexte : REQ-005/REQ-006 ; le prompt maître privilégie IronRDP et interdit le
  simple lancement d'un client externe ; adaptateur FreeRDP seulement après
  validation explicite si IronRDP est insuffisant.
- Décision proposée : IronRDP comme backend unique au départ ; prototype J0 de
  redirection de dossiers arbitraires (RDPDR) avec preuve de confinement sur
  Linux puis Windows ; extension Rust si lacune.
- Risque connu : maturité IronRDP (NLA/CredSSP, clipboard, audio, multi-écran)
  à vérifier fonction par fonction en J0/J3.
- 2026-11-26 — MSG-0001.

## ADR-0005 — DHCPv4 : dhcproto + socket2, moteur maison — PROPOSÉE (validation J0)
- Contexte : REQ-020/021 ; dhcproto fournit l'encodage/décodage des paquets,
  pas la politique (scopes, baux, priorités d'options).
- Décision proposée : moteur DORA maison sur Tokio + socket2 (broadcast par
  interface), baux en SQLite, éditeur d'options typées avec aperçu octets.
- Validation requise : tests unitaires d'encodage (43, 66, 67, 121, 150) puis
  tests paquets sur réseau isolé (J4).
- 2026-11-26 — MSG-0001.

## ADR-0006 — Stockage : rusqlite (SQLite) — PROPOSÉE
- Décision proposée : `rusqlite` avec feature `bundled` pour reproductibilité
  des builds (le système fournit aussi sqlite3 : à trancher en J0 selon audit
  licence/poids) ; migrations versionnées dans `crates/storage`.
- 2026-11-26 — MSG-0001.

## ADR-0007 — Coffre : AES-256-GCM + Argon2id + trousseau natif (keyring) — PROPOSÉE
- Décision proposée : maître de clé aléatoire ; si mot de passe maître :
  dérivation Argon2id ; enveloppe AES-256-GCM ; `zeroize`/`secrecy` pour la
  mémoire ; option trousseau système (`keyring`) ; aucun secret en logs.
- 2026-11-26 — MSG-0001.

## ADR-0008 — TFTP : module Rust asynchrone dédié (RFC 1350 + options) — VALIDÉE (imposée)
- Décision : implémentation maison Tokio (UDP 69 + ports de transfert), racine
  confinée, écriture désactivée par défaut, anti-traversée et anti-symlinks.
- 2026-11-26 — MSG-0001.

## ADR-0009 — Terminal : alacritty_terminal + portable-pty — PROPOSÉE (validation J0)
- Décision proposée : émulation VT par `alacritty_terminal` (crate autonome),
  PTY/ConPTY par `portable-pty`, rendu egui custom ; backpressure explicite.
- Alternative à comparer en J0 si besoin : vt100/vte (plus léger, moins complet).
- 2026-11-26 — MSG-0001.

## ADR-0010 — Traçabilité et qualité continues — VALIDÉE (imposée)
- Décision : fichiers de suivi (MESSAGES, OC, DECISIONS, JALONS, TODO, BUGS,
  TESTS*, REQUIREMENTS, AVANCEMENT…) maintenus à chaque intervention ; MSG id ;
  aucun résultat de test inventé ; recette utilisateur obligatoire par jalon.
- 2026-11-26 — MSG-0001.

## ADR-0011 — Icône du projet : `icon.png` (512×512 RGBA) — VALIDÉE
- Contexte : MSG-0002 ; l'utilisateur fournit `icon.png` comme icône des
  exécutables (packaging NSIS/DEB, J7).
- Décision : fichier versionné à la racine du dépôt (`icon.png`) ; conversions
  .ico/.png multi-tailles générées au build packaging (jamais versionnées).
- 2026-11-26 — MSG-0002.

## ADR-0012 — Backend filesystem RDPDR maison sur ironrdp-rdpdr — PROPOSÉE (prototype J0 validé)
- Contexte : ironrdp-rdpdr 0.9 fournit le canal MS-RDPEFS mais aucun backend
  filesystem ; REQ-006 exige la redirection de dossiers arbitraires confinée.
- Décision : extension Rust `ConfinedDriveBackend` (crates/protocol-rdp)
  implémentant `RdpdrBackend` (create/read/write/close) avec confinement strict
  à la racine du partage ; tests de confinement verts (TEST-RDP-002/003).
- Conséquences : IRP query/directory/security à implémenter en J3 ; connexion
  RDP complète (NLA, rendu) prototypée en J3 contre serveur réel.
- 2026-11-26 — MSG-0002.

## Décisions en attente (à trancher en J0 / avec l'utilisateur)
- D-ATT-01 : VNC — crate `vnc-rs` 0.6.1 (MIT OR Apache-2.0, client VNC async) :
  **évaluée OK licence** (MSG-0002) ; prototype de connexion à faire en J3.
- D-ATT-02 : Mosh — `mosh` 0.1.4 (mosh-rs) est **GPL-3.0-or-later** : interdite
  en lien statique selon LICENSES.md. Options : binaire séparé distribué à part
  (conforme GPL) ou report. **Décision requise utilisateur** ; défaut proposé :
  report J5 + adaptateur processus isolé documenté.
- D-ATT-03 : NFS — crate `nfs` 0.1.0 = client NFSv3/v4 userspace (Apache-2.0),
  pas un serveur. Serveur NFS local = composant système (nfs-ganesha / export
  kernel) piloté par l'app, documenté comme dépendance OS (proposition J5).
- D-ATT-04 : X11 forwarding / XDMCP — intégration native à valider (serveur X local).
- D-ATT-05 : capture PCAP — `pcap` 2.5.0 (MIT OR Apache-2.0) : licence OK ;
  dépendance système libpcap-dev à installer (Npcap sous Windows, non
  redistribué : installation documentée). Prototype J5.
- D-ATT-06 : helper privilégié — périmètre exact (bind < 1024 Linux) et IPC.
- D-ATT-07 : rusqlite bundled vs system (ADR-0006).
- D-ATT-08 : éditions/versions de référence PuTTYCM/MobaXterm (inventaire user).
- D-0007 (2026-11-27, MSG-0006) : provider crypto de russh = `ring` (features
  explicites flate2+ring+rsa, default-features=false). Motif : aws-lc-sys exige
  NASM sous Windows MSVC (CI windows-2022 en échec) ; ring se compile sans
  assembleur externe. Algorithmes SSH négociés inchangés pour nos tests réels.
  Réversible si NASM est installé sur les runners (setup nasm) — non retenu :
  dépendance d'outillage supplémentaire sans bénéfice fonctionnel.
