use std::path::PathBuf;
use std::process::ExitCode;

use facet::Facet;
use figue::{self as args, FigueBuiltins};

#[derive(Facet)]
struct Cli {
    #[facet(args::subcommand)]
    command: Commands,

    #[facet(flatten)]
    builtins: FigueBuiltins,
}

#[derive(Facet)]
#[repr(u8)]
enum Commands {
    /// Download and install a toolchain
    Install {
        /// Version to install (e.g. 23.1, 22.1, 24.1)
        #[facet(args::positional, default = chipsmith_core::DEFAULT_LATEST.to_string())]
        version: String,

        /// Path to a local installer (skips download)
        #[facet(args::named)]
        installer: Option<PathBuf>,
    },

    /// Build the FPGA project in the current (or given) directory
    Build {
        /// Project directory containing chipsmith.toml (default: current dir)
        #[facet(args::named, default = PathBuf::from("."))]
        project_dir: PathBuf,
    },

    /// Run a toolchain tool directly
    Run {
        /// Tool name (e.g. quartus_sh, quartus_map)
        #[facet(args::positional)]
        tool: String,

        /// Version to use (default: latest)
        #[facet(args::named, default = chipsmith_core::DEFAULT_LATEST.to_string())]
        version: String,

        /// Arguments passed to the tool
        #[facet(args::positional)]
        args: Vec<String>,
    },

    /// Show the install path for a toolchain version
    Which {
        /// Version (default: latest)
        #[facet(args::positional, default = chipsmith_core::DEFAULT_LATEST.to_string())]
        version: String,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli: Cli = figue::from_std_args().unwrap();

    let result = match cli.command {
        Commands::Install { version, installer } => match installer {
            Some(path) => chipsmith_core::install_from_local("quartus-prime", &path, &version).await,
            None => chipsmith_core::install("quartus-prime", &version).await.map(|_| ()),
        },

        Commands::Build { project_dir } => chipsmith_core::build(&project_dir).await.map(|_| ()),

        Commands::Run { tool, version, args } => {
            chipsmith_core::run_tool("quartus-prime", &version, &tool, &args, None).await
        }

        Commands::Which { version } => match chipsmith_core::which("quartus-prime", &version) {
            Ok((dir, true)) => {
                println!("{}", dir.display());
                Ok(())
            }
            Ok((dir, false)) => {
                eprintln!("Not installed (would install to {})", dir.display());
                return ExitCode::FAILURE;
            }
            Err(e) => Err(e),
        },
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
