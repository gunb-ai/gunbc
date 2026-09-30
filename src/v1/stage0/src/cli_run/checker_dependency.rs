//! THE CHECKER'S OWN INPUTS, observed from the build tool's record of what the checker compiled.
//!
//! `v2.workflow.floor_subject_seed` `checker_subject_rule` decides what a changed checker input
//! does to the required floor's prepared subject. This file only OBSERVES the two facts that
//! decision reads -- the checker's dependency paths and the run's changed paths -- and hands them
//! to that function in a frame. It lists no checker path by hand: a file enters the set by being
//! compiled into the binary that runs the floor, so a new infer helper, stage0 mirror or runtime
//! file cannot be missing from it (DESIGN section 3, one authority: the build tool's record).
//!
//! THE RECORD IS CARGO'S DEP-INFO BESIDE THE RUNNING BINARY (`target/<profile>/<bin>.d`), which
//! lists every source file rustc read for that binary, including `include_str!` inputs. Build
//! manifests decide which of those files and crates are compiled, so the manifest of every
//! workspace package the record reaches is an input too, as are the workspace's lockfile and
//! toolchain pin. Paths outside the workspace (registry crates) are governed by the lockfile and
//! are not repository paths a diff can name.

use super::*;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const CHECKER_SUBJECT_AUTHORITY: &str = "src/v2/workflow/floor_subject_seed.dag";

/// `v2.workflow.floor_subject_seed` `CheckerSubjectDecision`, decoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CheckerSubjectDecision {
    pub rule: String,
    pub changed_checker_paths: Vec<String>,
    pub reason: String,
}

impl CheckerSubjectDecision {
    pub(crate) fn selects_every_admitted_module(&self) -> bool {
        self.rule == "EveryAdmittedModule"
    }
    pub(crate) fn refused(&self) -> bool {
        self.rule == "CheckerSubjectRefused"
    }
}

/// The prerequisites of every rule in a Makefile-syntax dep-info file, with `\ ` unescaped.
pub(crate) fn parse_dep_info_prerequisites(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    // Continuation lines are joined first, so one rule is one logical line.
    let joined = text.replace("\\\n", " ");
    for line in joined.lines() {
        let Some(colon) = find_rule_colon(line) else {
            continue;
        };
        let mut current = String::new();
        let mut chars = line[colon + 1..].chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                '\\' if chars.peek() == Some(&' ') => {
                    current.push(' ');
                    chars.next();
                }
                ' ' | '\t' => {
                    if !current.is_empty() {
                        out.push(std::mem::take(&mut current));
                    }
                }
                _ => current.push(c),
            }
        }
        if !current.is_empty() {
            out.push(current);
        }
    }
    out
}

/// The first `:` that is followed by whitespace or the end of line, so a Windows drive letter or
/// an escaped path is not read as the rule separator.
fn find_rule_colon(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    (0..bytes.len()).find(|&i| {
        bytes[i] == b':' && (i + 1 == bytes.len() || bytes[i + 1] == b' ' || bytes[i + 1] == b'\t')
    })
}

/// Workspace-relative checker inputs: every dep-info prerequisite inside `workspace_root`, the
/// nearest `Cargo.toml` above each, and the workspace's `Cargo.toml`, `Cargo.lock` and
/// `rust-toolchain.toml` where they exist.
pub(crate) fn checker_input_paths(prerequisites: &[String], workspace_root: &Path) -> Vec<String> {
    let mut inputs: BTreeSet<String> = BTreeSet::new();
    let mut manifests: BTreeSet<PathBuf> = BTreeSet::new();
    for prerequisite in prerequisites {
        // rustc's own `deps/*.d` records spell workspace files relative to the workspace root,
        // where cargo runs it; cargo's top-level `<bin>.d` spells them absolute. Both readings
        // name the same file.
        let joined = workspace_root.join(prerequisite);
        let path = joined.as_path();
        let Ok(relative) = path.strip_prefix(workspace_root) else {
            continue;
        };
        inputs.insert(normalize_repo_path(&relative.to_string_lossy()));
        let mut dir = path.parent();
        while let Some(d) = dir {
            if !d.starts_with(workspace_root) {
                break;
            }
            let manifest = d.join("Cargo.toml");
            if manifest.is_file() {
                manifests.insert(manifest);
                break;
            }
            dir = d.parent();
        }
    }
    for name in ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml"] {
        let p = workspace_root.join(name);
        if p.is_file() {
            manifests.insert(p);
        }
    }
    for manifest in manifests {
        if let Ok(relative) = manifest.strip_prefix(workspace_root) {
            inputs.insert(normalize_repo_path(&relative.to_string_lossy()));
        }
    }
    inputs.into_iter().collect()
}

