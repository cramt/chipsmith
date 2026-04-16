use std::path::PathBuf;

use chipsmith_quartus_common::download;
use chipsmith_quartus_common::install::{
    cdn_url, install_device_support, lookup as common_lookup, DeviceSupport, KnownVersion,
    QuartusVersion,
};
use chipsmith_toolchain::error::ChipsmithError;

use crate::runner;

pub const VERSIONS: &[KnownVersion] = &[
    KnownVersion {
        key: "23.1",
        download: QuartusVersion {
            version: "23.1std.1",
            revision: "993",
            filename: "QuartusLiteSetup-23.1std.1.993-linux.run",
        },
        install_subdir: "23.1std",
        devices: &[
            DeviceSupport {
                family: "cyclonev",
                filename: "cyclonev-23.1std.1.993.qdz",
            },
            DeviceSupport {
                family: "cyclone10lp",
                filename: "cyclone10lp-23.1std.1.993.qdz",
            },
            DeviceSupport {
                family: "cyclone",
                filename: "cyclone-23.1std.1.993.qdz",
            },
            DeviceSupport {
                family: "max10",
                filename: "max10-23.1std.1.993.qdz",
            },
            DeviceSupport {
                family: "max",
                filename: "max-23.1std.1.993.qdz",
            },
        ],
    },
    KnownVersion {
        key: "22.1",
        download: QuartusVersion {
            version: "22.1std.2",
            revision: "922",
            filename: "QuartusLiteSetup-22.1std.2.922-linux.run",
        },
        install_subdir: "22.1std",
        devices: &[DeviceSupport {
            family: "cyclonev",
            filename: "cyclonev-22.1std.2.922.qdz",
        }],
    },
    KnownVersion {
        key: "24.1",
        download: QuartusVersion {
            version: "24.1std",
            revision: "1077",
            filename: "QuartusLiteSetup-24.1std.0.1077-linux.run",
        },
        install_subdir: "24.1std",
        devices: &[DeviceSupport {
            family: "cyclonev",
            filename: "cyclonev-24.1std.0.1077.qdz",
        }],
    },
];

pub const LATEST: &str = "23.1";

pub fn lookup(key: &str) -> Result<&'static KnownVersion, ChipsmithError> {
    common_lookup(VERSIONS, key)
}

pub fn install_dir_for(version: &KnownVersion) -> PathBuf {
    dirs::home_dir()
        .expect("could not determine home directory")
        .join("intelFPGA_lite")
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
            "Quartus Prime {} not found, downloading and installing...",
            version_key
        );

        let url = cdn_url(&version.download, version.download.filename);
        let installer = download::download_file(&url, version.download.filename).await?;
        download::make_executable(&installer).await?;

        eprintln!(
            "Installing Quartus Prime {} to {}",
            version_key,
            dir.display()
        );
        runner::install_quartus(&installer, &dir).await?;
    }

    install_device_support(version, &dir).await?;

    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_finds_all_versions() {
        assert!(lookup("23.1").is_ok());
        assert!(lookup("22.1").is_ok());
        assert!(lookup("24.1").is_ok());
    }

    #[test]
    fn lookup_rejects_unknown() {
        assert!(lookup("99.0").is_err());
    }

    #[test]
    fn latest_is_valid() {
        assert!(lookup(LATEST).is_ok());
    }

    #[test]
    fn install_dir_contains_version() {
        let ver = lookup("23.1").unwrap();
        let dir = install_dir_for(ver);
        assert!(dir.ends_with("intelFPGA_lite/23.1std"));
    }
}
