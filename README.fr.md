<p align="center"><img src="docs/banner-zoom-veille.png" alt="zoom-veille" width="100%"></p>

<p align="center"><img src="https://img.shields.io/badge/platform-Linux%20·%20KDE%20Plasma%20Wayland-000000?style=flat-square" alt="platform"> <img src="https://img.shields.io/badge/rust-2021-dea584?style=flat-square" alt="rust"> <img src="https://img.shields.io/badge/license-MIT-610089?style=flat-square" alt="license"></p>

<p align="center"><a href="README.md">🇬🇧 English version</a></p>

Ferme Zoom tout seul quand il traîne dans la zone de notification sans servir à rien.

Le client Zoom pour Linux consomme 2 à 10 % de CPU en permanence même réduit dans le tray, ce qui se voit sur l'autonomie d'un portable. Ce petit service le ferme au bout de 5 minutes s'il n'est vraiment pas utilisé.

## Quand Zoom est fermé

Seulement si, pendant 5 minutes d'affilée :

- aucune fenêtre Zoom n'existe (même réduite : une salle d'attente réduite est donc protégée) ;
- aucun flux audio Zoom n'est actif dans PipeWire ;
- la caméra n'est pas ouverte par Zoom.

Une notification « Zoom fermé » s'affiche quand ça arrive. Si KWin ne répond pas, le service ne ferme rien par sécurité.

## Comment il voit les fenêtres

Pas d'accès X11 sous Wayland, donc il passe par le runner « fenêtres » de KWin (celui de KRunner) via D-Bus, puis `getWindowInfo` pour vérifier la classe de chaque fenêtre. Ça évite qu'un onglet « Zoom pricing » dans le navigateur compte comme une fenêtre Zoom.

## Installation

```bash
./install.sh
```

Il faut Rust (`sudo pacman -S rust`) : le script compile le binaire et le met dans `~/.local/bin`. La dernière ligne doit afficher `active`. Si l'ancienne version Python était installée, elle est remplacée.

Réglages en haut de `src/main.rs` : `DELAY` (5 min), `STEP_ZOOM` (30 s), `STEP_IDLE` (60 s). Pour essayer sans attendre 5 minutes : `ZOOM_VEILLE_DELAI=20 target/release/zoom-veille`.

Désactiver : `systemctl --user disable --now zoom-veille.service`

## Consommation

Quand Zoom n'est pas lancé, le service jette juste un œil à `/proc` une fois par minute (avec 5 s de tolérance sur le réveil, pour que le noyau le groupe avec d'autres) : 0 % de CPU et moins de 3 Mo de RAM, contre une quinzaine pour l'ancienne version Python. Les vérifications PipeWire / KWin ne tournent que si Zoom est ouvert, toutes les 30 s.

Testé sur KDE Plasma 6 (Wayland). Une fois Zoom fermé, ses notifications (chat, invitations) n'arrivent plus jusqu'à ce qu'on le rouvre.
