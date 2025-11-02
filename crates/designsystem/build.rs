use std::{
    env,
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

use heck::ToUpperCamelCase;

fn main() -> io::Result<()> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let icons_dir = Path::new(&manifest_dir).join("icons");
    println!("cargo:rerun-if-changed={}", icons_dir.display());

    let mut entries: Vec<_> = fs::read_dir(&icons_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .map(|ext| ext == "svg")
                .unwrap_or(false)
        })
        .collect();

    entries.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("icon_data.rs");
    let mut file = File::create(dest_path)?;

    writeln!(
        file,
        "/// Strongly typed identifier for a design system icon.\n#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]\npub enum IconName {{"
    )?;

    for entry in &entries {
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        let stem = Path::new(&*file_name)
            .file_stem()
            .unwrap()
            .to_string_lossy();
        let variant = stem.replace('-', " ").to_upper_camel_case();
        writeln!(file, "    /// Icon asset generated from `{stem}.svg`.")?;
        writeln!(file, "    {variant},")?;
    }

    writeln!(file, "}}\n")?;

    writeln!(
        file,
        "#[allow(missing_docs)]\nimpl IconName {{\n    pub const ALL: &'static [IconName] = &["
    )?;
    for entry in &entries {
        let file_name = entry.file_name();
        let stem = Path::new(&file_name).file_stem().unwrap().to_string_lossy();
        let variant = stem.replace('-', " ").to_upper_camel_case();
        writeln!(file, "        IconName::{variant},")?;
    }
    writeln!(file, "    ];\n")?;

    writeln!(file, "    pub const fn asset_path(self) -> &'static str {{")?;
    writeln!(file, "        match self {{")?;
    for entry in &entries {
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        let stem = Path::new(&*file_name)
            .file_stem()
            .unwrap()
            .to_string_lossy();
        let variant = stem.replace('-', " ").to_upper_camel_case();
        writeln!(
            file,
            "            IconName::{variant} => \"designsystem/icons/{file_name}\","
        )?;
    }
    writeln!(file, "        }}\n    }}\n")?;

    writeln!(file, "    pub const fn svg(self) -> &'static str {{")?;
    writeln!(file, "        match self {{")?;
    for entry in &entries {
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        let stem = Path::new(&*file_name)
            .file_stem()
            .unwrap()
            .to_string_lossy();
        let variant = stem.replace('-', " ").to_upper_camel_case();
        let icon_path = icons_dir
            .strip_prefix(&manifest_dir)
            .unwrap()
            .join(&*file_name);
        writeln!(
            file,
            "            IconName::{variant} => include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/{}\")),",
            icon_path.display()
        )?;
    }
    writeln!(
        file,
        "        }}\n    }}\n}}
"
    )?;

    writeln!(
        file,
        "/// Lookup table that maps icon stems to strongly typed names.\npub(crate) const ICON_NAMES: &[(&'static str, IconName)] = &["
    )?;
    for entry in &entries {
        let file_name = entry.file_name();
        let stem = Path::new(&file_name).file_stem().unwrap().to_string_lossy();
        let variant = stem.replace('-', " ").to_upper_camel_case();
        writeln!(file, "    (\"{stem}\", IconName::{variant}),")?;
    }
    writeln!(file, "];\n")?;

    Ok(())
}
