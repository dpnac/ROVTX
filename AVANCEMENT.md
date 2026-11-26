# AVANCEMENT.md — Point de reprise autonome

- **Version** : 0.0.0 (J0 en cours)
- **Branche / dernier commit** : voir `git log -1` (commit J0 à venir)
- **Jalon courant** : J0 — prototypes ; socle compilable, 29 tests verts
- **État réel** : prototypes DHCP options, TFTP, coffre, SSH/SFTP (contre
  OpenSSH réel), terminal PTY, confinement RDPDR implémentés et testés ;
  workspace + toolchain épinglée + CI squelette en place. UI non commencée
  (J1).

## Avancement par exigence
Matrice complète : `REQUIREMENTS.md`. Synthèse J0 :
- REQ-001/012 (SSH2 clé + SFTP) : prototype testé contre OpenSSH 10.0 (état :
  testée-prototype ; MFA/agent/tunnels restants J2).
- REQ-006 (RDPDR confiné) : backend filesystem maison testé par IRP simulés
  (état : testée-prototype ; connexion réelle J3).
- REQ-011 (terminal) : émulation + PTY + Ctrl+C testés (état : testée-prototype).
- REQ-019 (coffre) : Argon2id + AES-256-GCM testés (état : testée-prototype).
- REQ-020/021 (DHCP options typées) : encodage + priorités testés, cross-check
  dhcproto (état : testée-prototype ; moteur DORA J4).
- REQ-022 (TFTP) : serveur complet testé en loopback (état : testée-prototype).
- REQ-025 (icône) : `icon.png` 512×512 versionné (ADR-0011) ; packaging J7.
- REQ-027 (traçabilité) : implémentée et appliquée.

## Modules opérationnels / non validés
- Opérationnels (prototypes testés) : service-dhcp (options), service-tftp,
  vault, terminal, protocol-ssh, protocol-rdp (confinement).
- Non validés : DORA complet + baux, RDP connexion/NLA/rendu/presse-papiers,
  VNC, Mosh (licence GPL bloquante), NFS, X11/XDMCP, pcap, UI egui, imports,
  packaging.

## Dernières demandes
- MSG-0002 (2026-11-26) : icône icon.png pour les exécutables ; lancement J0.
  Statut : implémentée (prototypes + tests), en attente de recette utilisateur
  TM-00-01→03 avant validation du jalon.

## Tests rouges / non exécutés
- Aucun test rouge. NOT_RUN : TEST-DHCP-006/007 (J4), recettes TM-00-*.

## Obstacles
- Mosh : crate GPL-3.0 → décision utilisateur requise (D-ATT-02).
- libpcap-dev absent de la toolbox (installable via kodeos-install en J5).
- openssh-server installé avec erreur postinst bénigne (chown /var/log/apt) ;
  sshd fonctionnel manuellement.

## Décisions en attente de l'utilisateur
1. Recette J0 (TM-00-01→03) → validation du jalon.
2. Mosh : binaire séparé GPL ou report (D-ATT-02).
3. Inventaire parité PuTTYCM/MobaXterm (D-ATT-08).
4. Coffre : trousseau système par défaut + mot de passe maître optionnel
   (proposition maintenue).

## Prochaines actions ordonnées
1. Recette utilisateur J0 ; corrections éventuelles.
2. J1 : crate `desktop` (egui/eframe), thèmes, arbre, SQLite, coffre UI, F12.
3. J2 : terminal egui complet, SSH/SFTP UI, tunnels, série.
