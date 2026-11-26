# DESIGN_SYSTEM.md — Principes d'interface (original, sans identité empruntée)

## Principes
- Design original : aucune reprise de l'identité graphique de PuTTYCM,
  MobaXterm ou d'autres produits.
- Fluide et sobre : hiérarchie claire (arbre à gauche, onglets au centre,
  panneaux outils/SFTP à droite, barre d'état en bas), densité adaptée à
  l'administration (tableaux, formulaires compacts mais lisibles).
- Accessible : navigation clavier complète, contrastes AA, tailles lisibles,
  HiDPI, focus visibles, libellés FR/EN.

## Structure type d'écran
```text
┌ Barre d'outils : connexions, outils, serveurs, préférences, recherche
├ Arborescence repliable │ Onglets (terminal/RDP/SFTP/outils) │ Panneau latéral
└ Barre d'état : état session, protocole, encodage, services, F12
```

## Thèmes
- Clair / sombre / automatique (suit l'OS) ; couleurs du terminal indépendantes
  (profils : fond, encre, ANSI 16, 24 bits, curseur, sélection).
- Jetons : couleurs sémantiques (succès, avertissement, erreur, info),
  espacements sur grille 4 px, rayons discrets, ombres minimales.

## Composants clés
- Arbre de connexions : DnD, favoris, tags, recherche instantanée.
- Onglets : réordonnables, détachables, indicateurs d'état (connecté, erreur,
  transfert), split horizontal/vertical.
- Fiches de session : sections protocole / authentification / tunnels /
  ressources locales (RDP) / post-commandes / options terminal.
- Éditeur d'options DHCP : tableau typé + aperçu hexadécimal des octets.
- Modale F12 : journal filtrable, niveaux, état des modules, auto-tests,
  export expurgé.

## Règles d'interaction
- Sélection = copie et clic droit = collage : deux réglages indépendants,
  surcharge par session ; avertissement avant collage multiligne.
- Raccourcis configurables ; Ctrl+Shift+C/V par défaut sans casser Ctrl+C du
  terminal ; combinaison alternative pour transmettre F12 à la session.
- Confirmations explicites : commandes automatiques dangereuses, démarrage
  DHCP/TFTP, écriture TFTP, multi-exécution (liste des cibles affichée).
- RDP/VNC : interactions natives conservées par défaut.

## États et retours utilisateur
- Tout service : arrêté / démarrage / actif / erreur, avec journal accessible.
- Transferts : progression, vitesse, annulation, reprise si protocole capable.
- Erreurs : message actionnable + conseil correctif + référence (REQ/TEST).
