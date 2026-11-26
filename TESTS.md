# TESTS.md — Catalogue de tests (ID stables, traçables)

Format : TEST-XXX-NNN | REQ/MSG/BUG | module | type | plateformes |
préconditions | données | procédure/commande | attendu | bloquant ?

Statuts d'exécution dans `TESTS_RESULTS.md` : PASS, FAIL, BLOCKED, NOT_RUN,
SKIPPED (motif). Une commande non exécutée n'est jamais PASS.

## Catalogue initial (à implémenter avec le code correspondant)
| ID | REQ | Module | Type | Attendu | Bloquant |
|---|---|---|---|---|---|
| TEST-DHCP-001 | REQ-021 | service-dhcp | unitaire | option 43 `01:04:C0:A8:32:0A` → octets `01 04 C0 A8 32 0A` | oui |
| TEST-DHCP-002 | REQ-021 | service-dhcp | unitaire | option 66 texte `192.168.50.10` → ASCII (pas hex) | oui |
| TEST-DHCP-003 | REQ-021 | service-dhcp | unitaire | option 67 `bootx64.efi` → ASCII | oui |
| TEST-DHCP-004 | REQ-021 | service-dhcp | unitaire | option 150 liste IPv4 → `C0 A8 32 0A` | oui |
| TEST-DHCP-005 | REQ-021 | service-dhcp | unitaire | option 121 routes structurées (préfixe/dest/passerelle) | oui |
| TEST-DHCP-006 | REQ-020 | service-dhcp | intégration | DORA complet sur interface loopback isolée, bail persistant | oui |
| TEST-DHCP-007 | REQ-020 | service-dhcp | sécurité | aucune réponse hors interface sélectionnée | oui |
| TEST-TFTP-001 | REQ-022 | service-tftp | intégration | GET fichier OK, blksize/tsize négociés | oui |
| TEST-TFTP-002 | REQ-022 | service-tftp | sécurité | traversée `../` refusée ; symlink hors racine refusé | oui |
| TEST-TFTP-003 | REQ-022 | service-tftp | sécurité | PUT refusé si écriture désactivée (défaut) | oui |
| TEST-VAULT-001 | REQ-019 | vault | unitaire | round-trip Argon2id + AES-256-GCM ; zéro secret en clair en mémoire après zeroize | oui |
| TEST-SSH-001 | REQ-001 | protocol-ssh | intégration | auth clé + KbdInt contre serveur de test local | oui |
| TEST-RDP-001 | REQ-006 | protocol-rdp | intégration | dossier local nommé visible en `\\tsclient\<nom>`, lecture/écriture | oui |
| TEST-RDP-002 | REQ-006 | protocol-rdp | sécurité | accès hors dossiers sélectionnés refusé | oui |
| TEST-TERM-001 | REQ-011 | terminal | intégration | Ctrl+C envoyé au processus, pas intercepté par l'UI | oui |
| TEST-TRACE-001 | REQ-027 | docs | manuel agent | chaque MSG a une entrée journalisée avant/après intervention | oui |

Les tests réseau privilégiés ou matériels s'exécutent en laboratoire isolé ou
VM dédiées, jamais en production.
