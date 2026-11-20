# wireguardian

A Linux desktop client for viewing and managing WireGuard tunnels. Use the
Settings menu to choose a theme.

## Flatpak

The GitHub Actions Flatpak workflow builds an x86-64 bundle on pushes, pull
requests, or manual runs. Download `wireguardian-flatpak` from the workflow
run, then install and launch it:

```sh
flatpak install --user wireguardian.flatpak
flatpak run io.github.HenryLimaBR.WireGuardian
```

The host needs WireGuard tools (`wg` and `wg-quick`), `python3`, `sudo`,
and a graphical sudo askpass helper (for example `ksshaskpass` or
`ssh-askpass`) for protected system configurations. Connecting and
disconnecting also requires root authorization through passwordless sudo or
polkit (`pkexec`). A polkit authentication agent must be running when pkexec
is used. The application does not install or configure these host tools.

**Security:** the Flatpak has access to the host filesystem and the Flatpak
host-command portal. It can run commands outside the sandbox, including
privileged WireGuard operations after authorization. This greatly weakens
Flatpak isolation; only install a bundle you trust. System configurations are
read on the host after sudo authorization and cached in application memory
until reloaded or the application exits. Personal configurations imported by
the Flatpak are stored in its own application data directory.

### Build locally

Install Flatpak, Flatpak Builder, AppStream Compose, Cargo, Python 3 and the
26.08 Freedesktop runtime, SDK and Rust extension from Flathub. On Ubuntu,
the Compose tool is provided by the `appstream-compose` package. Then, from
the project root:

```sh
flatpak --user remote-add --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
flatpak --user install flathub org.freedesktop.Platform//26.08 org.freedesktop.Sdk//26.08 org.freedesktop.Sdk.Extension.rust-stable//26.08
cargo vendor --locked vendor > flatpak/cargo-config.toml
flatpak-builder --user --force-clean --repo=repo build-dir flatpak/io.github.HenryLimaBR.WireGuardian.yml
flatpak build-bundle repo wireguardian.flatpak io.github.HenryLimaBR.WireGuardian
```

Cargo downloads the locked dependencies during `cargo vendor`; the Flatpak
build itself runs offline. Generated dependencies, build directories, and the
bundle are ignored by Git. The same application ID is used for the desktop
entry and window icon. For a native installation, install the desktop entry
and 512-pixel icon in the corresponding XDG applications and hicolor icon
directories; X11 also receives the icon directly from the window.

The workflow produces an artifact only; it does not publish to Flathub or
create a GitHub release. Before public distribution, confirm the GPL license
variant in the application metadata, and review the broad host permissions
and Flathub's submission requirements.
