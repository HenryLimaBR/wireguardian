use gpui_kit::assets::IconName;
use gpui_kit::component::*;
use gpui_kit::*;

use super::{Notice, NoticeKind};
use crate::wg::{self, Peer, PeerStatus};

pub(super) fn chip(theme: &Theme, text: impl Into<SharedString>, color: Hsla) -> Div {
    let _ = theme;
    h_flex()
        .items_center()
        .gap_1()
        .px_2()
        .py_1()
        .rounded_full()
        .bg(color.opacity(0.12))
        .border_1()
        .border_color(color.opacity(0.28))
        .child(div().size(px(6.)).rounded_full().bg(color))
        .child(div().text_xs().text_color(color).child(text.into()))
}

pub(super) fn panel(theme: &Theme) -> Div {
    v_flex()
        .gap_4()
        .p_5()
        .rounded_xl()
        .border_1()
        .border_color(theme.border)
        .bg(theme.colors.list)
}

pub(super) fn section_title(title: &'static str, theme: &Theme) -> Div {
    div()
        .text_sm()
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(theme.foreground)
        .child(title)
}

pub(super) fn detail_row(theme: &Theme, label: &'static str, value: String) -> Div {
    h_flex()
        .items_start()
        .gap_4()
        .py_1()
        .child(
            div()
                .w(px(120.))
                .flex_shrink_0()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(label),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_sm()
                .text_color(theme.foreground)
                .child(value),
        )
}

pub(super) fn stat(theme: &Theme, label: &'static str, value: String) -> Div {
    stat_colored(theme, label, value, theme.foreground)
}

pub(super) fn stat_colored(theme: &Theme, label: &'static str, value: String, color: Hsla) -> Div {
    v_flex()
        .gap_1()
        .min_w(px(120.))
        .child(
            div()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(label),
        )
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(color)
                .child(value),
        )
}

pub(super) fn notice_banner(notice: &Notice, theme: &Theme) -> AnyElement {
    let (color, icon) = match notice.kind {
        NoticeKind::Info => (theme.info, IconName::Info),
        NoticeKind::Success => (theme.success, IconName::CircleCheck),
        NoticeKind::Error => (theme.danger, IconName::CircleAlert),
    };
    h_flex()
        .items_start()
        .gap_2()
        .p_3()
        .rounded_md()
        .bg(color.opacity(0.1))
        .border_1()
        .border_color(color.opacity(0.28))
        .child(Icon::new(icon).size(px(16.)).text_color(color))
        .child(
            div()
                .flex_1()
                .text_sm()
                .text_color(color)
                .child(notice.text.replace('\n', " ")),
        )
        .into_any_element()
}

pub(super) fn render_peer(
    peer: &Peer,
    live: Option<&PeerStatus>,
    index: usize,
    now: u64,
    theme: &Theme,
) -> AnyElement {
    let handshake = live.map(|live| live.latest_handshake).unwrap_or(0);
    let (handshake_text, handshake_color) = handshake_label(handshake, now, theme);
    let endpoint = live
        .map(|live| live.endpoint.clone())
        .filter(|endpoint| !endpoint.is_empty() && endpoint != "(none)")
        .or_else(|| peer.endpoint.clone())
        .unwrap_or_else(|| "—".into());
    let allowed_ips = live
        .map(|live| live.allowed_ips.clone())
        .filter(|ips| !ips.is_empty())
        .unwrap_or_else(|| join_or_dash(&peer.allowed_ips));
    let keepalive = live
        .map(|live| live.persistent_keepalive.clone())
        .filter(|value| !value.is_empty() && value != "off")
        .or_else(|| peer.persistent_keepalive.map(|value| format!("{value}s")))
        .unwrap_or_else(|| "off".into());
    let transfer = live
        .map(|live| {
            format!(
                "↓ {}   ↑ {}",
                wg::human_bytes(live.rx),
                wg::human_bytes(live.tx)
            )
        })
        .unwrap_or_else(|| "—".into());

    let mut card = panel(theme)
        .gap_3()
        .child(
            h_flex()
                .items_center()
                .gap_2()
                .child(div().size(px(7.)).rounded_full().bg(handshake_color))
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(format!("Peer {}", index + 1)),
                )
                .child(div().flex_1())
                .child(
                    div()
                        .text_xs()
                        .text_color(handshake_color)
                        .child(handshake_text),
                ),
        )
        .child(
            div()
                .text_xs()
                .font_family(theme.mono_font_family.clone())
                .text_color(theme.muted_foreground)
                .child(chunk_key(&peer.public_key)),
        )
        .child(
            h_flex()
                .gap_6()
                .flex_wrap()
                .child(stat(theme, "Endpoint", endpoint))
                .child(stat(theme, "Allowed IPs", allowed_ips))
                .child(stat(theme, "Keepalive", keepalive))
                .child(stat(theme, "Transfer", transfer)),
        );

    if peer.has_preshared_key {
        card = card.child(
            div()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child("Preshared key configured"),
        );
    }

    card.into_any_element()
}

pub(super) fn handshake_label(handshake: u64, now: u64, theme: &Theme) -> (String, Hsla) {
    if handshake == 0 {
        ("No handshake yet".into(), theme.muted_foreground)
    } else {
        let age = now.saturating_sub(handshake);
        let color = if age < 180 {
            theme.success
        } else if age < 600 {
            theme.warning
        } else {
            theme.muted_foreground
        };
        (
            format!("Handshake {}", wg::human_ago(handshake, now)),
            color,
        )
    }
}

pub(super) fn join_or_dash(values: &[String]) -> String {
    if values.is_empty() {
        "—".into()
    } else {
        values.join(", ")
    }
}

/// Break base64 keys into groups so they wrap inside the detail pane instead
/// of being clipped by the pane's width.
pub(super) fn chunk_key(key: &str) -> String {
    let mut chunks = Vec::new();
    let mut current = String::new();
    for (index, ch) in key.chars().enumerate() {
        if index > 0 && index % 16 == 0 {
            chunks.push(std::mem::take(&mut current));
        }
        current.push(ch);
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks.join(" ")
}
