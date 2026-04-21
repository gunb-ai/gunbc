//! SG-2c-1 — Regenerate `parse_tables_generated.rs` from
//! `src/v3/compiler/parse_tables.dag`. Thin host shim: the render logic
//! lives in `v3_compiler::regen_parse_tables_emit::render_parse_tables_generated_rs`
//! so hermetic integration tests can compare in-memory without spawning
//! this binary through `cargo run` (which blows the 2s per-test ratchet
//! on cold CI).
//!
//! Scope (grammar-tables prototype, NOT SG-2c proper) — see
//! `src/v3/compiler/parse_tables.dag` for the full scope note and
//! dissolution trigger (recursive list-body emission over `List<Token>`).

use std::path::PathBuf;

use v3_compiler::generated_files::GENERATED_FILES;
use v3_compiler::render_parse_tables_generated_rs;

const GENERATED_FILE: &str = "src/v3/compiler/src/parse_tables_generated.rs";
const PARSE_TABLES_AUTHORITY_FILE: &str = "src/v3/compiler/parse_tables.dag";
const TOKENIZE_AUTHORITY_FILE: &str = "src/v3/compiler/tokenize.dag";

fn main() {
    assert!(
        GENERATED_FILES.contains(&GENERATED_FILE),
        "`regen_parse_tables` writes `{GENERATED_FILE}` but that path is not \
         registered in `REGEN_OUTPUTS` in `src/v3/compiler/build.rs`."
    );

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let tables_source = std::fs::read_to_string(manifest_dir.join("parse_tables.dag"))
        .expect("read parse_tables.dag");
    let tokenize_source =
        std::fs::read_to_string(manifest_dir.join("tokenize.dag")).expect("read tokenize.dag");

    let formatted = render_parse_tables_generated_rs(
        &tables_source,
        PARSE_TABLES_AUTHORITY_FILE,
        &tokenize_source,
        TOKENIZE_AUTHORITY_FILE,
    )
    .unwrap_or_else(|e| panic!("{e}"));

    let out_path = manifest_dir.join("src").join("parse_tables_generated.rs");
    std::fs::write(&out_path, &formatted).expect("write parse_tables_generated.rs");
    println!("wrote {}", out_path.display());
}
