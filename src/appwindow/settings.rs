use gpui_kit::assets::IconName;
use gpui_kit::component::Sizable;
use gpui_kit::component::button::Button;
use gpui_kit::component::menu::{DropdownMenu, PopupMenuItem};
use gpui_kit::*;

use super::AppWindow;
use crate::theme::ThemePreset;

impl AppWindow {
    pub(super) fn render_settings(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let current = self.theme_preset;
        let error = self.theme_error.clone();
        let app = cx.entity().downgrade();
        Button::new("settings")
            .small()
            .icon(IconName::Settings)
            .label(if error.is_some() {
                "Settings !"
            } else {
                "Settings"
            })
            .dropdown_menu(move |mut menu, _, _| {
                menu = menu.label("Theme");
                for (preset, label) in [
                    (ThemePreset::Dark, "Dark"),
                    (ThemePreset::Cyberpunk, "Cyberpunk"),
                ] {
                    let app = app.clone();
                    menu = menu.item(
                        PopupMenuItem::new(label)
                            .checked(preset == current)
                            .on_click(move |_, _, cx| {
                                let _ = app.update(cx, |this, cx| this.change_theme(preset, cx));
                            }),
                    );
                }
                if let Some(error) = &error {
                    menu = menu.separator().label(error.clone());
                }
                menu
            })
    }
}
