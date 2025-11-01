//! Icon pipeline that converts SVG assets into an enum and runtime loader.

use std::{borrow::Cow, collections::HashMap, sync::LazyLock};

use gpui::{AssetSource, Result, SharedString};

include!(concat!(env!("OUT_DIR"), "/icon_data.rs"));

static ICON_SOURCE: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for icon in IconName::ALL {
        map.insert(icon.asset_path(), icon.svg());
    }
    map
});

static ICON_BY_NAME: LazyLock<HashMap<&'static str, IconName>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for (name, icon) in ICON_NAMES {
        map.insert(*name, *icon);
    }
    map
});

/// Asset source backed by the statically generated SVG table.
#[derive(Debug, Default, Clone, Copy)]
pub struct IconAssetSource;

impl AssetSource for IconAssetSource {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        Ok(ICON_SOURCE
            .get(path)
            .map(|svg| Cow::Borrowed(svg.as_bytes())))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        if path.is_empty() {
            return Ok(vec![SharedString::from_static("designsystem/icons")]);
        }

        if path == "designsystem/icons" {
            return Ok(IconName::ALL
                .iter()
                .map(|icon| SharedString::from(icon.asset_path()))
                .collect());
        }

        Ok(vec![])
    }
}

/// Utility helpers for working with statically embedded SVG icons.
#[derive(Debug, Default, Clone, Copy)]
pub struct IconLoader;

impl IconLoader {
    /// Returns an [`AssetSource`] implementation that exposes the embedded
    /// icons.
    pub const fn asset_source() -> IconAssetSource {
        IconAssetSource
    }

    /// Returns the canonical asset path for the icon.
    pub const fn asset_path(name: IconName) -> &'static str {
        name.asset_path()
    }

    /// Returns the inline SVG markup for the icon.
    pub const fn svg(name: IconName) -> &'static str {
        name.svg()
    }

    /// Returns the strongly typed icon by its file stem.
    pub fn resolve(name: &str) -> Option<IconName> {
        ICON_BY_NAME.get(name).copied()
    }

    /// Exposes all icon stems alongside their strongly typed names.
    pub fn all() -> &'static [(&'static str, IconName)] {
        ICON_NAMES
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_lookup_round_trips() {
        for (stem, name) in IconLoader::all() {
            let resolved = IconLoader::resolve(stem).expect("icon must resolve");
            assert_eq!(&resolved, name);
            assert!(IconLoader::svg(*name).contains("<svg"));
        }
    }
}
