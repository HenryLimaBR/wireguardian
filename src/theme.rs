use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use gpui_kit::component::{Theme, ThemeMode};
use gpui_kit::{App, Hsla, px, rgb};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ThemePreset {
    #[default]
    Dark,
    Cyberpunk,
}

impl ThemePreset {
    fn name(self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Cyberpunk => "cyberpunk",
        }
    }

    fn palette(self) -> Palette {
        match self {
            Self::Dark => Palette {
                background: 0x0b1220,
                foreground: 0xeaf2fc,
                sidebar: 0x101a2a,
                border: 0x2a3a50,
                muted: 0x9caec4,
                surface: 0x152235,
                hover: 0x1d3046,
                active: 0x1c3852,
                primary: 0x69c4ff,
                primary_foreground: 0x071626,
                primary_hover: 0x90d3ff,
                primary_active: 0x46aaec,
                button: 0x1c2b3f,
                button_hover: 0x283c53,
                button_active: 0x304960,
                input: 0x344961,
                info: 0x8bcfff,
                success: 0x5be0a4,
                warning: 0xf6c66b,
                danger: 0xff8795,
            },
            Self::Cyberpunk => Palette {
                background: 0x0d0918,
                foreground: 0xf8f2ff,
                sidebar: 0x160d25,
                border: 0x67417b,
                muted: 0xbeb0ce,
                surface: 0x211432,
                hover: 0x322047,
                active: 0x3c2150,
                primary: 0xf6e45e,
                primary_foreground: 0x171121,
                primary_hover: 0xfff490,
                primary_active: 0xd8c73e,
                button: 0x301e42,
                button_hover: 0x44275a,
                button_active: 0x563270,
                input: 0x76538b,
                info: 0xff83df,
                success: 0x65efda,
                warning: 0xffdf78,
                danger: 0xff8b9b,
            },
        }
    }
}

struct Palette {
    background: u32,
    foreground: u32,
    sidebar: u32,
    border: u32,
    muted: u32,
    surface: u32,
    hover: u32,
    active: u32,
    primary: u32,
    primary_foreground: u32,
    primary_hover: u32,
    primary_active: u32,
    button: u32,
    button_hover: u32,
    button_active: u32,
    input: u32,
    info: u32,
    success: u32,
    warning: u32,
    danger: u32,
}

pub fn apply(preset: ThemePreset, cx: &mut App) {
    let palette = preset.palette();
    Theme::change(ThemeMode::Dark, None, cx);
    Theme::update(cx, |theme| {
        let colors = &mut theme.colors;
        colors.background = Hsla::from(rgb(palette.background));
        colors.foreground = Hsla::from(rgb(palette.foreground));
        colors.sidebar = Hsla::from(rgb(palette.sidebar));
        colors.border = Hsla::from(rgb(palette.border));
        colors.muted_foreground = Hsla::from(rgb(palette.muted));
        colors.list = Hsla::from(rgb(palette.surface));
        colors.list_hover = Hsla::from(rgb(palette.hover));
        colors.list_active = Hsla::from(rgb(palette.active));
        colors.primary = Hsla::from(rgb(palette.primary));
        colors.primary_foreground = Hsla::from(rgb(palette.primary_foreground));
        colors.primary_hover = Hsla::from(rgb(palette.primary_hover));
        colors.primary_active = Hsla::from(rgb(palette.primary_active));
        colors.button_primary = colors.primary;
        colors.button_primary_foreground = colors.primary_foreground;
        colors.button_primary_hover = colors.primary_hover;
        colors.button_primary_active = colors.primary_active;
        colors.button = Hsla::from(rgb(palette.button));
        colors.button_hover = Hsla::from(rgb(palette.button_hover));
        colors.button_active = Hsla::from(rgb(palette.button_active));
        colors.button_foreground = colors.foreground;
        colors.input = Hsla::from(rgb(palette.input));
        colors.ring = colors.primary;
        colors.info = Hsla::from(rgb(palette.info));
        colors.success = Hsla::from(rgb(palette.success));
        colors.warning = Hsla::from(rgb(palette.warning));
        colors.danger = Hsla::from(rgb(palette.danger));
        theme.radius = px(if preset == ThemePreset::Cyberpunk {
            3.
        } else {
            8.
        });
        theme.radius_lg = px(if preset == ThemePreset::Cyberpunk {
            6.
        } else {
            12.
        });
    });
}

fn preference_path() -> PathBuf {
    crate::wg::config_home().join("wireguardian").join("theme")
}

pub fn load() -> Result<ThemePreset, String> {
    let path = preference_path();
    load_from(&path)
        .map_err(|error| format!("Could not load theme from {}: {error}", path.display()))
}

fn load_from(path: &Path) -> io::Result<ThemePreset> {
    let value = match fs::read_to_string(path) {
        Ok(value) => value,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(ThemePreset::default()),
        Err(error) => return Err(error),
    };
    match value.trim() {
        "dark" => Ok(ThemePreset::Dark),
        "cyberpunk" => Ok(ThemePreset::Cyberpunk),
        _ => Err(io::Error::new(io::ErrorKind::InvalidData, "unknown theme")),
    }
}

pub fn save(preset: ThemePreset) -> Result<(), String> {
    let path = preference_path();
    save_to(&path, preset)
        .map_err(|error| format!("Could not save theme to {}: {error}", path.display()))
}

fn save_to(path: &Path, preset: ThemePreset) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, preset.name())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saves_loads_and_validates_theme_preference() {
        let dir = std::env::temp_dir().join(format!(
            "wireguardian-theme-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let path = dir.join("theme");
        assert_eq!(load_from(&path).unwrap(), ThemePreset::Dark);
        save_to(&path, ThemePreset::Cyberpunk).unwrap();
        assert_eq!(load_from(&path).unwrap(), ThemePreset::Cyberpunk);
        save_to(&path, ThemePreset::Dark).unwrap();
        assert_eq!(load_from(&path).unwrap(), ThemePreset::Dark);
        fs::write(&path, "invalid").unwrap();
        assert_eq!(
            load_from(&path).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        assert!(save_to(&dir, ThemePreset::Dark).is_err());
        fs::remove_file(path).unwrap();
        fs::remove_dir(dir).unwrap();
    }

    fn contrast(foreground: u32, background: u32) -> f64 {
        fn luminance(color: u32) -> f64 {
            let channel = |shift: u32| {
                let value = f64::from((color >> shift) & 0xff_u32) / 255.0;
                if value <= 0.04045 {
                    value / 12.92
                } else {
                    ((value + 0.055) / 1.055).powf(2.4)
                }
            };
            0.2126 * channel(16) + 0.7152 * channel(8) + 0.0722 * channel(0)
        }
        let a = luminance(foreground);
        let b = luminance(background);
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }

    #[test]
    fn themes_keep_text_legible() {
        for preset in [ThemePreset::Dark, ThemePreset::Cyberpunk] {
            let p = preset.palette();
            for (foreground, background) in [
                (p.foreground, p.background),
                (p.foreground, p.surface),
                (p.muted, p.sidebar),
                (p.muted, p.active),
                (p.primary_foreground, p.primary),
                (p.info, p.background),
                (p.success, p.surface),
                (p.warning, p.sidebar),
                (p.danger, p.background),
            ] {
                assert!(
                    contrast(foreground, background) >= 4.5,
                    "{preset:?}: {foreground:06x} on {background:06x}"
                );
            }
        }
    }
}
