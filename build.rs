//! Local link shims for development machines that ship the runtime shared
//! objects of GPUI's X11/Wayland backends without the `-devel` symlinks.
//!
//! This creates symlinks inside Cargo's OUT_DIR and adds that directory to the
//! linker search path. Nothing is written outside the build directory and no
//! system package is installed.

#[cfg(target_os = "linux")]
fn main() {
    use std::os::unix::fs::symlink;
    use std::path::{Path, PathBuf};

    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR is set by cargo"));
    let link_dir = out_dir.join("native-libs");
    std::fs::create_dir_all(&link_dir).expect("create native link directory");

    // (linker name, versioned runtime candidates)
    let libraries: &[(&str, &[&str])] = &[
        (
            "libxcb.so",
            &[
                "/usr/lib64/libxcb.so.1",
                "/usr/lib/libxcb.so.1",
                "/lib64/libxcb.so.1",
                "/lib/libxcb.so.1",
            ],
        ),
        (
            "libxkbcommon.so",
            &[
                "/usr/lib64/libxkbcommon.so.0",
                "/usr/lib/libxkbcommon.so.0",
                "/lib64/libxkbcommon.so.0",
                "/lib/libxkbcommon.so.0",
            ],
        ),
        (
            "libxkbcommon-x11.so",
            &[
                "/usr/lib64/libxkbcommon-x11.so.0",
                "/usr/lib/libxkbcommon-x11.so.0",
                "/lib64/libxkbcommon-x11.so.0",
                "/lib/libxkbcommon-x11.so.0",
            ],
        ),
    ];

    for (link_name, candidates) in libraries {
        let link = link_dir.join(link_name);
        if link.exists() {
            continue;
        }
        if let Some(target) = candidates.iter().find(|candidate| Path::new(candidate).exists()) {
            symlink(target, &link).expect("create local library symlink");
        }
    }

    println!("cargo:rustc-link-search=native={}", link_dir.display());
}

#[cfg(not(target_os = "linux"))]
fn main() {}
