# TESTS_RESULTS.md — Résultats d'exécution réels

Format : RUN-XXXX | date | version/commit | OS/arch | outillage | commandes
réellement lancées | code retour | durée | totaux | résultat par test | preuve.

## RUN-0001 — 2026-11-26T21:27Z
- Version/commit : commit initial de suivi (pré-J0).
- OS/arch : Linux x86_64 (toolbox KodeOS).
- Commandes lancées :
  - `cargo --version` → code 0, sortie `cargo 1.100.0 (3acc79c0c 2026-11-05)`
  - `rustc --version` → code 0, sortie `rustc 1.100.0 (0ef1bc386 2026-11-09)`
  - `curl -sS -o /dev/null -w %{http_code} https://static.crates.io/crates/serde/serde-1.0.0.crate` → 200
  - `curl -sS -o /dev/null -w %{http_code} https://index.crates.io/config.json` → 200
  - `pkg-config --list-all | grep -Ei 'pcap|sqlite|openssl|…'` → sqlite3, openssl, xcb, wayland, fontconfig présents ; pcap absent
- Totaux tests applicatifs : 0 exécutés (aucun code encore) ; TEST-* du
  catalogue : NOT_RUN.
- Conclusion : environnement de build viable ; registre crates.io accessible.
  Aucun test fonctionnel n'a été exécuté à ce stade.
