#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
install -Dm755 zoom-veille.py ~/.local/bin/zoom-veille.py
install -Dm644 zoom-veille.service ~/.config/systemd/user/zoom-veille.service
systemctl --user daemon-reload
systemctl --user enable --now zoom-veille.service
systemctl --user restart zoom-veille.service
systemctl --user is-active zoom-veille.service
