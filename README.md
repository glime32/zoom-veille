<p align="center"><img src="docs/banner-zoom-veille.png" alt="zoom-veille" width="100%"></p>

<p align="center"><img src="https://img.shields.io/badge/platform-Linux%20·%20KDE%20Plasma%20Wayland-000000?style=flat-square" alt="platform"> <img src="https://img.shields.io/badge/rust-2021-dea584?style=flat-square" alt="rust"> <img src="https://img.shields.io/badge/license-MIT-610089?style=flat-square" alt="license"></p>

<p align="center"><a href="README.fr.md">🇫🇷 Version française</a></p>

**Automatically closes Zoom when it's sitting idle in the system tray.**

The Linux Zoom client uses 2–10 % CPU even when minimized to the tray, which hurts laptop battery life. This small user service closes it after 5 minutes of real inactivity.

## When Zoom gets closed

Only if, for 5 minutes in a row:

- no Zoom window exists (even minimized — a minimized waiting room is safe);
- no Zoom audio stream is active in PipeWire;
- the camera isn't open by Zoom.

You get a "Zoom closed" notification. If KWin doesn't answer, nothing is closed.

<p align="center"><img src="docs/zoom-log.png" width="760" alt="log"></p>

No X11 on Wayland, so it lists windows through KWin's KRunner "windows" runner over D-Bus, then `getWindowInfo` to check each window class (a browser tab titled "Zoom pricing" doesn't count).

## Install

```bash
git clone https://github.com/glime32/zoom-veille && cd zoom-veille && ./install.sh
```

You need Rust (`sudo pacman -S rust`); the script builds the binary into `~/.local/bin` and replaces the old Python version if it was installed. Settings at the top of `src/main.rs`: `DELAY` (5 min), `STEP_ZOOM` (30 s), `STEP_IDLE` (60 s). To try it without waiting 5 minutes: `ZOOM_VEILLE_DELAI=20 target/release/zoom-veille`. Disable with `systemctl --user disable --now zoom-veille.service`.

When Zoom isn't running, the service only looks at `/proc` once a minute (with 5 s of timer slack so the kernel can batch the wakeup): 0 % CPU and under 3 MB of RAM, versus ~15 MB for the old Python version. The PipeWire / KWin checks only run while Zoom is open, every 30 s.
