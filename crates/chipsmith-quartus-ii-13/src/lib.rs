mod build;
mod install;
mod nixos;
mod runner;

use std::path::{Path, PathBuf};

use chipsmith_toolchain::error::ChipsmithError;
use chipsmith_toolchain::manifest::Manifest;
use chipsmith_toolchain::toolchain::Toolchain;

pub struct QuartusII13Toolchain;

pub const LATEST: &str = install::LATEST;

#[async_trait::async_trait]
impl Toolchain for QuartusII13Toolchain {
    fn name(&self) -> &str {
        "quartus-ii-13"
    }

    fn install_dir(&self, version: &str) -> Result<PathBuf, ChipsmithError> {
        let ver = install::lookup(version)?;
        Ok(install::install_dir_for(ver))
    }

    fn is_installed(&self, version: &str) -> Result<bool, ChipsmithError> {
        let ver = install::lookup(version)?;
        Ok(install::is_installed(ver))
    }

    async fn ensure_installed(&self, version: &str) -> Result<PathBuf, ChipsmithError> {
        install::ensure_installed(version).await
    }

    async fn install_from_local(
        &self,
        installer: &Path,
        version: &str,
    ) -> Result<(), ChipsmithError> {
        let ver = install::lookup(version)?;
        let dir = install::install_dir_for(ver);
        runner::install_quartus(installer, &dir).await
    }

    async fn run_tool(
        &self,
        version: &str,
        tool: &str,
        args: &[String],
        working_dir: Option<&Path>,
    ) -> Result<(), ChipsmithError> {
        let dir = install::ensure_installed(version).await?;
        runner::run_tool(&dir, tool, args, working_dir).await
    }

    async fn build(
        &self,
        project_dir: &Path,
        manifest: &Manifest,
    ) -> Result<PathBuf, ChipsmithError> {
        build::build(project_dir, manifest).await
    }
}
