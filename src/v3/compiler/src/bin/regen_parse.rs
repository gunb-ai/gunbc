//! Regenerate `parse_generated.rs` from `src/v3/std/parse_surface.dag` (Surface carriers)
//! plus `parse_parser_body.txt` (sibling file under `src/v3/compiler/`).
//!
//! **Parser staging (PR posture — option 1), not SG-2b closure:** `parse_surface.dag` is
//! the Surface **carrier** authority (shared with `parse_surface_generated.rs`); `parse_parser_body.txt`
//! remains temporary **semantic** algorithm authority until a follow-on lane lands true `.dag` parse
//! rules. **Dissolution:** same trigger as the header on `parse_parser_body.txt` (delete the fragment
//! once `regen_parse` emits the algorithm from `.dag` alone).

use std::path::PathBuf;

use v3_compiler::generated_files::GENERATED_FILES;
use v3_compiler::render_parse_generated_rs;

const GENERATED_FILE: &str = "src/v3/compiler/src/parse_generated.rs";
const SURFACE_TYPES_AUTHORITY_FILE: &str = "src/v3/std/parse_surface.dag";
const PARSER_BODY_REL: &str = "parse_parser_body.txt";

fn main() {
    assert!(
        GENERATED_FILES.contains(&GENERATED_FILE),
        "`regen_parse` writes `{GENERATED_FILE}` but that path is not \
         registered in `REGEN_OUTPUTS` in `src/v3/compiler/build.rs`."
    );

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dag_path = manifest_dir.join("..").join("std").join("parse_surface.dag");
    let source = std::fs::read_to_string(&dag_path).expect("read parse_surface.dag");
    let body_path = manifest_dir.join(PARSER_BODY_REL);
    let parser_body = std::fs::read_to_string(&body_path)
        .unwrap_or_else(|e| panic!("read parser body fragment `{}`: {e}", body_path.display()));

    let formatted = render_parse_generated_rs(&source, SURFACE_TYPES_AUTHORITY_FILE, &parser_body)
        .unwrap_or_else(|e| panic!("{e}"));

    let out_path = manifest_dir.join("src").join("parse_generated.rs");
    std::fs::write(&out_path, &formatted).expect("write parse_generated.rs");
    println!("wrote {}", out_path.display());
}
