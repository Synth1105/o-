use std::path::{Component, Path, PathBuf};

pub fn resolve(paths: &[&str]) -> String {
    let mut resolved = PathBuf::new();
    for &path in paths {
        if path.is_empty() {
            continue;
        }
        let p = Path::new(path);
        if p.is_absolute() {
            resolved = p.to_path_buf();
        } else {
            resolved.push(p);
        }
    }
    if resolved.as_os_str().is_empty() {
        return ".".to_string();
    }
    normalize(&resolved.to_string_lossy())
}

pub fn normalize(path: &str) -> String {
    let mut components = Vec::new();
    let is_absolute = path.starts_with('/');

    for component in Path::new(path).components() {
        match component {
            Component::Normal(seg) => components.push(seg.to_string_lossy().to_string()),
            Component::ParentDir => {
                if components.last().map(|s| s.as_str()) != Some("..") {
                    if !components.is_empty() {
                        components.pop();
                    } else if !is_absolute {
                        components.push("..".to_string());
                    }
                }
            }
            Component::RootDir => {}
            Component::CurDir => {}
            Component::Prefix(_) => {}
        }
    }

    let result = components.join("/");
    if is_absolute {
        format!("/{}", result)
    } else if result.is_empty() {
        ".".to_string()
    } else {
        result
    }
}

pub fn join(base: &str, paths: &[&str]) -> String {
    let mut result = base.to_string();
    for &p in paths {
        if result.is_empty() || result == "." {
            result = p.to_string();
        } else {
            result = format!("{}/{}", result, p);
        }
    }
    normalize(&result)
}

pub fn dirname(path: &str) -> String {
    let p = Path::new(path);
    p.parent()
        .map(|parent| {
            let s = parent.to_string_lossy().to_string();
            if s.is_empty() { ".".to_string() } else { s }
        })
        .unwrap_or_else(|| ".".to_string())
}

pub fn basename(path: &str, ext: Option<&str>) -> String {
    let p = Path::new(path);
    let name = p
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    if let Some(suffix) = ext {
        if !suffix.is_empty() && name.ends_with(suffix) {
            return name[..name.len() - suffix.len()].to_string();
        }
    }
    name
}

pub fn extname(path: &str) -> String {
    let p = Path::new(path);
    p.extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default()
}

pub fn is_absolute(path: &str) -> bool {
    Path::new(path).is_absolute()
}

pub fn relative(from: &str, to: &str) -> String {
    let from_path = Path::new(from)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(from));
    let to_path = Path::new(to)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(to));

    if from_path == to_path {
        return String::new();
    }

    let from_components: Vec<_> = from_path.components().collect();
    let to_components: Vec<_> = to_path.components().collect();

    let mut common = 0;
    let min_len = from_components.len().min(to_components.len());
    for i in 0..min_len {
        if from_components[i] == to_components[i] {
            common = i + 1;
        } else {
            break;
        }
    }

    let mut result = String::new();
    for _ in common..from_components.len() {
        if !result.is_empty() {
            result.push('/');
        }
        result.push_str("..");
    }

    for component in &to_components[common..] {
        if !result.is_empty() {
            result.push('/');
        }
        if let Component::Normal(seg) = component {
            result.push_str(&seg.to_string_lossy());
        }
    }

    if result.is_empty() {
        ".".to_string()
    } else {
        result
    }
}

pub struct ParsedPath {
    pub root: String,
    pub dir: String,
    pub base: String,
    pub ext: String,
    pub name: String,
}

pub fn parse(path: &str) -> ParsedPath {
    let p = Path::new(path);
    let root = if p.is_absolute() {
        "/".to_string()
    } else {
        String::new()
    };
    let dir = dirname(path);
    let base = basename(path, None);
    let ext = extname(path);
    let name = if !ext.is_empty() {
        base[..base.len() - ext.len()].to_string()
    } else {
        base.clone()
    };

    ParsedPath {
        root,
        dir,
        base,
        ext,
        name,
    }
}

pub fn format(parsed: &ParsedPath) -> String {
    let mut result = String::new();
    if !parsed.dir.is_empty() {
        result.push_str(&parsed.dir);
        result.push('/');
    } else if !parsed.root.is_empty() {
        result.push_str(&parsed.root);
    }
    result.push_str(&parsed.base);
    result
}

pub const SEP: &str = "/";
pub const DELIMITER: &str = ":";
