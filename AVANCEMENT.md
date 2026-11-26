# AVANCEMENT.md — Point de reprise autonome

- **Version** : 0.0.0 (pré-J0)
- **Branche / dernier commit** : voir `git log -1` (commit initial de suivi)
- **Jalon courant** : pré-J0 — cadrage ; J0 non autorisé encore
- **État réel** : aucun code applicatif ; documents de suivi initialisés ;
  environnement de build vérifié (cargo 1.100.0, crates.io joignable).

## Avancement par exigence
Aucune exigence implémentée. Matrice complète : `REQUIREMENTS.md`
(REQ-001 à REQ-030, toutes « analysée » sauf REQ-027 « implémentée (initial) »
pour la traçabilité elle-même).

## Modules opérationnels / non validés
- Opérationnels : néant.
- Non validés : tous (prototypes J0 à réaliser : IronRDP/RDPDR, russh/SFTP,
  terminal, DHCP options, TFTP, coffre, VNC/Mosh/NFS/X11/pcap).

## Dernières demandes
- MSG-0001 (2026-11-26) : démarrage du projet, cadrage complet, initialisation
  du suivi. Statut : analysée → implémentée pour le suivi ; cadrage remis.

## Tests rouges / non exécutés
- Néant (aucun test écrit encore) ; tout reste NOT_RUN par définition.

## Obstacles
- Aucun bloquant technique identifié ; libpcap-dev absent de la toolbox
  (installation possible via kodeos-install le moment venu).
- Décisions en attente : D-ATT-01 à D-ATT-08 (`DECISIONS.md`).

## Décisions en attente de l'utilisateur
1. Autorisation de lancer J0 (prototypes + workspace + CI squelette).
2. Inventaire des fonctions/éditions de référence (matrice de parité).
3. Choix coffre : mot de passe maître systématique ou trousseau système par
   défaut (les deux restent supportés).

## Prochaines actions ordonnées
1. Réponse de cadrage (périmètre, matrice, risques, jalons, questions).
2. Sur autorisation : TASK-0101 (workspace + toolchain + CI squelette).
3. Prototypes J0 par priorité de risque : TASK-0102 (RDP/RDPDR) puis 0103,
   0104, 0105, 0106, 0107, 0108, 0109.
