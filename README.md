# Gestionnaire de connexions et outils réseau (Rust)

Alternative autonome, native et open-source à PuTTY Connection Manager et
MobaXterm, enrichie d'un serveur DHCPv4 avancé et d'un serveur TFTP :
terminal multi-session, SSH/SFTP, RDP avec redirection de dossiers confinée,
VNC, série, tunnels, outils réseau et services locaux.

- **Langage / UI** : Rust (édition 2024), egui/eframe + wgpu, Tokio.
- **Cibles** : Windows 11 x64, Debian 13 amd64, Ubuntu 24.04 LTS amd64.
- **Livrables** : dépôt source, installateur `.exe` (NSIS), paquet `.deb`,
  documentation, tests, CI/CD.
- **Statut** : pré-J0 (cadrage). Aucun module n'est déclaré opérationnel.
  Voir `AVANCEMENT.md` et `JALONS.md`.

## Démarrer ici
1. `AGENTS.md` — règles permanentes pour tout intervenant (humain ou agent).
2. `OC.md` — instructions de reprise.
3. `AVANCEMENT.md` — état réel du projet.
4. `JALONS.md`, `TODO.md`, `MESSAGES.md` — planification et journal.
5. `docs/inputs/` — prompt maître v1.2 et cahier des charges v1.2 (sources
   d'exigences, copies de travail versionnées).

## Compiler (à partir de J0)
```text
cargo build --workspace            # développement
cargo build --workspace --release --locked
cargo test --workspace --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
```
Prérequis système Linux : voir `ARCHITECTURE.md` § dépendances natives.
Aucune dépendance manuelle non documentée n'est acceptée.

## Sécurité
Aucun secret dans les logs, arguments, télémétrie ou exports F12.
Coffre chiffré (AES-256-GCM, Argon2id), trousseau système optionnel.
Voir `SECURITY.md`.

## Licence
À définir en J0 après audit des dépendances (`LICENSES.md`).
