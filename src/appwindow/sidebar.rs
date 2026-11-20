use gpui_kit::assets::IconName;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::Input;
use gpui_kit::component::scroll::ScrollableElement as _;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::{AppWindow, SIDEBAR_WIDTH};
use crate::wg::Tunnel;

impl AppWindow {
    pub(super) fn render_sidebar(&self, theme: &Theme, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w(px(SIDEBAR_WIDTH))
            .flex_shrink_0()
            .h_full()
            .border_r_1()
            .border_color(theme.border)
            .bg(theme.sidebar)
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .px_4()
                    .pt_5()
                    .pb_3()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Your tunnels"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child(format!("{} configured", self.tunnels.len())),
                    ),
            )
            .child(self.render_tunnel_list(theme, cx))
            .child(self.render_import(theme, cx))
    }

    fn render_tunnel_list(&self, theme: &Theme, cx: &mut Context<Self>) -> impl IntoElement {
        let mut list = v_flex().gap_2().px_3().pb_3();

        if let Some(error) = &self.system_error {
            list = list.child(
                h_flex()
                    .items_start()
                    .gap_2()
                    .p_2()
                    .rounded_md()
                    .bg(theme.warning.opacity(0.1))
                    .border_1()
                    .border_color(theme.warning.opacity(0.25))
                    .child(
                        Icon::new(IconName::Info)
                            .size(px(14.))
                            .text_color(theme.warning),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_xs()
                            .text_color(theme.warning)
                            .child(format!("System configs unavailable: {error}")),
                    ),
            );
        }

        if let Some(error) = &self.system_auth_error {
            list = list.child(
                div()
                    .p_2()
                    .text_xs()
                    .text_color(theme.danger)
                    .child(error.clone()),
            );
        }

        if self.system_requires_privilege || self.system_configs.is_some() {
            list = list.child(
                Button::new("unlock-system-configs")
                    .small()
                    .label(if self.unlocking_system_configs {
                        "Requesting access…"
                    } else if self.system_configs.is_some() {
                        "Reload system configs"
                    } else {
                        "Unlock system configs"
                    })
                    .disabled(self.unlocking_system_configs)
                    .on_click(cx.listener(|this, _, _, cx| this.unlock_system_configs(cx))),
            );
            if self.system_configs.is_some() {
                list = list.child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child("Reload after changing system configurations."),
                );
            }
        }

        if self.tunnels.is_empty() {
            list = list.child(
                v_flex()
                    .items_center()
                    .gap_2()
                    .py_8()
                    .child(
                        Icon::new(IconName::Shield)
                            .size(px(28.))
                            .text_color(theme.muted_foreground),
                    )
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("No tunnels yet"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child("Import a .conf file below."),
                    ),
            );
        }

        for tunnel in &self.tunnels {
            list = list.child(self.render_tunnel_row(tunnel, theme, cx));
        }

        div()
            .id("tunnel-list")
            .flex_1()
            .min_h_0()
            .overflow_y_scrollbar()
            .child(list)
    }

    fn render_tunnel_row(
        &self,
        tunnel: &Tunnel,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let live = self.live.get(&tunnel.name);
        let up = live.map(|live| live.up).unwrap_or(false);
        let selected = self.selected.as_deref() == Some(tunnel.name.as_str());
        let busy = self.busy.as_deref() == Some(tunnel.name.as_str());
        let status_color = if up {
            theme.success
        } else {
            theme.muted_foreground
        };
        let endpoint = tunnel
            .primary_endpoint()
            .map(str::to_owned)
            .unwrap_or_else(|| {
                format!(
                    "{} peer{}",
                    tunnel.peers.len(),
                    if tunnel.peers.len() == 1 { "" } else { "s" }
                )
            });
        let detail = format!("{endpoint} · {}", tunnel.source.label());

        let select_name = tunnel.name.clone();
        let toggle_name = tunnel.name.clone();

        div()
            .id(SharedString::from(format!("row-{}", tunnel.name)))
            .cursor_pointer()
            .p_3()
            .rounded_xl()
            .border_1()
            .border_color(if selected {
                theme.primary.opacity(0.45)
            } else {
                theme.border
            })
            .bg(if selected {
                theme.colors.list_active
            } else {
                theme.colors.list
            })
            .hover(|style| style.bg(theme.colors.list_hover))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.selected = Some(select_name.clone());
                cx.notify();
            }))
            .child(
                h_flex()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .size(px(38.))
                            .rounded_lg()
                            .bg(if selected {
                                theme.primary.opacity(0.15)
                            } else {
                                theme.background
                            })
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(Icon::new(IconName::Shield).size(px(18.)).text_color(
                                if selected {
                                    theme.primary
                                } else {
                                    status_color
                                },
                            )),
                    )
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .gap_1()
                            .child(
                                h_flex()
                                    .items_center()
                                    .justify_between()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .truncate()
                                            .child(tunnel.name.clone()),
                                    )
                                    .child(div().text_xs().text_color(status_color).child(if up {
                                        "UP"
                                    } else {
                                        "DOWN"
                                    })),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.muted_foreground)
                                    .truncate()
                                    .child(detail),
                            ),
                    )
                    .child(
                        Button::new(SharedString::from(format!("toggle-{}", tunnel.name)))
                            .small()
                            .icon(IconName::Power)
                            .loading(busy)
                            .disabled(self.busy.is_some())
                            .tooltip(if up { "Disconnect" } else { "Connect" })
                            .when(!up, |button| button.primary())
                            .when(up, |button| button.secondary())
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.toggle(&toggle_name, cx);
                            })),
                    ),
            )
    }

    fn render_import(&self, theme: &Theme, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_3()
            .p_4()
            .border_t_1()
            .border_color(theme.border)
            .bg(theme.sidebar)
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.muted_foreground)
                    .child("IMPORT CONFIGURATION"),
            )
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .child(Input::new(&self.import_input).small()),
                    )
                    .child(
                        Button::new("import")
                            .small()
                            .icon(IconName::Plus)
                            .label("Import")
                            .loading(self.busy.as_deref() == Some("__import"))
                            .disabled(self.busy.is_some())
                            .on_click(cx.listener(|this, _, _, cx| this.import(cx))),
                    ),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .truncate()
                    .child(format!("Copied to {}", self.user_dir.display())),
            )
    }
}
