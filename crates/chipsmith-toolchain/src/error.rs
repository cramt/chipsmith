use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum ChipsmithError {
    #[error("process `{command}` failed with exit code {code:?}")]
    ProcessFailed { command: String, code: Option<i32> },

    #[error("failed to spawn `{command}`: {source}")]
    Spawn {
        command: String,
        #[source]
        source: std::io::Error,
    },

    #[error("toolchain not installed at {path}")]
    NotInstalled { path: PathBuf },

    #[error("installer not found: {path}")]
    InstallerNotFound { path: PathBuf },

    #[error("unknown tool: {name}")]
    UnknownTool { name: String },

    #[error("chipsmith.toml not found: {path}")]
    ManifestNotFound { path: PathBuf },

    #[error("chipsmith.toml parse error in {path}: {message}")]
    ManifestParse { path: PathBuf, message: String },

    #[error("no source files matched pattern: {pattern}")]
    NoSourceFiles { pattern: String },

    #[error("unknown version: {version} (available: {})", available.join(", "))]
    UnknownVersion {
        version: String,
        available: Vec<String>,
    },

    #[error("unknown toolchain backend: {name}")]
    UnknownBackend { name: String },

    #[error("nix eval failed for `{attr}`: {message}")]
    NixEval { attr: String, message: String },

    #[error("output file not found: {path} (have you run `chipsmith build`?)")]
    OutputNotFound { path: PathBuf },

    #[error("download failed: {message}")]
    Download { message: String },

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
