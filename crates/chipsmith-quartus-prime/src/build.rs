use std::path::{Path, PathBuf};

use chipsmith_toolchain::error::ChipsmithError;
use chipsmith_toolchain::manifest::Manifest;

use crate::install;
use crate::qsf;
use crate::runner;

pub async fn build(project_dir: &Path, manifest: &Manifest) -> Result<PathBuf, ChipsmithError> {
    let sources = manifest.resolve_sources(project_dir)?;
    let install_dir = install::ensure_installed(manifest.toolchain.version()).await?;

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

    let steps: &[(&str, &[&str])] = &[
        ("Synthesis", &["quartus_map", &manifest.project.name, "--read_settings_files=on"]),
        ("Fitter", &["quartus_fit", &manifest.project.name, "--read_settings_files=on"]),
        ("Assembler", &["quartus_asm", &manifest.project.name, "--read_settings_files=on"]),
        ("Timing analysis", &["quartus_sta", &manifest.project.name]),
    ];

    for (label, args) in steps {
        let tool = args[0];
        let tool_args: Vec<String> = args[1..].iter().map(|s| s.to_string()).collect();
        eprintln!("==> {} ({})", label, tool);
        runner::run_tool(&install_dir, tool, &tool_args, Some(&build_dir)).await?;
    }

    let sof = build_dir
        .join("output_files")
        .join(format!("{}.sof", manifest.project.name));

    eprintln!("Build complete: {}", sof.display());
    Ok(sof)
}
