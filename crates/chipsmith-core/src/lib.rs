pub use chipsmith_toolchain::{error, manifest, toolchain};

use std::path::{Path, PathBuf};

use error::ChipsmithError;
use manifest::Manifest;
use toolchain::Toolchain;

use chipsmith_quartus_ii_13::QuartusII13Toolchain;
use chipsmith_quartus_prime::QuartusPrimeToolchain;

pub const DEFAULT_LATEST: &str = chipsmith_quartus_prime::LATEST;

fn resolve_backend(name: &str) -> Result<Box<dyn Toolchain>, ChipsmithError> {
    match name {
        "quartus-prime" => Ok(Box::new(QuartusPrimeToolchain)),
        "quartus-ii-13" => Ok(Box::new(QuartusII13Toolchain)),
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
    resolve_backend(backend)?
        .install_from_local(installer, version)
        .await
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
