use std::process::Stdio;

use super::host;
use super::paths::command_exists;
use super::types::{ActionResult, Privilege, Tunnel};

pub fn set_tunnel(tunnel: &Tunnel, up: bool, privilege: Privilege) -> ActionResult {
    if !command_exists("wg-quick") {
        return ActionResult {
            ok: false,
            message: "wg-quick was not found on PATH".into(),
        };
    }
    if !privilege.can_elevate() {
        return ActionResult {
            ok: false,
            message: "Cannot change tunnel state without root, passwordless sudo, or pkexec".into(),
        };
    }

    let action = if up { "up" } else { "down" };
    let mut command = match privilege {
        Privilege::Root => host::command("wg-quick"),
        Privilege::Sudo => {
            let mut command = host::command("sudo");
            command.args(["-n", "wg-quick"]);
            command
        }
        Privilege::Pkexec => {
            let mut command = host::command("pkexec");
            command.arg("wg-quick");
            command
        }
        Privilege::Unavailable => unreachable!(),
    };

    let output = command
        .arg(action)
        .arg(tunnel.quick_target())
        .stdin(Stdio::null())
        .output();

    let output = match output {
        Ok(output) => output,
        Err(err) => {
            return ActionResult {
                ok: false,
                message: format!("Failed to launch wg-quick: {err}"),
            };
        }
    };

    let mut detail = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    if detail.is_empty() {
        detail = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    }

    if output.status.success() {
        return ActionResult {
            ok: true,
            message: format!(
                "Tunnel \"{}\" {}",
                tunnel.name,
                if up { "connected" } else { "disconnected" }
            ),
        };
    }

    if output.status.code() == Some(126) {
        return ActionResult {
            ok: false,
            message: "Authorization was dismissed or denied".into(),
        };
    }

    let lower = detail.to_lowercase();
    if lower.contains("operation not permitted")
        || lower.contains("permission denied")
        || lower.contains("must be run as root")
    {
        detail = format!(
            "{detail}\nHint: start WireGuardian with elevated privileges, configure passwordless sudo for wg-quick, or use polkit."
        );
    }
    if detail.is_empty() {
        detail = format!("wg-quick exited with {}", output.status);
    }

    ActionResult {
        ok: false,
        message: detail,
    }
}
