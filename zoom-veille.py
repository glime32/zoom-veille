#!/usr/bin/env python3
# zoom-veille : ferme Zoom quand il traîne dans le tray pour rien.
#
# Zoom bouffe 2 à 10 % de CPU même réduit dans le tray. Ce script le ferme
# s'il n'y a ni fenêtre Zoom (même réduite), ni son, ni caméra depuis DELAI.
# Quand Zoom n'est pas lancé, il se contente de lire /proc toutes les 30 s.
import os, re, json, time, signal, shutil, subprocess

DELAI = 5 * 60   # secondes d'inactivité avant de fermer
PAS = 30         # intervalle entre deux vérifs
NOMS = {"zoom", "zoomlauncher", "zoomwebviewhost", "aomhost", "zopen"}


def log(msg):
    print(time.strftime("%H:%M:%S ") + msg, flush=True)


def lancer(cmd):
    try:
        r = subprocess.run(cmd, capture_output=True, text=True, timeout=10)
        return r.stdout if r.returncode == 0 else None
    except (OSError, subprocess.TimeoutExpired):
        return None


def zoom_pids():
    res = {}
    for d in os.listdir("/proc"):
        if not d.isdigit():
            continue
        try:
            with open("/proc/" + d + "/cmdline", "rb") as f:
                exe = f.read().split(b"\0")[0].decode(errors="ignore")
        except OSError:
            continue
        base = os.path.basename(exe).lower()
        if base in NOMS and "zoom" in exe.lower():
            res[int(d)] = base
    return res


def en_appel(pids):
    # un flux audio Zoom actif dans PipeWire = réunion en cours
    ids = {str(p) for p in pids}
    out = lancer(["pw-dump"])
    if out:
        try:
            for o in json.loads(out):
                info = o.get("info") or {}
                p = info.get("props") or {}
                if not str(o.get("type", "")).endswith("Node") or not str(p.get("media.class", "")).startswith("Stream/"):
                    continue
                qui = " ".join(str(p.get(k, "")) for k in ("application.name", "application.process.binary", "node.name")).lower()
                if info.get("state") == "running" and ("zoom" in qui or str(p.get("application.process.id", "")) in ids):
                    return True
        except (ValueError, AttributeError):
            pass
    # caméra ouverte par un processus Zoom
    for pid in pids:
        try:
            for fd in os.listdir("/proc/%d/fd" % pid):
                if os.readlink("/proc/%d/fd/%s" % (pid, fd)).startswith("/dev/video"):
                    return True
        except OSError:
            pass
    return False


kwin_ok = True


def fenetre_ouverte():
    # On passe par le runner "fenêtres" de KWin (celui de KRunner) pour
    # lister les fenêtres, puis getWindowInfo pour vérifier la classe.
    global kwin_ok
    out = lancer(["busctl", "--user", "--json=short", "call", "org.kde.KWin",
                  "/WindowsRunner", "org.kde.krunner1", "Match", "s", "zoom"])
    if out is None:
        # dans le doute on ne ferme rien
        if kwin_ok:
            log("KWin ne répond pas, Zoom ne sera pas fermé")
            kwin_ok = False
        return True
    kwin_ok = True
    for uuid in set(re.findall(r"\d+_(\{[0-9a-fA-F-]{36}\})", out)):
        try:
            d = json.loads(lancer(["busctl", "--user", "--json=short", "call", "org.kde.KWin",
                                   "/KWin", "org.kde.KWin", "getWindowInfo", "s", uuid]))["data"][0]
            v = {k: x.get("data") for k, x in d.items()}
        except (TypeError, ValueError, KeyError, IndexError, AttributeError):
            return True
        # le titre "Zoom pricing - Helium" matche aussi, d'où le test sur la classe
        if "zoom" not in (str(v.get("resourceClass", "")) + " " + str(v.get("desktopFile", ""))).lower():
            continue
        # les petites fenêtres cachées de Zoom (popups, tray) ne comptent pas
        if v.get("skipTaskbar") or (v.get("width") or 0) < 100 or (v.get("height") or 0) < 100:
            continue
        return True
    return False


def fermer():
    for sig in (signal.SIGTERM, signal.SIGKILL):
        for pid in zoom_pids():
            try:
                os.kill(pid, sig)
            except OSError:
                pass
        for _ in range(20):
            if not zoom_pids():
                return
            time.sleep(0.5)


log("surveillance de Zoom démarrée")
depuis = None
while True:
    pids = zoom_pids()
    if "zoom" not in pids.values() or en_appel(pids) or fenetre_ouverte():
        depuis = None
    elif depuis is None:
        depuis = time.time()
        log("Zoom seul dans le tray, fermeture dans %d min si ça ne bouge pas" % (DELAI // 60))
    elif time.time() - depuis >= DELAI:
        fermer()
        depuis = None
        log("Zoom fermé")
        if shutil.which("notify-send"):
            lancer(["notify-send", "-a", "Zoom", "Zoom fermé",
                    "Il tournait dans le tray sans servir depuis %d min." % (DELAI // 60)])
    time.sleep(PAS)
