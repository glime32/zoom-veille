# zoom-veille

Ferme Zoom tout seul quand il traîne dans la zone de notification sans servir à rien.

Le client Zoom pour Linux consomme 2 à 10 % de CPU en permanence même réduit dans le tray, ce qui se voit sur l'autonomie d'un portable. Ce petit service le ferme au bout de 5 minutes s'il n'est vraiment pas utilisé.

## Quand Zoom est fermé

Seulement si, pendant 5 minutes d'affilée :

- aucune fenêtre Zoom n'existe (même réduite : une salle d'attente réduite est donc protégée) ;
- aucun flux audio Zoom n'est actif dans PipeWire ;
- la caméra n'est pas ouverte par Zoom.

Une notification « Zoom fermé » s'affiche quand ça arrive. Si KWin ne répond pas, le script ne ferme rien par sécurité.

## Comment il voit les fenêtres

Pas d'accès X11 sous Wayland, donc il passe par le runner « fenêtres » de KWin (celui de KRunner) via D-Bus, puis `getWindowInfo` pour vérifier la classe de chaque fenêtre. Ça évite qu'un onglet « Zoom pricing » dans le navigateur compte comme une fenêtre Zoom.

## Installation

```bash
./install.sh
```

La dernière ligne doit afficher `active`. Réglages (`DELAI`, `PAS`) en haut de `zoom-veille.py`.

Désactiver : `systemctl --user disable --now zoom-veille.service`

## Consommation

Quand Zoom n'est pas lancé, le script lit juste `/proc` toutes les 30 s : 0 % de CPU et une dizaine de Mo de RAM. Les vérifications PipeWire / KWin ne tournent que si Zoom est ouvert.

Testé sur KDE Plasma 6 (Wayland). Une fois Zoom fermé, ses notifications (chat, invitations) n'arrivent plus jusqu'à ce qu'on le rouvre.
