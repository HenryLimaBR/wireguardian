use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Stdio;

use super::SYSTEM_DIR;
use super::host;
use super::paths::user_tunnel_dir;
use super::types::{ActionResult, ImportResult, Peer, Source, Tunnel};

pub(super) fn scan_dir(dir: &Path, source: Source) -> (Vec<Tunnel>, Option<String>, bool) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) => {
            let denied = err.kind() == std::io::ErrorKind::PermissionDenied;
            return (
                Vec::new(),
                Some(format!("{}: {err}", dir.display())),
                denied,
            );
        }
    };

    let mut tunnels = Vec::new();
    let mut permission_error = None;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("conf") {
            continue;
        }
        let Some(name) = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .map(str::to_owned)
        else {
            continue;
        };
        let (tunnel, denied) = parse_config(&path, name, source);
        if denied && permission_error.is_none() {
            permission_error = Some(format!("{}: permission denied", path.display()));
        }
        tunnels.push(tunnel);
    }
    let denied = permission_error.is_some();
    (tunnels, permission_error, denied)
}

fn parse_config(path: &Path, name: String, source: Source) -> (Tunnel, bool) {
    match fs::read_to_string(path) {
        Ok(contents) => (parse_config_contents(path, name, source, &contents), false),
        Err(err) => {
            let denied = err.kind() == std::io::ErrorKind::PermissionDenied;
            (
                Tunnel {
                    name,
                    path: path.to_path_buf(),
                    source,
                    parse_error: Some(err.to_string()),
                    ..Default::default()
                },
                denied,
            )
        }
    }
}

pub(super) fn parse_config_contents(
    path: &Path,
    name: String,
    source: Source,
    contents: &str,
) -> Tunnel {
    let mut tunnel = Tunnel {
        name,
        path: path.to_path_buf(),
        source,
        ..Default::default()
    };

    #[derive(Clone, Copy, PartialEq)]
    enum Section {
        None,
        Interface,
        Peer,
    }

    let mut section = Section::None;
    let mut current_peer: Option<Peer> = None;

    for raw_line in contents.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            if let Some(peer) = current_peer.take() {
                tunnel.peers.push(peer);
            }
            section = match line[1..line.len() - 1].trim().to_ascii_lowercase().as_str() {
                "interface" => Section::Interface,
                "peer" => Section::Peer,
                _ => Section::None,
            };
            if section == Section::Peer {
                current_peer = Some(Peer::default());
            }
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim().to_ascii_lowercase();
        let value = value.trim();
        if value.is_empty() {
            continue;
        }

        match section {
            Section::Interface => match key.as_str() {
                "privatekey" => tunnel.private_key = Some(value.to_owned()),
                "address" => tunnel.addresses.extend(split_list(value)),
                "dns" => tunnel.dns.extend(split_list(value)),
                "listenport" => tunnel.listen_port = value.parse().ok(),
                "mtu" => tunnel.mtu = value.parse().ok(),
                "fwmark" => tunnel.fwmark = Some(value.to_owned()),
                _ => {}
            },
            Section::Peer => {
                let peer = current_peer.get_or_insert_with(Peer::default);
                match key.as_str() {
                    "publickey" => peer.public_key = value.to_owned(),
                    "presharedkey" => peer.has_preshared_key = true,
                    "endpoint" => peer.endpoint = Some(value.to_owned()),
                    "allowedips" => peer.allowed_ips.extend(split_list(value)),
                    "persistentkeepalive" => {
                        peer.persistent_keepalive = value.parse().ok();
                    }
                    _ => {}
                }
            }
            Section::None => {}
        }
    }

    if let Some(peer) = current_peer.take() {
        tunnel.peers.push(peer);
    }

    if tunnel.private_key.is_none() {
        tunnel.parse_error = Some("missing PrivateKey in [Interface]".into());
    } else if tunnel.peers.is_empty() {
        tunnel.parse_error = Some("no [Peer] sections found".into());
    }

    tunnel
}

fn split_list(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_owned)
        .collect()
}

