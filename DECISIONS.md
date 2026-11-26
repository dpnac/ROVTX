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

## Décisions en attente (à trancher en J0 / avec l'utilisateur)
- D-ATT-01 : VNC — crate Rust mature ? (évaluation J0 ; sinon adaptateur validé).
- D-ATT-02 : Mosh — pas de crate client mature connue ; prototype J0 ou report J5.
- D-ATT-03 : NFS serveur local — composant système (nfs-ganesha/export kernel)
  vs crate Rust ; décision après audit.
- D-ATT-04 : X11 forwarding / XDMCP — intégration native à valider (serveur X local).
- D-ATT-05 : capture PCAP — `pcap` (libpcap/Npcap) : dépendance native à auditer ;
  libpcap-dev absent de la toolbox actuelle (installation à demander).
- D-ATT-06 : helper privilégié — périmètre exact (bind < 1024 Linux) et IPC.
- D-ATT-07 : rusqlite bundled vs system (ADR-0006).
- D-ATT-08 : éditions/versions de référence PuTTYCM/MobaXterm (inventaire user).
