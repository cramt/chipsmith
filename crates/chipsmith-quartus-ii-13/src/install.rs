use std::path::PathBuf;

use chipsmith_quartus_common::download;
use chipsmith_quartus_common::install::{
    cdn_url, install_device_support, lookup as common_lookup, DeviceSupport, KnownVersion,
    QuartusVersion,
};
use chipsmith_toolchain::error::ChipsmithError;

use crate::runner;

pub const VERSIONS: &[KnownVersion] = &[KnownVersion {
    key: "13.0sp1",
    download: QuartusVersion {
        version: "13.0sp1",
        revision: "232",
        filename: "QuartusSetupWeb-13.0.1.232.run",
    },
    install_subdir: "13.0sp1",
    devices: &[
        DeviceSupport {
            family: "cyclone",
            filename: "cyclone_web-13.0.1.232.qdz",
        },
        DeviceSupport {
            family: "max",
            filename: "max_web-13.0.1.232.qdz",
        },
    ],
}];

pub const LATEST: &str = "13.0sp1";

pub fn lookup(key: &str) -> Result<&'static KnownVersion, ChipsmithError> {
    common_lookup(VERSIONS, key)
}

pub fn install_dir_for(version: &KnownVersion) -> PathBuf {
    dirs::home_dir()
        .expect("could not determine home directory")
        .join("altera")
        .join(version.install_subdir)
}

pub fn is_installed(version: &KnownVersion) -> bool {
    install_dir_for(version)
        .join("quartus")
        .join("bin")
        .join("quartus_sh")
        .exists()
}

pub async fn ensure_installed(version_key: &str) -> Result<PathBuf, ChipsmithError> {
    let version = lookup(version_key)?;
    let dir = install_dir_for(version);

    if !is_installed(version) {
        eprintln!(
            "Quartus II {} not found, downloading and installing...",
            version_key
        );

        let url = cdn_url(&version.download, version.download.filename);
        let installer = download::download_file(&url, version.download.filename).await?;
        download::make_executable(&installer).await?;

        eprintln!("Installing Quartus II {} to {}", version_key, dir.display());
        runner::install_quartus(&installer, &dir).await?;
    }

    install_device_support(version, &dir).await?;

    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_finds_known_version() {
        assert!(lookup("13.0sp1").is_ok());
    }

    #[test]
    fn lookup_rejects_unknown() {
        assert!(lookup("14.0").is_err());
    }

    #[test]
    fn latest_is_valid() {
        assert!(lookup(LATEST).is_ok());
    }

    #[test]
    fn install_dir_contains_version() {
        let ver = lookup("13.0sp1").unwrap();
        let dir = install_dir_for(ver);
        assert!(dir.ends_with("altera/13.0sp1"));
    }
}
