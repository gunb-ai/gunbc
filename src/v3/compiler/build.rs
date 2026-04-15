// Build script: enumerate every `*.dag` file in `src/v3/std/` and
// `src/v3/spec/` and generate a Rust array of `(path, content)`
// pairs the bootstrap loader consumes. Adding a new substrate
// declaration file or per-target language spec file becomes a
// pure file-system change — no Rust edits to `bootstrap.rs`, no
// fixture-array maintenance.
//
// **Why two directories.** `src/v3/std/` hosts v3 substrate
// declarations that are conceptually part of the shared standard
// library (their canonical eventual home is `dsl/std/`, per the
// THESIS.md "Bootstrap staging" note). `src/v3/spec/` hosts
// v3-only language spec files (v3_l1.dag, rust.dag) whose
// canonical eventual home is `dsl/extdeps/languages/`. Both are
// kept outside the `dsl/` tree today because v2's CI scanner
// doesn't know about v3-specific types; `src/v3/std/` and
// `src/v3/spec/` are staging locations during the v2→v3 window.
// When v2 retires, both directories collapse into their canonical
// homes under `dsl/`.
//
// **Why a build script.** The pre-unwind shape used hardcoded
// `const RUST_DAG: &str = include_str!("../../spec/rust.dag");`
// constants in `bootstrap.rs`. PR #445 review flagged this as a
// duplicate-authority bug: the on-disk spec files and the Rust
// fixture array are two parallel representations of the same set,
// and adding a new target requires editing both. This script
// removes the parallel representation by deriving the array from
// the filesystem.
//
// **Load order.** Two-phase bootstrap (collect_symbols then
// lower_bodies) makes cross-file references order-independent for
// symbol resolution. The order rule below is a readability /
// dependency-respecting convention, not a correctness
// requirement:
//
//   1. `v3_l1.dag` first (substrate markers, no dependencies).
//   2. `src/v3/std/*.dag` next, alphabetically (substrate
//      declarations — TypeConnective, Behavior, Declaration, etc.).
//   3. Other `src/v3/spec/*.dag` after, alphabetically (per-target
//      language specs that reference substrate declarations via
//      TypeRealization / FunctionRealization entries).
//
// The `v3_l1.dag` special case is preserved from the pre-split
// build script.
//
// **Output**: writes a Rust file `OUT_DIR/v3_specs.rs` containing:
//
//   pub static V3_SPECS: &[(&str, &str)] = &[
//       ("src/v3/spec/v3_l1.dag",    include_str!("...")),
//       ("src/v3/std/substrate.dag", include_str!("...")),
//       ("src/v3/spec/rust.dag",     include_str!("...")),
//   ];
//
// `bootstrap.rs` uses `include!(concat!(env!("OUT_DIR"),
// "/v3_specs.rs"));` to pull the array in. The `include_str!`
// calls inside the generated file resolve at compile time against
// absolute paths, so the runtime is still hermetic — no
// filesystem access at `Dag::new()` time.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// One .dag file discovered during the scan, with enough context
/// to produce a display path and sort it into the correct load
/// order.
struct Entry {
    /// Absolute path for `include_str!`.
    abs_path: PathBuf,
    /// Project-relative directory (`src/v3/std` or `src/v3/spec`).
    display_dir: &'static str,
    /// Just the file name (e.g. `substrate.dag`).
    file_name: String,
}

fn main() {
    // Resolve the v3 root relative to CARGO_MANIFEST_DIR
    // (which points at src/v3/compiler/). The sibling directories
    // are src/v3/std/ and src/v3/spec/, one level up and across.
    let manifest_dir =
        env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set by Cargo");
    let manifest_path = PathBuf::from(&manifest_dir);
    let v3_root = manifest_path
        .parent()
        .expect("compiler dir has parent");
    let std_dir = v3_root.join("std");
    let spec_dir = v3_root.join("spec");

    // Tell Cargo to re-run the script if any file in either
    // directory changes. Without this, adding a new .dag file
    // wouldn't trigger a rebuild and the loader would silently
    // miss it.
    println!("cargo:rerun-if-changed={}", std_dir.display());
    println!("cargo:rerun-if-changed={}", spec_dir.display());

    let mut entries: Vec<Entry> = Vec::new();
    entries.extend(scan_dag_files(&std_dir, "src/v3/std"));
    entries.extend(scan_dag_files(&spec_dir, "src/v3/spec"));

    // Sort into load order. `v3_l1.dag` first (special-cased),
    // then std/ files alphabetically, then remaining spec/ files
    // alphabetically. Encoded as a (priority, directory, name)
    // sort key so a single sort produces the final order.
    entries.sort_by(|a, b| {
        fn load_priority(e: &Entry) -> u8 {
            if e.file_name == "v3_l1.dag" {
                0
            } else if e.display_dir == "src/v3/std" {
                1
            } else {
                2
            }
        }
        (load_priority(a), a.display_dir, a.file_name.as_str())
            .cmp(&(load_priority(b), b.display_dir, b.file_name.as_str()))
    });

    // Generate the Rust source. Each entry becomes a
    // `(path_string, include_str!(absolute_path))` tuple. The
    // path string is the project-relative path used for
    // diagnostic spans (e.g. "src/v3/spec/rust.dag"); the
    // `include_str!` argument is an absolute path so the macro
    // resolves regardless of where the generated file lands.
    let mut generated = String::from(
        "// AUTO-GENERATED by build.rs. Do not edit.\n\
         //\n\
         // Spec files enumerated from src/v3/std/*.dag and\n\
         // src/v3/spec/*.dag. Adding a new substrate declaration\n\
         // or per-target language spec is a pure file-system\n\
         // change — drop the .dag file in the appropriate\n\
         // directory and the build script picks it up at the\n\
         // next compile.\n\
         pub static V3_SPECS: &[(&str, &str)] = &[\n",
    );
    for entry in &entries {
        let display_path = format!("{}/{}", entry.display_dir, entry.file_name);
        generated.push_str(&format!(
            "    (\"{}\", include_str!(\"{}\")),\n",
            display_path,
            entry.abs_path.display()
        ));
    }
    generated.push_str("];\n");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR must be set by Cargo");
    let out_path = Path::new(&out_dir).join("v3_specs.rs");
    fs::write(&out_path, generated)
        .unwrap_or_else(|e| panic!("failed to write {}: {}", out_path.display(), e));
}

/// Scan a directory for `*.dag` files. Tolerates a missing
/// directory (returns empty) so a build in a tree where one of
/// the staging dirs doesn't exist yet still works.
fn scan_dag_files(dir: &Path, display_dir: &'static str) -> Vec<Entry> {
    let read = match fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    read.filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("dag"))
        .map(|p| {
            let file_name = p
                .file_name()
                .and_then(|s| s.to_str())
                .expect("entry has utf-8 file name")
                .to_string();
            let abs_path = p.canonicalize().unwrap_or_else(|e| {
                panic!("failed to canonicalize {}: {}", p.display(), e)
            });
            Entry {
                abs_path,
                display_dir,
                file_name,
            }
        })
        .collect()
}
