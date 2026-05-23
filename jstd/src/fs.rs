use std::fs;
use std::io::{self, Write};
use std::path::Path;

#[derive(Debug)]
pub enum FsError {
    Io(io::Error),
    Utf8(std::string::FromUtf8Error),
    NotFound(String),
    PermissionDenied(String),
}

impl std::fmt::Display for FsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FsError::Io(e) => write!(f, "{}", e),
            FsError::Utf8(e) => write!(f, "{}", e),
            FsError::NotFound(path) => write!(f, "ENOENT: no such file or directory, '{}'", path),
            FsError::PermissionDenied(path) => write!(f, "EACCES: permission denied, '{}'", path),
        }
    }
}

impl From<io::Error> for FsError {
    fn from(e: io::Error) -> Self {
        match e.kind() {
            io::ErrorKind::NotFound => FsError::NotFound(e.to_string()),
            io::ErrorKind::PermissionDenied => FsError::PermissionDenied(e.to_string()),
            _ => FsError::Io(e),
        }
    }
}

pub type Result<T> = std::result::Result<T, FsError>;

pub fn read_file_sync(path: &str) -> Result<String> {
    let content = fs::read_to_string(path)?;
    Ok(content)
}

pub fn read_file_sync_binary(path: &str) -> Result<Vec<u8>> {
    let content = fs::read(path)?;
    Ok(content)
}

pub fn write_file_sync(path: &str, data: &str) -> Result<()> {
    fs::write(path, data)?;
    Ok(())
}

pub fn write_file_sync_binary(path: &str, data: &[u8]) -> Result<()> {
    fs::write(path, data)?;
    Ok(())
}

pub fn append_file_sync(path: &str, data: &str) -> Result<()> {
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}

pub fn exists_sync(path: &str) -> bool {
    Path::new(path).exists()
}

pub fn mkdir_sync(path: &str, recursive: bool) -> Result<()> {
    if recursive {
        fs::create_dir_all(path)?;
    } else {
        fs::create_dir(path)?;
    }
    Ok(())
}

pub fn rmdir_sync(path: &str, recursive: bool) -> Result<()> {
    if recursive {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_dir(path)?;
    }
    Ok(())
}

pub fn unlink_sync(path: &str) -> Result<()> {
    fs::remove_file(path)?;
    Ok(())
}

pub fn rename_sync(old_path: &str, new_path: &str) -> Result<()> {
    fs::rename(old_path, new_path)?;
    Ok(())
}

pub fn copy_file_sync(src: &str, dest: &str) -> Result<()> {
    fs::copy(src, dest)?;
    Ok(())
}

pub struct StatInfo {
    pub is_file: bool,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub size: u64,
}

pub fn stat_sync(path: &str) -> Result<StatInfo> {
    let metadata = fs::metadata(path)?;
    Ok(StatInfo {
        is_file: metadata.is_file(),
        is_dir: metadata.is_dir(),
        is_symlink: false,
        size: metadata.len(),
    })
}

pub fn lstat_sync(path: &str) -> Result<StatInfo> {
    let metadata = fs::symlink_metadata(path)?;
    Ok(StatInfo {
        is_file: metadata.is_file(),
        is_dir: metadata.is_dir(),
        is_symlink: metadata.file_type().is_symlink(),
        size: metadata.len(),
    })
}

pub fn readdir_sync(path: &str) -> Result<Vec<String>> {
    let entries = fs::read_dir(path)?;
    let mut names = Vec::new();
    for entry in entries {
        let entry = entry?;
        if let Some(name) = entry.file_name().to_str() {
            names.push(name.to_string());
        }
    }
    Ok(names)
}

pub fn realpath_sync(path: &str) -> Result<String> {
    let canonical = fs::canonicalize(path)?;
    Ok(canonical.to_string_lossy().to_string())
}

pub fn access_sync(path: &str) -> Result<()> {
    if !Path::new(path).exists() {
        return Err(FsError::NotFound(path.to_string()));
    }
    Ok(())
}

pub const O_RDONLY: i32 = 0;
pub const O_WRONLY: i32 = 1;
pub const O_RDWR: i32 = 2;
pub const F_OK: i32 = 0;
pub const R_OK: i32 = 4;
pub const W_OK: i32 = 2;
pub const X_OK: i32 = 1;
