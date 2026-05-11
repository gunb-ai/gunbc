//! **Layer:** integration
//!
//! R3 T-V2-Retirement — gate **`v2_oracle_no_remaining_test_consumers`**
//! ([`docs/r3-structure.md`](../../../../../../docs/r3-structure.md) §T-V2-Retirement;
//! program plan #41).
//!
//! Mechanical receipt aligned to G-1 in
//! [`docs/audit/t-v2-g2-deletion-plan-and-guardrails.md`](../../../../../../docs/audit/t-v2-g2-deletion-plan-and-guardrails.md):
//! `grep -rEn 'v2[_-]compiler(_tests|-tests)?' src/` excluding `src/v2/` → zero matches on
//! comment-stripped Rust sources; Cargo **dependency** tables (including `workspace.dependencies`)
//! parsed as TOML (not line regex)
//! must not link the legacy v2 crates from manifests outside `src/v2/` (including path deps whose
//! resolved tree lies under `src/v2/` even when the dep key / `package` spelling does not).

use crate::common::strip_rust_comments;
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
}

/// Spellings for `v2[_-]compiler(_tests|-tests)?` (G-1 grep). Longer keys first so diagnostics
/// prefer the specific crate name; all are substring-scanned on comment-stripped Rust (same as
/// `grep -E` on source text, including string literals — covers `v2-compiler`, `v2_compiler ::`, …).
const G1_V2_CRATE_SUBSTRINGS: &[&str] = &[
    concat!("v2-", "compiler-", "tests"),
    concat!("v2-", "compiler_", "tests"),
    concat!("v2_", "compiler-", "tests"),
    concat!("v2_", "compiler_", "tests"),
    concat!("v2-", "compiler"),
    concat!("v2_", "compiler"),
];

/// `path` is under `src_root` (the workspace `src/` directory). True when the relative path's
/// first component is `v2`.
fn is_under_src_v2(path: &Path, src_root: &Path) -> bool {
    if let Ok(rel) = path.strip_prefix(src_root) {
        if let Some(std::path::Component::Normal(first)) = rel.components().next() {
            return first == "v2";
        }
    }
    false
}

