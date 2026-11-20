use gpui_kit::*;

mod appwindow;
mod assets;
mod theme;
mod wg;

fn main() {
    if std::env::args_os().nth(1).as_deref() == Some(std::ffi::OsStr::new("--read-system-configs"))
    {
        if let Err(err) = wg::run_config_reader() {
            eprintln!("{err}");
            std::process::exit(1);
        }
        return;
    }

    application().with_assets(assets::AppAssets).run(|cx| {
        init(cx);
        let (theme, theme_error) = match theme::load() {
            Ok(theme) => (theme, None),
            Err(error) => (theme::ThemePreset::default(), Some(error)),
        };
        theme::apply(theme, cx);

        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(1180.), px(780.)),
                cx,
            ))),
            titlebar: Some(TitlebarOptions {
                title: Some("WireGuardian".into()),
                ..Default::default()
            }),
            window_min_size: Some(size(px(900.), px(600.))),
            app_id: Some("io.github.HenryLimaBR.WireGuardian".into()),
            icon: Some(std::sync::Arc::new(
                image::load_from_memory(include_bytes!("../assets/wireguardian-icon.png"))
                    .expect("embedded window icon is a valid PNG")
                    .into_rgba8(),
            )),
            ..Default::default()
        };

        open_window(options, cx, |window, cx| {
            cx.new(|cx| appwindow::AppWindow::new(window, cx, theme, theme_error))
        })
        .expect("Failed to open Window");
    });
}
