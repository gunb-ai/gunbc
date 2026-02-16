use std::path::{Component, Path, PathBuf};

pub fn resolve_default_root() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    default_root_from_cwd(&cwd)
}

pub fn normalize_cli_path(path: PathBuf) -> PathBuf {
    let absolute = if path.is_absolute() {
        path
    } else {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        cwd.join(path)
    };
    normalize_path_components(&absolute)
}

pub fn default_root_from_cwd(cwd: &Path) -> PathBuf {
    normalize_path_components(&cwd.join("dsl"))
}

pub fn normalize_path_components(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(Path::new(std::path::MAIN_SEPARATOR_STR)),
            Component::CurDir => {}
            Component::ParentDir => {
                let has_root = normalized.has_root();
                if normalized.as_os_str().is_empty() || normalized.ends_with("..") {
                    if !has_root {
                        normalized.push("..");
                    }
                } else if !normalized.pop() && !has_root {
                    normalized.push("..");
                }
            }
            Component::Normal(segment) => normalized.push(segment),
        }
    }
    if normalized.as_os_str().is_empty() && !path.has_root() {
        return PathBuf::from(".");
    }
    normalized
}
