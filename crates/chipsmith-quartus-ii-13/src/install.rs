use std::path::PathBuf;

use chipsmith_toolchain::error::ChipsmithError;

use crate::download;
use crate::runner;

pub struct QuartusVersion {
    pub version: &'static str,
    pub revision: &'static str,
    pub filename: &'static str,
}

pub struct DeviceSupport {
    pub family: &'static str,
    pub filename: &'static str,
}

pub struct KnownVersion {
    pub key: &'static str,
    pub download: QuartusVersion,
    pub install_subdir: &'static str,
    pub devices: &'static [DeviceSupport],
}

const QUARTUS_CDN: &str = "https://downloads.intel.com/akdlm/software/acdsinst";

fn cdn_url(ver: &QuartusVersion, filename: &str) -> String {
    format!(
        "{}/{}/{}/ib_installers/{}",
        QUARTUS_CDN, ver.version, ver.revision, filename
    )
}

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
    VERSIONS
        .iter()
        .find(|v| v.key == key)
        .ok_or_else(|| ChipsmithError::UnknownVersion {
            version: key.to_string(),
            available: VERSIONS.iter().map(|v| v.key.to_string()).collect(),
        })
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

    // Install any missing device support packages
    for device in version.devices {
        let marker = dir.join(".chipsmith_device_installed_").join(device.family);
        if marker.exists() {
            continue;
        }

        eprintln!("Installing {} device support...", device.family);
        let url = cdn_url(&version.download, device.filename);
        match download::download_file(&url, device.filename).await {
            Ok(qdz) => {
                if let Err(e) = download::unzip(&qdz, &dir).await {
                    eprintln!(
                        "Warning: failed to install {} device support: {}",
                        device.family, e
                    );
                    continue;
                }
            }
            Err(e) => {
                eprintln!(
                    "Warning: failed to download {} device support: {}",
                    device.family, e
                );
                continue;
            }
        }

        if let Some(parent) = marker.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&marker, "")?;
    }

    Ok(dir)
}
