use std::{error::Error, process::Command};

use clap::{Parser, Subcommand, ValueEnum};

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

impl Xtask {
    fn run(self) -> Result<(), Box<dyn Error>> {
        match self.command {
            XtaskCommand::Demo {
                scenario,
                standalone,
            } => {
                if standalone {
                    run(Command::new("cargo").args(["run", "--package", scenario.package_name()]))
                } else {
                    run(Command::new("cargo").args([
                        "run",
                        "--bin",
                        "workbench",
                        "--",
                        "--open",
                        scenario.launch_arg(),
                    ]))
                }
            }
            XtaskCommand::Gallery { target } => run(Command::new("cargo").args([
                "run",
                "--package",
                "gallery",
                "--",
                "--open",
                target.launch_arg(),
            ])),
            XtaskCommand::Docs { open } => {
                let mut command = Command::new("cargo");
                command.args(["doc", "--workspace", "--all-features", "--no-deps"]);
                if open {
                    command.arg("--open");
                }
                run(command)
            }
        }
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
