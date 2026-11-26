# LICENSES.md — Audit de licences des dépendances et composants

Statut : à compléter en J0 (TASK-0109) avec `cargo deny` et vérification
manuelle des crates retenues. Aucune dépendance adoptée sans licence
compatible documentée ici.

## Dépendances candidates (stack de référence du prompt maître)
| Crate / composant | Licence attendue (à vérifier) | Usage | Statut audit |
|---|---|---|---|
| egui, eframe, egui_dock | MIT OR Apache-2.0 | UI | à vérifier |
| tokio | MIT | async | à vérifier |
| alacritty_terminal | Apache-2.0 | émulation VT | à vérifier |
| portable-pty | MIT/Apache-2.0 | PTY/ConPTY | à vérifier |
| russh, russh-sftp | Apache-2.0 | SSH/SFTP | à vérifier |
| ironrdp (+ rdpdr) | MIT/Apache-2.0 | RDP | à vérifier |
| tokio-serial | MIT | série | à vérifier |
| dhcproto | MIT/Apache-2.0 | DHCP | à vérifier |
| socket2 | MIT/Apache-2.0 | sockets | à vérifier |
| rusqlite | MIT | SQLite | à vérifier |
| serde | MIT/Apache-2.0 | sérialisation | à vérifier |
| keyring | MIT/Apache-2.0 | trousseau | à vérifier |
| argon2, aes-gcm, secrecy, zeroize | MIT/Apache-2.0 | crypto | à vérifier |
| tracing, tracing-subscriber | MIT | journaux | à vérifier |
| arboard | MIT/Apache-2.0 | presse-papiers | à vérifier |
| pcap | MIT/Apache-2.0 | capture (libpcap/Npcap : BSD) | à vérifier |
| cargo-packager | MIT/Apache-2.0 | packaging | à vérifier |

## Composants système redistribués
- SQLite (domaine public), OpenSSL (Apache-2.0) si lié, libpcap (BSD-3),
  Npcap (licence propre : à auditer si redistribué — préférer lien dynamique
  et installation documentée).

## Règles
- Licences acceptées : MIT, Apache-2.0, BSD-2/3, ISC, Unicode-DFS, Zlib,
  domaine public (CC0 à valider au cas par cas).
- Copyleft (GPL/AGPL) : interdit dans le lien statique de l'application ;
  exception uniquement sur décision explicite documentée (ADR).
- Formats redistribués (polices, icônes) : licences vérifiées et citées ici.
