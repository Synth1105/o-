use flate2::read::GzDecoder;
use crate::lock::{LockCollector, write_lockfile};
use crate::report::Report;
use nodejs_semver::{Range, Version};
use package_json::PackageJson;
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::Value;
use ssri::Integrity;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tar::Archive;
use tempfile::tempdir;

pub fn read_manifest(path: &str) -> io::Result<PackageJson> {
    let manifest_path = find_manifest_path(Path::new(path))?;
    read_manifest_from_path(&manifest_path)
}

fn read_manifest_from_path(path: &Path) -> io::Result<PackageJson> {
    let source = fs::read_to_string(path)?;
    serde_json::from_str(&source).map_err(|source| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("failed to parse {}: {source}", path.display()),
        )
    })
}

fn find_manifest_path(start: &Path) -> io::Result<PathBuf> {
    let mut current = if start.is_dir() {
        start.to_path_buf()
    } else {
        start.parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."))
    };

    loop {
        let candidate = current.join("package.json");
        if candidate.is_file() {
            return Ok(candidate);
        }

        if !current.pop() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("failed to find package.json from {}", start.display()),
            ));
        }
    }
}

#[derive(Debug, Deserialize)]
struct Packument {
    versions: HashMap<String, RegistryVersion>,
}

#[derive(Debug, Deserialize)]
struct RegistryVersion {
    #[serde(default)]
    dependencies: HashMap<String, String>,
    dist: RegistryDist,
}

#[derive(Debug, Deserialize)]
struct RegistryDist {
    tarball: String,
    integrity: Option<String>,
}

#[derive(Debug)]
struct ResolvedPackage {
    name: String,
    version: String,
    dependencies: HashMap<String, String>,
    tarball_url: String,
    integrity: Option<String>,
}

#[derive(Debug)]
pub enum PmError {
    FindManifest { start: PathBuf, source: io::Error },
    ReadManifest { path: PathBuf, source: io::Error },
    ParseManifest { path: PathBuf, source: serde_json::Error },
    ProjectRootMissing { path: PathBuf },
    CreateDir { path: PathBuf, source: io::Error },
    FetchMetadata { package: String, source: reqwest::Error },
    MetadataStatus { package: String, source: reqwest::Error },
    ReadMetadataBody { package: String, source: reqwest::Error },
    ParseMetadata {
        package: String,
        source: serde_json::Error,
    },
    InvalidRange {
        package: String,
        range: String,
        source: String,
    },
    VersionNotFound { package: String, range: String },
    MissingResolvedVersion { package: String, version: String },
    DownloadTarball { package: String, source: reqwest::Error },
    TarballStatus { package: String, source: reqwest::Error },
    ReadTarballBody { package: String, source: reqwest::Error },
    InvalidIntegrity {
        package: String,
        version: String,
        source: String,
    },
    IntegrityMismatch {
        package: String,
        version: String,
        source: String,
    },
    ExtractTarball {
        package: String,
        source: io::Error,
    },
    MissingPackageDir { package: String, path: PathBuf },
    RemoveExistingInstall { path: PathBuf, source: io::Error },
    CopyInstall {
        from: PathBuf,
        to: PathBuf,
        source: io::Error,
    },
    ReadInstalledManifest { path: PathBuf, source: io::Error },
    MissingInstalledName { path: PathBuf },
    InvalidBinField { path: PathBuf },
    InvalidBinEntry { path: PathBuf, entry: String },
    MissingBinTarget {
        package_dir: PathBuf,
        target: PathBuf,
    },
    CreateBinLink {
        command: String,
        path: PathBuf,
        source: io::Error,
    },
    WriteLockfile { path: PathBuf, source: io::Error },
}

