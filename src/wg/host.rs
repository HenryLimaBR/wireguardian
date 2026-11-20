use std::path::Path;
use std::process::{Command, Stdio};

pub(super) fn in_flatpak() -> bool {
    Path::new("/.flatpak-info").exists()
}

pub(super) fn command(program: &str) -> Command {
    command_for(program, in_flatpak())
}

fn command_for(program: &str, flatpak: bool) -> Command {
    if flatpak {
        let mut command = Command::new("flatpak-spawn");
        command.args(["--host", "--watch-bus", program]);
        command
    } else {
        Command::new(program)
    }
}

pub(super) fn find_command(name: &str) -> Option<String> {
    let output = command("sh")
        .args(["-c", "command -v \"$1\"", "sh", name])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let path = String::from_utf8(output.stdout).ok()?.trim().to_owned();
    Path::new(&path).is_absolute().then_some(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commands_use_host_portal_only_in_flatpak() {
        let native = command_for("wg", false);
        assert_eq!(native.get_program(), "wg");
        assert_eq!(native.get_args().count(), 0);

        let flatpak = command_for("wg", true);
        assert_eq!(flatpak.get_program(), "flatpak-spawn");
        assert_eq!(
            flatpak.get_args().collect::<Vec<_>>(),
            ["--host", "--watch-bus", "wg"]
        );
    }
}
