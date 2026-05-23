use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

pub fn cwd() -> String {
    env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .to_string_lossy()
        .to_string()
}

pub fn chdir(dir: &str) -> Result<(), String> {
    env::set_current_dir(dir).map_err(|e| e.to_string())
}

pub fn env_vars() -> HashMap<String, String> {
    env::vars().collect()
}

pub fn get_env(key: &str) -> Option<String> {
    env::var(key).ok()
}

pub fn set_env(key: &str, value: &str) {
    unsafe { env::set_var(key, value) };
}

pub fn unset_env(key: &str) {
    unsafe { env::remove_var(key) };
}

pub fn pid() -> u32 {
    std::process::id()
}

pub fn ppid() -> u32 {
    #[cfg(unix)]
    {
        0
    }
    #[cfg(not(unix))]
    {
        0
    }
}

pub fn platform() -> &'static str {
    if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "windows") {
        "win32"
    } else {
        "unknown"
    }
}

pub fn arch() -> &'static str {
    if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else if cfg!(target_arch = "x86") {
        "ia32"
    } else {
        "unknown"
    }
}

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub fn argv() -> Vec<String> {
    env::args().collect()
}

pub fn exec_path() -> String {
    env::current_exe()
        .unwrap_or_else(|_| PathBuf::from("o-"))
        .to_string_lossy()
        .to_string()
}

pub fn exit(code: i32) -> ! {
    std::process::exit(code)
}

pub struct ProcessInfo {
    pub pid: u32,
    pub platform: &'static str,
    pub arch: &'static str,
    pub version: &'static str,
    pub cwd: String,
    pub exec_path: String,
    pub argv: Vec<String>,
}

pub fn get_process_info() -> ProcessInfo {
    ProcessInfo {
        pid: pid(),
        platform: platform(),
        arch: arch(),
        version: version(),
        cwd: cwd(),
        exec_path: exec_path(),
        argv: argv(),
    }
}
