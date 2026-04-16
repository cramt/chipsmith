use std::path::{Path, PathBuf};

use crate::error::ChipsmithError;
use crate::manifest::Manifest;

pub trait Toolchain: Send + Sync {
    fn name(&self) -> &str;

    fn install_dir(&self, version: &str) -> Result<PathBuf, ChipsmithError>;

    fn is_installed(&self, version: &str) -> Result<bool, ChipsmithError>;

    fn ensure_installed(
        &self,
        version: &str,
    ) -> impl std::future::Future<Output = Result<PathBuf, ChipsmithError>> + Send;

    fn run_tool(
        &self,
        version: &str,
        tool: &str,
        args: &[String],
        working_dir: Option<&Path>,
    ) -> impl std::future::Future<Output = Result<(), ChipsmithError>> + Send;

    fn build(
        &self,
        project_dir: &Path,
        manifest: &Manifest,
    ) -> impl std::future::Future<Output = Result<PathBuf, ChipsmithError>> + Send;
}