/// Observe the checker's inputs from the dep-info beside the running executable. A missing,
/// unreadable or foreign record is a refusal the `.dag` rule consumes, never an empty set.
pub(crate) fn observe_checker_input_paths(workspace_root: &Path) -> Result<Vec<String>, String> {
    let exe = std::env::current_exe()
        .map_err(|e| format!("the running checker's own path is unreadable: {e}"))?;
    let mut dep_info = exe.clone().into_os_string();
    dep_info.push(".d");
    let dep_info = PathBuf::from(dep_info);
    let text = std::fs::read_to_string(&dep_info).map_err(|e| {
        format!(
            "the checker's dependency record {} is unreadable ({e}); the floor must run a binary \
             cargo built in this workspace",
            dep_info.display()
        )
    })?;
    let canonical_root = workspace_root
        .canonicalize()
        .unwrap_or_else(|_| workspace_root.to_path_buf());
    checker_inputs_from_record(&text, &dep_info, &canonical_root)
}

/// The checker's inputs from one record's text, or the reason the record cannot stand for them.
///
/// TWO RECORDS SHARE THE `.d` FORMAT AND ONLY ONE IS THE CHECKER'S. Cargo writes `<bin>.d` beside
/// a binary it builds, listing every file of every workspace crate the binary links, and the
/// package manifests with them. rustc writes `deps/<crate>-<hash>.d` for ONE crate and omits its
/// dependency crates. The seed's infer lives in its own crate (`v1-stage0-v1-infer`), so a
/// per-crate record is missing the very files gunbc#12441 edited. The two are told apart
/// structurally: only cargo's record lists a `Cargo.toml`. A record listing none is refused.
pub(crate) fn checker_inputs_from_record(
    text: &str,
    record: &Path,
    canonical_root: &Path,
) -> Result<Vec<String>, String> {
    let prerequisites = parse_dep_info_prerequisites(text);
    if !prerequisites.iter().any(|p| p.ends_with("Cargo.toml")) {
        return Err(format!(
            "the dependency record {} lists no Cargo.toml, so it is rustc's per-crate record and \
             omits the checker's dependency crates; the floor must run a binary whose cargo \
             record lists every crate it links",
            record.display()
        ));
    }
    let inputs = checker_input_paths(&prerequisites, canonical_root);
    // THE ANCHOR: this file is compiled into the checker, and `file!()` is rustc's own spelling
    // of it. A record that omits it came from another build, or was relativized against the
    // wrong root, so it is refused instead of read as "no checker input changed".
    let anchor = normalize_repo_path(file!());
    if !inputs.contains(&anchor) {
        return Err(format!(
            "the checker's dependency record {} does not list {anchor}, which this binary was \
             compiled from; it describes another build or another checkout ({} inputs under {})",
            record.display(),
            inputs.len(),
            canonical_root.display()
        ));
    }
    Ok(inputs)
}

