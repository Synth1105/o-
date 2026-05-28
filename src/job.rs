use crate::pm::{Manifest, read_manifest};
use crate::report::Report;
use std::io;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Debug)]
pub enum JobError {
    ReadManifest {
        path: PathBuf,
        source: io::Error,
    },
    MissingScript {
        name: String,
    },
    SpawnScript {
        name: String,
        command: String,
        source: io::Error,
    },
    ScriptFailed {
        name: String,
        command: String,
        status: String,
    },
}

impl JobError {
    pub fn report(&self) -> Report {
        match self {
            Self::ReadManifest { path, source } => Report::new("failed to read package.json")
                .detail(format!("path: {}", path.display()))
                .detail(format!("cause: {source}")),
            Self::MissingScript { name } => Report::new(format!("script `{name}` is not defined"))
                .detail("add it to the `scripts` section in package.json"),
            Self::SpawnScript {
                name,
                command,
                source,
            } => Report::new(format!("failed to start script `{name}`"))
                .detail(format!("command: {command}"))
                .detail(format!("cause: {source}")),
            Self::ScriptFailed {
                name,
                command,
                status,
            } => Report::new(format!("script `{name}` failed"))
                .detail(format!("command: {command}"))
                .detail(format!("status: {status}")),
        }
    }
}

pub fn run(name: &str) -> Result<(), JobError> {
    let manifest_path = std::env::current_dir()
        .map_err(|source| JobError::ReadManifest {
            path: PathBuf::from("."),
            source,
        })?
        .join("package.json");
    let manifest = read_manifest(".").map_err(|source| JobError::ReadManifest {
        path: manifest_path.clone(),
        source,
    })?;
    run_with_manifest(name, &manifest)
}

fn run_with_manifest(name: &str, manifest: &Manifest) -> Result<(), JobError> {
    let Some(scripts) = manifest.scripts.as_ref() else {
        return Err(JobError::MissingScript {
            name: name.to_string(),
        });
    };

    let Some(command) = scripts.get(name) else {
        return Err(JobError::MissingScript {
            name: name.to_string(),
        });
    };

    let status = shell_command(command)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|source| JobError::SpawnScript {
            name: name.to_string(),
            command: command.clone(),
            source,
        })?;

    if !status.success() {
        return Err(JobError::ScriptFailed {
            name: name.to_string(),
            command: command.clone(),
            status: status
                .code()
                .map(|code| code.to_string())
                .unwrap_or_else(|| status.to_string()),
        });
    }

    Ok(())
}

fn shell_command(command: &str) -> Command {
    if cfg!(windows) {
        let mut shell = Command::new("cmd");
        shell.arg("/C").arg(command);
        shell
    } else {
        let mut shell = Command::new("sh");
        shell.arg("-c").arg(command);
        shell
    }
}
