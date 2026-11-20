use std::borrow::Cow;

use gpui_kit::{AssetSource, Result, SharedString, assets::AllAssets};

pub const LOGO: &str = "branding/wireguardian.png";

pub struct AppAssets;

impl AssetSource for AppAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path == LOGO {
            Ok(Some(Cow::Borrowed(include_bytes!(
                "../assets/wireguardian-logo.png"
            ))))
        } else {
            AllAssets.load(path)
        }
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut assets = AllAssets.list(path)?;
        if LOGO.starts_with(path) {
            assets.push(LOGO.into());
        }
        Ok(assets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serves_logo_and_existing_icons() {
        let assets = AppAssets;
        let logo = assets.load(LOGO).unwrap().unwrap();
        assert!(logo.starts_with(b"\x89PNG\r\n\x1a\n"));
        assert!(assets.list("branding/").unwrap().contains(&LOGO.into()));
        assert!(assets.load("icons/shield-check.svg").unwrap().is_some());
        assert!(
            assets
                .list("icons/")
                .unwrap()
                .contains(&"icons/shield-check.svg".into())
        );
    }
}
