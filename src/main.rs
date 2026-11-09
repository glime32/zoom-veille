// zoom-veille: closes Zoom when it sits alone in the tray (no Zoom window,
// no call) for DELAY seconds. Rust port of the old Python script.
//
// Battery: when Zoom is not running, one look at /proc per minute and nothing
// else; with a 5 s timer slack the kernel can batch the wakeup with others.

use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

const DELAY: u64 = 5 * 60;
const STEP_ZOOM: u64 = 30; // Zoom running: look every 30 s
const STEP_IDLE: u64 = 60; // Zoom absent: once a minute is enough
const NAMES: [&str; 5] = ["zoom", "zoomlauncher", "zoomwebviewhost", "aomhost", "zopen"];

fn log(msg: &str) {
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    let t = unsafe { libc::time(std::ptr::null_mut()) };
    unsafe { libc::localtime_r(&t, &mut tm) };
    println!("{:02}:{:02}:{:02} {msg}", tm.tm_hour, tm.tm_min, tm.tm_sec);
}

fn run(cmd: &[&str]) -> Option<String> {
    let out = Command::new(cmd[0]).args(&cmd[1..]).stdin(Stdio::null()).stderr(Stdio::null()).output().ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

fn zoom_pids() -> HashMap<i32, String> {
    let mut found = HashMap::new();
    let Ok(dir) = fs::read_dir("/proc") else { return found };
    for entry in dir.flatten() {
        let name = entry.file_name();
        let Some(pid) = name.to_str().and_then(|s| s.parse::<i32>().ok()) else { continue };
        let Ok(cmdline) = fs::read(format!("/proc/{pid}/cmdline")) else { continue };
        let exe = String::from_utf8_lossy(cmdline.split(|&b| b == 0).next().unwrap_or_default()).to_lowercase();
        let base = exe.rsplit('/').next().unwrap_or_default().to_string();
        if NAMES.contains(&base.as_str()) && exe.contains("zoom") {
            found.insert(pid, base);
        }
    }
    found
}

// Zoom is in a call if it plays or records sound, or has the camera open
fn in_call(pids: &HashMap<i32, String>) -> bool {
    let ids: HashSet<String> = pids.keys().map(|p| p.to_string()).collect();
    if let Some(out) = run(&["pw-dump"]) {
        if let Ok(Value::Array(objects)) = serde_json::from_str::<Value>(&out) {
            for o in &objects {
                let info = &o["info"];
                let props = &info["props"];
                let s = |v: &Value| match v {
                    Value::String(s) => s.clone(),
                    Value::Null => String::new(),
                    other => other.to_string(),
                };
                if !s(&o["type"]).ends_with("Node") || !s(&props["media.class"]).starts_with("Stream/") {
                    continue;
                }
                let who = format!("{} {} {}", s(&props["application.name"]), s(&props["application.process.binary"]), s(&props["node.name"])).to_lowercase();
                if s(&info["state"]) == "running" && (who.contains("zoom") || ids.contains(&s(&props["application.process.id"]))) {
                    return true;
                }
            }
        }
    }
    pids.keys().any(|pid| {
        fs::read_dir(format!("/proc/{pid}/fd")).map_or(false, |fds| {
            fds.flatten().any(|fd| fs::read_link(fd.path()).is_ok_and(|l| l.starts_with("/dev/video")))
        })
    })
}

// window ids in KRunner's answer look like "0_{8-4-4-4-12}"
fn window_uuids(text: &str) -> HashSet<String> {
    let mut found = HashSet::new();
    let b = text.as_bytes();
    let mut i = 1;
    while i + 38 <= b.len() {
        if b[i] == b'_' && b[i + 1] == b'{' && b[i - 1].is_ascii_digit() && b[i + 38] == b'}' {
            let id = &text[i + 1..i + 39];
            if id[1..37].bytes().all(|c| c.is_ascii_hexdigit() || c == b'-') {
                found.insert(id.to_string());
            }
        }
        i += 1;
    }
    found
}

struct Kwin {
    ok: bool,
}

impl Kwin {
    // Does a Zoom window exist (even minimised)? We ask KWin.
    fn zoom_window(&mut self) -> bool {
        let Some(out) = run(&["busctl", "--user", "--json=short", "call", "org.kde.KWin", "/WindowsRunner", "org.kde.krunner1", "Match", "s", "zoom"]) else {
            if self.ok {
                log("KWin ne répond pas : Zoom ne sera pas fermé par sécurité");
                self.ok = false;
            }
            return true;
        };
        self.ok = true;
        for uuid in window_uuids(&out) {
            let info = run(&["busctl", "--user", "--json=short", "call", "org.kde.KWin", "/KWin", "org.kde.KWin", "getWindowInfo", "s", &uuid])
                .and_then(|o| serde_json::from_str::<Value>(&o).ok());
            let Some(d) = info.as_ref().and_then(|v| v["data"].get(0)).and_then(|d| d.as_object()) else {
                return true; // unreadable: assume a window, never close by mistake
            };
            let get = |k: &str| d.get(k).map(|x| &x["data"]).cloned().unwrap_or(Value::Null);
            let text = |v: Value| v.as_str().map(str::to_string).unwrap_or_default();
            let class = format!("{} {}", text(get("resourceClass")), text(get("desktopFile"))).to_lowercase();
            if !class.contains("zoom") {
                continue;
            }
            let small = |k: &str| get(k).as_f64().unwrap_or(0.0) < 100.0;
            if get("skipTaskbar").as_bool().unwrap_or(false) || small("width") || small("height") {
                continue;
            }
            return true;
        }
        false
    }
}

fn close_zoom() {
    for sig in [libc::SIGTERM, libc::SIGKILL] {
        for pid in zoom_pids().keys() {
            unsafe { libc::kill(*pid, sig) };
        }
        for _ in 0..20 {
            if zoom_pids().is_empty() {
                return;
            }
            sleep(Duration::from_millis(500));
        }
    }
}

fn main() {
    unsafe { libc::prctl(libc::PR_SET_TIMERSLACK, 5_000_000_000u64) };
    log("surveillance de Zoom démarrée");
    // ZOOM_VEILLE_DELAI (seconds) only shortens the wait for tests
    let delay = std::env::var("ZOOM_VEILLE_DELAI").ok().and_then(|v| v.parse().ok()).unwrap_or(DELAY);
    let mut kwin = Kwin { ok: true };
    let mut since: Option<Instant> = None;
    loop {
        let pids = zoom_pids();
        let running = pids.values().any(|n| n == "zoom");
        if !running || in_call(&pids) || kwin.zoom_window() {
            since = None;
        } else if let Some(t) = since {
            if t.elapsed().as_secs() >= delay {
                close_zoom();
                since = None;
                log("Zoom fermé (inutilisé)");
                let _ = run(&["notify-send", "-a", "Zoom", "Zoom fermé", &format!("Il tournait dans le tray sans servir depuis {} min.", DELAY / 60)]);
            }
        } else {
            since = Some(Instant::now());
            log(&format!("Zoom tourne seul dans le tray : fermeture dans {} min si rien ne change", DELAY / 60));
        }
        sleep(Duration::from_secs(if running { STEP_ZOOM } else { STEP_IDLE }));
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn finds_window_ids() {
        let out = r#"{"type":"a(sssida{sv})","data":[[["0_{0a1b2c3d-1111-2222-3333-444455556666}","Zoom",""],["12_{ffffffff-aaaa-bbbb-cccc-dddddddddddd}","x",""]]]}"#;
        let ids = super::window_uuids(out);
        assert!(ids.contains("{0a1b2c3d-1111-2222-3333-444455556666}"));
        assert!(ids.contains("{ffffffff-aaaa-bbbb-cccc-dddddddddddd}"));
        assert_eq!(ids.len(), 2);
    }
}
