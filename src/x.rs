use crate::pm::{PmError, install_from};
use clap::Parser;
use serde_json::Value;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::Stdio;

#[derive(Parser, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Args {
    pub package: String,
    #[arg(last = true)]
    pub args: Vec<String>,
}

pub fn parse_package(package: &str) -> Result<(String, String), PmError> {
    let package = package.trim();
    if package.is_empty() {
        return Err(PmError::InvalidPackageSpec {
            spec: package.to_string(),
        });
    }

    if package.starts_with('@') {
        let slash = package
            .find('/')
            .ok_or_else(|| PmError::InvalidPackageSpec {
                spec: package.to_string(),
            })?;
        let tail = &package[slash + 1..];

        if let Some(at) = tail.rfind('@') {
            let split_index = slash + 1 + at;
            let name = &package[..split_index];
            let version = &package[split_index + 1..];
            if version.is_empty() {
                return Err(PmError::InvalidPackageSpec {
                    spec: package.to_string(),
                });
            }
            return Ok((name.to_string(), version.to_string()));
        }

        return Ok((package.to_string(), "latest".to_string()));
    }

    if let Some((name, version)) = package.rsplit_once('@') {
        if name.is_empty() || version.is_empty() {
            return Err(PmError::InvalidPackageSpec {
                spec: package.to_string(),
            });
        }
        return Ok((name.to_string(), version.to_string()));
    }

    Ok((package.to_string(), "latest".to_string()))
}

pub fn process(package: &str, version: &str, args: &[String]) -> Result<(), PmError> {
    let temp = tempfile::tempdir().map_err(|source| PmError::CreateTempDir { source })?;
    let path = temp.path().to_owned();
    let current_dir = env::current_dir().map_err(|source| PmError::CurrentDir { source })?;

    let manifest = format!(
        r#"{{
  "name": "____o-x-generated____",
  "version": "1.0.0",
  "private": true,
  "dependencies": {{
    "{}": "{}"
  }}
}}"#,
        package, version
    );

    let manifest_path = path.join("package.json");
    fs::write(&manifest_path, manifest).map_err(|source| PmError::WriteGeneratedManifest {
        path: manifest_path.clone(),
        source,
    })?;

    let path_str = path
        .to_str()
        .ok_or_else(|| PmError::InvalidTempPath { path: path.clone() })?;

    install_from(path_str)?;

    let package_dir = install_dir(&path.join("node_modules"), package);
    let package_json_path = package_dir.join("package.json");
    let command_name = resolve_bin_command(package, &package_json_path)?;
    let command_path = resolve_shim_path(&path.join("node_modules"), &command_name);
    if !command_path.is_file() {
        return Err(PmError::MissingPackageBinary {
            package: package.to_string(),
            command: command_name,
            path: command_path,
        });
    }

    let status = Command::new(&command_path)
        .args(args)
        .current_dir(&current_dir)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|source| PmError::SpawnPackageBinary {
            package: package.to_string(),
            command: command_path.clone(),
            source,
        })?;

    if !status.success() {
        return Err(PmError::PackageBinaryFailed {
            package: package.to_string(),
            command: command_path,
            status: status
                .code()
                .map(|code| code.to_string())
                .unwrap_or_else(|| status.to_string()),
            stderr: None,
        });
    }

    Ok(())
}

fn resolve_bin_command(package: &str, package_json_path: &Path) -> Result<String, PmError> {
    let source =
        fs::read_to_string(package_json_path).map_err(|source| PmError::ReadInstalledManifest {
            path: package_json_path.to_path_buf(),
            source,
        })?;
    let value: Value = serde_json::from_str(&source).map_err(|source| PmError::ParseManifest {
        path: package_json_path.to_path_buf(),
        source,
    })?;

    let default_name = default_bin_name(package);
    let Some(bin_value) = value.get("bin") else {
        return Ok(default_name);
    };

    match bin_value {
        Value::String(_) => Ok(default_name),
        Value::Object(entries) => {
            if entries.contains_key(&default_name) {
                return Ok(default_name);
            }

            if entries.len() == 1 {
                if let Some((name, _)) = entries.iter().next() {
                    return Ok(name.clone());
                }
            }

            Err(PmError::AmbiguousBinEntry {
                package: package.to_string(),
                path: package_json_path.to_path_buf(),
                available: entries.keys().cloned().collect(),
            })
        }
        Value::Null => Ok(default_name),
        _ => Err(PmError::InvalidBinField {
            path: package_json_path.to_path_buf(),
        }),
    }
}

fn default_bin_name(package: &str) -> String {
    package
        .rsplit_once('/')
        .map(|(_, name)| name.to_string())
        .unwrap_or_else(|| package.to_string())
}

fn install_dir(node_modules_dir: &Path, package_name: &str) -> PathBuf {
    if let Some((scope, name)) = package_name.split_once('/') {
        node_modules_dir.join(scope).join(name)
    } else {
        node_modules_dir.join(package_name)
    }
}

#[cfg(unix)]
fn resolve_shim_path(node_modules_dir: &Path, command_name: &str) -> PathBuf {
    node_modules_dir.join(".bin").join(command_name)
}

#[cfg(windows)]
fn resolve_shim_path(node_modules_dir: &Path, command_name: &str) -> PathBuf {
    node_modules_dir
        .join(".bin")
        .join(format!("{command_name}.cmd"))
}
