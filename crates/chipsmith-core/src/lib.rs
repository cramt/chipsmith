pub use chipsmith_toolchain::{error, manifest, toolchain};

use std::path::{Path, PathBuf};

use error::ChipsmithError;
use manifest::Manifest;
use toolchain::Toolchain;

use chipsmith_quartus_prime::QuartusPrimeToolchain;
use chipsmith_quartus_ii_13::QuartusII13Toolchain;

pub const DEFAULT_LATEST: &str = chipsmith_quartus_prime::LATEST;

enum AnyToolchain {
    QuartusPrime(QuartusPrimeToolchain),
    QuartusII13(QuartusII13Toolchain),
}

impl AnyToolchain {
    async fn ensure_installed(&self, version: &str) -> Result<PathBuf, ChipsmithError> {
        match self {
            Self::QuartusPrime(t) => t.ensure_installed(version).await,
            Self::QuartusII13(t) => t.ensure_installed(version).await,
        }
    }

    async fn run_tool(
        &self,
        version: &str,
        tool: &str,
        args: &[String],
        working_dir: Option<&Path>,
    ) -> Result<(), ChipsmithError> {
        match self {
            Self::QuartusPrime(t) => t.run_tool(version, tool, args, working_dir).await,
            Self::QuartusII13(t) => t.run_tool(version, tool, args, working_dir).await,
        }
    }

    async fn build(&self, project_dir: &Path, manifest: &Manifest) -> Result<PathBuf, ChipsmithError> {
        match self {
            Self::QuartusPrime(t) => t.build(project_dir, manifest).await,
            Self::QuartusII13(t) => t.build(project_dir, manifest).await,
        }
    }

    fn install_dir(&self, version: &str) -> Result<PathBuf, ChipsmithError> {
        match self {
            Self::QuartusPrime(t) => t.install_dir(version),
            Self::QuartusII13(t) => t.install_dir(version),
        }
    }

    fn is_installed(&self, version: &str) -> Result<bool, ChipsmithError> {
        match self {
            Self::QuartusPrime(t) => t.is_installed(version),
            Self::QuartusII13(t) => t.is_installed(version),
        }
    }
}

fn resolve_backend(name: &str) -> Result<AnyToolchain, ChipsmithError> {
    match name {
        "quartus-prime" => Ok(AnyToolchain::QuartusPrime(QuartusPrimeToolchain)),
        "quartus-ii-13" => Ok(AnyToolchain::QuartusII13(QuartusII13Toolchain)),
        _ => Err(ChipsmithError::UnknownBackend {
            name: name.to_string(),
        }),
    }
}

/// Install a toolchain version. Downloads if needed.
pub async fn install(backend: &str, version: &str) -> Result<PathBuf, ChipsmithError> {
    resolve_backend(backend)?.ensure_installed(version).await
}

/// Install from a local installer file.
pub async fn install_from_local(
    backend: &str,
    installer: &Path,
    version: &str,
) -> Result<(), ChipsmithError> {
    match resolve_backend(backend)? {
        AnyToolchain::QuartusPrime(t) => t.install_from_local(installer, version).await,
        AnyToolchain::QuartusII13(t) => t.install_from_local(installer, version).await,
    }
}

/// Build the FPGA project described by the manifest.
pub async fn build(project_dir: &Path) -> Result<PathBuf, ChipsmithError> {
    let manifest = Manifest::load(project_dir)?;
    let backend = resolve_backend(manifest.toolchain.backend())?;
    backend.build(project_dir, &manifest).await
}

/// Run a tool from the given backend.
pub async fn run_tool(
    backend: &str,
    version: &str,
    tool: &str,
    args: &[String],
    working_dir: Option<&Path>,
) -> Result<(), ChipsmithError> {
    resolve_backend(backend)?
        .run_tool(version, tool, args, working_dir)
        .await
}

/// Get the install directory for a backend + version.
pub fn which(backend: &str, version: &str) -> Result<(PathBuf, bool), ChipsmithError> {
    let b = resolve_backend(backend)?;
    let dir = b.install_dir(version)?;
    let installed = b.is_installed(version)?;
    Ok((dir, installed))
}
