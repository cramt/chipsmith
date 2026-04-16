use std::path::{Path, PathBuf};
use tokio::process::Command;

use chipsmith_toolchain::error::ChipsmithError;
use crate::nixos;

const KNOWN_TOOLS: &[&str] = &[
    "quartus_sh",
    "quartus_map",
    "quartus_fit",
    "quartus_asm",
    "quartus_sta",
    "quartus_pgm",
    "quartus_cpf",
    "quartus",
    "jtagconfig",
];

pub fn tool_path(install_dir: &Path, tool: &str) -> Result<PathBuf, ChipsmithError> {
    if !KNOWN_TOOLS.contains(&tool) {
        return Err(ChipsmithError::UnknownTool {
            name: tool.to_string(),
        });
    }
    Ok(install_dir.join("quartus").join("bin").join(tool))
}

pub async fn run_tool(
    install_dir: &Path,
    tool: &str,
    args: &[String],
    working_dir: Option<&Path>,
) -> Result<(), ChipsmithError> {
    let bin = tool_path(install_dir, tool)?;

    if !bin.exists() {
        return Err(ChipsmithError::NotInstalled {
            path: install_dir.to_path_buf(),
        });
    }

    let mut cmd = Command::new(&bin);
    cmd.args(args);
    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    let status = if nixos::is_nixos() {
        let compat = nixos::NixCompat::init().await?;
        if let Some(mut bwrap_cmd) = compat.bwrap_command(&bin) {
            bwrap_cmd.args(args);
            if let Some(dir) = working_dir {
                bwrap_cmd.current_dir(dir);
            }
            bwrap_cmd
                .env("LD_LIBRARY_PATH", &compat.ld_library_path)
                .spawn()
                .map_err(|source| ChipsmithError::Spawn {
                    command: bin.display().to_string(),
                    source,
                })?
                .wait()
                .await?
        } else {
            cmd.env("LD_LIBRARY_PATH", &compat.ld_library_path)
                .spawn()
                .map_err(|source| ChipsmithError::Spawn {
                    command: bin.display().to_string(),
                    source,
                })?
                .wait()
                .await?
        }
    } else {
        cmd.spawn()
            .map_err(|source| ChipsmithError::Spawn {
                command: bin.display().to_string(),
                source,
            })?
            .wait()
            .await?
    };

    if !status.success() {
        return Err(ChipsmithError::ProcessFailed {
            command: format!("{} {}", tool, args.join(" ")),
            code: status.code(),
        });
    }

    Ok(())
}

pub async fn install_quartus(installer: &Path, install_dir: &Path) -> Result<(), ChipsmithError> {
    if !installer.exists() {
        return Err(ChipsmithError::InstallerNotFound {
            path: installer.to_path_buf(),
        });
    }

    // Quartus II 13.0sp1 does not have --accept_eula flag
    let install_args = &["--mode", "unattended", "--unattendedmodeui", "none", "--installdir"];

    if nixos::is_nixos() {
        let compat = nixos::NixCompat::init().await?;

        // Quartus II 13.0sp1 installer is a 32-bit statically linked binary
        // that internally loads /lib/ld-linux.so.2. Use bwrap to provide it.
        let status = if let Some(mut cmd) = compat.bwrap_command(installer) {
            eprintln!("Using bubblewrap for 32-bit installer...");
            cmd.args(install_args)
                .arg(install_dir)
                .env("LD_LIBRARY_PATH", &compat.ld_library_path)
                .spawn()
                .map_err(|source| ChipsmithError::Spawn {
                    command: installer.display().to_string(),
                    source,
                })?
                .wait()
                .await?
        } else {
            eprintln!("Patching installer for NixOS...");
            compat.patch_elf(installer).await?;
            Command::new(installer)
                .args(install_args)
                .arg(install_dir)
                .env("LD_LIBRARY_PATH", &compat.ld_library_path)
                .spawn()
                .map_err(|source| ChipsmithError::Spawn {
                    command: installer.display().to_string(),
                    source,
                })?
                .wait()
                .await?
        };

        if !status.success() {
            return Err(ChipsmithError::ProcessFailed {
                command: installer.display().to_string(),
                code: status.code(),
            });
        }

        compat
            .patch_install(&[
                install_dir.join("quartus").join("bin"),
                install_dir.join("quartus").join("linux64"),
                install_dir.join("quartus").join("adm"),
            ])
            .await?;
    } else {
        let status = Command::new(installer)
            .args(install_args)
            .arg(install_dir)
            .spawn()
            .map_err(|source| ChipsmithError::Spawn {
                command: installer.display().to_string(),
                source,
            })?
            .wait()
            .await?;

        if !status.success() {
            return Err(ChipsmithError::ProcessFailed {
                command: installer.display().to_string(),
                code: status.code(),
            });
        }
    }

    Ok(())
}
