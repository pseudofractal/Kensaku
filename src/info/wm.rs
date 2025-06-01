use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process::Command;

pub fn get_window_manager() -> String {
    let de = env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| env::var("XDG_DESKTOP_SESSION"))
        .or_else(|_| env::var("DESKTOP_SESSION"))
        .ok();

    if let Some(de_name) = de {
        return de_name;
    }

    let wm_from_xinitrc = dirs::home_dir()
    .and_then(|home| {
        let path = home.join(".xinitrc");
        File::open(path).ok().and_then(|file| {
            let reader = BufReader::new(file);
            let lines = reader.lines().collect::<Result<Vec<_>, _>>().ok()?;
            lines.last().cloned()
        })
    })
    .and_then(|line| line.split_whitespace().last().map(|s| s.to_string()));

    if let Some(wm) = wm_from_xinitrc {
        return wm;
    }

    let known_wms = [
        "hyprland", "sway", "i3", "bspwm", "openbox", "fluxbox",
        "xmonad", "herbstluftwm", "awesome", "kwin", "mutter", "marco",
    ];

    if let Ok(output) = Command::new("ps").arg("axo").arg("comm").output() {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            for line in stdout.lines() {
                let proc_name = line.trim().to_lowercase();
                if known_wms.contains(&proc_name.as_str()) {
                    return proc_name;
                }
            }
        }
    }

    "Unknown".to_string()
}

