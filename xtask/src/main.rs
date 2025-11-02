use std::{
    env,
    error::Error,
    fs,
    io::{self, Read},
    path::PathBuf,
    process::Command,
};

use clap::{Parser, Subcommand, ValueEnum};
use heck::ToKebabCase;

fn main() {
    if let Err(err) = Xtask::parse().run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

#[derive(Debug, Parser)]
#[command(
    name = "xtask",
    about = "Workspace utilities for gpui-component demos",
    version
)]
struct Xtask {
    #[command(subcommand)]
    command: XtaskCommand,
}

#[derive(Debug, Subcommand)]
enum XtaskCommand {
    /// Launch a demo either inside the workbench shell or as a standalone
    /// binary.
    Demo {
        #[arg(value_enum)]
        scenario: DemoScenario,
        /// Run the standalone binary instead of routing through the workbench
        /// shell.
        #[arg(long)]
        standalone: bool,
    },
    /// Focus the gallery on a specific category or quick-start overlay.
    Gallery {
        #[arg(value_enum)]
        target: GalleryScenario,
    },
    /// Build API documentation for the entire workspace.
    Docs {
        /// Pass --open to cargo doc so rustdoc opens a browser window locally.
        #[arg(long)]
        open: bool,
    },
    /// Normalize raw SVGs into the design system's icon pack.
    Icons {
        /// Directory containing raw SVG assets.
        input: PathBuf,
        /// Target pack that determines the output prefix.
        #[arg(long, value_enum, default_value_t = IconPack::Product)]
        pack: IconPack,
        /// Remove existing icons for the selected pack before importing.
        #[arg(long)]
        clean: bool,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum DemoScenario {
    #[value(name = "data-explorer")]
    DataExplorer,
    #[value(name = "markdown-notes")]
    MarkdownNotes,
    #[value(name = "code-playground")]
    CodePlayground,
    #[value(name = "operations-dashboard")]
    OperationsDashboard,
    #[value(name = "webview")]
    WebviewDocs,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum GalleryScenario {
    #[value(name = "category=inputs")]
    Inputs,
    #[value(name = "category=navigation")]
    Navigation,
    #[value(name = "category=feedback")]
    Feedback,
    #[value(name = "category=data")]
    Data,
    #[value(name = "category=layout")]
    Layout,
    #[value(name = "category=overlays")]
    Overlays,
    #[value(name = "category=editors")]
    Editors,
    #[value(name = "palette")]
    PaletteOverlay,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum IconPack {
    Core,
    Product,
}

impl IconPack {
    fn prefix(self) -> &'static str {
        match self {
            Self::Core => "core",
            Self::Product => "product",
        }
    }
}

impl Xtask {
    fn run(self) -> Result<(), Box<dyn Error>> {
        match self.command {
            XtaskCommand::Demo {
                scenario,
                standalone,
            } => {
                let mut command = Command::new("cargo");
                if standalone {
                    command.args(["run", "--package", scenario.package_name()]);
                } else {
                    command.args([
                        "run",
                        "--bin",
                        "workbench",
                        "--",
                        "--open",
                        scenario.launch_arg(),
                    ]);
                }
                run(command)
            }
            XtaskCommand::Gallery { target } => {
                let mut command = Command::new("cargo");
                command.args([
                    "run",
                    "--package",
                    "gallery",
                    "--",
                    "--open",
                    target.launch_arg(),
                ]);
                run(command)
            }
            XtaskCommand::Docs { open } => {
                let mut command = Command::new("cargo");
                command.args(["doc", "--workspace", "--all-features", "--no-deps"]);
                if open {
                    command.arg("--open");
                }
                run(command)
            }
            XtaskCommand::Icons { input, pack, clean } => self.import_icons(input, pack, clean),
        }
    }
}

impl Xtask {
    fn import_icons(
        &self,
        input: PathBuf,
        pack: IconPack,
        clean: bool,
    ) -> Result<(), Box<dyn Error>> {
        let input_dir = if input.is_relative() {
            std::env::current_dir()?.join(input)
        } else {
            input
        };

        if !input_dir.exists() {
            return Err(format!("input directory '{}' not found", input_dir.display()).into());
        }

        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let workspace_root = manifest_dir
            .parent()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "workspace root"))?
            .to_path_buf();
        let dest_dir = workspace_root.join("crates/designsystem/icons");

        fs::create_dir_all(&dest_dir)?;

        if clean {
            for entry in fs::read_dir(&dest_dir)? {
                let entry = entry?;
                if entry
                    .path()
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.starts_with(pack.prefix()))
                    .unwrap_or(false)
                {
                    fs::remove_file(entry.path())?;
                }
            }
        }

        let mut imported = 0usize;
        for entry in fs::read_dir(&input_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("svg") {
                continue;
            }
            let mut svg = String::new();
            fs::File::open(&path)?.read_to_string(&mut svg)?;
            let normalized = normalize_svg(&svg);
            let stem = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or("icon")
                .to_kebab_case();
            let filename = format!("{}-{}.svg", pack.prefix(), stem);
            let dest_path = dest_dir.join(filename);
            fs::write(&dest_path, normalized)?;
            imported += 1;
        }

        println!(
            "Imported {imported} {} icon(s) into {}",
            pack.prefix(),
            dest_dir.display()
        );

        Ok(())
    }
}

impl DemoScenario {
    fn package_name(self) -> &'static str {
        match self {
            Self::DataExplorer => "data_explorer",
            Self::MarkdownNotes => "markdown_notes",
            Self::CodePlayground => "code_playground",
            Self::OperationsDashboard => "dashboard",
            Self::WebviewDocs => "webview",
        }
    }

    fn launch_arg(self) -> &'static str {
        match self {
            Self::DataExplorer => "demo=data-explorer",
            Self::MarkdownNotes => "demo=markdown-notes",
            Self::CodePlayground => "demo=code-playground",
            Self::OperationsDashboard => "demo=operations-dashboard",
            Self::WebviewDocs => "demo=webview",
        }
    }
}

impl GalleryScenario {
    fn launch_arg(self) -> &'static str {
        match self {
            Self::Inputs => "category=inputs",
            Self::Navigation => "category=navigation",
            Self::Feedback => "category=feedback",
            Self::Data => "category=data",
            Self::Layout => "category=layout",
            Self::Overlays => "category=overlays",
            Self::Editors => "category=editors",
            Self::PaletteOverlay => "palette",
        }
    }
}

fn run(mut command: Command) -> Result<(), Box<dyn Error>> {
    let status = command.status()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("command exited with status {status}").into())
    }
}

fn normalize_svg(source: &str) -> String {
    let mut svg = source.trim().to_string();
    if !svg.contains("fill=\"none\"") {
        svg = svg.replacen("<svg", "<svg fill=\"none\"", 1);
    }
    if !svg.contains("stroke=\"currentColor\"") {
        svg = svg.replacen("<svg", "<svg stroke=\"currentColor\"", 1);
    }
    if !svg.contains("stroke-width=") {
        svg = svg.replacen("<svg", "<svg stroke-width=\"2\"", 1);
    }
    if !svg.contains("stroke-linecap=") {
        svg = svg.replacen("<svg", "<svg stroke-linecap=\"round\"", 1);
    }
    if !svg.contains("stroke-linejoin=") {
        svg = svg.replacen("<svg", "<svg stroke-linejoin=\"round\"", 1);
    }
    if !svg.ends_with('\n') {
        svg.push('\n');
    }
    svg
}
