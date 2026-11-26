# ARCHITECTURE.md — Architecture cible et décisions

## Vue d'ensemble
```text
┌──────────────────────────────────────────────────────────────┐
│ Interface native Rust — egui/eframe + wgpu                    │
│ onglets (egui_dock) · arbre · SFTP · outils · F12 · thèmes    │
├──────────────────────────────────────────────────────────────┤
│ Cœur applicatif Rust — Tokio                                  │
│ modèles · événements · orchestration · canaux bornés          │
├───────────────┬──────────────────────┬───────────────────────┤
│ Protocoles    │ Services             │ Stockage / sécurité   │
│ SSH·RDP·VNC·  │ DHCPv4 · TFTP ·      │ SQLite · coffre       │
│ série·legacy  │ autres serveurs ·    │ (AES-GCM/Argon2id) ·  │
│               │ IPC helper           │ keyring · imports     │
├───────────────┴──────────────────────┴───────────────────────┤
│ platform : intégrations OS (PTY/ConPTY, trousseaux, FFI isolée)│
└──────────────────────────────────────────────────────────────┘
Cibles : Windows 11, Debian 13, Ubuntu 24.04 — Cargo · CI GitHub Actions
```

## Crates du workspace (noms indicatifs, ADR-0002)
| Crate | Rôle |
|---|---|
| desktop | egui/eframe, navigation, modales, thèmes, F12 |
| core | modèles, événements, orchestration, backpressure |
| sessions | cycle de vie des sessions, héritage de paramètres |
| terminal | émulation VT (alacritty_terminal), rendu, PTY (portable-pty) |
| protocol-ssh | russh/russh-sftp, tunnels, known_hosts, agent |
| protocol-rdp | IronRDP, rendu, RDPDR confiné |
| protocol-vnc | client VNC (crate à valider en J0) |
| protocol-legacy | Telnet, Rlogin/Rsh, série (tokio-serial), X11/XDMCP, Mosh |
| storage | rusqlite, migrations, sauvegardes |
| vault | AES-256-GCM/XChaCha20-Poly1305, Argon2id, keyring, zeroize |
| service-dhcp | moteur DORA, baux, options typées, journal |
| service-tftp | RFC 1350 + blksize/timeout/tsize, racine confinée |
| service-other | HTTP/HTTPS, FTP/FTPS, SSH/SFTP serveur, NFS, cron local |
| network-tools | ping, traceroute, DNS, ports, routes, capture pcap |
| privileged-helper | IPC authentifié, privilèges minimaux, désactivé par défaut |
| diagnostics | tracing, modale F12, export expurgé, auto-tests |
| platform | FFI/native isolé (ConPTY, trousseaux, interfaces réseau) |

## Principes
- Traits `ProtocolAdapter` / `NetworkService` : erreurs typées, état, arrêt
  propre ; chaque protocole isolé derrière son crate.
- Séparation stricte état UI / I/O : le thread graphique ne fait que rendre et
  produire des commandes ; tout I/O est asynchrone (Tokio) avec canaux bornés
  et tokens d'annulation.
- Backpressure : terminal (flux octets → grille), rendu RDP/VNC (frames),
  transferts SFTP/TFTP (fenêtres glissantes bornées).
- Sécurité : coffre authentifié ; IPC authentifié vers helper privilégié
  (optionnel, désactivé) ; UI jamais en admin/root par défaut ; vérification
  des clés d'hôte et certificats non affaiblie ; zéro secret en logs.
- `unsafe`/FFI : uniquement dans `platform` ou adaptateurs dédiés, justifié,
  documenté, testé (ADR-0002).

## Dépendances natives système (Linux)
- Présentes dans l'environnement de build actuel : libsqlite3, openssl,
  xcb/x11, wayland-client, fontconfig/freetype (requis eframe/wgpu).
- À installer si besoin : `libpcap-dev` (capture), `libsecret`/secret-service
  (trousseau, optionnel avec fallback).
- Windows : ConPTY natif ; Npcap/libpcap seulement si la capture est retenue.

## Packaging
`cargo-packager` : NSIS `.exe` x64 et `.deb` amd64, compilation native par OS
sur runners CI ; helpers privilégiés optionnels désactivés par défaut.
