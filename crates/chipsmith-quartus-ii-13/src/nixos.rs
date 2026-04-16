use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use chipsmith_toolchain::error::ChipsmithError;

pub fn is_nixos() -> bool {
    Path::new("/etc/NIXOS").exists()
}

/// Resolve all required nix store paths in a single `nix eval` call.
async fn resolve_nix_paths() -> Result<NixPaths, ChipsmithError> {
    let expr = r#"
        let pkgs = import <nixpkgs> {};
        in {
            glibc = pkgs.glibc.outPath;
            gcc-lib = pkgs.gcc-unwrapped.lib.outPath;
            zlib = pkgs.zlib.outPath;
            patchelf = pkgs.patchelf.outPath;
            bash = pkgs.bash.outPath;
            ncurses = pkgs.ncurses.outPath;
            freetype = pkgs.freetype.outPath;
            fontconfig = pkgs.fontconfig.lib.outPath;
            libxrender = pkgs.xorg.libXrender.outPath;
            libxext = pkgs.xorg.libXext.outPath;
            libx11 = pkgs.xorg.libX11.outPath;
            libxi = pkgs.xorg.libXi.outPath;
            libxtst = pkgs.xorg.libXtst.outPath;
            libxft = pkgs.xorg.libXft.outPath;
            dbus = pkgs.dbus.lib.outPath;
            glib = pkgs.glib.out.outPath;
            libpng = pkgs.libpng.outPath;
            expat = pkgs.expat.outPath;
            libxml2 = pkgs.libxml2.outPath;
            libxcb = pkgs.xorg.libxcb.outPath;
            libxau = pkgs.xorg.libXau.outPath;
            libxdmcp = pkgs.xorg.libXdmcp.outPath;
            libxcrypt-legacy = pkgs.libxcrypt-legacy.outPath;
            libsm = pkgs.xorg.libSM.outPath;
            libice = pkgs.xorg.libICE.outPath;
            krb5 = pkgs.krb5.lib.outPath;
            bzip2 = pkgs.bzip2.out.outPath;
            systemd = pkgs.systemd.outPath;
            libxfixes = pkgs.xorg.libXfixes.outPath;
            libxdamage = pkgs.xorg.libXdamage.outPath;
            libxcomposite = pkgs.xorg.libXcomposite.outPath;
            libxrandr = pkgs.xorg.libXrandr.outPath;
            libxcursor = pkgs.xorg.libXcursor.outPath;
            libxinerama = pkgs.xorg.libXinerama.outPath;
            libuuid = pkgs.util-linux.lib.outPath;
            glibc32 = pkgs.pkgsi686Linux.glibc.outPath;
            gcc-lib32 = pkgs.pkgsi686Linux.gcc-unwrapped.lib.outPath;
            zlib32 = pkgs.pkgsi686Linux.zlib.outPath;
            bubblewrap = pkgs.bubblewrap.outPath;
        }
    "#;

    let output = tokio::process::Command::new("nix")
        .args(["eval", "--impure", "--json", "--expr", expr])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|source| ChipsmithError::Spawn {
            command: "nix eval".to_string(),
            source,
        })?
        .wait_with_output()
        .await?;

    if !output.status.success() {
        return Err(ChipsmithError::NixEval {
            attr: "quartus dependencies".to_string(),
            message: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }

    let map: HashMap<String, String> =
        facet_json::from_slice(&output.stdout).map_err(|e| ChipsmithError::NixEval {
            attr: "JSON parse".to_string(),
            message: e.to_string(),
        })?;

    Ok(NixPaths { paths: map })
}

struct NixPaths {
    paths: HashMap<String, String>,
}

impl NixPaths {
    fn get(&self, key: &str) -> Option<PathBuf> {
        self.paths.get(key).map(PathBuf::from)
    }

    fn dynamic_linker(&self) -> Option<PathBuf> {
        self.get("glibc")
            .map(|p| p.join("lib").join("ld-linux-x86-64.so.2"))
    }

    fn dynamic_linker_32(&self) -> Option<PathBuf> {
        self.get("glibc32")
            .map(|p| p.join("lib").join("ld-linux.so.2"))
    }

    fn patchelf(&self) -> Option<PathBuf> {
        self.get("patchelf").map(|p| p.join("bin").join("patchelf"))
    }

    fn bash(&self) -> Option<PathBuf> {
        self.get("bash").map(|p| p.join("bin").join("bash"))
    }

    fn bubblewrap(&self) -> Option<PathBuf> {
        self.get("bubblewrap").map(|p| p.join("bin").join("bwrap"))
    }

    fn lib_paths(&self) -> Vec<PathBuf> {
        self.paths
            .values()
            .map(|p| PathBuf::from(p).join("lib"))
            .filter(|p| p.exists())
            .collect()
    }
}

fn build_ld_library_path(lib_paths: &[PathBuf]) -> String {
    lib_paths
        .iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join(":")
}

fn make_writable(path: &Path) -> Result<(), ChipsmithError> {
    use std::os::unix::fs::PermissionsExt;
    let meta = std::fs::metadata(path)?;
    let mut perms = meta.permissions();
    let mode = perms.mode();
    if mode & 0o200 == 0 {
        perms.set_mode(mode | 0o200);
        std::fs::set_permissions(path, perms)?;
    }
    Ok(())
}

/// Everything needed to run FHS binaries on NixOS.
pub struct NixCompat {
    pub ld_library_path: String,
    pub dynamic_linker_32: Option<PathBuf>,
    pub bwrap_bin: Option<PathBuf>,
    bash_path: PathBuf,
    patchelf_bin: PathBuf,
    dynamic_linker: PathBuf,
}

impl NixCompat {
    pub async fn init() -> Result<Self, ChipsmithError> {
        eprintln!("Resolving NixOS library paths...");
        let paths = resolve_nix_paths().await?;

        let dynamic_linker = paths
            .dynamic_linker()
            .ok_or_else(|| ChipsmithError::NixEval {
                attr: "glibc".to_string(),
                message: "could not find dynamic linker".to_string(),
            })?;

        let patchelf_bin = paths.patchelf().ok_or_else(|| ChipsmithError::NixEval {
            attr: "patchelf".to_string(),
            message: "could not find patchelf".to_string(),
        })?;

        let bash_path = paths.bash().ok_or_else(|| ChipsmithError::NixEval {
            attr: "bash".to_string(),
            message: "could not find bash".to_string(),
        })?;

        let dynamic_linker_32 = paths.dynamic_linker_32();
        let bwrap_bin = paths.bubblewrap();
        let ld_library_path = build_ld_library_path(&paths.lib_paths());

        Ok(Self {
            ld_library_path,
            dynamic_linker_32,
            bwrap_bin,
            bash_path,
            patchelf_bin,
            dynamic_linker,
        })
    }

    /// Build a bwrap command that provides /lib/ld-linux.so.2 for 32-bit binaries.
    pub fn bwrap_command(&self, program: &Path) -> Option<tokio::process::Command> {
        let bwrap = self.bwrap_bin.as_ref()?;
        let ld32 = self.dynamic_linker_32.as_ref()?;

        let mut cmd = tokio::process::Command::new(bwrap);
        cmd.arg("--ro-bind")
            .arg("/")
            .arg("/")
            .arg("--bind")
            .arg("/tmp")
            .arg("/tmp")
            .arg("--bind")
            .arg(dirs::home_dir().unwrap_or_default())
            .arg(dirs::home_dir().unwrap_or_default())
            .arg("--tmpfs")
            .arg("/lib")
            .arg("--symlink")
            .arg(ld32)
            .arg("/lib/ld-linux.so.2")
            .arg("--dev")
            .arg("/dev")
            .arg("--proc")
            .arg("/proc")
            .arg(program);
        Some(cmd)
    }

    pub async fn patch_elf(&self, binary: &Path) -> Result<bool, ChipsmithError> {
        make_writable(binary)?;

        let status = tokio::process::Command::new(&self.patchelf_bin)
            .arg("--set-interpreter")
            .arg(&self.dynamic_linker)
            .arg(binary)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|source| ChipsmithError::Spawn {
                command: "patchelf".to_string(),
                source,
            })?
            .wait()
            .await?;

        Ok(status.success())
    }

    /// Fix `#!/bin/bash` shebangs to point to the nix bash.
    fn patch_shebang(&self, path: &Path, content: &[u8]) -> Result<bool, ChipsmithError> {
        let shebang = b"#!/bin/bash";
        if content.len() < shebang.len() || &content[..shebang.len()] != shebang {
            return Ok(false);
        }

        make_writable(path)?;

        let replacement = format!("#!{}", self.bash_path.display());
        let mut new_content = replacement.into_bytes();
        new_content.extend_from_slice(&content[shebang.len()..]);
        std::fs::write(path, &new_content)?;

        Ok(true)
    }

    /// Walk a directory and patch all ELF binaries and bash scripts.
    pub async fn patch_dir(&self, dir: &Path) -> Result<(u32, u32), ChipsmithError> {
        let mut elfs = 0u32;
        let mut scripts = 0u32;

        let mut entries = tokio::fs::read_dir(dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                continue;
            }

            if let Ok(bytes) = tokio::fs::read(&path).await {
                if bytes.len() > 4 && &bytes[0..4] == b"\x7fELF" {
                    if self.patch_elf(&path).await? {
                        elfs += 1;
                    }
                } else if self.patch_shebang(&path, &bytes)? {
                    scripts += 1;
                }
            }
        }

        Ok((elfs, scripts))
    }

    /// Patch all ELF and shell script files in the given directories.
    pub async fn patch_install(&self, dirs_to_patch: &[PathBuf]) -> Result<(), ChipsmithError> {
        eprintln!("Patching installation for NixOS...");

        let mut total_elfs = 0u32;
        let mut total_scripts = 0u32;

        for dir in dirs_to_patch {
            if dir.exists() {
                let (elfs, scripts) = self.patch_dir(dir).await?;
                total_elfs += elfs;
                total_scripts += scripts;
            }
        }

        eprintln!(
            "Patched {} ELF binaries, {} shell scripts",
            total_elfs, total_scripts
        );
        Ok(())
    }
}
