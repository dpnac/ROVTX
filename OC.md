# OC.md — Instructions de reprise (cohérent avec AGENTS.md)

## Ordre de lecture pour reprendre le projet
1. `MESSAGES.md` (dernières entrées) — où en est l'échange avec l'utilisateur.
2. `AVANCEMENT.md` — version, branche, dernier commit, jalon, état réel,
   blocages, prochaines actions ordonnées.
3. `JALONS.md` + `TODO.md` — ce qui est attendu et ce qui reste à faire.
4. `DECISIONS.md` — ADR validés vs proposés vs en attente.
5. `REQUIREMENTS.md` — matrice de traçabilité.
6. `TESTS_RESULTS.md` + `TESTS_MANUELS.md` — ce qui est prouvé, ce qui attend
   une recette utilisateur (un test manuel non retourné n'est pas réussi).
7. `docs/inputs/` — prompt maître v1.2 et cahier des charges v1.2 (sources).
8. `git log --oneline -20` et `git status` — état du dépôt.

## Reprendre le travail
- Si le contexte est insuffisant : utiliser le sommaire du CDC (page 1) et les
  fichiers de suivi comme index, puis relire les sources nécessaires.
- Continuer la prochaine action ordonnée de `AVANCEMENT.md` sauf instruction
  contraire de l'utilisateur ; journaliser le nouveau message (MSG-XXXX).
- Avant tout changement d'agent ou interruption : actualiser les fichiers de
  suivi (état, blocages, prochaine action) et committer.

## Environnement de build connu (toolbox KodeOS, 2026-11-26)
- Linux ; cargo/rustc 1.100.0 ; registre crates.io joignable via proxy HTTP.
- Présents : sqlite3, openssl, xcb, wayland, fontconfig, cmake, gcc/g++.
- Absents : libpcap-dev (à installer via `kodeos-install libpcap-dev` si besoin).
- Builds à diriger vers `/build/<nom>` (hors dépôt) ; le dépôt ne contient que
  des sources (+ node_modules/vendor si outils les y exigent).