/// Evaluate `checker_subject_decision` over the observation. The rule lives in the `.dag`; this
/// function decodes its three-field wire form and decides nothing.
pub(crate) fn checker_subject_decision(
    source_roots: &[String],
    observation: &Result<Vec<String>, String>,
    changed_paths: &[String],
) -> Result<CheckerSubjectDecision, String> {
    use v1_interpreter::Value;
    // Anchored at the workspace root rather than the process cwd, so the decision is the same
    // from the floor (run at the root) and from a test (run in the crate directory).
    let root = process_workspace_root();
    let anchored = |p: &str| {
        let path = Path::new(p);
        if path.is_absolute() {
            p.to_string()
        } else {
            root.join(path).to_string_lossy().into_owned()
        }
    };
    let roots: Vec<String> = source_roots.iter().map(|r| anchored(r)).collect();
    let (graph, indices) = resolve_entry_graph_shared(&roots, &anchored(CHECKER_SUBJECT_AUTHORITY))
        .map_err(|e| format!("checker subject authority resolve: {e}"))?;
    let ctx = make_eval_context(&graph, indices, v1_interpreter::ExecutionMode::Hermetic);
    let list = |items: &[String]| {
        v1_interpreter::list_value(
            items
                .iter()
                .map(v1_interpreter::str_value)
                .collect::<Vec<_>>(),
        )
    };
    let (deps, refusal) = match observation {
        Ok(paths) => (list(paths), v1_interpreter::str_value("")),
        Err(reason) => (list(&[]), v1_interpreter::str_value(reason)),
    };
    let args = [
        (Some("dependency_paths".to_string()), deps),
        (Some("observation_refusal".to_string()), refusal),
        (Some("changed_paths".to_string()), list(changed_paths)),
    ];
    let result =
        v1_interpreter::run_in_context_with_args(&ctx, "checker_subject_decision", &args, false)
            .map_err(|e| format!("checker_subject_decision: {e}"))?;
    let fields = match &result {
        Value::Record { fields, .. } | Value::Variant { fields, .. } => fields,
        other => {
            return Err(format!(
                "checker_subject_decision returned `{}`, expected CheckerSubjectDecision",
                ctx.format_value(other)
            ))
        }
    };
    let text = |name: &str| match ctx.field(fields, name) {
        Some(Value::Str(s)) => Ok(s.to_string()),
        _ => Err(format!("CheckerSubjectDecision missing String `{name}`")),
    };
    let changed_checker_paths = match ctx.field(fields, "changed_checker_paths") {
        Some(v) => decoded_string_list(&ctx, v)?,
        None => return Err("CheckerSubjectDecision missing `changed_checker_paths`".to_string()),
    };
    Ok(CheckerSubjectDecision {
        rule: text("rule")?,
        changed_checker_paths,
        reason: text("reason")?,
    })
}

/// A `List<String>` as the frame returns it: a host list, or the structural `FreeMonoid` chain a
/// `.dag` fold such as `filter` builds.
fn decoded_string_list(
    ctx: &v1_interpreter::InterpContext,
    value: &v1_interpreter::Value,
) -> Result<Vec<String>, String> {
    use v1_interpreter::Value;
    let mut out = Vec::new();
    let mut cur = value;
    loop {
        match cur {
            Value::List(_) => {
                out.extend(string_list_from_value(cur, "changed_checker_paths")?);
                return Ok(out);
            }
            Value::Variant {
                variant_name,
                fields,
                ..
            } if ctx.sym_eq(*variant_name, "Empty") => {
                let _ = fields;
                return Ok(out);
            }
            Value::Variant {
                variant_name,
                fields,
                ..
            } if ctx.sym_eq(*variant_name, "Cons") => {
                match (ctx.field(fields, "head"), ctx.field(fields, "tail")) {
                    (Some(Value::Str(s)), Some(tail)) => {
                        out.push(s.to_string());
                        cur = tail;
                    }
                    _ => {
                        return Err(
                            "changed_checker_paths: a Cons without a String head".to_string()
                        )
                    }
                }
            }
            other => {
                return Err(format!(
                    "changed_checker_paths is not a list: `{}`",
                    ctx.format_value(other)
                ))
            }
        }
    }
}

