use gpui_kit::assets::IconName;
use gpui_kit::component::button::Button;
use gpui_kit::component::*;
use gpui_kit::*;

use super::AppWindow;
use super::components::chip;
use crate::wg;

impl AppWindow {
    pub(super) fn render_header(&self, theme: &Theme, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .h(px(76.))
            .flex_shrink_0()
            .items_center()
            .gap_2()
            .px_4()
            .border_b_1()
            .border_color(theme.border)
            .bg(theme.background)
            .child(img(crate::assets::LOGO).size(px(64.)))
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("WireGuardian"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child("WIREGUARD CONTROL CENTER"),
                    ),
            )
            .child(div().flex_1())
            .child(chip(
                theme,
                if self.wg_quick_available {
                    "wg-quick ready"
                } else {
                    "wg-quick missing"
                },
                if self.wg_quick_available {
                    theme.success
                } else {
                    theme.danger
                },
            ))
            .child(chip(
                theme,
                self.privilege.label(),
                if self.privilege.can_elevate() {
                    theme.info
                } else {
                    theme.warning
                },
            ))
            .child(
                Button::new("refresh")
                    .small()
                    .icon(IconName::RefreshCw)
                    .label("Refresh")
                    .on_click(cx.listener(|this, _, _, cx| this.refresh_now(cx))),
            )
            .child(self.render_settings(cx))
    }

    pub(super) fn render_footer(&self, theme: &Theme) -> impl IntoElement {
        h_flex()
            .h(px(34.))
            .flex_shrink_0()
            .items_center()
            .justify_between()
            .gap_4()
            .px_4()
            .border_t_1()
            .border_color(theme.border)
            .bg(theme.sidebar)
            .child(
                h_flex()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child(format!(
                                "wg-quick: {}",
                                if self.wg_quick_available {
                                    "found"
                                } else {
                                    "missing"
                                }
                            )),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child(format!(
                                "wg: {}",
                                if self.wg_available {
                                    "found"
                                } else {
                                    "missing"
                                }
                            )),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child(format!("privilege: {}", self.privilege.label())),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .truncate()
                            .child(format!("configs: {}", self.user_dir.display())),
                    ),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .child(format!(
                        "updated {}",
                        wg::human_ago(self.last_refresh, wg::now_epoch())
                    )),
            )
    }
}