impl PmError {
    pub fn report(&self) -> Report {
        match self {
            Self::FindManifest { start, source } => Report::new("failed to find package.json")
                .detail(format!("start: {}", start.display()))
                .detail(format!("cause: {source}")),
            Self::ReadManifest { path, source } => Report::new("failed to read package.json")
                .detail(format!("path: {}", path.display()))
                .detail(format!("cause: {source}")),
            Self::ParseManifest { path, source } => Report::new("failed to parse package.json")
                .detail(format!("path: {}", path.display()))
                .detail(format!("cause: {source}")),
            Self::ProjectRootMissing { path } => Report::new("failed to resolve project root")
                .detail(format!("path: {}", path.display())),
            Self::CreateDir { path, source } => Report::new("failed to create directory")
                .detail(format!("path: {}", path.display()))
                .detail(format!("cause: {source}")),
            Self::FetchMetadata { package, source } => {
                Report::new(format!("failed to fetch package metadata for `{package}`"))
                    .detail(format!("cause: {source}"))
            }
            Self::MetadataStatus { package, source } => {
                Report::new(format!("registry returned an error for `{package}`"))
                    .detail(format!("cause: {source}"))
            }
            Self::ReadMetadataBody { package, source } => {
                Report::new(format!("failed to read package metadata body for `{package}`"))
                    .detail(format!("cause: {source}"))
            }
            Self::ParseMetadata { package, source } => {
                Report::new(format!("failed to decode package metadata for `{package}`"))
                    .detail(format!("cause: {source}"))
            }
            Self::InvalidRange {
                package,
                range,
                source,
            } => Report::new(format!("invalid semver range for `{package}`"))
                .detail(format!("range: {range}"))
                .detail(format!("cause: {source}")),
            Self::VersionNotFound { package, range } => {
                Report::new(format!("no version of `{package}` satisfies the requested range"))
                    .detail(format!("range: {range}"))
            }
            Self::MissingResolvedVersion { package, version } => {
                Report::new(format!("registry metadata is incomplete for `{package}`"))
                    .detail(format!("version: {version}"))
            }
            Self::DownloadTarball { package, source } => {
                Report::new(format!("failed to download tarball for `{package}`"))
                    .detail(format!("cause: {source}"))
            }
            Self::TarballStatus { package, source } => {
                Report::new(format!("tarball request failed for `{package}`"))
                    .detail(format!("cause: {source}"))
            }
            Self::ReadTarballBody { package, source } => {
                Report::new(format!("failed to read tarball body for `{package}`"))
                    .detail(format!("cause: {source}"))
            }
            Self::InvalidIntegrity {
                package,
                version,
                source,
            } => Report::new(format!("registry integrity is invalid for `{package}`"))
                .detail(format!("version: {version}"))
                .detail(format!("cause: {source}")),
            Self::IntegrityMismatch {
                package,
                version,
                source,
            } => Report::new(format!("integrity check failed for `{package}`"))
                .detail(format!("version: {version}"))
                .detail(format!("cause: {source}")),
            Self::ExtractTarball { package, source } => {
                Report::new(format!("failed to extract tarball for `{package}`"))
                    .detail(format!("cause: {source}"))
            }
            Self::MissingPackageDir { package, path } => {
                Report::new(format!("downloaded tarball for `{package}` is malformed"))
                    .detail(format!("missing: {}", path.display()))
            }
            Self::RemoveExistingInstall { path, source } => {
                Report::new("failed to remove existing package installation")
                    .detail(format!("path: {}", path.display()))
                    .detail(format!("cause: {source}"))
            }
            Self::CopyInstall { from, to, source } => Report::new("failed to copy package files")
                .detail(format!("from: {}", from.display()))
                .detail(format!("to: {}", to.display()))
                .detail(format!("cause: {source}")),
            Self::ReadInstalledManifest { path, source } => {
                Report::new("failed to read installed package manifest")
                    .detail(format!("path: {}", path.display()))
                    .detail(format!("cause: {source}"))
            }
            Self::MissingInstalledName { path } => {
                Report::new("installed package manifest is missing `name`")
                    .detail(format!("path: {}", path.display()))
            }
            Self::InvalidBinField { path } => Report::new("installed package has an invalid `bin` field")
                .detail(format!("path: {}", path.display())),
            Self::InvalidBinEntry { path, entry } => {
                Report::new("installed package has an invalid `bin` entry")
                    .detail(format!("path: {}", path.display()))
                    .detail(format!("entry: {entry}"))
            }
            Self::MissingBinTarget { package_dir, target } => {
                Report::new("installed package bin target does not exist")
                    .detail(format!("package: {}", package_dir.display()))
                    .detail(format!("target: {}", target.display()))
            }
            Self::CreateBinLink {
                command,
                path,
                source,
            } => Report::new(format!("failed to create bin link `{command}`"))
                .detail(format!("path: {}", path.display()))
                .detail(format!("cause: {source}")),
            Self::WriteLockfile { path, source } => Report::new("failed to write package-lock.json")
                .detail(format!("path: {}", path.display()))
                .detail(format!("cause: {source}")),
        }
    }
}

pub fn install() -> Result<Report, PmError> {
    install_from(".")
}

