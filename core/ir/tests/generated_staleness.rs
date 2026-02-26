//! Staleness check: ensures `core/ir/src/generated/mod.rs` matches
//! what `daglang gen-types` would produce from the current DSL files.
//!
//! If this test fails, regenerate with:
//!   cargo run -p daglang-cli -- gen-types dsl/std \
//!     --module std.symbols --module std.unicode --module std.width \
//!     --module std.render --module std.box_draw \
//!     --output core/ir/src/generated/mod.rs

#[allow(clippy::disallowed_methods)] // Test infrastructure: needs Command::new and fs::read_to_string
#[test]
fn generated_types_are_not_stale() {
    let output = std::process::Command::new(env!("CARGO"))
        .args([
            "run",
            "-p",
            "daglang-cli",
            "--",
            "gen-types",
            "dsl/std",
            "--module",
            "std.symbols",
            "--module",
            "std.unicode",
            "--module",
            "std.width",
            "--module",
            "std.render",
            "--module",
            "std.box_draw",
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR").to_string() + "/../..")
        .output()
        .expect("failed to run daglang gen-types");

    assert!(
        output.status.success(),
        "gen-types failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let fresh = String::from_utf8(output.stdout).expect("gen-types produced non-UTF-8");
    let on_disk =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/generated/mod.rs"))
            .expect("could not read generated/mod.rs");

    if fresh != on_disk {
        panic!(
            "core/ir/src/generated/mod.rs is stale. Regenerate with:\n  \
             cargo run -p daglang-cli -- gen-types dsl/std \
             --module std.symbols --module std.unicode --module std.width \
             --module std.render --module std.box_draw \
             --output core/ir/src/generated/mod.rs"
        );
    }
}
