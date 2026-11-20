//! Minimal WireGuard backend for WireGuardian.
//!
//! Nothing is installed or configured system-wide: the module only reads
//! configuration files and shells out to the `wg` / `wg-quick` binaries that
//! are already on the machine. Elevation is optional and best-effort.

mod actions;
mod config;
mod formatting;
mod host;
mod paths;
mod privileged;
mod status;
mod types;

#[allow(unused_imports)]
pub use actions::set_tunnel;
#[allow(unused_imports)]
pub use config::{delete_config, import_config};
#[allow(unused_imports)]
pub use formatting::{human_ago, human_bytes, now_epoch};
pub(crate) use paths::config_home;
#[allow(unused_imports)]
pub use paths::{command_exists, expand_path, user_tunnel_dir};
pub use privileged::{read_system_configs, run_config_reader};
#[allow(unused_imports)]
pub use status::{detect_privilege, interface_is_up, snapshot};
#[allow(unused_imports)]
pub use types::{
    ActionResult, ImportResult, LiveStatus, Peer, PeerStatus, Privilege, Snapshot, Source, Tunnel,
};

pub const SYSTEM_DIR: &str = "/etc/wireguard";