pub fn install_from(path: &str) -> Result<Report, PmError> {
    let manifest_path = find_manifest_path(Path::new(path)).map_err(|source| PmError::FindManifest {
        start: PathBuf::from(path),
        source,
    })?;
    let project_root = manifest_path
        .parent()
        .ok_or_else(|| PmError::ProjectRootMissing {
            path: manifest_path.clone(),
        })?;
    let manifest = read_manifest_from_path(&manifest_path).map_err(|source| map_manifest_error(&manifest_path, source))?;
    let node_modules = project_root.join("node_modules");
    fs::create_dir_all(&node_modules).map_err(|source| PmError::CreateDir {
        path: node_modules.clone(),
        source,
    })?;

    let mut installed = HashSet::new();
    let mut lock = LockCollector::new();
    lock.insert_root(&manifest);
    let client = Client::new();

    let root_dependencies = manifest.dependencies.clone().unwrap_or_default();
    let dependency_count = root_dependencies.len();

    install_dependency_set(
        &client,
        project_root,
        &root_dependencies,
        &node_modules,
        &mut installed,
        &mut lock,
    )?;

    let lockfile = lock.into_lockfile(&manifest);
    let lockfile_path = write_lockfile(project_root, &lockfile).map_err(|source| PmError::WriteLockfile {
        path: project_root.join("package-lock.json"),
        source,
    })?;

    Ok(Report::new("installed project dependencies")
        .detail(format!("root: {}", project_root.display()))
        .detail(format!("dependencies: {dependency_count}"))
        .detail(format!("lockfile: {}", lockfile_path.display())))
}

fn install_dependency_set(
    client: &Client,
    project_root: &Path,
    dependencies: &HashMap<String, String>,
    node_modules_dir: &Path,
    installed: &mut HashSet<String>,
    lock: &mut LockCollector,
) -> Result<(), PmError> {
    for (name, range) in dependencies {
        install_dependency(client, project_root, name, range, node_modules_dir, installed, lock)?;
    }

    Ok(())
}

