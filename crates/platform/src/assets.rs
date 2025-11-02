//! Asset bundling helpers for fonts and images that ship with the workspace.

/// Metadata describing an embedded asset file.
#[derive(Debug, Clone, Copy)]
pub struct AssetSpec {
    /// Logical path that the asset should be exposed under at runtime.
    pub logical_path: &'static str,
    /// Raw embedded bytes for the asset.
    pub bytes: &'static [u8],
}

impl AssetSpec {
    /// Creates a new specification for an embedded asset.
    pub const fn new(logical_path: &'static str, bytes: &'static [u8]) -> Self {
        Self { logical_path, bytes }
    }

    /// Returns the logical path used when installing the asset source.
    #[must_use]
    pub const fn logical_path(self) -> &'static str {
        self.logical_path
    }

    /// Returns the embedded bytes for the asset.
    #[must_use]
    pub const fn bytes(self) -> &'static [u8] {
        self.bytes
    }
}

/// Grouping of embedded font and image assets used by the platform crate.
#[derive(Debug, Clone, Copy)]
pub struct AssetBundle {
    fonts: &'static [AssetSpec],
    images: &'static [AssetSpec],
}

impl AssetBundle {
    /// Returns the embedded font assets.
    #[must_use]
    pub const fn fonts(self) -> &'static [AssetSpec] {
        self.fonts
    }

    /// Returns the embedded image assets.
    #[must_use]
    pub const fn images(self) -> &'static [AssetSpec] {
        self.images
    }

    /// Registers the assets with the provided callbacks.
    pub fn register_with<F, I>(&self, mut font_loader: F, mut image_loader: I)
    where
        F: FnMut(&str, &'static [u8]),
        I: FnMut(&str, &'static [u8]),
    {
        for font in self.fonts {
            font_loader(font.logical_path, font.bytes);
        }
        for image in self.images {
            image_loader(image.logical_path, image.bytes);
        }
    }

}

include!(concat!(env!("OUT_DIR"), "/platform_asset_manifest.rs"));

/// Collection of assets bundled with the platform crate.
pub const EMBEDDED_ASSETS: AssetBundle = AssetBundle {
    fonts: FONT_ASSETS,
    images: IMAGE_ASSETS,
};
