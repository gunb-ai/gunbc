//! Regenerate `tokenize_generated.rs` from `src/v3/compiler/tokenize.dag`.
//!
//! Implementation lives in `v3_compiler::regen_tokenize` so integration tests
//! can invoke the same pipeline in-process (no nested `cargo run`).

fn main() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    v3_compiler::regen_tokenize::write_tokenize_generated_rs(&manifest_dir);
}
