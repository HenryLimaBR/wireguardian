mod actions;
mod cards;
mod chrome;
mod components;
mod details;
mod settings;
mod sidebar;

use std::collections::HashMap;
use std::path::PathBuf;

use gpui_kit::component::input::InputState;
use gpui_kit::component::*;
use gpui_kit::*;

use crate::theme::ThemePreset;
use crate::wg::{self, LiveStatus, Privilege, Tunnel};

const SIDEBAR_WIDTH: f32 = 320.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum NoticeKind {
    Info,
    Success,
    Error,
}

struct Notice {
    kind: NoticeKind,
    text: String,
}

impl Notice {
    fn info(text: impl Into<String>) -> Self {
        Self {
            kind: NoticeKind::Info,
            text: text.into(),
        }
    }

    fn success(text: impl Into<String>) -> Self {
        Self {
            kind: NoticeKind::Success,
            text: text.into(),
        }
    }

    fn error(text: impl Into<String>) -> Self {
        Self {
            kind: NoticeKind::Error,
            text: text.into(),
        }
    }
}

pub struct AppWindow {
    tunnels: Vec<Tunnel>,
    live: HashMap<String, LiveStatus>,
    selected: Option<String>,
    privilege: Privilege,
    busy: Option<String>,
    notice: Option<Notice>,
    import_input: Entity<InputState>,
    theme_preset: ThemePreset,
    theme_error: Option<String>,
    last_refresh: u64,
    system_error: Option<String>,
    system_auth_error: Option<String>,
    system_requires_privilege: bool,
    system_configs: Option<Vec<Tunnel>>,
    system_configs_generation: u64,
    unlocking_system_configs: bool,
    wg_available: bool,
    wg_quick_available: bool,
    user_dir: PathBuf,
}

impl AppWindow {
    pub fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
        theme_preset: ThemePreset,
        theme_error: Option<String>,
    ) -> Self {
        let import_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("Path to a .conf file"));

        let mut this = Self {
            tunnels: Vec::new(),
            live: HashMap::new(),
            selected: None,
            privilege: Privilege::Unavailable,
            busy: None,
            notice: None,
            import_input,
            theme_preset,
            theme_error,
            last_refresh: 0,
            system_error: None,
            system_auth_error: None,
            system_requires_privilege: false,
            system_configs: None,
            system_configs_generation: 0,
            unlocking_system_configs: false,
            wg_available: false,
            wg_quick_available: false,
            user_dir: wg::user_tunnel_dir(),
        };
        this.apply_snapshot(wg::snapshot(None), cx);
        if this.system_requires_privilege {
            this.unlock_system_configs(cx);
        }
        this.start_refresh_loop(cx);
        this
    }
}

impl Render for AppWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();

        v_flex()
            .size_full()
            .bg(theme.background)
            .text_color(theme.foreground)
            .child(self.render_header(&theme, cx))
            .child(
                h_flex()
                    .flex_1()
                    .min_h_0()
                    .child(self.render_sidebar(&theme, cx))
                    .child(self.render_details(&theme, cx)),
            )
            .child(self.render_footer(&theme))
    }
}
