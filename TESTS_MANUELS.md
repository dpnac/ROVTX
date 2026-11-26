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