pub(super) fn derive_public_key(private_key: &str) -> Option<String> {
    let mut child = host::command("wg")
        .arg("pubkey")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    {
        let stdin = child.stdin.as_mut()?;
        stdin.write_all(private_key.trim().as_bytes()).ok()?;
        stdin.write_all(b"\n").ok()?;
    }
    let output = child.wait_with_output().ok()?;
    if !output.status.success() {
        return None;
    }
    let key = String::from_utf8(output.stdout).ok()?.trim().to_owned();
    (!key.is_empty()).then_some(key)
}

pub fn import_config(source: &Path) -> ImportResult {
    if source.as_os_str().is_empty() {
        return ImportResult {
            ok: false,
            message: "Enter the path of a WireGuard .conf file".into(),
            name: None,
        };
    }
    let source = if source.is_absolute() {
        source.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(source))
            .unwrap_or_else(|_| source.to_path_buf())
    };
    if !source.is_file() {
        return ImportResult {
            ok: false,
            message: format!("{} is not a file", source.display()),
            name: None,
        };
    }

    let Some(stem) = source.file_stem().and_then(|stem| stem.to_str()) else {
        return ImportResult {
            ok: false,
            message: "The file name is not valid UTF-8".into(),
            name: None,
        };
    };
    let name = sanitize_name(stem);
    if name.is_empty() {
        return ImportResult {
            ok: false,
            message: "The tunnel name must contain letters, digits, '-' or '_'".into(),
            name: None,
        };
    }

    let (parsed, _) = parse_config(&source, name.clone(), Source::User);
    if let Some(error) = parsed.parse_error {
        return ImportResult {
            ok: false,
            message: format!("Invalid configuration: {error}"),
            name: None,
        };
    }

    let dir = user_tunnel_dir();
    if let Err(err) = fs::create_dir_all(&dir) {
        return ImportResult {
            ok: false,
            message: format!("Could not create {}: {err}", dir.display()),
            name: None,
        };
    }
    let _ = fs::set_permissions(&dir, fs::Permissions::from_mode(0o700));

    let destination = dir.join(format!("{name}.conf"));
    if let Err(err) = fs::copy(&source, &destination) {
        return ImportResult {
            ok: false,
            message: format!("Could not copy config: {err}"),
            name: None,
        };
    }
    let _ = fs::set_permissions(&destination, fs::Permissions::from_mode(0o600));

    ImportResult {
        ok: true,
        message: format!("Imported \"{name}\" into {}", dir.display()),
        name: Some(name),
    }
}

pub fn delete_config(tunnel: &Tunnel) -> ActionResult {
    if tunnel.source == Source::System {
        return ActionResult {
            ok: false,
            message: format!("System tunnels must be removed from {SYSTEM_DIR} as root"),
        };
    }
    match fs::remove_file(&tunnel.path) {
        Ok(()) => ActionResult {
            ok: true,
            message: format!("Removed \"{}\"", tunnel.name),
        },
        Err(err) => ActionResult {
            ok: false,
            message: format!("Could not remove config: {err}"),
        },
    }
}

fn sanitize_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        .collect::<String>()
        .trim_matches('.')
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_unreadable_system_directory_and_config() {
        let dir = std::env::temp_dir().join(format!(
            "wireguardian-permissions-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&dir).unwrap();
        let config = dir.join("office.conf");
        fs::write(&config, "[Interface]\nPrivateKey = secret\n[Peer]\n").unwrap();

        fs::set_permissions(&dir, fs::Permissions::from_mode(0o000)).unwrap();
        let dir_denied = fs::read_dir(&dir)
            .err()
            .is_some_and(|err| err.kind() == std::io::ErrorKind::PermissionDenied);
        let dir_result = scan_dir(&dir, Source::System);
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();

        fs::set_permissions(&config, fs::Permissions::from_mode(0o000)).unwrap();
        let file_denied = fs::read(&config)
            .err()
            .is_some_and(|err| err.kind() == std::io::ErrorKind::PermissionDenied);
        let file_result = scan_dir(&dir, Source::System);
        fs::remove_file(config).unwrap();
        fs::remove_dir(dir).unwrap();

        if dir_denied {
            assert!(dir_result.2);
            assert!(dir_result.1.is_some());
        }
        if file_denied {
            assert!(file_result.2);
            assert!(file_result.1.is_some());
            assert!(file_result.0[0].parse_error.is_some());
        }
    }
}
