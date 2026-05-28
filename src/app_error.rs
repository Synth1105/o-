use std::io;
use std::path::PathBuf;

use crate::job::JobError;
use crate::pm::PmError;
use crate::report::Report;

#[derive(Debug)]
pub enum AppError {
    HomeDirUnavailable,
    ReadConfig {
        path: PathBuf,
        source: io::Error,
    },
    ParseConfigToml(toml::de::Error),
    CreateDir {
        path: PathBuf,
        source: io::Error,
    },
    InstallToolchain {
        user: String,
        repo: String,
        source: String,
    },
    MissingToolchainSelection,
    ToolchainNotInstalled {
        toolchain: String,
        path: PathBuf,
    },
    MoveToolchain {
        from: PathBuf,
        to: PathBuf,
        source: io::Error,
    },
    RemoveToolchain {
        path: PathBuf,
        source: io::Error,
    },
    ReadScript {
        path: PathBuf,
        source: io::Error,
    },
    PackageManager(PmError),
    Job(JobError),
    ToolchainExecution {
        toolchain: String,
        message: String,
    },
}

impl AppError {
    pub fn report(&self) -> Report {
        match self {
            Self::HomeDirUnavailable => Report::new("could not resolve home directory")
                .detail("`$HOME` is unavailable in the current environment"),
            Self::ReadConfig { path, source } => Report::new("failed to read config")
                .detail(format!("path: {}", path.display()))
                .detail(format!("cause: {source}")),
            Self::ParseConfigToml(source) => Report::new("failed to parse config")
                .detail("expected a valid TOML document")
                .detail(format!("cause: {source}")),
            Self::CreateDir { path, source } => Report::new("failed to prepare directory")
                .detail(format!("path: {}", path.display()))
                .detail(format!("cause: {source}")),
            Self::InstallToolchain { user, repo, source } => {
                Report::new(format!("failed to install toolchain `{repo}`"))
                    .detail(format!("source: {user}/{repo}"))
                    .detail(format!("cause: {source}"))
            }
            Self::MissingToolchainSelection => Report::new("no toolchain is selected")
                .detail("set `toolchain.name` in `~/.config/o-/config.toml`"),
            Self::ToolchainNotInstalled { toolchain, path } => {
                Report::new(format!("toolchain `{toolchain}` is not installed"))
                    .detail(format!("path: {}", path.display()))
                    .detail("install it with `o- toolchain add <user> <repo>`")
            }
            Self::MoveToolchain { from, to, source } => {
                Report::new("failed to place installed toolchain")
                    .detail(format!("from: {}", from.display()))
                    .detail(format!("to: {}", to.display()))
                    .detail(format!("cause: {source}"))
            }
            Self::RemoveToolchain { path, source } => Report::new("failed to remove toolchain")
                .detail(format!("path: {}", path.display()))
                .detail(format!("cause: {source}")),
            Self::ReadScript { path, source } => Report::new("failed to read script")
                .detail(format!("path: {}", path.display()))
                .detail(format!("cause: {source}")),
            Self::PackageManager(error) => error.report(),
            Self::Job(error) => error.report(),
            Self::ToolchainExecution { toolchain, message } => {
                Report::new(format!("toolchain `{toolchain}` failed")).detail(message.clone())
            }
        }
    }
}
