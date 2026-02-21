use std::path::{Component, Path, PathBuf};

pub use daglang_resolve::{has_dag_extension, has_dag_like_extension};

/// Returns an error message if the path has a dag-like extension with wrong casing.
/// Returns `None` if the path is fine (either lowercase `.dag` or not dag-like at all).
pub fn check_dag_extension_casing(path: &Path) -> Option<String> {
    if has_dag_like_extension(path) && !has_dag_extension(path) {
        Some(format!(
            "path `{}` has wrong-cased extension; rename to `.dag` (lowercase)",
            path.display()
        ))
    } else {
        None
    }
}

pub fn is_single_file_target(path: &Path) -> bool {
    has_dag_extension(path) && !path.is_dir()
}

/// Returns an error message if `path` has a `.dag` extension but is a directory.
///
/// This rejects the ambiguous case where a directory is named `something.dag`
/// — callers should treat directories as module-discovery roots without a
/// `.dag` suffix, or reference individual `.dag` files inside the directory.
pub fn check_dag_directory_conflict(path: &Path) -> Option<String> {
    if has_dag_extension(path) && path.is_dir() {
        Some(format!(
            "failed to read {}: target is a directory. \
             `.dag` paths are treated as single-file targets; \
             pass the directory path without the `.dag` suffix, \
             or reference a `.dag` file inside it",
            path.display(),
        ))
    } else {
        None
    }
}

pub fn resolve_default_root(cwd: &Path) -> PathBuf {
    normalize_path_components(&cwd.join("dsl"))
}

pub fn resolve_single_file_root(cwd: &Path, file: &Path) -> PathBuf {
    for ancestor in file.ancestors() {
        if ancestor
            .file_name()
            .is_some_and(|name| name.eq_ignore_ascii_case("dsl"))
        {
            return normalize_path_components(ancestor);
        }
    }
    file.parent()
        .map(normalize_path_components)
        .unwrap_or_else(|| normalize_path_components(cwd))
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
                if has_root {
                    // Absolute path: pop one component but never above root.
                    // `pop()` returns false when already at root, so no action needed.
                    normalized.pop();
                } else if normalized.as_os_str().is_empty() || normalized.ends_with("..") {
                    // Relative path at the start or after leading `..` segments:
                    // preserve the `..` so the relative offset is kept.
                    normalized.push("..");
                } else if !normalized.pop() {
                    // Relative path where pop failed (empty after earlier ops):
                    // emit a leading `..`.
                    normalized.push("..");
                }
            }
            Component::Normal(segment) => normalized.push(segment),
        }
    }
    if normalized.as_os_str().is_empty() {
        if path.has_root() {
            return PathBuf::from(std::path::MAIN_SEPARATOR_STR);
        }
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

        assert!(is_single_file_target(&file));

        std::fs::remove_dir_all(file.parent().expect("fixture file should have parent"))
            .expect("failed to cleanup fixture directory");
    }

    #[test]
    fn is_single_file_target_returns_false_for_dag_directory() {
        let dir = unique_temp_path("dag_directory").join("sample.dag");
        std::fs::create_dir_all(&dir).expect("failed to create dag-named directory");

        assert!(
            !is_single_file_target(&dir),
            ".dag directories should never be treated as single-file targets"
        );

        std::fs::remove_dir_all(dir).expect("failed to cleanup dag-named directory");
    }

    #[test]
    fn check_dag_directory_conflict_rejects_dag_directory() {
        let dir = unique_temp_path("dag_dir_conflict").join("sample.dag");
        std::fs::create_dir_all(&dir).expect("failed to create dag-named directory");

        let error = check_dag_directory_conflict(&dir);
        assert!(
            error.is_some(),
            ".dag directory should produce a conflict error"
        );
        assert!(
            error.as_ref().unwrap().contains("is a directory"),
            "error should mention directory: {:?}",
            error
        );

        std::fs::remove_dir_all(dir).expect("failed to cleanup dag-named directory");
    }

    #[test]
    fn check_dag_directory_conflict_accepts_dag_file() {
        let file = unique_temp_path("dag_file_ok").join("sample.dag");
        std::fs::create_dir_all(file.parent().expect("fixture file should have parent"))
            .expect("failed to create fixture directory");
        std::fs::write(&file, "module sample.file\nfn run() -> Unit {}")
            .expect("failed to write fixture file");

        assert!(
            check_dag_directory_conflict(&file).is_none(),
            ".dag file should not produce a conflict error"
        );

        std::fs::remove_dir_all(file.parent().expect("fixture file should have parent"))
            .expect("failed to cleanup fixture directory");
    }

    #[test]
    fn is_single_file_target_returns_false_for_non_dag_path() {
        let path = PathBuf::from("sample/not_dag.txt");
        assert!(!is_single_file_target(&path));
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

    #[test]
    fn normalize_path_components_clamps_parent_at_root_to_root() {
        assert_eq!(normalize_path_components(&PathBuf::from("/..")), PathBuf::from("/"));
        assert_eq!(normalize_path_components(&PathBuf::from("/../..")), PathBuf::from("/"));
        assert_eq!(normalize_path_components(&PathBuf::from("/../../../../")), PathBuf::from("/"));
    }

    #[test]
    fn normalize_path_components_preserves_multiple_leading_parent_segments() {
        assert_eq!(
            normalize_path_components(&PathBuf::from("../../foo")),
            PathBuf::from("../../foo")
        );
        assert_eq!(
            normalize_path_components(&PathBuf::from("../../../bar/baz")),
            PathBuf::from("../../../bar/baz")
        );
    }

    #[test]
    fn normalize_path_components_returns_dot_for_fully_cancelled_relative_path() {
        assert_eq!(
            normalize_path_components(&PathBuf::from("foo/..")),
            PathBuf::from(".")
        );
    }

    #[test]
    fn resolve_single_file_root_prefers_dsl_ancestor() {
        let cwd = PathBuf::from("/workspace");
        let file = PathBuf::from("/workspace/dsl/cloud/gcp/credential.dag");
        assert_eq!(
            resolve_single_file_root(&cwd, &file),
            PathBuf::from("/workspace/dsl")
        );
    }

    #[test]
    fn resolve_single_file_root_falls_back_to_parent_without_dsl_ancestor() {
        let cwd = PathBuf::from("/workspace");
        let file = PathBuf::from("/tmp/custom/module.dag");
        assert_eq!(
            resolve_single_file_root(&cwd, &file),
            PathBuf::from("/tmp/custom")
        );
    }
}
