use std::process::Command;

pub fn get_package_count() -> Option<String> {
    let native_managers = [
        ("pacman", &["-Q"][..], 0),
    ];

    let mut native_count = 0;
    for (cmd, args, skip_lines) in native_managers.iter() {
        if Command::new("which").arg(cmd).output().ok()?.status.success() {
            let output = Command::new(cmd).args(*args).output().ok()?;
            let stdout = String::from_utf8_lossy(&output.stdout);

            native_count = match *cmd {
                "zypper" => stdout.lines().filter(|l| l.contains("| i ")).count(),
                _ => stdout.lines().skip(*skip_lines).filter(|l| !l.trim().is_empty()).count(),
            };
            break;
        }
    }

    let flatpak_count = if Command::new("which").arg("flatpak").output().ok()?.status.success() {
        let output = Command::new("flatpak").arg("list").output().ok()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout.lines().filter(|l| !l.trim().is_empty()).count()
    } else {
        0
    };

    Some(format!("{} (native), {} (flatpak)", native_count, flatpak_count))
}

