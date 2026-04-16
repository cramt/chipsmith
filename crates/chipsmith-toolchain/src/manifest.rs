use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use facet::Facet;

use crate::error::ChipsmithError;

#[derive(Debug, Facet)]
pub struct Manifest {
    pub project: Project,
    pub toolchain: ToolchainSpec,
    pub target: Target,
    pub hdl: Hdl,
    #[facet(default)]
    pub pins: BTreeMap<String, PinMapping>,
}

#[derive(Debug, Facet)]
pub struct Project {
    pub name: String,
    pub top: String,
}

/// Parsed from `[toolchain]` — exactly one key (the backend name) with a version string.
/// e.g. `quartus = "23.1"` or `vivado = "2024.1"`
#[derive(Debug, Facet)]
#[facet(transparent)]
pub struct ToolchainSpec(BTreeMap<String, String>);

impl ToolchainSpec {
    pub fn backend(&self) -> &str {
        self.0.keys().next().unwrap()
    }

    pub fn version(&self) -> &str {
        self.0.values().next().unwrap()
    }

    fn validate(&self) -> Result<(), String> {
        if self.0.len() != 1 {
            return Err(
                "[toolchain] must have exactly one entry (e.g. quartus-prime = \"23.1\")"
                    .to_string(),
            );
        }
        Ok(())
    }
}

#[derive(Debug, Facet)]
pub struct Target {
    pub family: String,
    pub device: String,
}

#[derive(Debug, Facet)]
pub struct Hdl {
    #[facet(default = "VHDL_2008".to_string())]
    pub standard: String,
    pub sources: Vec<String>,
}

#[derive(Debug, Facet)]
#[facet(untagged)]
#[repr(u8)]
pub enum PinMapping {
    Single(String),
    Bus(Vec<String>),
}

impl Manifest {
    pub fn load(project_dir: &Path) -> Result<Self, ChipsmithError> {
        let path = project_dir.join("chipsmith.toml");
        let content = std::fs::read_to_string(&path)
            .map_err(|_| ChipsmithError::ManifestNotFound { path: path.clone() })?;
        let manifest: Manifest =
            facet_toml::from_str(&content).map_err(|e| ChipsmithError::ManifestParse {
                path: path.clone(),
                message: e.to_string(),
            })?;
        manifest
            .toolchain
            .validate()
            .map_err(|e| ChipsmithError::ManifestParse { path, message: e })?;
        Ok(manifest)
    }

    pub fn resolve_sources(&self, project_dir: &Path) -> Result<Vec<PathBuf>, ChipsmithError> {
        let mut files = Vec::new();
        for pattern in &self.hdl.sources {
            let full_pattern = project_dir.join(pattern);
            let matches: Vec<_> = glob::glob(full_pattern.to_str().unwrap_or(pattern))
                .map_err(|e| ChipsmithError::ManifestParse {
                    path: project_dir.join("chipsmith.toml"),
                    message: format!("invalid glob pattern '{}': {}", pattern, e),
                })?
                .filter_map(|r| r.ok())
                .collect();

            if matches.is_empty() {
                return Err(ChipsmithError::NoSourceFiles {
                    pattern: pattern.clone(),
                });
            }
            files.extend(matches);
        }
        Ok(files)
    }
}
