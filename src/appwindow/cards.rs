use gpui_kit::assets::IconName;
use gpui_kit::component::*;
use gpui_kit::*;

use super::AppWindow;
use super::components::*;
use crate::wg::{self, LiveStatus, Tunnel};

impl AppWindow {
    pub(super) fn render_connection_card(
        &self,
        tunnel: &Tunnel,
        live: Option<&LiveStatus>,
        up: bool,
        theme: &Theme,
    ) -> impl IntoElement {
        let now = wg::now_epoch();
        let handshake = live.map(|live| live.latest_handshake()).unwrap_or(0);
        let (handshake_text, handshake_color) = handshake_label(handshake, now, theme);
        let listen_port = live
            .and_then(|live| live.listen_port)
            .or(tunnel.listen_port);

        let mut card = panel(theme)
            .child(section_title("Connection", theme))
            .child(
                h_flex()
                    .gap_8()
                    .flex_wrap()
                    .child(stat(
                        theme,
                        "Status",
                        if up { "Up" } else { "Down" }.to_string(),
                    ))
                    .child(stat_colored(
                        theme,
                        "Latest handshake",
                        handshake_text,
                        handshake_color,
                    ))
                    .child(stat(
                        theme,
                        "Listen port",
                        listen_port
                            .map(|port| port.to_string())
                            .unwrap_or_else(|| "auto".into()),
                    ))
                    .child(stat(
                        theme,
                        "Transfer",
                        live.map(|live| {
                            format!(
                                "↓ {}   ↑ {}",
                                wg::human_bytes(live.rx_bytes),
                                wg::human_bytes(live.tx_bytes)
                            )
                        })
                        .unwrap_or_else(|| "—".into()),
                    )),
            );

        if let Some(error) = live.and_then(|live| live.stats_error.as_deref()) {
            card = card.child(
                h_flex()
                    .items_start()
                    .gap_2()
                    .child(
                        Icon::new(IconName::Info)
                            .size(px(14.))
                            .text_color(theme.muted_foreground),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child(format!("Live peer stats unavailable: {error}")),
                    ),
            );
        }

        card
    }

    pub(super) fn render_interface_card(&self, tunnel: &Tunnel, theme: &Theme) -> impl IntoElement {
        panel(theme)
            .child(section_title("Interface configuration", theme))
            .child(
                v_flex()
                    .gap_1()
                    .child(detail_row(
                        theme,
                        "Address",
                        join_or_dash(&tunnel.addresses),
                    ))
                    .child(detail_row(theme, "DNS", join_or_dash(&tunnel.dns)))
                    .child(detail_row(
                        theme,
                        "Listen port",
                        tunnel
                            .listen_port
                            .map(|port| port.to_string())
                            .unwrap_or_else(|| "auto".into()),
                    ))
                    .child(detail_row(
                        theme,
                        "MTU",
                        tunnel
                            .mtu
                            .map(|mtu| mtu.to_string())
                            .unwrap_or_else(|| "auto".into()),
                    ))
                    .child(detail_row(
                        theme,
                        "Public key",
                        tunnel
                            .public_key
                            .as_deref()
                            .map(chunk_key)
                            .unwrap_or_else(|| "unavailable".into()),
                    ))
                    .child(detail_row(
                        theme,
                        "Config file",
                        tunnel.path.display().to_string(),
                    )),
            )
    }

    pub(super) fn render_peers(
        &self,
        tunnel: &Tunnel,
        live: Option<&LiveStatus>,
        up: bool,
        theme: &Theme,
    ) -> impl IntoElement {
        let now = wg::now_epoch();
        let mut section = v_flex().gap_3().child(
            h_flex()
                .items_center()
                .gap_2()
                .child(section_title("Peers", theme))
                .child(chip(
                    theme,
                    format!("{}", tunnel.peers.len()),
                    theme.muted_foreground,
                )),
        );

        if tunnel.peers.is_empty() {
            section = section.child(
                div()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child("This configuration does not define any peers."),
            );
        }

        for (index, peer) in tunnel.peers.iter().enumerate() {
            let live_peer = live.filter(|_| up).and_then(|live| {
                live.peers
                    .iter()
                    .find(|status| status.public_key == peer.public_key)
            });
            section = section.child(render_peer(peer, live_peer, index, now, theme));
        }

        section
    }
}
