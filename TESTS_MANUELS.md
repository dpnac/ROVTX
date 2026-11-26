# TESTS_MANUELS.md — Recettes utilisateur par jalon

Format : TM-XX-NN | plateforme | prérequis | étapes exactes | attendu |
observé (à renseigner par l'utilisateur) | preuve facultative | statut.
L'utilisateur répond par ID : OK / KO + description + capture expurgée.
Un test non retourné n'est pas considéré réussi.

## Pré-J0 / J0
Aucune recette fonctionnelle : rien n'est encore exécutable. Recettes J0 à
ajouter avec chaque prototype (commandes de démonstration reproductibles).

## J1 (à compléter à l'ouverture du jalon)
- TM-01-01 | Win+Linux | build J1 | lancer l'app, basculer clair/sombre/auto,
  fermer/rouvrir | préférences restaurées | observé : — | statut : NOT_RUN
- TM-01-02 | Win+Linux | build J1 | F12 | modale de diagnostic, export sans
  secret | observé : — | statut : NOT_RUN

## J2 et suivants
Recettes SSH/SFTP (TM-02-*), RDP confinement (TM-03-*), DHCP laboratoire isolé
(TM-04-*), TFTP (TM-04-*), installateurs VM propres (TM-07-*) : créées à
l'ouverture de chaque jalon, reprises de REC-01 à REC-07 du cahier des charges.

## J0 — recettes des prototypes (à rejouer par l'utilisateur sur sa machine)
- TM-00-01 | Linux | dépôt + cargo | Prérequis sshd de test :
  `mkdir -p /tmp/sshd-test && ssh-keygen -q -t ed25519 -f /tmp/sshd-test/host_ed25519 -N ""`
  puis config minimale (Port 2222, ListenAddress 127.0.0.1, UsePAM no,
  Subsystem sftp) et `sshd -f …` ; clé client dans authorized_keys ;
  fixture `/tmp/sshd-test/known_fingerprints.txt` = sorties de
  `ssh-keygen -lf host_*.pub` préfixées « 127.0.0.1 ».
  Étapes : `cargo test -p protocol-ssh` | attendu : 2 PASS | observé (agent) :
  2 PASS | statut : OK agent, en attente recette utilisateur.
- TM-00-02 | Linux/Win | dépôt + cargo | `cargo test --workspace` |
  attendu : 29 PASS, 0 FAIL | observé (agent) : 29/0 | statut : en attente
  recette utilisateur.
- TM-00-03 | Linux | dépôt + cargo | `cargo test -p service-tftp` |
  attendu : 5 PASS dont refus écriture par défaut et anti-traversée |
  observé (agent) : 5 PASS | statut : en attente recette utilisateur.
