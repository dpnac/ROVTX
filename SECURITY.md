# SECURITY.md — Politique de sécurité du projet

## Principes
- Aucun secret (mot de passe, clé privée, jeton, master key) dans : logs,
  arguments de processus, télémétrie, rapports F12, fichiers de suivi, commits,
  captures d'écran, exports par défaut.
- Coffre : maître de clé aléatoire 256 bits ; enveloppe AES-256-GCM
  (ou XChaCha20-Poly1305) ; dérivation Argon2id si mot de passe maître ;
  `secrecy`/`zeroize` pour les buffers sensibles ; verrouillage manuel et
  automatique ; export sans secrets par défaut, export chiffré sur demande.
- Trousseau système (keyring) supporté en alternative/complément.
- Vérification des clés d'hôte SSH (known_hosts) et des certificats TLS/RDP
  jamais affaiblie ; protocoles historiques (Telnet, Rlogin/Rsh, TFTP, FTP)
  disponibles mais signalés comme non chiffrés.
- Privilèges minimaux : UI jamais en administrateur/root par défaut ; helper
  privilégié optionnel, désactivé par défaut, IPC authentifié, surface minimale.
- Services réseau : DHCP/TFTP/serveurs arrêtés par défaut ; démarrage explicite
  avec confirmation ; interface obligatoire pour DHCP ; écriture TFTP désactivée
  par défaut ; racine TFTP confinée (anti-traversée, anti-symlinks) ; scans et
  serveurs historiques réservés aux réseaux autorisés.
- RDPDR : redirection limitée aux dossiers sélectionnés, nommés ; aucun partage
  de disque entier par défaut ; preuve d'absence d'évasion requise (REQ-006).

## Pratiques de développement
- `cargo audit` et `cargo deny` en CI ; SBOM ; dépendances auditées (licences
  et provenance) avant adoption ; FFI/`unsafe` justifiés, confinés, testés.
- Fuzzing des parseurs réseau (cargo fuzz) et tests de propriété (proptest).
- Reproduction des bugs par test échouant avant correctif ; test de
  non-régression conservé.
- Signatures de release et SHA-256 lorsque les certificats sont disponibles ;
  procédure de rollback et sauvegarde avant migration.

## Signalement
Toute vulnérabilité constatée : entrée BUG-XXXX avec sévérité, reproduction,
correctif et test de non-régression ; pas de divulgation de secret dans le
journal.