fn collect_rs_files(dir: &Path, src_root: &Path, out: &mut Vec<PathBuf>) {
    if is_under_src_v2(dir, src_root) {
        return;
    }
    let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("read_dir {}: {e}", dir.display()));
    for ent in entries {
        let ent = ent.unwrap_or_else(|e| panic!("read_dir entry {}: {e}", dir.display()));
        let p = ent.path();
        if p.is_dir() {
            collect_rs_files(&p, src_root, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

fn collect_cargo_toml_files(dir: &Path, src_root: &Path, out: &mut Vec<PathBuf>) {
    if is_under_src_v2(dir, src_root) {
        return;
    }
    let cargo = dir.join("Cargo.toml");
    if cargo.is_file() {
        out.push(cargo);
    }
    let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("read_dir {}: {e}", dir.display()));
    for ent in entries {
        let ent = ent.unwrap_or_else(|e| panic!("read_dir entry {}: {e}", dir.display()));
        let p = ent.path();
        if p.is_dir() {
            collect_cargo_toml_files(&p, src_root, out);
        }
    }
}

fn rust_source_first_g1_match(stripped: &str) -> Option<&'static str> {
    G1_V2_CRATE_SUBSTRINGS
        .iter()
        .find(|&&needle| stripped.contains(needle))
        .copied()
}

fn is_g1_v2_manifest_name(name: &str) -> bool {
    G1_V2_CRATE_SUBSTRINGS.contains(&name)
}

/// Resolve `.` / `..` in `path` without touching the filesystem (Cargo path deps are relative to
/// the manifest directory).
fn lexical_normalize(path: &Path) -> PathBuf {
    use std::path::Component;
    let mut out = PathBuf::new();
    for comp in path.components() {
        match comp {
            Component::CurDir => {}
            Component::ParentDir => {
                if matches!(out.components().next_back(), Some(Component::Normal(_))) {
                    out.pop();
                }
            }
            _ => out.push(comp.as_os_str()),
        }
    }
    out
}

fn dep_path_reaches_src_v2(manifest_path: &Path, workspace_root: &Path, spec: &Value) -> bool {
    let Some(t) = spec.as_table() else {
        return false;
    };
    let Some(rel) = t.get("path").and_then(Value::as_str) else {
        return false;
    };
    let dir = manifest_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let joined = lexical_normalize(&dir.join(rel));
    let src_v2 = lexical_normalize(&workspace_root.join("src").join("v2"));

    match (fs::canonicalize(&joined), fs::canonicalize(&src_v2)) {
        (Ok(ref jc), Ok(ref sv)) => jc.starts_with(sv),
        // Missing path components: canonicalize fails even when the author clearly aimed
        // under `src/v2/` (exploratory fail-open called out in review). Fall back to
        // lexical `..` resolution vs workspace `src/v2`.
        _ => joined.starts_with(&src_v2),
    }
}

fn dep_spec_links_v2_crate(name: &str, spec: &Value) -> bool {
    let name_hit = is_g1_v2_manifest_name(name);
    let pkg_hit = spec
        .as_table()
        .and_then(|t| t.get("package"))
        .and_then(Value::as_str)
        .is_some_and(is_g1_v2_manifest_name);
    if !(name_hit || pkg_hit) {
        return false;
    }
    match spec {
        Value::String(_) => name_hit,
        // Inline tables: key or `package =` already names the crate; count registry `version`, git
        // subtables, etc., not only `path` / `workspace` / `git` top-level keys.
        Value::Table(_) => true,
        _ => false,
    }
}

fn scan_dep_table(
    table: &toml::map::Map<String, Value>,
    scan_ctx: Option<(&Path, &Path)>,
) -> Option<String> {
    for (dep_key, dep_spec) in table {
        if dep_spec_links_v2_crate(dep_key, dep_spec) {
            return Some(format!(
                "dependency key or `package =` rename targets legacy v2 crate ({dep_key})",
            ));
        }
        if let Some((manifest_path, workspace_root)) = scan_ctx {
            if dep_path_reaches_src_v2(manifest_path, workspace_root, dep_spec) {
                return Some(format!(
                    "dependency `{dep_key}` path resolves under src/v2 (legacy tree link)",
                ));
            }
        }
    }
    None
}

fn scan_dependencies_value(sect: &Value, scan_ctx: Option<(&Path, &Path)>) -> Option<String> {
    let table = sect.as_table()?;
    scan_dep_table(table, scan_ctx)
}

fn manifest_violation_g1_v2_edge(
    root: &toml::map::Map<String, Value>,
    scan_ctx: Option<(&Path, &Path)>,
) -> Option<String> {
    for key in ["dependencies", "build-dependencies", "dev-dependencies"] {
        if let Some(sect) = root.get(key) {
            if let Some(msg) = scan_dependencies_value(sect, scan_ctx) {
                return Some(format!("{key}: {msg}"));
            }
        }
    }

    if let Some(Value::Table(ws)) = root.get("workspace") {
        if let Some(sect) = ws.get("dependencies") {
            if let Some(msg) = scan_dependencies_value(sect, scan_ctx) {
                return Some(format!("workspace.dependencies: {msg}"));
            }
        }
    }

    if let Some(Value::Table(targets)) = root.get("target") {
        for (triple, tv) in targets {
            let Some(t) = tv.as_table() else {
                continue;
            };
            for key in ["dependencies", "build-dependencies", "dev-dependencies"] {
                if let Some(sect) = t.get(key) {
                    if let Some(msg) = scan_dependencies_value(sect, scan_ctx) {
                        return Some(format!("target.{triple} {key}: {msg}"));
                    }
                }
            }
        }
    }

    None
}

#[test]
fn g1_manifest_workspace_dependencies_detected() {
    let v2_pkg = concat!("v2-", "compiler");
    let raw = format!(
        concat!(
            "[package]\nname = \"x\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n",
            "[workspace]\nmembers = [\".\"]\n\n",
            "[workspace.dependencies]\n{0} = {{ path = \"src/v2/stage0\" }}\n",
        ),
        v2_pkg,
    );
    let v: Value = toml::from_str(&raw).expect("fixture toml");
    let root = v.as_table().expect("root table");
    let viol = manifest_violation_g1_v2_edge(root, None);
    assert!(
        viol.is_some(),
        "expected workspace.dependencies v2 path dep to register; got {viol:?}"
    );
}

#[test]
fn g1_rust_substrings_match_whitespace_path_and_hyphen_forms() {
    let v2_cc = concat!("v2_", "compiler");
    let v2_hy = concat!("v2-", "compiler");
    let src_path = format!("fn f() {{ let _ = {v2_cc} :: foo::bar(); }}\n");
    assert_eq!(
        rust_source_first_g1_match(&strip_rust_comments(&src_path)),
        Some(v2_cc),
    );
    let src_lit = format!("const S: &str = \"{v2_hy}\";\n");
    assert_eq!(
        rust_source_first_g1_match(&strip_rust_comments(&src_lit)),
        Some(v2_hy),
    );
}

#[test]
fn g1_manifest_dotted_dev_deps_table_detected() {
    let raw = concat!(
        "[package]\nname = \"x\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n",
        "[dev-dependencies.",
        "v2-",
        "compiler]\n",
        "path = \"../stage0\"\n",
    );
    let v: Value = toml::from_str(raw).expect("fixture toml");
    let root = v.as_table().expect("root table");
    let viol = manifest_violation_g1_v2_edge(root, None);
    assert!(
        viol.is_some(),
        "expected dotted dev-dependencies table to register as v2 edge; got {viol:?}"
    );
}

#[test]
fn g1_manifest_package_rename_path_detected() {
    let raw = format!(
        concat!(
            "[package]\nname = \"x\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n",
            "[dependencies]\n",
            "st-{0} = {{ package = \"{1}\", path = \"../stage0\" }}\n",
        ),
        "shim",
        concat!("v2-", "compiler"),
    );
    let v: Value = toml::from_str(&raw).expect("fixture toml");
    let root = v.as_table().expect("root table");
    let viol = manifest_violation_g1_v2_edge(root, None);
    assert!(
        viol.is_some(),
        "expected package-rename path dep to register; got {viol:?}"
    );
}

#[test]
fn g1_manifest_registry_version_inline_table_detected() {
    let v2_pkg = concat!("v2-", "compiler");
    let raw = format!(
        concat!(
            "[package]\nname = \"x\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n",
            "[dependencies]\n{0} = {{ version = \"0.0.1\" }}\n",
        ),
        v2_pkg,
    );
    let v: Value = toml::from_str(&raw).expect("fixture toml");
    let root = v.as_table().expect("root table");
    let viol = manifest_violation_g1_v2_edge(root, None);
    assert!(
        viol.is_some(),
        "expected registry version inline-table dep to register; got {viol:?}"
    );
}

#[test]
fn g1_manifest_package_rename_version_inline_table_detected() {
    let v2_pkg = concat!("v2-", "compiler");
    let raw = format!(
        concat!(
            "[package]\nname = \"x\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n",
            "[dependencies]\n",
            "st-shim = {{ package = \"{0}\", version = \"0.0.1\" }}\n",
        ),
        v2_pkg,
    );
    let v: Value = toml::from_str(&raw).expect("fixture toml");
    let root = v.as_table().expect("root table");
    let viol = manifest_violation_g1_v2_edge(root, None);
    assert!(
        viol.is_some(),
        "expected package-rename registry dep to register; got {viol:?}"
    );
}

#[test]
fn g1_manifest_neutral_key_path_into_src_v2_detected() {
    let workspace_root = workspace_root();
    let manifest_path = workspace_root
        .join("src")
        .join("v3")
        .join("compiler")
        .join("Cargo.toml");
    assert!(
        manifest_path.is_file(),
        "fixture requires {}",
        manifest_path.display(),
    );

    let raw = concat!(
        "[package]\nname = \"x\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n",
        "[dependencies]\n",
        "shim-crate = { path = \"../../v2/stage0\" }\n",
    );
    let v: Value = toml::from_str(raw).expect("fixture toml");
    let root = v.as_table().expect("root table");
    let viol = manifest_violation_g1_v2_edge(root, Some((&manifest_path, &workspace_root)));
    assert!(
        viol.is_some(),
        "expected neutral dep key with path into src/v2 to register; got {viol:?}"
    );
}

#[test]
fn g1_manifest_neutral_key_path_lexically_under_missing_src_v2_entry_detected() {
    let workspace_root = workspace_root();
    let manifest_path = workspace_root
        .join("src")
        .join("v3")
        .join("compiler")
        .join("Cargo.toml");
    assert!(
        manifest_path.is_file(),
        "fixture requires {}",
        manifest_path.display(),
    );

    let missing_under_v2 = concat!("../../v2/__gate41_missing_path_probe");
    let raw = format!(
        concat!(
            "[package]\nname = \"x\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n",
            "[dependencies]\n",
            "shim-crate = {{ path = \"{0}\" }}\n",
        ),
        missing_under_v2,
    );
    assert!(
        !workspace_root
            .join("src")
            .join("v2")
            .join("__gate41_missing_path_probe")
            .exists(),
        "fixture path segment must not exist on disk (canonicalize fail)",
    );
    let v: Value = toml::from_str(&raw).expect("fixture toml");
    let root = v.as_table().expect("root table");
    let viol = manifest_violation_g1_v2_edge(root, Some((&manifest_path, &workspace_root)));
    assert!(
        viol.is_some(),
        "expected lexical path under src/v2 to register when target is missing; got {viol:?}"
    );
}

#[test]
fn v2_oracle_no_remaining_test_consumers_rust_sources() {
    let workspace_root = workspace_root();
    let src_root = workspace_root.join("src");
    assert!(src_root.is_dir(), "expected src/ at {}", src_root.display());

    let mut rs_files = Vec::new();
    collect_rs_files(&src_root, &src_root, &mut rs_files);
    rs_files.sort();

    for path in rs_files {
        let raw =
            fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let stripped = strip_rust_comments(&raw);
        if let Some(needle) = rust_source_first_g1_match(&stripped) {
            let rel = path.strip_prefix(&workspace_root).unwrap_or(&path);
            panic!(
                "v2_oracle_no_remaining_test_consumers / G-1: `{}` matches `v2[_-]compiler(_tests|-tests)?` \
                 audit surface ({needle:?} after comment strip). Remove the legacy v2 crate reference \
                 or relocate under `src/v2/`.",
                rel.display(),
            );
        }
    }
}

#[test]
fn v2_oracle_no_remaining_test_consumers_workspace_manifests() {
    let workspace_root = workspace_root();
    let src_root = workspace_root.join("src");

    let mut manifests = Vec::with_capacity(16);
    let root_toml = workspace_root.join("Cargo.toml");
    assert!(
        root_toml.is_file(),
        "expected workspace Cargo.toml at {}",
        root_toml.display()
    );
    manifests.push(root_toml);

    if src_root.is_dir() {
        collect_cargo_toml_files(&src_root, &src_root, &mut manifests);
    }
    manifests.sort();

    for path in manifests {
        let raw =
            fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let root: toml::map::Map<String, Value> =
            match toml::from_str::<Value>(&raw).expect("Cargo.toml must parse as TOML") {
                Value::Table(t) => t,
                _ => panic!("expected Cargo.toml root table at {}", path.display()),
            };
        if let Some(detail) = manifest_violation_g1_v2_edge(&root, Some((&path, &workspace_root))) {
            panic!(
                "v2_oracle_no_remaining_test_consumers / G-1: `{}` links legacy v2 crate in a \
                 dependency table: {detail}",
                path.strip_prefix(&workspace_root)
                    .unwrap_or(&path)
                    .display(),
            );
        }
    }
}
