use package_json::PackageJson;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LockFile {
    pub name: Option<String>,
    pub version: Option<String>,
    #[serde(rename = "lockfileVersion")]
    pub lockfile_version: u8,
    pub packages: BTreeMap<String, LockedPackage>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct LockedPackage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integrity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "devDependencies")]
    pub dev_dependencies: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "optionalDependencies")]
    pub optional_dependencies: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "peerDependencies")]
    pub peer_dependencies: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Default)]
pub struct LockCollector {
    packages: BTreeMap<String, LockedPackage>,
}

impl LockCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_root(&mut self, manifest: &PackageJson) {
        self.insert_root_fields(
            &manifest.name,
            &manifest.version,
            &manifest.dependencies.clone().unwrap_or_default(),
            &manifest.dev_dependencies.clone().unwrap_or_default(),
            &manifest.optional_dependencies.clone().unwrap_or_default(),
            &manifest.peer_dependencies.clone().unwrap_or_default(),
        );
    }

    pub fn insert_root_fields(
        &mut self,
        name: &str,
        version: &str,
        dependencies: &HashMap<String, String>,
        dev_dependencies: &HashMap<String, String>,
        optional_dependencies: &HashMap<String, String>,
        peer_dependencies: &HashMap<String, String>,
    ) {
        self.packages.insert(
            String::new(),
            LockedPackage {
                name: Some(name.to_string()),
                version: Some(version.to_string()),
                resolved: None,
                integrity: None,
                dependencies: Some(to_btree(dependencies.clone())),
                dev_dependencies: Some(to_btree(dev_dependencies.clone())),
                optional_dependencies: Some(to_btree(optional_dependencies.clone())),
                peer_dependencies: Some(to_btree(peer_dependencies.clone())),
            },
        );
    }

    pub fn insert_package(
        &mut self,
        project_root: &Path,
        package_dir: &Path,
        name: &str,
        version: &str,
        resolved: &str,
        integrity: Option<&str>,
        dependencies: &HashMap<String, String>,
        optional_dependencies: &HashMap<String, String>,
        peer_dependencies: &HashMap<String, String>,
    ) -> io::Result<()> {
        let key = lockfile_key(project_root, package_dir)?;
        self.packages.insert(
            key,
            LockedPackage {
                name: Some(name.to_string()),
                version: Some(version.to_string()),
                resolved: Some(resolved.to_string()),
                integrity: integrity.map(str::to_string),
                dependencies: Some(to_btree(dependencies.clone())),
                dev_dependencies: None,
                optional_dependencies: Some(to_btree(optional_dependencies.clone())),
                peer_dependencies: Some(to_btree(peer_dependencies.clone())),
            },
        );
        Ok(())
    }

    pub fn into_lockfile(self, manifest: &PackageJson) -> LockFile {
        self.into_lockfile_fields(&manifest.name, &manifest.version)
    }

    pub fn into_lockfile_fields(self, name: &str, version: &str) -> LockFile {
        LockFile {
            name: Some(name.to_string()),
            version: Some(version.to_string()),
            lockfile_version: 3,
            packages: self.packages,
        }
    }
}

pub fn write_lockfile(project_root: &Path, lockfile: &LockFile) -> io::Result<PathBuf> {
    let path = project_root.join("package-lock.json");
    let temp_path = project_root.join(".package-lock.json.tmp");
    let mut json = serde_json::to_vec_pretty(lockfile).map_err(io::Error::other)?;
    json.push(b'\n');
    fs::write(&temp_path, json)?;
    fs::rename(&temp_path, &path)?;
    Ok(path)
}

fn lockfile_key(project_root: &Path, package_dir: &Path) -> io::Result<String> {
    let relative = package_dir
        .strip_prefix(project_root)
        .map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "package path `{}` is outside project root `{}`",
                    package_dir.display(),
                    project_root.display()
                ),
            )
        })?;

    Ok(relative.to_string_lossy().replace('\\', "/"))
}

fn to_btree(map: HashMap<String, String>) -> BTreeMap<String, String> {
    map.into_iter().collect()
}
