use std::collections::HashMap;
use std::path::Path;

use gunbc_exec::{ExecError, Executable, Value};
use sha2::{Digest, Sha256};

use crate::types::{CrateInfo, MakegenConfig, Rule, Target, UpsertStatus};

/// The operation type for makegen nodes.
#[derive(Debug, Clone)]
pub enum MakegenOp {
    /// Initialize context from config
    Context { config: MakegenConfig },
    /// Check existing Makefile state (Observe)
    Check,
    /// Resolve upsert state from check + generated content (Pure)
    Resolve,
    /// Parse workspace Cargo.toml (Pure)
    ParseWorkspace,
    /// Generate target definitions (Pure)
    GenerateTargets,
    /// Generate rules from targets (Pure)
    GenerateRules,
    /// Compose final Makefile content (Pure)
    ComposeMakefile,
    /// Write content to file (WritesWorld)
    WriteFile,
    /// Print content to stdout (Observe - for dry-run)
    PrintStdout,
}

impl Executable for MakegenOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            MakegenOp::Context { config } => {
                let abs_path = std::fs::canonicalize(&config.workspace_path)
                    .unwrap_or_else(|_| std::path::PathBuf::from(&config.workspace_path));
                let mut out = HashMap::new();
                out.insert("workspace_path".into(), Value::Str(abs_path.to_string_lossy().into_owned()));
                out.insert("per_crate_targets".into(), Value::Bool(config.per_crate_targets));
                out.insert("lint_targets".into(), Value::Bool(config.lint_targets));
                out.insert("output_path".into(), Value::Str(config.output_path.clone()));
                out.insert("force".into(), Value::Bool(config.force));
                Ok(out)
            }

            MakegenOp::Check => {
                let workspace_path = inputs.get("workspace_path")
                    .and_then(|v| if let Value::Str(s) = v { Some(s.clone()) } else { None })
                    .unwrap_or_else(|| ".".into());
                let output_path = inputs.get("output_path")
                    .and_then(|v| if let Value::Str(s) = v { Some(s.clone()) } else { None })
                    .unwrap_or_else(|| "Makefile".into());
                let force = inputs.get("force")
                    .and_then(|v| if let Value::Bool(b) = v { Some(*b) } else { None })
                    .unwrap_or(false);
                let per_crate = inputs.get("per_crate_targets")
                    .and_then(|v| if let Value::Bool(b) = v { Some(*b) } else { None })
                    .unwrap_or(true);
                let lint = inputs.get("lint_targets")
                    .and_then(|v| if let Value::Bool(b) = v { Some(*b) } else { None })
                    .unwrap_or(true);

                // Compute input hash from Cargo.toml mtime + config
                let cargo_toml = Path::new(&workspace_path).join("Cargo.toml");
                let mtime = std::fs::metadata(&cargo_toml)
                    .and_then(|m| m.modified())
                    .map(|t| format!("{:?}", t))
                    .unwrap_or_else(|_| "unknown".into());

                let config_str = format!("per_crate={},lint={}", per_crate, lint);
                let input_hash = compute_hash(&format!("{}{}", mtime, config_str));

                // Read existing Makefile to check hash
                let makefile_path = Path::new(&workspace_path).join(&output_path);
                let existing_hash = if makefile_path.exists() {
                    std::fs::read_to_string(&makefile_path)
                        .ok()
                        .and_then(|content| extract_hash(&content))
                } else {
                    None
                };

                let mut out = HashMap::new();
                out.insert("input_hash".into(), Value::Str(input_hash.clone()));
                out.insert("makefile_path".into(), Value::Str(makefile_path.to_string_lossy().into_owned()));

                let needs_generate = force || existing_hash.as_ref() != Some(&input_hash);
                out.insert("needs_generate".into(), Value::Bool(needs_generate));
                out.insert("file_exists".into(), Value::Bool(makefile_path.exists()));

                // Pass through values for generation pipeline
                out.insert("workspace_path".into(), Value::Str(workspace_path));
                out.insert("per_crate_targets".into(), Value::Bool(per_crate));
                out.insert("lint_targets".into(), Value::Bool(lint));

                Ok(out)
            }

            MakegenOp::Resolve => {
                let content = inputs.get("content")
                    .and_then(|v| if let Value::Str(s) = v { Some(s.clone()) } else { None });
                let input_hash = inputs.get("input_hash")
                    .and_then(|v| if let Value::Str(s) = v { Some(s.clone()) } else { None })
                    .unwrap_or_default();
                let makefile_path = inputs.get("makefile_path")
                    .and_then(|v| if let Value::Str(s) = v { Some(s.clone()) } else { None })
                    .unwrap_or_else(|| "Makefile".into());
                let needs_generate = inputs.get("needs_generate")
                    .and_then(|v| if let Value::Bool(b) = v { Some(*b) } else { None })
                    .unwrap_or(true);
                let file_exists = inputs.get("file_exists")
                    .and_then(|v| if let Value::Bool(b) = v { Some(*b) } else { None })
                    .unwrap_or(false);

                let mut out = HashMap::new();

                match (content, needs_generate) {
                    (Some(c), true) => {
                        out.insert("content".into(), Value::Str(c));
                        out.insert("hash".into(), Value::Str(input_hash));
                        out.insert("needs_write".into(), Value::Bool(true));
                        out.insert("makefile_path".into(), Value::Str(makefile_path));
                        out.insert("file_existed".into(), Value::Bool(file_exists));
                    }
                    (_, false) => {
                        out.insert("content".into(), Value::Skipped);
                        out.insert("hash".into(), Value::Str(input_hash));
                        out.insert("needs_write".into(), Value::Bool(false));
                        out.insert("makefile_path".into(), Value::Str(makefile_path));
                        out.insert("file_existed".into(), Value::Bool(file_exists));
                    }
                    (None, true) => {
                        // Content was skipped upstream - propagate
                        out.insert("content".into(), Value::Skipped);
                        out.insert("hash".into(), Value::Str(input_hash));
                        out.insert("needs_write".into(), Value::Bool(false));
                        out.insert("makefile_path".into(), Value::Str(makefile_path));
                        out.insert("file_existed".into(), Value::Bool(file_exists));
                    }
                }

                Ok(out)
            }

            MakegenOp::ParseWorkspace => {
                let workspace_path = inputs.get("workspace_path")
                    .and_then(|v| if let Value::Str(s) = v { Some(s.clone()) } else { None })
                    .unwrap_or_else(|| ".".into());

                let cargo_toml = Path::new(&workspace_path).join("Cargo.toml");
                let content = std::fs::read_to_string(&cargo_toml)
                    .map_err(|e| ExecError(format!("Failed to read Cargo.toml: {}", e)))?;

                let parsed: toml::Value = content.parse()
                    .map_err(|e| ExecError(format!("Failed to parse Cargo.toml: {}", e)))?;

                let mut crates = Vec::new();

                // Check if it's a workspace
                if let Some(workspace) = parsed.get("workspace") {
                    if let Some(members) = workspace.get("members").and_then(|m| m.as_array()) {
                        for member in members {
                            if let Some(member_path) = member.as_str() {
                                if let Some(crate_info) = parse_member_crate(&workspace_path, member_path) {
                                    crates.push(crate_info);
                                }
                            }
                        }
                    }
                } else if let Some(package) = parsed.get("package") {
                    // Single crate project
                    if let Some(name) = package.get("name").and_then(|n| n.as_str()) {
                        let has_lib = Path::new(&workspace_path).join("src/lib.rs").exists();
                        let has_main = Path::new(&workspace_path).join("src/main.rs").exists();
                        crates.push(CrateInfo {
                            name: name.to_string(),
                            path: workspace_path.clone(),
                            is_binary: has_main,
                            is_library: has_lib,
                        });
                    }
                }

                let crate_names: Vec<String> = crates.iter().map(|c| c.name.clone()).collect();
                let crate_paths: Vec<String> = crates.iter().map(|c| c.path.clone()).collect();
                let crate_is_bin: Vec<String> = crates.iter().map(|c| c.is_binary.to_string()).collect();
                let crate_is_lib: Vec<String> = crates.iter().map(|c| c.is_library.to_string()).collect();

                let mut out = HashMap::new();
                out.insert("crate_names".into(), Value::StrList(crate_names));
                out.insert("crate_paths".into(), Value::StrList(crate_paths));
                out.insert("crate_is_bin".into(), Value::StrList(crate_is_bin));
                out.insert("crate_is_lib".into(), Value::StrList(crate_is_lib));
                Ok(out)
            }

            MakegenOp::GenerateTargets => {
                let crate_names = inputs.get("crate_names")
                    .and_then(|v| if let Value::StrList(s) = v { Some(s.clone()) } else { None })
                    .unwrap_or_default();
                let per_crate = inputs.get("per_crate_targets")
                    .and_then(|v| if let Value::Bool(b) = v { Some(*b) } else { None })
                    .unwrap_or(true);
                let lint = inputs.get("lint_targets")
                    .and_then(|v| if let Value::Bool(b) = v { Some(*b) } else { None })
                    .unwrap_or(true);

                let mut targets = Vec::new();

                // Core targets
                targets.push(Target {
                    name: "all".into(),
                    dependencies: vec!["build".into(), "test".into()],
                    phony: true,
                });
                targets.push(Target {
                    name: "build".into(),
                    dependencies: vec![],
                    phony: true,
                });
                targets.push(Target {
                    name: "test".into(),
                    dependencies: vec!["build".into()],
                    phony: true,
                });
                targets.push(Target {
                    name: "clean".into(),
                    dependencies: vec![],
                    phony: true,
                });
                targets.push(Target {
                    name: "doc".into(),
                    dependencies: vec![],
                    phony: true,
                });

                if lint {
                    targets.push(Target {
                        name: "lint".into(),
                        dependencies: vec![],
                        phony: true,
                    });
                    targets.push(Target {
                        name: "fmt".into(),
                        dependencies: vec![],
                        phony: true,
                    });
                }

                if per_crate {
                    for name in &crate_names {
                        targets.push(Target {
                            name: format!("build-{}", name),
                            dependencies: vec![],
                            phony: true,
                        });
                        targets.push(Target {
                            name: format!("test-{}", name),
                            dependencies: vec![format!("build-{}", name)],
                            phony: true,
                        });
                    }
                }

                // Serialize targets as JSON-ish strings for transport
                let target_strs: Vec<String> = targets.iter()
                    .map(|t| format!("{}|{}|{}", t.name, t.dependencies.join(","), t.phony))
                    .collect();

                let mut out = HashMap::new();
                out.insert("targets".into(), Value::StrList(target_strs));
                Ok(out)
            }

            MakegenOp::GenerateRules => {
                let target_strs = inputs.get("targets")
                    .and_then(|v| if let Value::StrList(s) = v { Some(s.clone()) } else { None })
                    .unwrap_or_default();
                // crate_names available for future per-crate command customization
                let _crate_names = inputs.get("crate_names")
                    .and_then(|v| if let Value::StrList(s) = v { Some(s.clone()) } else { None })
                    .unwrap_or_default();

                // Parse targets back
                let targets: Vec<Target> = target_strs.iter()
                    .filter_map(|s| {
                        let parts: Vec<&str> = s.split('|').collect();
                        if parts.len() == 3 {
                            Some(Target {
                                name: parts[0].into(),
                                dependencies: if parts[1].is_empty() {
                                    vec![]
                                } else {
                                    parts[1].split(',').map(String::from).collect()
                                },
                                phony: parts[2] == "true",
                            })
                        } else {
                            None
                        }
                    })
                    .collect();

                let mut rules = Vec::new();

                for target in &targets {
                    let commands = match target.name.as_str() {
                        "all" => vec![],
                        "build" => vec!["cargo build --workspace".into()],
                        "test" => vec!["cargo test --workspace".into()],
                        "clean" => vec!["cargo clean".into()],
                        "doc" => vec!["cargo doc --workspace --no-deps".into()],
                        "lint" => vec!["cargo clippy --workspace -- -D warnings".into()],
                        "fmt" => vec!["cargo fmt --all".into()],
                        name if name.starts_with("build-") => {
                            let crate_name = &name[6..];
                            vec![format!("cargo build -p {}", crate_name)]
                        }
                        name if name.starts_with("test-") => {
                            let crate_name = &name[5..];
                            vec![format!("cargo test -p {}", crate_name)]
                        }
                        _ => vec![],
                    };

                    rules.push(Rule {
                        target: target.clone(),
                        commands,
                    });
                }

                // Serialize rules
                let rule_strs: Vec<String> = rules.iter()
                    .map(|r| {
                        let cmds = r.commands.join(";;");
                        format!("{}|{}|{}|{}", r.target.name, r.target.dependencies.join(","), r.target.phony, cmds)
                    })
                    .collect();

                let mut out = HashMap::new();
                out.insert("rules".into(), Value::StrList(rule_strs));
                Ok(out)
            }

            MakegenOp::ComposeMakefile => {
                let rule_strs = inputs.get("rules")
                    .and_then(|v| if let Value::StrList(s) = v { Some(s.clone()) } else { None })
                    .unwrap_or_default();
                let hash = inputs.get("input_hash")
                    .and_then(|v| if let Value::Str(s) = v { Some(s.clone()) } else { None })
                    .unwrap_or_else(|| "unknown".into());

                // Parse rules back
                let rules: Vec<Rule> = rule_strs.iter()
                    .filter_map(|s| {
                        let parts: Vec<&str> = s.splitn(4, '|').collect();
                        if parts.len() == 4 {
                            Some(Rule {
                                target: Target {
                                    name: parts[0].into(),
                                    dependencies: if parts[1].is_empty() {
                                        vec![]
                                    } else {
                                        parts[1].split(',').map(String::from).collect()
                                    },
                                    phony: parts[2] == "true",
                                },
                                commands: if parts[3].is_empty() {
                                    vec![]
                                } else {
                                    parts[3].split(";;").map(String::from).collect()
                                },
                            })
                        } else {
                            None
                        }
                    })
                    .collect();

                let mut content = String::new();
                content.push_str("# Generated by gunbc-makegen\n");
                content.push_str(&format!("# Hash: {}\n\n", hash));

                // Collect phony targets
                let phony_targets: Vec<&str> = rules.iter()
                    .filter(|r| r.target.phony)
                    .map(|r| r.target.name.as_str())
                    .collect();

                if !phony_targets.is_empty() {
                    content.push_str(&format!(".PHONY: {}\n\n", phony_targets.join(" ")));
                }

                // Write rules
                for rule in &rules {
                    let deps = if rule.target.dependencies.is_empty() {
                        String::new()
                    } else {
                        format!(" {}", rule.target.dependencies.join(" "))
                    };
                    content.push_str(&format!("{}:{}\n", rule.target.name, deps));
                    for cmd in &rule.commands {
                        content.push_str(&format!("\t{}\n", cmd));
                    }
                    content.push('\n');
                }

                let mut out = HashMap::new();
                out.insert("content".into(), Value::Str(content));
                Ok(out)
            }

            MakegenOp::WriteFile => {
                let content = inputs.get("content")
                    .and_then(|v| if let Value::Str(s) = v { Some(s.clone()) } else { None });
                let needs_write = inputs.get("needs_write")
                    .and_then(|v| if let Value::Bool(b) = v { Some(*b) } else { None })
                    .unwrap_or(false);
                let makefile_path = inputs.get("makefile_path")
                    .and_then(|v| if let Value::Str(s) = v { Some(s.clone()) } else { None })
                    .unwrap_or_else(|| "Makefile".into());
                let file_existed = inputs.get("file_existed")
                    .and_then(|v| if let Value::Bool(b) = v { Some(*b) } else { None })
                    .unwrap_or(false);

                let mut out = HashMap::new();

                if !needs_write {
                    out.insert("status".into(), Value::Str(UpsertStatus::Unchanged.to_string()));
                    eprintln!("Makefile is up-to-date");
                    return Ok(out);
                }

                match content {
                    Some(c) => {
                        std::fs::write(&makefile_path, &c)
                            .map_err(|e| ExecError(format!("Failed to write Makefile: {}", e)))?;
                        let status = if file_existed {
                            UpsertStatus::Updated
                        } else {
                            UpsertStatus::Created
                        };
                        eprintln!("{}: {}", status, makefile_path);
                        out.insert("status".into(), Value::Str(status.to_string()));
                    }
                    None => {
                        out.insert("status".into(), Value::Str(UpsertStatus::Unchanged.to_string()));
                    }
                }

                Ok(out)
            }

            MakegenOp::PrintStdout => {
                let content = inputs.get("content")
                    .and_then(|v| if let Value::Str(s) = v { Some(s.clone()) } else { None });
                let needs_write = inputs.get("needs_write")
                    .and_then(|v| if let Value::Bool(b) = v { Some(*b) } else { None })
                    .unwrap_or(false);

                let mut out = HashMap::new();

                if !needs_write {
                    eprintln!("[DRY RUN] Makefile is up-to-date");
                    out.insert("status".into(), Value::Str(UpsertStatus::Unchanged.to_string()));
                    return Ok(out);
                }

                match content {
                    Some(c) => {
                        eprintln!("[DRY RUN] Would write Makefile:\n");
                        println!("{}", c);
                        out.insert("status".into(), Value::Str(UpsertStatus::DryRun.to_string()));
                    }
                    None => {
                        eprintln!("[DRY RUN] No content to write");
                        out.insert("status".into(), Value::Str(UpsertStatus::Unchanged.to_string()));
                    }
                }

                Ok(out)
            }
        }
    }
}