fn install_dependency(
    client: &Client,
    project_root: &Path,
    name: &str,
    range: &str,
    node_modules_dir: &Path,
    installed: &mut HashSet<String>,
    lock: &mut LockCollector,
) -> Result<(), PmError> {
    let resolved = resolve_package(client, name, range)?;
    let install_key = format!("{}@{}::{}", resolved.name, resolved.version, node_modules_dir.display());

    if !installed.insert(install_key) {
        return Ok(());
    }

    let target_dir = install_dir(node_modules_dir, &resolved.name);
    if is_matching_install(&target_dir, &resolved.version)? {
        lock.insert_package(
            project_root,
            &target_dir,
            &resolved.name,
            &resolved.version,
            &resolved.tarball_url,
            resolved.integrity.as_deref(),
            &resolved.dependencies,
        ).map_err(|source| PmError::WriteLockfile {
            path: project_root.join("package-lock.json"),
            source,
        })?;
        install_dependency_set(
            client,
            project_root,
            &resolved.dependencies,
            &target_dir.join("node_modules"),
            installed,
            lock,
        )?;
        return Ok(());
    }

    if let Some(parent) = target_dir.parent() {
        fs::create_dir_all(parent).map_err(|source| PmError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    let package_root = download_and_extract_package(client, &resolved)?;

    if target_dir.exists() {
        fs::remove_dir_all(&target_dir).map_err(|source| PmError::RemoveExistingInstall {
            path: target_dir.clone(),
            source,
        })?;
    }
    copy_dir_all(&package_root, &target_dir).map_err(|source| PmError::CopyInstall {
        from: package_root.clone(),
        to: target_dir.clone(),
        source,
    })?;
    create_bin_links(node_modules_dir, &target_dir)?;
    lock.insert_package(
        project_root,
        &target_dir,
        &resolved.name,
        &resolved.version,
        &resolved.tarball_url,
        resolved.integrity.as_deref(),
        &resolved.dependencies,
    ).map_err(|source| PmError::WriteLockfile {
        path: project_root.join("package-lock.json"),
        source,
    })?;

    let nested_node_modules = target_dir.join("node_modules");
    fs::create_dir_all(&nested_node_modules).map_err(|source| PmError::CreateDir {
        path: nested_node_modules.clone(),
        source,
    })?;
    install_dependency_set(
        client,
        project_root,
        &resolved.dependencies,
        &nested_node_modules,
        installed,
        lock,
    )?;

    Ok(())
}

fn resolve_package(client: &Client, name: &str, range: &str) -> Result<ResolvedPackage, PmError> {
    let url = resolve_npm_url(name);
    let response = client
        .get(url)
        .send()
        .map_err(|source| PmError::FetchMetadata {
            package: name.to_string(),
            source,
        })?;

    let response = response.error_for_status().map_err(|source| {
        PmError::MetadataStatus {
            package: name.to_string(),
            source,
        }
    })?;

    let body = response.text().map_err(|source| {
        PmError::ReadMetadataBody {
            package: name.to_string(),
            source,
        }
    })?;

    let packument: Packument = serde_json::from_str(&body).map_err(|source| {
        PmError::ParseMetadata {
            package: name.to_string(),
            source,
        }
    })?;

    let range: Range = range.parse().map_err(|source: nodejs_semver::SemverError| {
        PmError::InvalidRange {
            package: name.to_string(),
            range: range.to_string(),
            source: source.to_string(),
        }
    })?;

    let version = packument
        .versions
        .keys()
        .filter_map(|raw_version| {
            Version::parse(raw_version)
                .ok()
                .map(|parsed| (raw_version, parsed))
        })
        .filter(|(_, parsed)| parsed.satisfies(&range))
        .map(|(_, parsed)| parsed)
        .max()
        .ok_or_else(|| PmError::VersionNotFound {
            package: name.to_string(),
            range: range.to_string(),
        })?;

    let version_string = version.to_string();
    let metadata = packument.versions.get(&version_string).ok_or_else(|| {
        PmError::MissingResolvedVersion {
            package: name.to_string(),
            version: version_string.clone(),
        }
    })?;

    Ok(ResolvedPackage {
        name: name.to_string(),
        version: version_string,
        dependencies: metadata.dependencies.clone(),
        tarball_url: metadata.dist.tarball.clone(),
        integrity: metadata.dist.integrity.clone(),
    })
}

fn download_and_extract_package(client: &Client, package: &ResolvedPackage) -> Result<PathBuf, PmError> {
    let response = client
        .get(&package.tarball_url)
        .send()
        .map_err(|source| PmError::DownloadTarball {
            package: package.name.clone(),
            source,
        })?;
    let response = response.error_for_status().map_err(|source| {
        PmError::TarballStatus {
            package: package.name.clone(),
            source,
        }
    })?;

    let bytes = response.bytes().map_err(|source| {
        PmError::ReadTarballBody {
            package: package.name.clone(),
            source,
        }
    })?;
    verify_integrity(package, bytes.as_ref())?;

    let temp = tempdir().map_err(|source| PmError::ExtractTarball {
        package: package.name.clone(),
        source,
    })?;
    let temp_path = temp.keep();

    let tar = GzDecoder::new(bytes.as_ref());
    let mut archive = Archive::new(tar);
    archive.unpack(&temp_path).map_err(|source| PmError::ExtractTarball {
        package: package.name.clone(),
        source,
    })?;

    let package_root = temp_path.join("package");
    if !package_root.is_dir() {
        return Err(PmError::MissingPackageDir {
            package: package.name.clone(),
            path: package_root,
        });
    }

    Ok(package_root)
}

fn verify_integrity(package: &ResolvedPackage, bytes: &[u8]) -> Result<(), PmError> {
    let Some(integrity) = &package.integrity else {
        return Ok(());
    };

    let parsed: Integrity = integrity.parse().map_err(|source: ssri::Error| {
        PmError::InvalidIntegrity {
            package: package.name.clone(),
            version: package.version.clone(),
            source: source.to_string(),
        }
    })?;

    parsed.check(bytes).map_err(|source: ssri::Error| {
        PmError::IntegrityMismatch {
            package: package.name.clone(),
            version: package.version.clone(),
            source: source.to_string(),
        }
    })?;

    Ok(())
}

fn install_dir(node_modules: &Path, package_name: &str) -> PathBuf {
    if let Some((scope, name)) = package_name.split_once('/') {
        node_modules.join(scope).join(name)
    } else {
        node_modules.join(package_name)
    }
}

fn is_matching_install(path: &Path, version: &str) -> Result<bool, PmError> {
    let manifest_path = path.join("package.json");
    if !manifest_path.is_file() {
        return Ok(false);
    }

    let manifest = read_manifest_from_path(&manifest_path)
        .map_err(|source| map_manifest_error(&manifest_path, source))?;
    Ok(manifest.version == version)
}

fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_all(&from, &to)?;
        } else {
            fs::copy(&from, &to)?;
        }
    }

    Ok(())
}

