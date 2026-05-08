//! §1.8 gate **`method_template_projection_emit_shim_retirement_coherence`** (ledger row **#97**).
//!
//! Q-V2-Retirement-Boundary-Matrix consumer: PB owns **`src/v2/`** deletion; Grounding owns retiring the
//! Gap-4 **`Map<String, String>` → ephemeral `.dag`** producer (**`pb_method_template_projection_dag_emit`**) and
//! the **`emit_method_template_projection`** Cargo bin (**Gunbc issue #1982**).
//!
//! **Pass condition.** Two coherence implications (Rust integration test fails closed on violation):
//!
//! - v2 compiler tree present ⇒ Gap-4 shim artifacts must remain (stage0/regen consumers still wired).
//! - v2 compiler tree absent ⇒ Gap-4 shim sources must not remain (forces Grounding completion after PB landing).
//!
//! At HEAD (`src/v2/stage0` still present) both hold; when PB deletes `src/v2/`, this test blocks until the
//! producer + bin are removed and census lists are updated.

use std::path::PathBuf;

#[test]
fn method_template_projection_emit_shim_retirement_coherence() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.join("..").join("..").join("..");
    assert!(
        workspace_root.join("Cargo.toml").is_file(),
        "expected workspace root at {}",
        workspace_root.display()
    );

    let v2_stage0 = workspace_root.join("src/v2/stage0");
    let shim_lib =
        workspace_root.join("src/v3/compiler/src/pb_method_template_projection_dag_emit.rs");
    let shim_bin =
        workspace_root.join("src/v3/compiler/src/bin/emit_method_template_projection.rs");

    let v2_present = v2_stage0.is_dir();
    let shim_lib_present = shim_lib.is_file();
    let shim_bin_present = shim_bin.is_file();

    assert!(
        !v2_present || (shim_lib_present && shim_bin_present),
        "§1.8 gate #97: with v2 tree at {}, Gap-4 emit shim must remain \
         (`pb_method_template_projection_dag_emit` + `emit_method_template_projection` bin); \
         saw shim_lib={shim_lib_present} shim_bin={shim_bin_present}",
        v2_stage0.display()
    );

    assert!(
        v2_present || (!shim_lib_present && !shim_bin_present),
        "§1.8 gate #97: v2 tree removed but Gap-4 emit shim still present — retire \
         `pb_method_template_projection_dag_emit.rs`, `emit_method_template_projection.rs`, \
         `lib.rs` export, and census entries per Gunbc #1982 (Grounding G6)"
    );
}
