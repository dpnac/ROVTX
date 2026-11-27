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

---

## MSG-0002 — 2026-11-26T21:41:03Z

- **Demande** : (1) l'icône du projet est `icon.png`, à utiliser pour les
  exécutables (packaging) ; (2) autorisation de commencer J0.
- **Exigences** : REQ-025 (packaging, icône), ensemble J0 (TASK-0101 à 0109).
- **Priorité** : maximale. **Jalon** : J0.
- **Critères de réussite** : workspace Cargo compilable (build + clippy + fmt +
  tests verts) ; prototypes DHCP options, TFTP, coffre, SSH, terminal,
  confinement RDPDR démontrés par tests réellement exécutés ; évaluations
  VNC/Mosh/NFS/pcap documentées ; versions verrouillées dans Cargo.lock.
- **Statut** : en cours.
- **Ambiguïtés** : fichier `icon.png` non trouvé dans le dépôt à la réception
  du message → à fournir par l'utilisateur ou à générer (décision ADR-0011) ;
  l'icône n'est pas bloquante pour J0 (packaging = J7).
- **Intervention** : création du workspace (6 crates), rust-toolchain.toml
  (1.100.0), CI squelette (.github/workflows/ci.yml), copie de icon.png
  (ADR-0011), prototypes + tests : service-dhcp (options typées, priorités,
  cross-check dhcproto), service-tftp (RFC 1350 + options + confinement),
  vault (Argon2id + AES-256-GCM), protocol-ssh (russh contre OpenSSH 10.0
  local), terminal (alacritty_terminal + portable-pty, Ctrl+C transmis),
  protocol-rdp (backend RDPDR confiné, extension Rust d'ironrdp-rdpdr,
  ADR-0012) ; évaluations licences VNC/Mosh/NFS/pcap (D-ATT-01→05).
- **Fichiers touchés** : Cargo.toml, Cargo.lock, rust-toolchain.toml,
  .github/workflows/ci.yml, icon.png, crates/{vault,service-dhcp,service-tftp,
  protocol-ssh,terminal,protocol-rdp}/*, TESTS_RESULTS.md (RUN-0002),
  TESTS_MANUELS.md (TM-00-01→03), DECISIONS.md (ADR-0011/0012, D-ATT),
  TODO.md, AVANCEMENT.md, LICENSES.md à compléter en J1 (cargo deny).
- **Commandes exécutées** : voir RUN-0002 (build, 29 tests, clippy -D warnings,
  fmt --check, build release --locked : tous code 0) ; sshd de fixture démarré
  sur 127.0.0.1:2222.
- **Résultats** : 29 PASS / 0 FAIL ; aucune régression (rien de préexistant).
- **Blocages** : aucun technique ; décisions user attendues (recette J0,
  Mosh GPL, inventaire parité, coffre).
- **Statut** : implémentée, testée, **en attente de recette** (TM-00-01→03).
- **Prochaine action** : recette utilisateur J0 ; puis J1 (crate desktop egui,
  SQLite, keyring, F12).
| MSG-0003 | 2026-11-26 | utilisateur | Question : « Ou se trouve le .deb » — réponse : aucun .deb produit à ce stade, le packaging Debian est le jalon J6 (après J1 exécutable applicatif) ; état de l'outillage vérifié (dpkg-deb présent ; fakeroot/lintian/dh absents) | aucune (question d'information) |
| MSG-0004 | 2026-11-27 | utilisateur | Erreur GitHub Actions « Invalid workflow file … line 2 » (run 45945501983) : cause = ligne 1 `name = "ci"` (syntaxe non YAML). Correctif : `name: ci`, clé `on` entre guillemets, `reports/.gitkeep` (dossier vide => upload-artifact échouerait), audit `continue-on-error` en J0. Validé YAML localement (ruby/psych). Push impossible depuis la toolbox (proxy 403 github.com) : push à faire côté utilisateur | commit correctif ci.yml + reports/.gitkeep |
| MSG-0005 | 2026-11-27 | utilisateur | Demande d'explication des recettes TM-00-01/02/03. Réponse : procédure pas à pas + script `scripts/tm00-setup-sshd.sh` (fixture sshd automatisée, idempotente). Vérifié en toolbox : fixture recréée de zéro, TM-00-01 = 2 PASS, TM-00-03 = 5 PASS, TM-00-02 = 29 PASS / 0 FAIL | script + TESTS_MANUELS.md mis à jour |
| MSG-0006 | 2026-11-27 | utilisateur | Échec CI runner windows-2022 : panic aws-lc-sys 0.47.0 « NASM command not found ». Cause : feature par défaut `aws-lc-rs` de russh, dont aws-lc-sys exige NASM sous MSVC. Correctif : `russh = { default-features = false, features = ["flate2", "ring", "rsa"] }` (provider ring, mêmes algorithmes). Vérifié : aws-lc absent de Cargo.lock, 29/29 PASS (SSH réel inclus), clippy/fmt OK, release --locked OK | commit protocol-ssh/Cargo.toml + Cargo.lock |
