#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
command -v cargo >/dev/null || { echo "cargo introuvable : installe rust (sudo pacman -S rust)"; exit 1; }
cargo build --release --locked
install -Dm755 target/release/zoom-veille ~/.local/bin/zoom-veille
rm -f ~/.local/bin/zoom-veille.py
install -Dm644 zoom-veille.service ~/.config/systemd/user/zoom-veille.service
systemctl --user daemon-reload
systemctl --user enable --now zoom-veille.service
systemctl --user restart zoom-veille.service
systemctl --user is-active zoom-veille.service
