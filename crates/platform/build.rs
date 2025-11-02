use std::{
    env,
    fs::{self, File},
    io::{self, ErrorKind, Write},
    path::{Path, PathBuf},
};

use example_plot::generate_accessibility_plot;

fn main() -> io::Result<()> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let fonts_dir = manifest_dir.join("assets/fonts");
    let images_dir = manifest_dir.join("assets/images");

    println!("cargo:rerun-if-changed={}", fonts_dir.display());
    println!("cargo:rerun-if-changed={}", images_dir.display());

    fs::create_dir_all(&images_dir)?;
    generate_accessibility_plot(images_dir.join("accessibility-checklist.png"))
        .map_err(|err| io::Error::new(ErrorKind::Other, err))?;

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("out dir"));
    let manifest_path = out_dir.join("platform_asset_manifest.rs");
    let mut manifest = File::create(&manifest_path)?;

    write_section(&mut manifest, &manifest_dir, &fonts_dir, "FONT_ASSETS")?;
    write_section(&mut manifest, &manifest_dir, &images_dir, "IMAGE_ASSETS")?;

    Ok(())
}

fn write_section(
    file: &mut File,
    manifest_root: &Path,
    section_dir: &Path,
    const_name: &str,
) -> io::Result<()> {
    writeln!(
        file,
        "pub(super) const {const_name}: &[crate::assets::AssetSpec] = &["
    )?;

    if section_dir.exists() {
        let entries = collect_assets(section_dir)?;
        for entry in entries {
            let logical_path = entry
                .strip_prefix(manifest_root.join("assets"))
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            let include_path = entry
                .strip_prefix(manifest_root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            writeln!(
                file,
                "    crate::assets::AssetSpec::new(\"{logical_path}\", include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/{include_path}\"))),"
            )?;
        }
    }

    writeln!(file, "];\n")?;
    Ok(())
}

fn collect_assets(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    if !dir.exists() {
        return Ok(files);
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            files.extend(collect_assets(&path)?);
        } else if path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| {
                matches!(
                    ext.to_lowercase().as_str(),
                    "svg" | "png" | "jpg" | "jpeg" | "otf" | "ttf" | "woff" | "woff2"
                )
            })
            .unwrap_or(false)
        {
            files.push(path);
        }
    }

    files.sort();
    Ok(files)
}
