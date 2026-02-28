#[cfg(test)]
mod tests {
    use gunbc_ir::WorkspaceLayout;
    use std::collections::HashSet;
    use std::fs;
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        WorkspaceLayout::from_env_manifest_dir()
            .expect("resolve workspace layout")
            .workspace_root
    }

    #[test]
    fn repo_root_resolves_workspace_layout_root() {
        let root = repo_root();
        assert!(root.join("Cargo.toml").is_file());
        assert!(root.join("core").is_dir());
        assert!(root.join("lib").is_dir());
    }

    fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    collect_rs_files(&path, out);
                } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                    out.push(path);
                }
            }
        }
    }

    fn default_disallowed_methods_allowlist() -> HashSet<String> {
        HashSet::from(["lib/transport/".to_string()])
    }

    fn load_disallowed_methods_allowlist(root: &Path) -> HashSet<String> {
        let path = root.join("tools/disallowed-methods-allowlist.txt");
        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(_) => {
                // The shell guardrail script was removed; this generated file may
                // be absent on fresh checkouts. Fall back to the canonical
                // transport-boundary prefix used by pragma policy.
                return default_disallowed_methods_allowlist();
            }
        };
        let mut allowed = HashSet::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let line = line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            allowed.insert(line.to_string());
        }
        if allowed.is_empty() {
            default_disallowed_methods_allowlist()
        } else {
            allowed
        }
    }

    struct PragmaLintPolicy {
        allow_dead_code: HashSet<String>,
        allow_lints: HashSet<String>,
    }

    fn load_pragma_lint_policy(root: &Path) -> PragmaLintPolicy {
        let path = root.join("tools/pragma-lint-policy.txt");
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("missing pragma policy file: {}", path.display()));

        let mut section = "";
        let mut allow_dead_code = HashSet::new();
        let mut allow_lints = HashSet::new();

        for raw in content.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                section = &line[1..line.len() - 1];
                continue;
            }
            match section {
                "allow.dead_code" => {
                    allow_dead_code.insert(line.to_string());
                }
                "allow.lints" => {
                    allow_lints.insert(line.to_string());
                }
                _ => {}
            }
        }

        PragmaLintPolicy {
            allow_dead_code,
            allow_lints,
        }
    }

    fn is_allowed_disallowed_methods(path: &str, allowed_prefixes: &HashSet<String>) -> bool {
        if path.contains("/tests/") {
            return true;
        }
        allowed_prefixes
            .iter()
            .any(|prefix| path.starts_with(prefix))
    }

    fn extract_allow_lints(line: &str) -> Vec<String> {
        let mut lints = Vec::new();
        let mut rest = line;
        while let Some(start) = rest.find("allow(") {
            let after = &rest[start + "allow(".len()..];
            if let Some(end) = after.find(')') {
                let inside = &after[..end];
                for item in inside.split(',') {
                    let lint = item.trim();
                    if !lint.is_empty() {
                        lints.push(lint.to_string());
                    }
                }
                rest = &after[end + 1..];
            } else {
                break;
            }
        }
        lints
    }

    #[test]
    fn lint_allow_pragmas_and_migrations() {
        let root = repo_root();
        let mut files = Vec::new();
        for dir in ["core", "lib", "gunbc-dag"] {
            collect_rs_files(&root.join(dir), &mut files);
        }

        let allowed_disallowed_methods = load_disallowed_methods_allowlist(&root);
        let policy = load_pragma_lint_policy(&root);

        let mut dead_code_allows = Vec::new();
        let mut disallowed_method_allows = Vec::new();
        let mut disallowed_type_allows = Vec::new();
        let mut migration_tags = Vec::new();
        let mut forbidden_allows = Vec::new();

        for file in files {
            let rel = file
                .strip_prefix(&root)
                .unwrap_or(&file)
                .to_string_lossy()
                .replace('\\', "/");
            let Ok(content) = fs::read_to_string(&file) else {
                continue;
            };

            for (idx, line) in content.lines().enumerate() {
                let line_no = idx + 1;
                let trimmed = line.trim_start();
                if trimmed.starts_with("#") && trimmed.contains("allow(") {
                    for lint in extract_allow_lints(trimmed) {
                        match lint.as_str() {
                            "dead_code" => {
                                if !policy.allow_dead_code.contains(rel.as_str()) {
                                    dead_code_allows.push(format!("{}:{} {}", rel, line_no, lint));
                                }
                            }
                            "clippy::disallowed_methods" => {
                                if !is_allowed_disallowed_methods(&rel, &allowed_disallowed_methods)
                                {
                                    disallowed_method_allows
                                        .push(format!("{}:{} {}", rel, line_no, lint));
                                }
                            }
                            "clippy::disallowed_types" => {
                                if !is_allowed_disallowed_methods(&rel, &allowed_disallowed_methods)
                                {
                                    disallowed_type_allows
                                        .push(format!("{}:{} {}", rel, line_no, lint));
                                }
                            }
                            _ => {
                                if !policy.allow_lints.contains(lint.as_str()) {
                                    forbidden_allows.push(format!("{}:{} {}", rel, line_no, lint));
                                }
                            }
                        }
                    }
                }
                if (trimmed.starts_with("//")
                    || trimmed.starts_with("///")
                    || trimmed.starts_with("/*")
                    || trimmed.starts_with('*'))
                    && trimmed.contains("MIGRATION:")
                {
                    migration_tags.push(format!("{}:{}", rel, line_no));
                }
            }
        }

        assert!(
            dead_code_allows.is_empty(),
            "Found #[allow(dead_code)] outside allowlist:\n{}",
            dead_code_allows.join("\n")
        );

        assert!(
            disallowed_method_allows.is_empty(),
            "Found #[allow(clippy::disallowed_methods)] outside allowlist:\n{}",
            disallowed_method_allows.join("\n")
        );
        assert!(
            disallowed_type_allows.is_empty(),
            "Found #[allow(clippy::disallowed_types)] outside allowlist:\n{}",
            disallowed_type_allows.join("\n")
        );

        assert!(
            forbidden_allows.is_empty(),
            "Found forbidden #[allow(...)] lints:\n{}",
            forbidden_allows.join("\n")
        );

        assert!(
            migration_tags.is_empty(),
            "Found MIGRATION tags (unfinished work):\n{}",
            migration_tags.join("\n")
        );
    }

    /// M7 audit: expose_plaintext_for_transport() callsites are in approved modules only.
    ///
    /// This test greps the source tree and asserts that non-test, non-doc callsites
    /// appear only in modules that are explicitly approved transport boundaries.
    #[test]
    fn expose_plaintext_callsites_are_in_approved_modules() {
        let approved = &[
            "core/ir/src/transport/credential.rs",
            "core/ir/src/resource/mod.rs",
            "core/ir/src/resource/handle.rs",
            "core/ir/src/value.rs",
            "core/exec/src/execute/mod.rs",
            "core/exec/src/display.rs",
            "gunbc-dag/src/resolve_service.rs",
            "lib/gcp-ops/src/ops.rs",
            "lib/tools/clippy/src/config.rs",
            "lib/transport/src/ops.rs",
        ];

        let workspace_root = repo_root();
        let mut files = Vec::new();
        for dir in ["core", "lib", "gunbc-dag"] {
            collect_rs_files(&workspace_root.join(dir), &mut files);
        }

        let mut unapproved = Vec::new();
        for file in files {
            let rel = file
                .strip_prefix(&workspace_root)
                .unwrap_or(&file)
                .to_string_lossy()
                .replace('\\', "/");

            let Ok(content) = fs::read_to_string(&file) else {
                continue;
            };

            let mut in_test_mod = false;
            for line in content.lines() {
                let trimmed = line.trim();

                if trimmed == "#[cfg(test)]" {
                    in_test_mod = true;
                    continue;
                }

                if trimmed.starts_with("//") {
                    continue;
                }

                if in_test_mod {
                    continue;
                }

                if !trimmed.contains(".expose_plaintext_for_transport()") {
                    continue;
                }

                if !approved.iter().any(|a| rel == *a) {
                    unapproved.push(rel.clone());
                    break;
                }
            }
        }

        assert!(
            unapproved.is_empty(),
            "expose_plaintext_for_transport() called in unapproved modules: {unapproved:?}\n\
             If these are legitimate transport boundaries, add them to the approved list."
        );
    }
}
