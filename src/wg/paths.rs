use std::path::PathBuf;

pub fn user_tunnel_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("WIREGUARDIAN_TUNNELS") {
        return PathBuf::from(dir);
    }
    config_home().join("wireguardian").join("tunnels")
}

pub(crate) fn config_home() -> PathBuf {
    if let Some(dir) = std::env::var_os("XDG_CONFIG_HOME") {
        let dir = PathBuf::from(dir);
        if dir.is_absolute() {
            return dir;
        }
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join(".config");
    }
    PathBuf::from(".config")
}

pub fn expand_path(raw: &str) -> PathBuf {
    let raw = raw.trim();
    if let Some(rest) = raw.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }
    PathBuf::from(raw)
}

pub fn command_exists(name: &str) -> bool {
    if super::host::in_flatpak() {
        return super::host::find_command(name).is_some();
    }
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| {
        let candidate = dir.join(name);
        candidate.is_file()
    })
}
