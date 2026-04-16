use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use chipsmith_toolchain::error::ChipsmithError;
use chipsmith_toolchain::manifest::{Manifest, PinMapping};

pub fn generate_qpf(manifest: &Manifest) -> String {
    format!(
        "QUARTUS_VERSION = \"{ver}\"\n\
         DATE = \"00:00:00 January 01, 2024\"\n\
         PROJECT_REVISION = \"{name}\"\n",
        ver = manifest.toolchain.version(),
        name = manifest.project.name,
    )
}

pub fn generate_qsf(
    manifest: &Manifest,
    sources: &[PathBuf],
    project_dir: &Path,
) -> Result<String, ChipsmithError> {
    let mut qsf = String::new();

    writeln!(qsf, "set_global_assignment -name FAMILY \"{}\"", manifest.target.family).unwrap();
    writeln!(qsf, "set_global_assignment -name DEVICE {}", manifest.target.device).unwrap();
    writeln!(qsf, "set_global_assignment -name TOP_LEVEL_ENTITY {}", manifest.project.top).unwrap();
    writeln!(qsf, "set_global_assignment -name VHDL_INPUT_VERSION {}", manifest.hdl.standard).unwrap();
    writeln!(qsf, "set_global_assignment -name PROJECT_OUTPUT_DIRECTORY output_files").unwrap();
    writeln!(qsf, "set_global_assignment -name NUM_PARALLEL_PROCESSORS ALL").unwrap();
    writeln!(qsf).unwrap();

    for source in sources {
        let relative = source.strip_prefix(project_dir).unwrap_or(source);
        writeln!(qsf, "set_global_assignment -name VHDL_FILE ../{}", relative.display()).unwrap();
    }
    writeln!(qsf).unwrap();

    for (signal, mapping) in &manifest.pins {
        match mapping {
            PinMapping::Single(pin) => {
                writeln!(qsf, "set_location_assignment {pin} -to {signal}").unwrap();
            }
            PinMapping::Bus(pins) => {
                for (i, pin) in pins.iter().enumerate() {
                    writeln!(qsf, "set_location_assignment {pin} -to {signal}[{i}]").unwrap();
                }
            }
        }
    }

    Ok(qsf)
}
