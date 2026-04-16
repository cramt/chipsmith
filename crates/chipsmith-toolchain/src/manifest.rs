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
    /// Construct from a pre-validated single-entry map. Panics if map is empty.
    /// Intended for tests and internal use after validation.
    pub fn from_map(map: BTreeMap<String, String>) -> Self {
        assert!(
            map.len() == 1,
            "[toolchain] must have exactly one entry, got {}",
            map.len()
        );
        Self(map)
    }

    /// The backend name (e.g. "quartus-prime"). Safe to call after validation.
    pub fn backend(&self) -> &str {
        self.0
            .keys()
            .next()
            .expect("ToolchainSpec invariant: validated to have exactly one entry")
    }

    /// The version string (e.g. "23.1"). Safe to call after validation.
    pub fn version(&self) -> &str {
        self.0
            .values()
            .next()
            .expect("ToolchainSpec invariant: validated to have exactly one entry")
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

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_TOML: &str = r#"
[project]
name = "blinky"
top = "blinky"

[toolchain]
quartus-prime = "23.1"

[target]
family = "Cyclone V"
device = "5CSEBA6U23I7"

[hdl]
sources = ["src/*.vhd"]

[pins]
clk = "PIN_Y2"
led = ["PIN_V16", "PIN_W16", "PIN_V17", "PIN_W17"]
"#;

    fn parse_manifest(toml: &str) -> Result<Manifest, String> {
        let manifest: Manifest = facet_toml::from_str(toml).map_err(|e| e.to_string())?;
        manifest.toolchain.validate().map_err(|e| e.to_string())?;
        Ok(manifest)
    }

    #[test]
    fn parses_valid_manifest() {
        let m = parse_manifest(VALID_TOML).unwrap();
        assert_eq!(m.project.name, "blinky");
        assert_eq!(m.project.top, "blinky");
        assert_eq!(m.toolchain.backend(), "quartus-prime");
        assert_eq!(m.toolchain.version(), "23.1");
        assert_eq!(m.target.family, "Cyclone V");
        assert_eq!(m.target.device, "5CSEBA6U23I7");
        assert_eq!(m.hdl.sources, vec!["src/*.vhd"]);
    }

    #[test]
    fn default_hdl_standard_is_vhdl_2008() {
        let m = parse_manifest(VALID_TOML).unwrap();
        assert_eq!(m.hdl.standard, "VHDL_2008");
    }

    #[test]
    fn parses_single_pin() {
        let m = parse_manifest(VALID_TOML).unwrap();
        match m.pins.get("clk").unwrap() {
            PinMapping::Single(pin) => assert_eq!(pin, "PIN_Y2"),
            _ => panic!("expected Single pin mapping"),
        }
    }

    #[test]
    fn parses_bus_pins() {
        let m = parse_manifest(VALID_TOML).unwrap();
        match m.pins.get("led").unwrap() {
            PinMapping::Bus(pins) => {
                assert_eq!(pins.len(), 4);
                assert_eq!(pins[0], "PIN_V16");
                assert_eq!(pins[3], "PIN_W17");
            }
            _ => panic!("expected Bus pin mapping"),
        }
    }

    #[test]
    fn rejects_multiple_toolchain_entries() {
        let toml = r#"
[project]
name = "test"
top = "test"

[toolchain]
quartus-prime = "23.1"
vivado = "2024.1"

[target]
family = "Cyclone V"
device = "test"

[hdl]
sources = ["*.vhd"]
"#;
        let result = parse_manifest(toml);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("exactly one entry"));
    }

    #[test]
    fn rejects_missing_project_name() {
        let toml = r#"
[project]
top = "test"

[toolchain]
quartus-prime = "23.1"

[target]
family = "Cyclone V"
device = "test"

[hdl]
sources = ["*.vhd"]
"#;
        assert!(parse_manifest(toml).is_err());
    }
}
