//! Regenerate `bootstrap_*_generated.rs` snapshots from on-disk `.dag` sources.
//!
//! This binary is the sole `v3-compiler` target with `required-features =
//! ["bootstrap-regen-fresh"]`, so the fresh-parse bootstrap subgraph is not part
//! of the default library build graph.
//!
//! PB-1-e mechanism (ii): the **fresh-compile vs committed snapshot** gate lives
//! here, not in `cargo test`. Use `--verify` in CI (or before push) to assert the
//! checked-in `bootstrap_*.rs` files match a from-scratch compile; omit `--verify`
//! to write updated snapshots after intentional `.dag` changes.
//!
//! Local invocation (feature is required):
//! `cargo run -p v3-compiler --features bootstrap-regen-fresh --bin regen_bootstrap`
//! (append `-- --verify` to check without writing).

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use v3_compiler::{
    compile_full_bootstrap_dag_from_std_seed,
    compile_full_bootstrap_without_parse_surface_dag_from_std_seed, compile_std_bootstrap_dag,
    generated_files::GENERATED_FILES, render_bootstrap_generated_rs,
    render_bootstrap_std_generated_rs,
};

const GENERATED_STD_FILE: &str = "src/v3/compiler/src/bootstrap_std_generated.rs";
const GENERATED_FULL_FILE: &str = "src/v3/compiler/src/bootstrap_generated.rs";
const GENERATED_NO_PARSE_SURFACE_FILE: &str =
    "src/v3/compiler/src/bootstrap_generated_without_parse_surface.rs";

fn main() {
    let verify_only = env::args().skip(1).any(|a| a == "--verify");

    for generated_file in [
        GENERATED_STD_FILE,
        GENERATED_FULL_FILE,
        GENERATED_NO_PARSE_SURFACE_FILE,
    ] {
        assert!(
            GENERATED_FILES.contains(&generated_file),
            "`regen_bootstrap` writes `{generated_file}` but that path is not \
             registered in `REGEN_OUTPUTS` in `src/v3/compiler/build.rs`."
        );
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let std_dag = compile_std_bootstrap_dag();
    let std_formatted = render_bootstrap_std_generated_rs(&std_dag)
        .unwrap_or_else(|e| panic!("regen_bootstrap std: {e}"));

    let full_dag = compile_full_bootstrap_dag_from_std_seed(std_dag.clone());
    let full_formatted = render_bootstrap_generated_rs(
        &full_dag,
        "dsl/std/*.dag + src/v3/std/*.dag + src/v3/spec/*.dag + src/v3/compiler/*.dag minus tokenize.dag + dsl/extdeps/languages/rust/primitives.dag",
        "bootstrapped_fixture_dag",
    )
    .unwrap_or_else(|e| panic!("regen_bootstrap full: {e}"));

    let full_no_parse_surface_dag =
        compile_full_bootstrap_without_parse_surface_dag_from_std_seed(std_dag);
    let full_no_parse_surface_formatted = render_bootstrap_generated_rs(
        &full_no_parse_surface_dag,
        "dsl/std/*.dag + src/v3/std/*.dag + src/v3/spec/*.dag + src/v3/compiler/*.dag minus tokenize.dag and src/v3/std/parse_surface.dag + dsl/extdeps/languages/rust/primitives.dag",
        "bootstrapped_fixture_without_parse_surface_dag",
    )
    .unwrap_or_else(|e| panic!("regen_bootstrap no-parse-surface: {e}"));

    if verify_only {
        assert_disk_matches(&manifest_dir, "bootstrap_std_generated.rs", &std_formatted);
        assert_disk_matches(&manifest_dir, "bootstrap_generated.rs", &full_formatted);
        assert_disk_matches(
            &manifest_dir,
            "bootstrap_generated_without_parse_surface.rs",
            &full_no_parse_surface_formatted,
        );
        println!("regen_bootstrap --verify: committed snapshots match fresh compile.");
        return;
    }

    write_generated(&manifest_dir, "bootstrap_std_generated.rs", &std_formatted);
    write_generated(&manifest_dir, "bootstrap_generated.rs", &full_formatted);
    write_generated(
        &manifest_dir,
        "bootstrap_generated_without_parse_surface.rs",
        &full_no_parse_surface_formatted,
    );
}

fn assert_disk_matches(manifest_dir: &Path, file_name: &str, expected: &str) {
    let path = manifest_dir.join("src").join(file_name);
    let on_disk =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    if on_disk != expected {
        panic!(
            "{file_name}: committed snapshot does not match fresh compile from .dag sources. \
             Run `cargo run -p v3-compiler --features bootstrap-regen-fresh --bin regen_bootstrap` \
             (without `--verify`) to \
             regenerate after intentional authority edits, or fix unintended drift. \
             (byte length on_disk={}, expected={})",
            on_disk.len(),
            expected.len(),
        );
    }
}

fn write_generated(manifest_dir: &Path, file_name: &str, contents: &str) {
    let out_path = manifest_dir.join("src").join(file_name);
    std::fs::write(&out_path, contents).unwrap_or_else(|e| panic!("write {file_name}: {e}"));
    println!("wrote {}", out_path.display());
}
