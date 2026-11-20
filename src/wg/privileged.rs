use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};

use super::SYSTEM_DIR;
use super::config::parse_config_contents;
use super::host;
use super::types::{Source, Tunnel};

const MAGIC: &[u8] = b"WGCONF1";

pub fn read_system_configs() -> Result<Vec<Tunnel>, String> {
    let flatpak = host::in_flatpak();
    let helper = if flatpak {
        std::env::var_os("SUDO_ASKPASS").or_else(|| {
            ["ksshaskpass", "ssh-askpass", "lxqt-sudo-askpass"]
                .into_iter()
                .find_map(host::find_command)
                .map(Into::into)
        })
    } else if std::env::var_os("SUDO_ASKPASS").is_none() {
        std::env::var_os("PATH").and_then(|path| {
            std::env::split_paths(&path)
                .filter(|dir| dir.is_absolute())
                .flat_map(|dir| {
                    ["ksshaskpass", "ssh-askpass", "lxqt-sudo-askpass"]
                        .map(move |name| dir.join(name))
                })
                .find(|path| path.is_file())
                .map(Into::into)
        })
    } else {
        None
    };

    let mut command = if flatpak {
        let mut command = Command::new("flatpak-spawn");
        command.args(["--host", "--watch-bus"]);
        if let Some(helper) = helper {
            let mut env = std::ffi::OsString::from("--env=SUDO_ASKPASS=");
            env.push(helper);
            command.arg(env);
        }
        command.arg("sudo");
        command
    } else {
        let mut command = Command::new("sudo");
        if let Some(helper) = helper {
            command.env("SUDO_ASKPASS", helper);
        }
        command
    };
    command
        .args([
            "-A",
            "-p",
            "WireGuardian needs access to system WireGuard configurations: ",
            "--",
        ])
        .stdin(Stdio::null());
    if flatpak {
        command.args(["python3", "-c", include_str!("host_reader.py"), SYSTEM_DIR]);
    } else {
        let executable = std::env::current_exe()
            .map_err(|err| format!("Could not locate WireGuardian: {err}"))?;
        command.arg(executable).arg("--read-system-configs");
    }

    let output = command
        .output()
        .map_err(|err| format!("Could not start sudo: {err}"))?;
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        return Err(if detail.is_empty() {
            format!(
                "sudo could not read system configurations ({})",
                output.status
            )
        } else {
            format!("Could not read system configurations: {detail}")
        });
    }
    decode_configs(&output.stdout)
        .map_err(|err| format!("Invalid response from system configuration reader: {err}"))
}

pub fn run_config_reader() -> io::Result<()> {
    let stdout = io::stdout();
    write_configs(Path::new(SYSTEM_DIR), &mut stdout.lock())
}

fn write_configs(dir: &Path, output: &mut impl Write) -> io::Result<()> {
    output.write_all(MAGIC)?;
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("conf") {
            continue;
        }
        let Some(filename) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let contents = fs::read(&path)?;
        write_field(output, filename.as_bytes())?;
        write_field(output, &contents)?;
    }
    output.write_all(&0u64.to_le_bytes())
}

fn write_field(output: &mut impl Write, value: &[u8]) -> io::Result<()> {
    output.write_all(&(value.len() as u64).to_le_bytes())?;
    output.write_all(value)
}

fn read_field<'a>(input: &mut &'a [u8]) -> io::Result<&'a [u8]> {
    let mut length = [0; 8];
    input.read_exact(&mut length)?;
    let length = usize::try_from(u64::from_le_bytes(length))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "field too large"))?;
    if length > input.len() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "truncated field",
        ));
    }
    let (value, rest) = input.split_at(length);
    *input = rest;
    Ok(value)
}