fn create_bin_links(node_modules_dir: &Path, package_dir: &Path) -> Result<(), PmError> {
    let bin_entries = read_bin_entries(package_dir)?;
    if bin_entries.is_empty() {
        return Ok(());
    }

    let bin_dir = node_modules_dir.join(".bin");
    fs::create_dir_all(&bin_dir).map_err(|source| PmError::CreateDir {
        path: bin_dir.clone(),
        source,
    })?;

    for (command_name, relative_target) in bin_entries {
        let target = package_dir.join(normalize_package_relative_path(&relative_target));
        if !target.is_file() {
            return Err(PmError::MissingBinTarget {
                package_dir: package_dir.to_path_buf(),
                target,
            });
        }

        create_bin_link(&bin_dir, &command_name, &target)?;
    }

    Ok(())
}

fn read_bin_entries(package_dir: &Path) -> Result<Vec<(String, String)>, PmError> {
    let package_json_path = package_dir.join("package.json");
    let source = fs::read_to_string(&package_json_path).map_err(|source| PmError::ReadInstalledManifest {
        path: package_json_path.clone(),
        source,
    })?;
    let value: Value = serde_json::from_str(&source).map_err(|source| PmError::ParseManifest {
        path: package_json_path.clone(),
        source,
    })?;

    let package_name = value
        .get("name")
        .and_then(Value::as_str)
        .map(default_bin_name)
        .ok_or_else(|| PmError::MissingInstalledName {
            path: package_json_path.clone(),
        })?;

    let Some(bin_value) = value.get("bin") else {
        return Ok(Vec::new());
    };

    match bin_value {
        Value::String(path) => Ok(vec![(package_name, path.clone())]),
        Value::Object(entries) => {
            let mut bins = Vec::with_capacity(entries.len());
            for (command_name, target) in entries {
                let target = target.as_str().ok_or_else(|| {
                    PmError::InvalidBinEntry {
                        path: package_json_path.clone(),
                        entry: command_name.clone(),
                    }
                })?;
                bins.push((command_name.clone(), target.to_string()));
            }
            Ok(bins)
        }
        Value::Null => Ok(Vec::new()),
        _ => Err(PmError::InvalidBinField {
            path: package_json_path,
        }),
    }
}

fn default_bin_name(package_name: &str) -> String {
    package_name
        .rsplit_once('/')
        .map(|(_, name)| name.to_string())
        .unwrap_or_else(|| package_name.to_string())
}

fn normalize_package_relative_path(path: &str) -> PathBuf {
    let trimmed = path.strip_prefix("./").unwrap_or(path);
    PathBuf::from(trimmed)
}

#[cfg(unix)]
fn create_bin_link(bin_dir: &Path, command_name: &str, target: &Path) -> Result<(), PmError> {
    use std::os::unix::fs::symlink;

    let link_path = bin_dir.join(command_name);
    remove_existing_link_path(&link_path).map_err(|source| PmError::CreateBinLink {
        command: command_name.to_string(),
        path: link_path.clone(),
        source,
    })?;
    symlink(target, &link_path).map_err(|source| PmError::CreateBinLink {
        command: command_name.to_string(),
        path: link_path,
        source,
    })
}

#[cfg(windows)]
fn create_bin_link(bin_dir: &Path, command_name: &str, target: &Path) -> Result<(), PmError> {
    let link_path = bin_dir.join(format!("{command_name}.cmd"));
    remove_existing_link_path(&link_path).map_err(|source| PmError::CreateBinLink {
        command: command_name.to_string(),
        path: link_path.clone(),
        source,
    })?;
    let script = format!("@ECHO off\r\nnode \"{}\" %*\r\n", target.display());
    fs::write(&link_path, script).map_err(|source| PmError::CreateBinLink {
        command: command_name.to_string(),
        path: link_path,
        source,
    })
}

fn remove_existing_link_path(path: &Path) -> io::Result<()> {
    if !path.exists() {
        return Ok(());
    }

    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}

fn resolve_npm_url(package: &str) -> String {
    let encoded = package.replace('@', "%40").replace('/', "%2F");
    format!("https://registry.npmjs.org/{encoded}")
}



pub fn uninstall(_name: &str) {}

fn map_manifest_error(path: &Path, source: io::Error) -> PmError {
    match source.kind() {
        io::ErrorKind::InvalidData => {
            let parse_source = serde_json::Error::io(io::Error::new(source.kind(), source.to_string()));
            PmError::ParseManifest {
                path: path.to_path_buf(),
                source: parse_source,
            }
        }
        _ => PmError::ReadManifest {
            path: path.to_path_buf(),
            source,
        },
    }
}
