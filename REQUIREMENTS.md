# REQUIREMENTS.md — Matrice de traçabilité REQ → MSG → TASK/ADR → TEST → état

Règle : toute nouvelle exigence ajoute une ligne ; aucune exigence n'est
supprimée sans accord explicite de l'utilisateur. Statuts : reçue, analysée,
en cours, implémentée, testée, en attente de recette, validée, rejetée.

| REQ | Fonction | Source | MSG | TASK / ADR | TEST | État |
|-----|----------|--------|-----|------------|------|------|
| REQ-001 | SSH2 : mot de passe, clés, agent, keyboard-interactive/MFA, known_hosts | CDC §01/03, PM §2 | MSG-0001 | ADR-0003 | à créer (J2) | analysée |
| REQ-002 | Telnet | CDC §01, PM §2 | MSG-0001 | — | à créer (J2) | analysée |
| REQ-003 | Rlogin/Rsh (hérités, avertissement clair) | CDC §01, PM §2 | MSG-0001 | — | à créer (J5) | analysée |
| REQ-004 | Mosh | CDC §01, PM §2 | MSG-0001 | proto J0 | à créer (J5) | analysée |
| REQ-005 | RDP intégré : NLA/CredSSP, TLS, certificats, domaine, RD Gateway, clavier, résolution dynamique, plein écran, multi-écran, presse-papiers, audio | CDC §03, PM §4 | MSG-0001 | ADR-0004 | REC-03 | analysée |
| REQ-006 | RDPDR : redirection de dossiers arbitraires nommés, confinement strict, lecture/écriture prouvées Win+Linux | CDC §03, PM §4 | MSG-0001 | ADR-0004, proto J0 | REC-03 | analysée |
| REQ-007 | VNC | CDC §01, PM §2 | MSG-0001 | proto J0 | à créer (J3) | analysée |
| REQ-008 | X11 forwarding / XDMCP | CDC §01, PM §2 | MSG-0001 | proto J0 | à créer (J5) | analysée |
| REQ-009 | Série RS232/USB (tokio-serial) | CDC §01, PM §2 | MSG-0001 | — | à créer (J2) | analysée |
| REQ-010 | Shell local (Bash, PowerShell, CMD) | CDC §01/05, PM §2 | MSG-0001 | — | à créer (J2) | analysée |
| REQ-011 | Terminal VT/xterm : ANSI, Unicode, 24 bits, scrollback, recherche, redimensionnement, sélection, raccourcis, logs, profils, reconnexion, plein écran, onglets détachables, split, macros, favoris, multi-exécution | CDC §02, PM §2/7 | MSG-0001 | ADR-0009 | REC-02 | analysée |
| REQ-012 | SFTP graphique automatique avec SSH : navigation, transferts, DnD, permissions, renommage, progression, annulation, édition distante avec conflits | CDC §01, PM §2 | MSG-0001 | ADR-0003 | REC-02 | analysée |
| REQ-013 | SCP, FTP/FTPS, comparaison de fichiers selon capacités | CDC §01, PM §2 | MSG-0001 | — | à créer (J5) | analysée |
| REQ-014 | Tunnels SSH locaux/distants/SOCKS, rebonds multiples, proxy, état | CDC §01/05, PM §2 | MSG-0001 | — | à créer (J2/J5) | analysée |
| REQ-015 | Outils réseau : ping, traceroute, DNS, ports/scan autorisés, interfaces/IP/routes, capture PCAP | CDC §05, PM §2 | MSG-0001 | proto J0 (pcap) | à créer (J5) | analysée |
| REQ-016 | Serveurs locaux : TFTP, HTTP/HTTPS, FTP/FTPS, SSH/SFTP, NFS ; ordonnanceur local type cron | CDC §05, PM §2/6 | MSG-0001 | proto J0 (NFS) | à créer (J5) | analysée |
| REQ-017 | Gestionnaire de connexions : arbre illimité, DnD, recherche, favoris, tags, duplication, historique, héritage explicite (jamais de secrets), confirmation commandes dangereuses | CDC §03, PM §3 | MSG-0001 | — | REC-01 | analysée |
| REQ-018 | Imports PuTTY (registre/export), PuTTYCM XML, MobaXterm ; export JSON versionné ; sauvegarde/restauration | CDC §01, PM §3 | MSG-0001 | — | à créer (J6) | analysée |
| REQ-019 | Coffre : AES-256-GCM ou XChaCha20-Poly1305, Argon2id, trousseau natif, verrouillage, masquage, export sans secrets par défaut | CDC §03, PM §3 | MSG-0001 | ADR-0007 | REC-01 | analysée |
| REQ-020 | DHCPv4 avancé : interface obligatoire, scopes, plages, exclusions, réservations MAC/client-ID, baux persistants, journal DORA, RELEASE/DECLINE/INFORM, conflits, relais/classes à valider | CDC §04, PM §5 | MSG-0001 | ADR-0005 | REC-04 | analysée |
| REQ-021 | Éditeur d'options DHCP typées (HEX, texte, IPv4, liste IPv4, uint8/16/32, bool, TLV) + aperçu octets ; codes 3, 6, 15, 43, 51, 60, 66, 67, 121, 150 ; priorité réservation > classe > scope > global | CDC §04, PM §5 | MSG-0001 | ADR-0005 | REC-04 | analysée |
| REQ-022 | TFTP RFC 1350 + blksize/timeout/tsize, racine confinée, écriture désactivée par défaut, anti-traversée, limites, journal, arrêt immédiat | CDC §05, PM §6 | MSG-0001 | ADR-0008 | REC-05 | analysée |
| REQ-023 | UX/UI : FR/EN, HiDPI, thèmes clair/sombre/auto, couleurs terminal indépendantes, onglets, split, panneaux, barre d'état, préférences persistantes | CDC §02, PM §7 | MSG-0001 | ADR-0001 | REC-06 | analysée |
| REQ-024 | F12 : modale de diagnostic (logs, filtres, état modules, erreurs, export expurgé) + panneau Auto-tests non destructifs ; combo alternative pour envoyer F12 à la session | CDC §02/10, PM §7/10 | MSG-0001 | — | REC-06 | analysée |
| REQ-025 | Packaging : installateur Windows x64 .exe (NSIS) et paquet amd64 .deb via cargo-packager ; cibles Windows 11, Debian 13, Ubuntu 24.04 | CDC §07, PM §1 | MSG-0001 | — | REC-07 | analysée |
| REQ-026 | CI Linux+Windows : fmt, clippy -D warnings, tests, build release --locked, audit, deny, SBOM, proptest, fuzz parseurs | CDC §07/10, PM §8 | MSG-0001 | — | à créer (J0+) | analysée |
| REQ-027 | Traçabilité : MESSAGES, OC, DECISIONS, JALONS, TODO, BUGS, TESTS*, REQUIREMENTS, AVANCEMENT maintenus ; MSG id par demande | CDC §09, PM §9 | MSG-0001 | ADR-0010 | TEST-TRACE-001 | implémentée (initial) |
| REQ-028 | Tests : unitaires, intégration, E2E, sécurité, perf, recette utilisateur par jalon (TESTS_MANUELS.md), auto-tests F12 | CDC §10, PM §10 | MSG-0001 | — | à créer | analysée |
| REQ-029 | Sécurité : aucun secret en logs/args/télémétrie/F12 ; IPC authentifié ; privilèges minimaux ; UI jamais admin par défaut ; vérification clés d'hôt/certs non affaiblie | CDC §06, PM §1/3 | MSG-0001 | ADR-0007 | à créer | analysée |
| REQ-030 | Macros, commandes favorites, multi-exécution explicite (liste cibles, désactivation immédiate) | CDC §02, PM §2 | MSG-0001 | — | à créer (J5) | analysée |

Hors périmètre initial (décision explicite du CDC) : DHCPv6 (extension future).
À inventorier avec l'utilisateur : fonctions réellement utilisées des produits
de référence (matrice de parité détaillée, J0).
