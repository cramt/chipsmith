use std::path::{Path, PathBuf};

use chipsmith_toolchain::error::ChipsmithError;
use chipsmith_toolchain::manifest::Manifest;

use crate::qsf;

pub struct BuildStep {
    pub label: &'static str,
    pub tool: &'static str,
    pub args: Vec<String>,
}

pub struct BuildPlan {
    pub build_dir: PathBuf,
    pub steps: Vec<BuildStep>,
    pub output_sof: PathBuf,
}

/// Prepare the build directory (QSF/QPF files, step list) without running anything.
/// Each backend calls this, then runs the steps through its own runner.
pub fn prepare_build(project_dir: &Path, manifest: &Manifest) -> Result<BuildPlan, ChipsmithError> {
    let sources = manifest.resolve_sources(project_dir)?;

    let build_dir = project_dir.join("build");
    std::fs::create_dir_all(&build_dir)?;

    let qsf_content = qsf::generate_qsf(manifest, &sources, project_dir)?;
    std::fs::write(
        build_dir.join(format!("{}.qsf", manifest.project.name)),
        &qsf_content,
    )?;

    let qpf_content = qsf::generate_qpf(manifest);
    std::fs::write(
        build_dir.join(format!("{}.qpf", manifest.project.name)),
        &qpf_content,
    )?;

    let name = &manifest.project.name;
    let steps = vec![
        BuildStep {
            label: "Synthesis",
            tool: "quartus_map",
            args: vec![name.clone(), "--read_settings_files=on".to_string()],
        },
        BuildStep {
            label: "Fitter",
            tool: "quartus_fit",
            args: vec![name.clone(), "--read_settings_files=on".to_string()],
        },
        BuildStep {
            label: "Assembler",
            tool: "quartus_asm",
            args: vec![name.clone(), "--read_settings_files=on".to_string()],
        },
        BuildStep {
            label: "Timing analysis",
            tool: "quartus_sta",
            args: vec![name.clone()],
        },
    ];

    let output_sof = build_dir
        .join("output_files")
        .join(format!("{}.sof", manifest.project.name));

    Ok(BuildPlan {
        build_dir,
        steps,
        output_sof,
    })
}
