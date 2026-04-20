//! Regenerate `lower_generated.rs` from the canonical hand-maintained
//! `src/v3/compiler/src/lower.rs` (the `lower` module).
//!
//! **SG-3f-prep (Option B):** `lower.rs` remains on the SG-0 census as the real
//! lowering implementation. This binary is a pass-through + header + rustfmt —
//! it does not dissolve the algorithm into `.dag`. The emitted
//! `lower_generated.rs` is **not** imported by `lib.rs`; it exists so the
//! regen + snapshot test harness stay wired for a future `lower.dag` cutover
//! after substrate reflection (`Surface*` in `substrate.dag`, SELF_HOSTING.md §4).
//! Do not treat `lower_generated.rs` as the live authority until that migration lands.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use v3_compiler::generated_files::GENERATED_FILES;

const GENERATED_FILE: &str = "src/v3/compiler/src/lower_generated.rs";
/// Canonical lowering source (hand-maintained; counted by SG-0).
const LOWER_RS: &str = "src/lower.rs";

const HEADER: &str = "// AUTO-GENERATED from `src/v3/compiler/src/lower.rs` via\n\
     // `regen_lower`. Regenerate instead of hand-editing.\n\
     //\n\
     // SG-3f-prep: not wired into `lib.rs` — canonical implementation is\n\
     // `lower.rs` pending `lower.dag` + reflected `Surface*` (SELF_HOSTING.md §4).\n\
     \n";

fn main() {
    assert!(
        GENERATED_FILES.contains(&GENERATED_FILE),
        "`regen_lower` writes `{GENERATED_FILE}` but that path is not \
         registered in `REGEN_OUTPUTS` in `src/v3/compiler/build.rs`."
    );

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let lower_path = manifest_dir.join(LOWER_RS);
    let body = std::fs::read_to_string(&lower_path).unwrap_or_else(|e| {
        panic!(
            "read canonical lowering source `{}`: {e}",
            lower_path.display()
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
