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

    /// Flash a .sof file to the FPGA via JTAG using quartus_pgm.
    async fn flash(
        &self,
        sof: &Path,
        manifest: &Manifest,
        cable: Option<&str>,
    ) -> Result<(), ChipsmithError> {
        if !sof.exists() {
            return Err(ChipsmithError::OutputNotFound {
                path: sof.to_path_buf(),
            });
        }

        let version = manifest.toolchain.version();
        let mut args = vec![
            "-m".to_string(),
            "jtag".to_string(),
            "-o".to_string(),
            format!("P;{}", sof.display()),
        ];
        if let Some(cable) = cable {
            args.push("-c".to_string());
            args.push(cable.to_string());
        }

        self.run_tool(version, "quartus_pgm", &args, None).await
    }
}
