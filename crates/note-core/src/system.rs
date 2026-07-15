use std::{
    process::{Command, Stdio},
    str::FromStr,
};

#[derive(Debug, Clone)]
pub struct Toplevel {
    pub app_id: String,
    pub title: String,
}

pub fn run_quiet(command: &str, args: &[&str]) -> bool {
    Command::new(command)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

pub fn command_output(command: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(command).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

pub fn list_toplevels(filter: Option<&str>) -> Vec<Toplevel> {
    let mut args = vec!["toplevel", "list"];
    if let Some(filter) = filter {
        args.push(filter);
    }
    let Some(output) = command_output("wlrctl", &args) else {
        return Vec::new();
    };
    output
        .lines()
        .filter_map(|line| {
            let (app_id, title) = line.split_once(':')?;
            Some(Toplevel {
                app_id: app_id.trim().to_owned(),
                title: title.trim().to_owned(),
            })
        })
        .collect()
}

pub fn focus_toplevel(toplevel: &Toplevel) -> bool {
    run_quiet(
        "wlrctl",
        &[
            "toplevel",
            "focus",
            &format!("app_id:{}", toplevel.app_id),
            &format!("title:{}", toplevel.title),
        ],
    )
}

pub fn volume_percent() -> u8 {
    let Some(output) = command_output("wpctl", &["get-volume", "@DEFAULT_AUDIO_SINK@"]) else {
        return 0;
    };
    output
        .split_whitespace()
        .find_map(|token| f64::from_str(token).ok())
        .map(|value| (value * 100.0).round().clamp(0.0, 150.0) as u8)
        .unwrap_or(0)
}

pub fn set_volume(percent: f64) {
    let value = format!("{}%", percent.round().clamp(0.0, 150.0));
    let _ = run_quiet("wpctl", &["set-volume", "@DEFAULT_AUDIO_SINK@", &value]);
}

pub fn brightness_percent() -> Option<u8> {
    let output = command_output("brightnessctl", &["-m"])?;
    output
        .split(',')
        .find(|part| part.trim_end().ends_with('%'))
        .and_then(|part| part.trim().trim_end_matches('%').parse().ok())
}

pub fn set_brightness(percent: f64) {
    let value = format!("{}%", percent.round().clamp(1.0, 100.0));
    let _ = run_quiet("brightnessctl", &["set", &value]);
}

pub fn wifi_enabled() -> bool {
    command_output("nmcli", &["radio", "wifi"])
        .map(|value| value.eq_ignore_ascii_case("enabled"))
        .unwrap_or(false)
}

pub fn set_wifi(enabled: bool) {
    let state = if enabled { "on" } else { "off" };
    let _ = run_quiet("nmcli", &["radio", "wifi", state]);
}

pub fn bluetooth_enabled() -> bool {
    command_output("bluetoothctl", &["show"])
        .map(|output| output.lines().any(|line| line.trim() == "Powered: yes"))
        .unwrap_or(false)
}

pub fn set_bluetooth(enabled: bool) {
    let state = if enabled { "on" } else { "off" };
    let _ = run_quiet("bluetoothctl", &["power", state]);
}
