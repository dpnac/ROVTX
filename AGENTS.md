# AGENTS.md — Règles permanentes pour tout intervenant

## Avant toute intervention
1. Lire `OC.md`, `AVANCEMENT.md`, `JALONS.md`, les dernières entrées de
   `MESSAGES.md`, `DECISIONS.md`, `TODO.md`, `BUGS.md`, `TESTS_RESULTS.md`,
   puis les sections du cahier des charges (`docs/inputs/`) utiles au sujet.
2. Vérifier `git status`, le dernier commit et les changements non committés.
3. Ne jamais écraser le travail existant ni recommencer le projet sans raison
   documentée et validée.
4. Attribuer un identifiant `MSG-XXXX` croissant à chaque message utilisateur
   relatif au projet et le journaliser dans `MESSAGES.md` **avant** de coder.

## Pendant
- Respecter les ADR de `DECISIONS.md` ; toute nouvelle décision y est ajoutée
  (proposée puis validée), jamais imposée silencieusement.
- Mettre à jour `REQUIREMENTS.md` pour toute nouvelle exigence
  (REQ → MSG → TASK/ADR → TEST → état).
- Code Rust : édition 2024, workspace, clippy propre (`-D warnings`),
  `unsafe` confiné/justifié/testé, thread graphique jamais bloqué, canaux
  bornés et annulation explicite.
- Aucun secret (mot de passe, clé, jeton) dans un fichier de suivi, un log,
  un commit, un rapport ou une capture.

## Après
- Mettre à jour `MESSAGES.md` (bilan réel), `AVANCEMENT.md`, `TODO.md`,
  `TESTS_RESULTS.md` (commandes réellement exécutées, codes retour, statuts
  PASS/FAIL/BLOCKED/NOT_RUN/SKIPPED), `BUGS.md` le cas échéant, `CHANGELOG.md`.
- Une commande non exécutée n'est jamais « PASS » ; un build réussi ne valide
  pas un protocole ; ne jamais prétendre avoir compilé/testé/livré sans preuve.
- Fournir un bilan de fin d'intervention : MSG, demandes traitées,
  modifications, fichiers, commandes et résultats, limites, tests manuels,
  statut du jalon, commit réel, prochaine action.

## Interdits
- Neutraliser un test, masquer une erreur, modifier des critères pour obtenir
  un succès artificiel.
- Démarrer DHCP/TFTP, scanner ou accéder à distance sans consentement explicite.
- Tester DHCP sur un réseau de production.
- Présenter un placeholder comme fonctionnel ; annoncer une parité sans preuve.