fn compute_hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)[..16].to_string()
}

fn extract_hash(content: &str) -> Option<String> {
    for line in content.lines() {
        if line.starts_with("# Hash: ") {
            return Some(line[8..].trim().to_string());
        }
    }
    None
}

fn parse_member_crate(workspace_path: &str, member_path: &str) -> Option<CrateInfo> {
    let full_path = Path::new(workspace_path).join(member_path);
    let cargo_toml = full_path.join("Cargo.toml");

    let content = std::fs::read_to_string(&cargo_toml).ok()?;
    let parsed: toml::Value = content.parse().ok()?;
    let name = parsed.get("package")?.get("name")?.as_str()?;

    let has_lib = full_path.join("src/lib.rs").exists();
    let has_main = full_path.join("src/main.rs").exists();
    let has_bin_dir = full_path.join("src/bin").exists();

    Some(CrateInfo {
        name: name.to_string(),
        path: member_path.to_string(),
        is_binary: has_main || has_bin_dir,
        is_library: has_lib,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_sets_paths() {
        let config = MakegenConfig {
            workspace_path: ".".into(),
            per_crate_targets: true,
            lint_targets: true,
            output_path: "Makefile".into(),
            force: false,
        };
        let op = MakegenOp::Context { config };
        let out = op.execute(HashMap::new()).unwrap();
        assert!(matches!(out.get("workspace_path"), Some(Value::Str(_))));
        assert!(matches!(out.get("per_crate_targets"), Some(Value::Bool(true))));
    }

    #[test]
    fn compose_makefile_includes_hash() {
        let mut inputs = HashMap::new();
        inputs.insert("rules".into(), Value::StrList(vec![
            "all|build,test|true|".into(),
            "build||true|cargo build --workspace".into(),
        ]));
        inputs.insert("input_hash".into(), Value::Str("abc123".into()));

        let op = MakegenOp::ComposeMakefile;
        let out = op.execute(inputs).unwrap();
        if let Value::Str(content) = &out["content"] {
            assert!(content.contains("# Hash: abc123"));
            assert!(content.contains(".PHONY: all build"));
            assert!(content.contains("cargo build --workspace"));
        } else {
            panic!("expected Str");
        }
    }

    #[test]
    fn extract_hash_finds_hash() {
        let content = "# Generated by gunbc-makegen\n# Hash: abc123\n\n.PHONY: all";
        assert_eq!(extract_hash(content), Some("abc123".into()));
    }

    #[test]
    fn extract_hash_returns_none_for_missing() {
        let content = "# No hash here\n.PHONY: all";
        assert_eq!(extract_hash(content), None);
    }

    #[test]
    fn compute_hash_is_deterministic() {
        let h1 = compute_hash("test input");
        let h2 = compute_hash("test input");
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 16);
    }

    #[test]
    fn generate_targets_respects_flags() {
        let mut inputs = HashMap::new();
        inputs.insert("crate_names".into(), Value::StrList(vec!["foo".into()]));
        inputs.insert("per_crate_targets".into(), Value::Bool(false));
        inputs.insert("lint_targets".into(), Value::Bool(false));

        let op = MakegenOp::GenerateTargets;
        let out = op.execute(inputs).unwrap();
        if let Value::StrList(targets) = &out["targets"] {
            assert!(!targets.iter().any(|t| t.starts_with("build-foo")));
            assert!(!targets.iter().any(|t| t.starts_with("lint")));
        } else {
            panic!("expected StrList");
        }
    }

    #[test]
    fn dry_run_prints_content() {
        let mut inputs = HashMap::new();
        inputs.insert("content".into(), Value::Str("test content".into()));
        inputs.insert("needs_write".into(), Value::Bool(true));

        let op = MakegenOp::PrintStdout;
        let out = op.execute(inputs).unwrap();
        assert!(matches!(out.get("status"), Some(Value::Str(s)) if s == "DryRun"));
    }

    #[test]
    fn resolve_propagates_unchanged() {
        let mut inputs = HashMap::new();
        inputs.insert("content".into(), Value::Str("test".into()));
        inputs.insert("input_hash".into(), Value::Str("abc".into()));
        inputs.insert("makefile_path".into(), Value::Str("Makefile".into()));
        inputs.insert("needs_generate".into(), Value::Bool(false));
        inputs.insert("file_exists".into(), Value::Bool(true));

        let op = MakegenOp::Resolve;
        let out = op.execute(inputs).unwrap();
        assert!(matches!(out.get("needs_write"), Some(Value::Bool(false))));
    }
}
