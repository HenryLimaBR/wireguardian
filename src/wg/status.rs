use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Stdio;

use super::SYSTEM_DIR;
use super::config::{derive_public_key, scan_dir};
use super::host;
use super::paths::{command_exists, config_home, user_tunnel_dir};
use super::types::{LiveStatus, PeerStatus, Privilege, Snapshot, Source, Tunnel};

pub fn detect_privilege() -> Privilege {
    if effective_uid() == Some(0) {
        return Privilege::Root;
    }
    let sudo_works = host::command("sudo")
        .args(["-n", "true"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if sudo_works {
        return Privilege::Sudo;
    }
    if command_exists("pkexec") {
        return Privilege::Pkexec;
    }
    Privilege::Unavailable
}

fn effective_uid() -> Option<u32> {
    let status = fs::read_to_string("/proc/self/status").ok()?;
    let line = status.lines().find(|line| line.starts_with("Uid:"))?;
    line.split_whitespace().nth(2)?.parse().ok()
}

pub fn snapshot(system_configs: Option<Vec<Tunnel>>) -> Snapshot {
    let privilege = detect_privilege();
    let wg_available = command_exists("wg");
    let wg_quick_available = command_exists("wg-quick");
    let user_dir = user_tunnel_dir();

    let (system_tunnels, system_error, system_requires_privilege) = match system_configs {
        Some(tunnels) => (tunnels, None, false),
        None if host::in_flatpak() => (
            Vec::new(),
            Some("System configs require host authorization".into()),
            true,
        ),
        None => scan_dir(Path::new(SYSTEM_DIR), Source::System),
    };
    let (personal_tunnels, personal_error, _) = scan_dir(&user_dir, Source::User);
    let (legacy_tunnels, _, _) = scan_dir(&config_home().join("wireguard"), Source::User);

    let mut tunnels: Vec<Tunnel> = Vec::new();
    let mut seen: HashMap<String, usize> = HashMap::new();
    for tunnel in system_tunnels
        .into_iter()
        .chain(personal_tunnels)
        .chain(legacy_tunnels)
    {
        if let Some(&index) = seen.get(&tunnel.name) {
            // Prefer a system config over a personal one with the same name.
            if tunnels[index].source == Source::User && tunnel.source == Source::System {
                tunnels[index] = tunnel;
            }
            continue;
        }
        seen.insert(tunnel.name.clone(), tunnels.len());
        tunnels.push(tunnel);
    }
    tunnels.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    if wg_available {
        for tunnel in tunnels.iter_mut() {
            if tunnel.public_key.is_none() {
                tunnel.public_key = tunnel.private_key.as_deref().and_then(derive_public_key);
            }
        }
    }

    let mut live = HashMap::new();
    for tunnel in &tunnels {
        live.insert(
            tunnel.name.clone(),
            read_live_status(&tunnel.name, privilege, wg_available),
        );
    }

    let system_error = system_error.or_else(|| {
        if !host::in_flatpak() && !Path::new(SYSTEM_DIR).exists() {
            Some(format!("{SYSTEM_DIR} does not exist"))
        } else {
            None
        }
    });
    let _ = personal_error;

    Snapshot {
        tunnels,
        live,
        privilege,
        system_error,
        system_requires_privilege,
        wg_available,
        wg_quick_available,
        user_dir,
    }
}

pub fn interface_is_up(name: &str) -> bool {
    let path = Path::new("/sys/class/net").join(name);
    if host::in_flatpak() {
        return host::command("test")
            .arg("-e")
            .arg(path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success());
    }
    path.exists()
}

fn read_live_status(name: &str, privilege: Privilege, wg_available: bool) -> LiveStatus {
    let mut status = LiveStatus {
        up: interface_is_up(name),
        ..Default::default()
    };
    let (rx, tx) = read_sysfs_counters(name);
    status.rx_bytes = rx;
    status.tx_bytes = tx;

    if !status.up || !wg_available {
        return status;
    }

    match show_dump(name, privilege) {
        Ok(dump) => {
            parse_dump(&dump, &mut status);
            status.stats_error = None;
        }
        Err(err) => status.stats_error = Some(err),
    }
    status
}

fn read_sysfs_counters(name: &str) -> (u64, u64) {
    let base = Path::new("/sys/class/net").join(name).join("statistics");
    let read = |file: &str| {
        let path = base.join(file);
        let contents = if host::in_flatpak() {
            host::command("cat")
                .arg(path)
                .stdin(Stdio::null())
                .stderr(Stdio::null())
                .output()
                .ok()
                .filter(|output| output.status.success())
                .and_then(|output| String::from_utf8(output.stdout).ok())
        } else {
            fs::read_to_string(path).ok()
        };
        contents
            .and_then(|value| value.trim().parse().ok())
            .unwrap_or(0)
    };
    (read("rx_bytes"), read("tx_bytes"))
}

/// `wg show` needs CAP_NET_ADMIN. Use the non-interactive path only; polling
/// must never pop an authentication dialog.
fn show_dump(name: &str, privilege: Privilege) -> Result<String, String> {
    let mut command = match privilege {
        Privilege::Root => host::command("wg"),
        Privilege::Sudo => {
            let mut command = host::command("sudo");
            command.args(["-n", "wg"]);
            command
        }
        Privilege::Pkexec | Privilege::Unavailable => host::command("wg"),
    };
    let output = command
        .args(["show", name, "dump"])
        .stdin(Stdio::null())
        .output()
        .map_err(|err| err.to_string())?;

    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    if stderr.is_empty() {
        Err(format!("wg show exited with {}", output.status))
    } else {
        Err(stderr)
    }
}

fn parse_dump(dump: &str, status: &mut LiveStatus) {
    let mut lines = dump.lines().filter(|line| !line.trim().is_empty());
    let Some(interface) = lines.next() else {
        return;
    };
    let fields: Vec<&str> = interface.split('\t').collect();
    if fields.len() >= 3 {
        status.public_key = clean_field(fields[1]);
        status.listen_port = fields[2].parse().ok();
    }
    for line in lines {
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 8 {
            continue;
        }
        status.peers.push(PeerStatus {
            public_key: fields[0].to_owned(),
            endpoint: fields[2].to_owned(),
            allowed_ips: fields[3].to_owned(),
            latest_handshake: fields[4].parse().unwrap_or(0),
            rx: fields[5].parse().unwrap_or(0),
            tx: fields[6].parse().unwrap_or(0),
            persistent_keepalive: fields[7].to_owned(),
        });
    }
}

fn clean_field(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() || value == "(none)" || value == "off" {
        None
    } else {
        Some(value.to_owned())
    }
}
