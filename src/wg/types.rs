use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Source {
    #[default]
    System,
    User,
}

impl Source {
    pub fn label(self) -> &'static str {
        match self {
            Source::System => "system",
            Source::User => "personal",
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Peer {
    pub public_key: String,
    pub has_preshared_key: bool,
    pub endpoint: Option<String>,
    pub allowed_ips: Vec<String>,
    pub persistent_keepalive: Option<u32>,
}

#[derive(Clone, Debug, Default)]
pub struct Tunnel {
    pub name: String,
    pub path: PathBuf,
    pub source: Source,
    pub private_key: Option<String>,
    pub public_key: Option<String>,
    pub addresses: Vec<String>,
    pub dns: Vec<String>,
    pub listen_port: Option<u32>,
    pub mtu: Option<u32>,
    pub fwmark: Option<String>,
    pub peers: Vec<Peer>,
    pub parse_error: Option<String>,
}

impl Tunnel {
    /// The argument handed to `wg-quick`. System tunnels live in the default
    /// search path, personal ones are addressed by their full path.
    pub fn quick_target(&self) -> String {
        match self.source {
            Source::System => self.name.clone(),
            Source::User => self.path.to_string_lossy().into_owned(),
        }
    }

    pub fn primary_endpoint(&self) -> Option<&str> {
        self.peers.iter().find_map(|p| p.endpoint.as_deref())
    }
}

#[derive(Clone, Debug, Default)]
pub struct PeerStatus {
    pub public_key: String,
    pub endpoint: String,
    pub allowed_ips: String,
    pub latest_handshake: u64,
    pub rx: u64,
    pub tx: u64,
    pub persistent_keepalive: String,
}

#[derive(Clone, Debug, Default)]
pub struct LiveStatus {
    pub up: bool,
    pub public_key: Option<String>,
    pub listen_port: Option<u32>,
    pub peers: Vec<PeerStatus>,
    pub stats_error: Option<String>,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}

impl LiveStatus {
    pub fn latest_handshake(&self) -> u64 {
        self.peers
            .iter()
            .map(|p| p.latest_handshake)
            .max()
            .unwrap_or(0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Privilege {
    Root,
    Sudo,
    Pkexec,
    Unavailable,
}

impl Privilege {
    pub fn label(self) -> &'static str {
        match self {
            Privilege::Root => "running as root",
            Privilege::Sudo => "passwordless sudo",
            Privilege::Pkexec => "polkit (pkexec)",
            Privilege::Unavailable => "no elevation available",
        }
    }

    pub fn can_elevate(self) -> bool {
        self != Privilege::Unavailable
    }
}

#[derive(Clone, Debug)]
pub struct Snapshot {
    pub tunnels: Vec<Tunnel>,
    pub live: HashMap<String, LiveStatus>,
    pub privilege: Privilege,
    pub system_error: Option<String>,
    pub system_requires_privilege: bool,
    pub wg_available: bool,
    pub wg_quick_available: bool,
    pub user_dir: PathBuf,
}

#[derive(Clone, Debug)]
pub struct ActionResult {
    pub ok: bool,
    pub message: String,
}

#[derive(Clone, Debug)]
pub struct ImportResult {
    pub ok: bool,
    pub message: String,
    pub name: Option<String>,
}
