use std::path::{Path, PathBuf};

use crate::error::ChipsmithError;
use crate::manifest::Manifest;

#[async_trait::async_trait]
pub trait Toolchain: Send + Sync {
    fn name(&self) -> &str;

    fn install_dir(&self, version: &str) -> Result<PathBuf, ChipsmithError>;

    fn is_installed(&self, version: &str) -> Result<bool, ChipsmithError>;

    async fn ensure_installed(&self, version: &str) -> Result<PathBuf, ChipsmithError>;

    async fn install_from_local(
        &self,
        installer: &Path,
        version: &str,
    ) -> Result<(), ChipsmithError>;

    async fn run_tool(
        &self,
        version: &str,
        tool: &str,
        args: &[String],
        working_dir: Option<&Path>,
    ) -> Result<(), ChipsmithError>;

    async fn build(
        &self,
        project_dir: &Path,
        manifest: &Manifest,
    ) -> Result<PathBuf, ChipsmithError>;
}
