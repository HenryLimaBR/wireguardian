use gpui_kit::assets::IconName;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::scroll::ScrollableElement as _;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::AppWindow;
use super::components::*;
use crate::wg::Source;

impl AppWindow {
    pub(super) fn render_details(&self, theme: &Theme, cx: &mut Context<Self>) -> AnyElement {
        let Some(tunnel) = self.selected_tunnel() else {
            let mut empty = v_flex()
                .flex_1()
                .h_full()
                .items_center()
                .justify_center()
                .gap_3()
                .child(
                    div()
                        .size(px(76.))
                        .rounded_xl()
                        .bg(theme.primary.opacity(0.12))
                        .border_1()
                        .border_color(theme.primary.opacity(0.25))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            Icon::new(IconName::ShieldCheck)
                                .size(px(34.))
                                .text_color(theme.primary),
                        ),
                )
                .child(
                    div()
                        .text_lg()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("No tunnel selected"),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(theme.muted_foreground)
                        .child("Pick a tunnel on the left, or import a configuration."),
                );
            if let Some(notice) = &self.notice {
                empty = empty.child(notice_banner(notice, theme));
            }
            return empty.into_any_element();
        };

        let live = self.live.get(&tunnel.name);
        let up = live.map(|live| live.up).unwrap_or(false);
        let busy = self.busy.as_deref() == Some(tunnel.name.as_str());
        let status_color = if up {
            theme.success
        } else {
            theme.muted_foreground
        };

        let toggle_name = tunnel.name.clone();
        let remove_name = tunnel.name.clone();
        let is_user = tunnel.source == Source::User;

        let mut body = v_flex().gap_4().p_6();

        if let Some(notice) = &self.notice {
            body = body.child(notice_banner(notice, theme));
        }

        if let Some(error) = &tunnel.parse_error {
            body = body.child(
                h_flex()
                    .items_start()
                    .gap_3()
                    .p_3()
                    .rounded_md()
                    .bg(theme.danger.opacity(0.1))
                    .border_1()
                    .border_color(theme.danger.opacity(0.25))
                    .child(
                        Icon::new(IconName::CircleAlert)
                            .size(px(16.))
                            .text_color(theme.danger),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_sm()
                            .text_color(theme.danger)
                            .child(format!("Configuration problem: {error}")),
                    ),
            );
        }

        body = body
            .child(self.render_connection_card(tunnel, live, up, theme))
            .child(self.render_interface_card(tunnel, theme))
            .child(self.render_peers(tunnel, live, up, theme));

        v_flex()
            .flex_1()
            .min_w_0()
            .h_full()
            .child(
                v_flex()
                    .flex_shrink_0()
                    .gap_2()
                    .px_6()
                    .py_5()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        h_flex()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .size(px(46.))
                                    .rounded_lg()
                                    .bg(theme.primary.opacity(0.12))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(
                                        Icon::new(IconName::ShieldCheck)
                                            .size(px(22.))
                                            .text_color(theme.primary),
                                    ),
                            )
                            .child(
                                div()
                                    .text_2xl()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .min_w_0()
                                    .truncate()
                                    .child(tunnel.name.clone()),
                            )
                            .child(chip(
                                theme,
                                if up { "Connected" } else { "Disconnected" },
                                status_color,
                            ))
                            .child(div().flex_1())
                            .child(
                                Button::new(SharedString::from(format!(
                                    "detail-toggle-{}",
                                    tunnel.name
                                )))
                                .icon(if up {
                                    IconName::PowerOff
                                } else {
                                    IconName::Power
                                })
                                .label(if up { "Disconnect" } else { "Connect" })
                                .loading(busy)
                                .disabled(self.busy.is_some())
                                .when(!up, |button| button.primary())
                                .when(up, |button| button.danger())
                                .on_click(cx.listener(
                                    move |this, _, _, cx| {
                                        this.toggle(&toggle_name, cx);
                                    },
                                )),
                            )
                            .when(is_user, |row| {
                                row.child(
                                    Button::new(SharedString::from(format!(
                                        "remove-{}",
                                        tunnel.name
                                    )))
                                    .ghost()
                                    .icon(IconName::Trash)
                                    .tooltip("Remove this personal config")
                                    .disabled(self.busy.is_some())
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.remove(&remove_name, cx);
                                    })),
                                )
                            }),
                    )
                    .child(
                        h_flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.muted_foreground)
                                    .child(tunnel.source.label()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.muted_foreground)
                                    .child("·"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.muted_foreground)
                                    .child(tunnel.path.display().to_string()),
                            ),
                    ),
            )
            .child(
                div()
                    .id("details-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scrollbar()
                    .child(body),
            )
            .into_any_element()
    }
}