/// The module seeds a decision contributes: every module the corpus admits when the rule chose
/// `EveryAdmittedModule`, and none otherwise. The runner and its route control both call this, so
/// the control prepares exactly what the floor prepares.
pub(crate) fn checker_module_seeds(
    decision: &CheckerSubjectDecision,
    corpus: &SourceCorpusRead,
) -> Vec<String> {
    if !decision.selects_every_admitted_module() {
        return Vec::new();
    }
    corpus
        .inventory
        .iter()
        .map(|view| view.module_path.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dep_info_prerequisites_unescape_spaces_and_join_continuations() {
        let text = "/w/target/release/claim_executor: /w/src/a.rs /w/src/b\\ c.rs \\\n /w/src/d.rs\n\n/w/src/a.rs:\n";
        assert_eq!(
            parse_dep_info_prerequisites(text),
            vec!["/w/src/a.rs", "/w/src/b c.rs", "/w/src/d.rs"]
        );
    }

    // THE ROSTER-COMPLETENESS CONTROL: a file the checker compiles cannot be missing from the
    // checker inputs, because the inputs ARE the compiled set. Read against this binary's own
    // dep-info when cargo wrote one beside the test executable's profile directory.
    #[test]
    fn every_workspace_file_the_record_lists_is_a_checker_input() {
        let root = tempdir_with_manifest();
        let a = root.join("crate/src/infer_helper.rs");
        let prerequisites = vec![
            a.to_string_lossy().to_string(),
            "crate/src/relative_helper.rs".to_string(),
            "/registry/serde/src/lib.rs".to_string(),
        ];
        let inputs = checker_input_paths(&prerequisites, &root);
        assert!(inputs.contains(&"crate/src/infer_helper.rs".to_string()));
        assert!(inputs.contains(&"crate/src/relative_helper.rs".to_string()));
        assert!(inputs.contains(&"crate/Cargo.toml".to_string()));
        assert!(inputs.contains(&"Cargo.lock".to_string()));
        assert!(!inputs.iter().any(|p| p.contains("serde")));
        let _ = std::fs::remove_dir_all(&root);
    }

    // THE OBSERVATION RUNS FOR REAL, over this test binary. Its record is rustc's per-crate one
    // (`deps/v1_compiler-<hash>.d`), which lists this crate's files and not the infer crate's:
    // measured 2026-09-30, 159 inputs and no v1_compiler_infer.rs. Read as the checker's inputs,
    // it would have left the gunbc#12441 edit outside the roster. So the observation must refuse
    // it, and this control is red if it ever reads such a record as complete.
    #[test]
    fn a_per_crate_record_that_omits_the_infer_crate_is_refused() {
        let refusal = observe_checker_input_paths(&process_workspace_root())
            .expect_err("a test binary carries only rustc's per-crate record");
        assert!(refusal.contains("per-crate"), "{refusal}");
    }

    // THE CARGO FORM, which the floor's claim_executor carries: manifests listed, the infer
    // crate's files listed, and this file (the anchor) listed. Admitted, and dropping the anchor
    // refuses it.
    #[test]
    fn a_cargo_record_is_admitted_only_with_its_anchor() {
        let root = process_workspace_root();
        let root = root.canonicalize().unwrap_or(root);
        let abs = |p: &str| root.join(p).to_string_lossy().into_owned();
        let with_anchor = format!(
            "{}: {} {} {}\n",
            abs("target/release/claim_executor"),
            abs("src/v1/stage0/Cargo.toml"),
            abs("src/v1/stage0/src/v1_compiler_infer.rs"),
            abs(file!())
        );
        let inputs = checker_inputs_from_record(&with_anchor, Path::new("claim_executor.d"), &root)
            .expect("a cargo record with its anchor is the checker's inputs");
        assert!(inputs.contains(&"src/v1/stage0/src/v1_compiler_infer.rs".to_string()));
        let without_anchor = format!(
            "{}: {} {}\n",
            abs("target/release/claim_executor"),
            abs("src/v1/stage0/Cargo.toml"),
            abs("src/v1/stage0/src/v1_compiler_infer.rs")
        );
        let refusal =
            checker_inputs_from_record(&without_anchor, Path::new("claim_executor.d"), &root)
                .expect_err("a record that omits the running checker's own file is refused");
        assert!(refusal.contains("does not list"), "{refusal}");
    }

    // THE REAL FRAME DECIDES: the rule is evaluated from v2.workflow.floor_subject_seed, not
    // restated here.
    #[test]
    fn the_modeled_rule_widens_only_for_a_changed_checker_input() {
        let roots = default_source_roots();
        let record = Ok(vec!["src/v1/stage0/src/v1_compiler_infer.rs".to_string()]);
        let widened = checker_subject_decision(
            &roots,
            &record,
            &["src/v1/stage0/src/v1_compiler_infer.rs".to_string()],
        )
        .expect("decision");
        assert!(widened.selects_every_admitted_module(), "{widened:?}");
        let narrow =
            checker_subject_decision(&roots, &record, &["src/v2/workflow/x.dag".to_string()])
                .expect("decision");
        assert_eq!(narrow.rule, "CheckerInputsUnchanged");
        let refused = checker_subject_decision(
            &roots,
            &Err("dep-info unreadable".to_string()),
            &["src/v2/workflow/x.dag".to_string()],
        )
        .expect("decision");
        assert!(refused.refused(), "{refused:?}");
        assert_eq!(refused.reason, "dep-info unreadable");
    }

    fn tempdir_with_manifest() -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "checker_dependency_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(root.join("crate/src")).unwrap();
        std::fs::write(root.join("crate/Cargo.toml"), "").unwrap();
        std::fs::write(root.join("Cargo.lock"), "").unwrap();
        std::fs::write(root.join("crate/src/infer_helper.rs"), "").unwrap();
        root
    }
}
