use std::path::{Component, Path, PathBuf};

pub use daglang_resolve::has_dag_extension;

pub fn is_single_file_target(path: &Path, treat_dag_directories_as_files: bool) -> bool {
    if !has_dag_extension(path) {
        return false;
    }
    if treat_dag_directories_as_files {
        return true;
    }
    !path.is_dir()
}

pub fn resolve_default_root(cwd: &Path) -> PathBuf {
    normalize_path_components(&cwd.join("dsl"))
}

pub fn normalize_cli_path(cwd: &Path, path: &Path) -> PathBuf {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    };
    normalize_path_components(&absolute)
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

#[cfg(test)]
#[allow(clippy::disallowed_methods, clippy::disallowed_types)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "daglang_path_utils_{name}_{}_{}",
            std::process::id(),
            nanos
        ))
    }

    #[test]
    fn is_single_file_target_returns_true_for_dag_file() {
        let file = unique_temp_path("dag_file").join("sample.dag");
        std::fs::create_dir_all(file.parent().expect("fixture file should have parent"))
            .expect("failed to create fixture directory");
        std::fs::write(&file, "module sample.file\nfn run() -> Unit {}")
            .expect("failed to write fixture file");

        assert!(is_single_file_target(&file, true));
        assert!(is_single_file_target(&file, false));

        std::fs::remove_dir_all(
            file.parent()
                .expect("fixture file should have parent"),
        )
        .expect("failed to cleanup fixture directory");
    }

    #[test]
    fn is_single_file_target_respects_dag_directory_mode_flag() {
        let dir = unique_temp_path("dag_directory").join("sample.dag");
        std::fs::create_dir_all(&dir).expect("failed to create dag-named directory");

        assert!(
            is_single_file_target(&dir, true),
            "compile-style mode should treat .dag directories as single-file targets"
        );
        assert!(
            !is_single_file_target(&dir, false),
            "check-style mode should keep .dag directories in directory mode"
        );

        std::fs::remove_dir_all(dir).expect("failed to cleanup dag-named directory");
    }

    #[test]
    fn is_single_file_target_returns_false_for_non_dag_path() {
        let path = PathBuf::from("sample/not_dag.txt");
        assert!(!is_single_file_target(&path, true));
        assert!(!is_single_file_target(&path, false));
    }

    #[test]
    fn normalize_path_components_preserves_leading_parent_segments() {
        let input = PathBuf::from("../workspace/./dsl/../tools");
        assert_eq!(
            normalize_path_components(&input),
            PathBuf::from("../workspace/tools")
        );
    }

    #[test]
    fn normalize_path_components_never_traverses_above_absolute_root() {
        let input = PathBuf::from("/tmp/../../etc");
        assert_eq!(normalize_path_components(&input), PathBuf::from("/etc"));
    }
}
