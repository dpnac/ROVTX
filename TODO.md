# TODO.md — Tâches, dépendances, priorités, critères de fin

Format : TASK-XXXX | priorité | jalon | dépendances | critères de fin | réf.

## Pré-J0 / cadrage
- TASK-0001 | P0 | pré-J0 | — | Fichiers de suivi créés et versionnés ; cadrage
  remis à l'utilisateur. | MSG-0001, REQ-027. **Critère : fait (commit initial).**
- TASK-0002 | P0 | pré-J0 | TASK-0001 | Inventaire avec l'utilisateur des
  fonctions/éditions de référence (matrice de parité détaillée). | REQ-017/018.
- TASK-0003 | P0 | pré-J0 | TASK-0001 | Autorisation explicite de lancer J0. | PM « Première réponse attendue ». **FAIT (MSG-0002).**

## J0
- TASK-0101 | P0 | J0 | autorisation user | Créer workspace Cargo + rust-toolchain.toml
  + CI squelette ; `cargo build --workspace` vert. | ADR-0001/0002. **FAIT (RUN-0002).**
- TASK-0102 | P0 | J0 | TASK-0101 | Prototype IronRDP + RDPDR confinement dossiers
  arbitraires (Linux d'abord, Windows ensuite) ; preuve lecture/écriture +
  absence d'évasion. | REQ-005/006, ADR-0004. **FAIT côté confinement (tests IRP
  simulés, RUN-0002) ; connexion réelle Windows/Linux = J3 (TASK-0301).**
- TASK-0103 | P0 | J0 | TASK-0101 | Prototype russh : auth clé + KbdInt + SFTP
  upload/download contre serveur de test local. | REQ-001/012.
- TASK-0104 | P0 | J0 | TASK-0101 | Prototype terminal egui + alacritty_terminal +
  portable-pty : rendu, sélection/copie, collage, Ctrl+C préservé. | REQ-011.
  **FAIT sans rendu egui (RUN-0002) ; rendu egui = J1/J2.**
- TASK-0105 | P0 | J0 | TASK-0101 | Prototype DHCP : encodage/décodage options
  43/66/67/121/150 (tests unitaires octet par octet) + DORA loopback isolé. | REQ-020/021.
  **FAIT pour encodage + cross-check dhcproto (RUN-0002) ; DORA loopback = J4.**
- TASK-0106 | P0 | J0 | TASK-0101 | Prototype TFTP : get/put, blksize/tsize,
  refus traversée et symlinks. | REQ-022. **FAIT (RUN-0002).**
- TASK-0107 | P1 | J0 | TASK-0101 | Prototype coffre : Argon2id + AES-256-GCM
  round-trip + keyring (Linux secret-service, fallback fichier chiffré). | REQ-019.
  **FAIT sans keyring (RUN-0002) ; keyring = J1.**
- TASK-0108 | P1 | J0 | — | Évaluation VNC / Mosh / NFS / X11-XDMCP / pcap :
  bilan capacités + licences + décision (ADR). | REQ-004/007/008/015/016.
  **FAIT (DECISIONS.md D-ATT-01→05 ; mosh GPL = décision user).**
- TASK-0109 | P1 | J0 | TASK-0101..0108 | Audit licences (cargo deny config) +
  LICENSES.md rempli ; versions verrouillées dans Cargo.lock. | REQ-026.
  **PARTIEL : Cargo.lock verrouillé, licences crates évaluées ; config cargo
  deny = J1.**

## J1+
- À décliner à l'ouverture de chaque jalon depuis JALONS.md et REQUIREMENTS.md.
