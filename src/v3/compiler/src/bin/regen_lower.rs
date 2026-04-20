//! Regenerate `lower_generated.rs` from `src/v3/compiler/lowering_rust.authority`.
//!
//! SG-3b: the merged compiler crate must not carry a hand-maintained
//! `lower.rs`. The lowering walk stays staged in `lowering_rust.authority`
//! (not `*.rs`, so SG-0 does not treat it as handwritten Rust surface)
//! until `Surface*` lives in the substrate and `lower.dag` can own the
//! algorithm (SELF_HOSTING.md §4).

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use v3_compiler::generated_files::GENERATED_FILES;

const GENERATED_FILE: &str = "src/v3/compiler/src/lower_generated.rs";
const AUTHORITY_FILE: &str = "lowering_rust.authority";

const HEADER: &str = "// AUTO-GENERATED from `src/v3/compiler/lowering_rust.authority` via\n\
     // `regen_lower`. Regenerate instead of hand-editing.\n\
     //\n\
     // Authority staging lives in `lowering_rust.authority` until `Surface*`\n\
     // declarations and `lower.dag` absorb this walk (SELF_HOSTING.md §4).\n\
     \n";

fn main() {
    assert!(
        GENERATED_FILES.contains(&GENERATED_FILE),
        "`regen_lower` writes `{GENERATED_FILE}` but that path is not \
         registered in `REGEN_OUTPUTS` in `src/v3/compiler/build.rs`."
    );

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let authority_path = manifest_dir.join(AUTHORITY_FILE);
    let body = std::fs::read_to_string(&authority_path).unwrap_or_else(|e| {
        panic!(
            "read lowering authority `{}`: {e}",
            authority_path.display()
        )
    });
    let combined = format!("{HEADER}{body}");

    let mut child = Command::new("rustfmt")
        .arg("--emit")
        .arg("stdout")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn rustfmt");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(combined.as_bytes())
        .unwrap();
    let output = child.wait_with_output().expect("rustfmt");
    assert!(output.status.success(), "rustfmt failed");
    let formatted = String::from_utf8(output.stdout).expect("utf8");

    let out_path = manifest_dir.join("src").join("lower_generated.rs");
    std::fs::write(&out_path, &formatted).expect("write lower_generated.rs");
    println!("wrote {}", out_path.display());
}
