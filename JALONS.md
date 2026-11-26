# JALONS.md — Jalons, critères de recette et dépendances

Règle : chaque jalon = code réel + tests automatisés + démonstration + bilan +
procédure de recette utilisateur + critères mesurés ; validation utilisateur
avant passage au jalon suivant. Aucun placeholder présenté comme fonctionnel.

## J0 · Faisabilité (prototypes et risques)
- Prototypes : (1) IronRDP connexion + RDPDR dossiers arbitraires confinés
  (Linux puis Windows) ; (2) russh auth clé + KbdInt/MFA + SFTP transfert ;
  (3) terminal alacritty_terminal + portable-pty dans egui (rendu, collage,
  Ctrl+C préservé) ; (4) DHCP encodage options 43/66/67/121/150 + DORA loopback
  isolé ; (5) TFTP lecture/écriture + anti-traversée ; (6) évaluation VNC, Mosh,
  NFS, X11/XDMCP, pcap ; (7) keyring/Argon2/AES-GCM round-trip.
- Livrables : architecture figée, ADR validés, matrice de parité initiale
  complétée avec l'utilisateur, audit licences, `rust-toolchain.toml`,
  workspace compilable, CI squelette.
- Critères : chaque prototype démontré par commande/test réel ; risques listés
  avec décision (faire / adapter / reporter) ; aucune fonction déclarée
  opérationnelle sans preuve.

## J1 · Socle
- UI egui/eframe (fenêtre, barre d'outils, arbre, onglets, split, barre d'état),
  thèmes clair/sombre/auto, FR/EN, HiDPI ; SQLite + migrations ; coffre
  (verrouillage, masquage) ; préférences persistantes ; modale F12 + export
  expurgé ; auto-tests F12.
- Recette : REC-01 (partie stockage/coffre), REC-06.

## J2 · Accès texte
- Terminal complet (VT, scrollback, recherche, profils), SSH2 (tous modes
  d'auth), SFTP automatique, série, shell local, tunnels de base,
  copier/coller selon réglages, multi-exécution explicite.
- Recette : REC-02.

## J3 · Bureau distant
- RDP intégré (NLA, TLS, gateway, clipboard, résolution dynamique), RDPDR
  dossiers nommés confinés ; VNC ; presse-papiers.
- Recette : REC-03.

## J4 · Services clés
- DHCPv4 avancé (scopes, réservations, baux persistants, journal DORA, éditeur
  d'options typées avec aperçu octets, priorités) ; TFTP complet ; association
  PXE DHCP+TFTP sans démarrage croisé automatique.
- Recette : REC-04, REC-05 (tests sur réseau isolé uniquement).

## J5 · Parité élargie
- Protocoles restants (Telnet, Rlogin/Rsh, Mosh, X11/XDMCP, FTP/FTPS, SCP),
  outils réseau (ping, traceroute, DNS, ports, capture), serveurs locaux
  (HTTP, FTP, SSH, NFS), ordonnanceur local, macros, favoris, comparaisons.
- Recette : matrice de parité mise à jour avec preuves.

## J6 · Migration
- Imports PuTTY / PuTTYCM XML / MobaXterm, export JSON versionné,
  sauvegarde/restauration chiffrée, rapport d'import.
- Recette : round-trip import→export→restauration sur jeux de données fournis.

## J7 · Release
- Sécurité complète (audit, fuzz, cargo deny, SBOM), packaging .exe NSIS +
  .deb, installation/désinstallation sur VM propres Windows 11 / Debian 13 /
  Ubuntu 24.04, documentation utilisateur/admin, changelog, rollback.
- Recette : REC-07 + rapport de tests complet.