fn decode_configs(mut input: &[u8]) -> io::Result<Vec<Tunnel>> {
    if !input.starts_with(MAGIC) {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid header"));
    }
    input = &input[MAGIC.len()..];
    let mut tunnels = Vec::new();
    loop {
        let filename = read_field(&mut input)?;
        if filename.is_empty() {
            if !input.is_empty() {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "trailing data"));
            }
            return Ok(tunnels);
        }
        let filename = std::str::from_utf8(filename)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
        let path = Path::new(filename);
        let name = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .filter(|_| path.extension().and_then(|ext| ext.to_str()) == Some("conf"))
            .filter(|_| path.components().count() == 1)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid filename"))?;
        let path = Path::new(SYSTEM_DIR).join(path);
        let contents = read_field(&mut input)?;
        let tunnel = match std::str::from_utf8(contents) {
            Ok(contents) => parse_config_contents(&path, name.to_owned(), Source::System, contents),
            Err(_) => Tunnel {
                name: name.to_owned(),
                path,
                source: Source::System,
                parse_error: Some("stream did not contain valid UTF-8".into()),
                ..Default::default()
            },
        };
        tunnels.push(tunnel);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_only_config_files_from_directory() {
        let dir = std::env::temp_dir().join(format!(
            "wireguardian-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&dir).unwrap();
        let config = dir.join("office.conf");
        let ignored = dir.join("notes.txt");
        fs::write(
            &config,
            "[Interface]\nPrivateKey = secret\n[Peer]\nPublicKey = peer\n",
        )
        .unwrap();
        fs::write(&ignored, "not a config").unwrap();

        let mut data = Vec::new();
        let result = write_configs(&dir, &mut data).and_then(|_| decode_configs(&data));
        let host_output = Command::new("python3")
            .args(["-c", include_str!("host_reader.py")])
            .arg(&dir)
            .output()
            .expect("python3 is required to test the Flatpak host reader");
        let host_result = decode_configs(&host_output.stdout);
        fs::remove_file(config).unwrap();
        fs::remove_file(ignored).unwrap();
        fs::remove_dir(dir).unwrap();

        let tunnels = result.unwrap();
        assert_eq!(tunnels.len(), 1);
        assert_eq!(tunnels[0].name, "office");
        assert_eq!(tunnels[0].peers[0].public_key, "peer");
        assert!(host_output.status.success());
        let host_tunnels = host_result.unwrap();
        assert_eq!(host_tunnels.len(), 1);
        assert_eq!(host_tunnels[0].name, "office");
        assert_eq!(host_tunnels[0].peers[0].public_key, "peer");
    }

    #[test]
    fn decodes_system_configs_and_reports_invalid_utf8() {
        let mut data = MAGIC.to_vec();
        write_field(&mut data, b"office.conf").unwrap();
        write_field(
            &mut data,
            b"[Interface]\nPrivateKey = secret\n[Peer]\nPublicKey = peer\n",
        )
        .unwrap();
        write_field(&mut data, b"broken.conf").unwrap();
        write_field(&mut data, &[0xff]).unwrap();
        data.extend_from_slice(&0u64.to_le_bytes());

        let tunnels = decode_configs(&data).unwrap();
        assert_eq!(tunnels.len(), 2);
        assert_eq!(tunnels[0].name, "office");
        assert_eq!(tunnels[0].path, Path::new(SYSTEM_DIR).join("office.conf"));
        assert_eq!(tunnels[0].private_key.as_deref(), Some("secret"));
        assert_eq!(tunnels[0].peers[0].public_key, "peer");
        assert!(tunnels[1].parse_error.is_some());
    }

    #[test]
    fn rejects_truncated_and_invalid_responses() {
        assert!(decode_configs(b"bad header").is_err());
        let mut data = MAGIC.to_vec();
        write_field(&mut data, b"../outside.conf").unwrap();
        write_field(&mut data, b"").unwrap();
        data.extend_from_slice(&0u64.to_le_bytes());
        assert!(decode_configs(&data).is_err());
        let mut truncated = MAGIC.to_vec();
        truncated.extend_from_slice(&10u64.to_le_bytes());
        truncated.extend_from_slice(b"short");
        assert!(decode_configs(&truncated).is_err());
    }
}
