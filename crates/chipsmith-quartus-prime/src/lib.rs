mod build;
mod download;
mod install;
mod nixos;
mod qsf;
mod runner;

use std::path::{Path, PathBuf};

use chipsmith_toolchain::error::ChipsmithError;
use chipsmith_toolchain::manifest::Manifest;
use chipsmith_toolchain::toolchain::Toolchain;

pub struct QuartusPrimeToolchain;

pub const LATEST: &str = install::LATEST;

impl QuartusPrimeToolchain {
    pub async fn install_from_local(&self, installer: &Path, version: &str) -> Result<(), ChipsmithError> {
        let ver = install::lookup(version)?;
        let dir = install::install_dir_for(ver);
        runner::install_quartus(installer, &dir).await
    }
}

impl Toolchain for QuartusPrimeToolchain {
    fn name(&self) -> &str {
        "quartus-prime"
    }

    fn install_dir(&self, version: &str) -> Result<PathBuf, ChipsmithError> {
        let ver = install::lookup(version)?;
        Ok(install::install_dir_for(ver))
    }

    fn is_installed(&self, version: &str) -> Result<bool, ChipsmithError> {
        let ver = install::lookup(version)?;
        Ok(install::is_installed(ver))
    }

    fn ensure_installed(
        &self,
        version: &str,
    ) -> impl std::future::Future<Output = Result<PathBuf, ChipsmithError>> + Send {
        install::ensure_installed(version)
    }

    fn run_tool(
        &self,
        version: &str,
        tool: &str,
        args: &[String],
        working_dir: Option<&Path>,
    ) -> impl std::future::Future<Output = Result<(), ChipsmithError>> + Send {
        let version = version.to_string();
        let tool = tool.to_string();
        let args = args.to_vec();
        let working_dir = working_dir.map(|p| p.to_path_buf());
        async move {
            let dir = install::ensure_installed(&version).await?;
            runner::run_tool(&dir, &tool, &args, working_dir.as_deref()).await
        }
    }

    fn build(
        &self,
        project_dir: &Path,
        manifest: &Manifest,
    ) -> impl std::future::Future<Output = Result<PathBuf, ChipsmithError>> + Send {
        build::build(project_dir, manifest)
    }
}
