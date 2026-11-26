# MESSAGES.md — Journal chronologique des échanges (append-only, versionné)

Règles : une entrée par message utilisateur relatif au projet (MSG-XXXX croissant) ;
date ISO 8601 ; demande expurgée ; contexte ; exigences REQ ; priorité ; jalon ;
critères de réussite ; statut ; ambiguïtés ; puis bilan après intervention
(fichiers touchés, décisions, tests réellement exécutés, commandes, résultats,
commit, blocages, prochaine action). Ne jamais effacer ni réécrire une entrée :
corriger par une entrée liée. Aucun secret, aucun raisonnement privé.

Statuts possibles : reçue, analysée, en cours, bloquée, implémentée, testée,
en attente de recette, validée, rejetée/annulée.

---

## MSG-0001 — 2026-11-26T21:27:22Z

- **Demande** : démarrage du projet « Gestionnaire de connexions et outils réseau »
  à partir du prompt maître v1.2 et du cahier des charges v1.2 (pièces jointes),
  avec le schéma d'architecture cible. Première réponse attendue : périmètre
  consolidé, matrice de parité initiale, décisions techniques, risques et
  prototypes prioritaires, arborescence, jalons avec critères de recette,
  questions bloquantes ; initialisation des fichiers de suivi avant J0.
- **Exigences** : ensemble de la matrice REQUIREMENTS.md (REQ-001 à REQ-030).
- **Priorité** : maximale. **Jalon** : pré-J0 (cadrage).
- **Critères de réussite** : documents de suivi créés et versionnés ; cadrage
  complet remis ; aucune implémentation fonctionnelle non autorisée ; aucun
  résultat inventé.
- **Statut** : analysée → implémentée (suivi initialisé ; cadrage remis).
- **Ambiguïtés relevées** :
  1. Éditions/versions exactes des produits de référence (PuTTYCM, MobaXterm)
     non fournies : inventaire à faire avec l'utilisateur (matrice de parité).
  2. DHCPv6 = extension distincte, non exigée initialement (confirmé CDC §04).
  3. VNC, Mosh, NFS, X11/XDMCP, capture pcap : bibliothèques Rust à évaluer en
     J0, intégration soumise à validation.
  4. « Helpers privilégiés » optionnels et désactivés par défaut : périmètre
     exact à confirmer (bind < 1024 sous Linux uniquement ?).
- **Intervention** : lecture intégrale du prompt maître (98 lignes) et du CDC
  (11 pages) ; vérification environnement ; création des fichiers de suivi ;
  copie des pièces jointes dans `docs/inputs/` ; commit initial de suivi.
- **Fichiers touchés** : README.md, AGENTS.md, OC.md, ARCHITECTURE.md,
  AVANCEMENT.md, DESIGN_SYSTEM.md, FUNCTIONS.md, TESTS.md, TESTS_RESULTS.md,
  TESTS_MANUELS.md, CHANGELOG.md, SECURITY.md, LICENSES.md, MESSAGES.md,
  DECISIONS.md, JALONS.md, TODO.md, BUGS.md, REQUIREMENTS.md, docs/inputs/*.
- **Commandes exécutées** : `cargo --version` → 1.100.0 ; `rustc --version` →
  1.100.0 ; `curl -o /dev/null -w %{http_code} https://static.crates.io/…` →
  200 ; `curl … https://index.crates.io/config.json` → 200 ;
  `pkg-config --list-all | grep …` → sqlite3, openssl, xcb, wayland, fontconfig
  présents ; libpcap absent.
- **Résultats** : environnement Linux de build opérationnel ; registre crates.io
  joignable via proxy ; pas de toolchain Rust épinglée encore créée (J0).
- **Blocages** : développement J0 en attente d'autorisation explicite de
  l'utilisateur (conformément au prompt maître).
- **Prochaine action** : réponse de cadrage au user ; sur autorisation, lancer
  J0 (prototypes + verrouillage dépendances).
