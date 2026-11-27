#!/usr/bin/env bash
# TM-00-01 — prépare la fixture sshd locale (127.0.0.1:2222) exigée par
# `cargo test -p protocol-ssh` (TEST-SSH-001 / TEST-SFTP-001).
#
# Usage :  bash scripts/tm00-setup-sshd.sh
# Stop  :  kill "$(cat /tmp/sshd-test/sshd.pid)"
#
# Idempotent : relançable sans risque. Ne touche qu'à /tmp/sshd-test et à
# ~/.ssh/authorized_keys (ajout de la clé de test uniquement, jamais de suppression).
# Prérequis : Linux, openssh-server installé (Debian/Ubuntu : sudo apt install openssh-server).
# Pas besoin de root : port 2222 (>1024) et sshd lancé sous votre utilisateur.
set -euo pipefail

D=/tmp/sshd-test
mkdir -p "$D"

# 1. Clés (hôte + client), si absentes.
[ -f "$D/host_ed25519" ]   || ssh-keygen -q -t ed25519 -f "$D/host_ed25519" -N ""
[ -f "$D/host_rsa" ]       || ssh-keygen -q -t rsa -b 3072 -f "$D/host_rsa" -N ""
[ -f "$D/client_ed25519" ] || ssh-keygen -q -t ed25519 -f "$D/client_ed25519" -N ""

# 2. Configuration sshd isolée : écoute uniquement sur 127.0.0.1:2222.
cat > "$D/sshd_config" <<EOF
Port 2222
ListenAddress 127.0.0.1
HostKey $D/host_ed25519
HostKey $D/host_rsa
PidFile $D/sshd.pid
UsePAM no
PasswordAuthentication no
PermitRootLogin no
Subsystem sftp internal-sftp
EOF

# 3. Clé client autorisée pour VOTRE utilisateur courant (le test se connecte en $USER).
mkdir -p ~/.ssh && chmod 700 ~/.ssh
touch ~/.ssh/authorized_keys && chmod 600 ~/.ssh/authorized_keys
PUB="$(cat "$D/client_ed25519.pub")"
grep -qxF "$PUB" ~/.ssh/authorized_keys || echo "$PUB" >> ~/.ssh/authorized_keys

# 4. Fixture known_hosts attendue par les tests : empreintes SHA256 des clés d'hôte.
: > "$D/known_fingerprints.txt"
for k in "$D"/host_*.pub; do
  echo "127.0.0.1 $(ssh-keygen -lf "$k" | awk '{print $2}')" >> "$D/known_fingerprints.txt"
done

# 5. Démarrage (si pas déjà actif).
SSHD="$(command -v sshd || echo /usr/sbin/sshd)"
if [ -f "$D/sshd.pid" ] && kill -0 "$(cat "$D/sshd.pid")" 2>/dev/null; then
  echo "sshd déjà actif (pid $(cat "$D/sshd.pid"))"
else
  "$SSHD" -f "$D/sshd_config" -E "$D/sshd.log"
  sleep 1
  echo "sshd démarré (pid $(cat "$D/sshd.pid"))"
fi
grep -m1 "Server listening" "$D/sshd.log" || true
echo "Fixture prête. Lancez : cargo test -p protocol-ssh"
