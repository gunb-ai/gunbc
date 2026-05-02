//! DB-8 — fixed-point ratchet driver (`docs/design-fixed-point-ratchet.md`).
//!
//! Full cycle (emit → `rustc` → run → byte-diff) requires a v3-parseable
//! `dsl/gunbc/compiler.dag` **and** an emitted CLI that can re-run emission on
//! that file. Until Lane 3 Stage 3c, `compiler.dag` is a cycle meta-model that
//! does not yet parse under the v3 surface grammar; this binary still:
//! - proves the **pipeline snapshot** fixed point on [`v3_compiler::default_fixed_point_source`]
//!   (same contract as `regen_v3`, includes the emit stage);
//! - probes `compiler.dag`, emits `target/self_host/stage1.rs` when parse succeeds;
//! - attempts `rustc` when compilation is meaningful;
//! - writes `target/self_host/receipt.json` for trend monitoring (DB-8 §Open questions, Q3).
//!
//! Single authority for byte-identical emission across processes: [`v3_compiler::emit::emit`]
//! + determinism tests (`tests/determinism_test.rs`) — see `feedback_substrate_principle_audit` Q5.
//!
//! **Exit status (Invariant D-1):** when `compiler.dag` parses and the emit→`rustc`→run→diff slice
//! runs, any failure on that slice returns **`Err`** from [`run`] so the process exits **non-zero**
//! after writing `receipt.json`. Workflow policy (“do not block merges yet”) is enforced by
//! `continue-on-error` on the CI job/step — not by treating a failed slice as `Ok`.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::process::{Command, Stdio};

use v3_compiler::compile_to_dag;
use v3_compiler::emit::{emit, EmitTarget};
use v3_compiler::self_host_receipt_p0 as receipt_p0;
use v3_compiler::{
    compare_stage_snapshots, compile_stage_snapshots, default_fixed_point_source, CompileError,
};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
}

fn write_receipt(path: &Path, json: &str) {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(path, json);
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            let _ = writeln!(io::stderr(), "{e}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<(), String> {
    let root = workspace_root();
    let out_dir = root.join("target").join("self_host");
    fs::create_dir_all(&out_dir).map_err(|e| format!("create {}: {e}", out_dir.display()))?;

    let receipt_path = out_dir.join("receipt.json");

    // Staged ratchet (always): pipeline snapshots identical across two compiles of the default source.
    let pass1 = compile_stage_snapshots(default_fixed_point_source(), "fixed_point_input.v3")
        .map_err(|e| format!("pipeline snapshot pass1: {e:?}"))?;
    let pass2 = compile_stage_snapshots(default_fixed_point_source(), "fixed_point_input.v3")
        .map_err(|e| format!("pipeline snapshot pass2: {e:?}"))?;
    compare_stage_snapshots(&pass1, &pass2).map_err(|m| m.detail.clone())?;

    let compiler_rel = Path::new("dsl/gunbc/compiler.dag");
    let compiler_abs = root.join(compiler_rel);
    let compiler_dag_text = fs::read_to_string(&compiler_abs)
        .map_err(|e| format!("read {}: {e}", compiler_abs.display()))?;

    let compiler_parse = match compile_to_dag(&compiler_dag_text, "dsl/gunbc/compiler.dag") {
        Ok(dag) => Ok(dag),
        Err(CompileError::Semantic(d)) => Err(format!(
            "compiler.dag semantic errors: {:?}",
            d.diagnostics()
        )),
        Err(e) => Err(format!("compiler.dag: {e:?}")),
    };

    // When `compiler.dag` parses and we exercise emit→rustc→run→diff, any failure on that
    // slice must exit non-zero (Invariant D-1 / DB-8 fail-closed). Parse failure alone stays
    // exit 0 — expected until v3 grammar + Lane 1e land (staged ratchet).
    let mut self_host_slice_failed: Option<String> = None;

    // P0 / DB-8 `receipt.json` always-emitted keys — checked before write by
    // `receipt_p0::validate_receipt_json_always_emitted_keys` / `top_level_property_needle`
    // (two spaces + `"key":`). Keep these three emission anchors aligned with that helper:
    // (1) pipeline field right after `{`, (2) `K_COMPILER_DAG_V3_PARSE` in `Ok` + `Err` arms,
    // (3) `K_STATUS` on the closing field before `}`.
    let mut receipt = String::new();
    receipt.push_str("{\n");
    receipt.push_str(&format!(
        "  \"{}\": \"ok\",\n",
        receipt_p0::K_PIPELINE_FIXED_POINT_DEFAULT_SOURCE
    ));

    match compiler_parse {
        Ok(dag) => {
            receipt.push_str(&format!(
                "  \"{}\": \"ok\",\n",
                receipt_p0::K_COMPILER_DAG_V3_PARSE
            ));
            let stage1 =
                emit(&dag, EmitTarget::Rust).map_err(|e| format!("emit compiler.dag: {e:?}"))?;
            let stage1_path = out_dir.join("stage1.rs");
            fs::write(&stage1_path, &stage1.text)
                .map_err(|e| format!("write {}: {e}", stage1_path.display()))?;
            receipt.push_str(&format!("  \"stage1_rs_bytes\": {},\n", stage1.text.len()));

            let bin_path = out_dir.join("stage1_bin");
            let rustc_inv = Command::new("rustc")
                .arg("--edition=2021")
                .arg(&stage1_path)
                .arg("-o")
                .arg(&bin_path)
                .stderr(Stdio::piped())
                .output()
                .map_err(|e| format!("rustc: {e}"))?;

            if rustc_inv.status.success() {
                receipt.push_str("  \"stage1_rustc\": \"ok\",\n");
                let run = Command::new(&bin_path)
                    .arg(&compiler_abs)
                    .output()
                    .map_err(|e| format!("run stage1_bin: {e}"))?;
                if run.status.success() {
                    let stage2_path = out_dir.join("stage2.rs");
                    receipt.push_str(&format!(
                        "  \"self_host_run_stderr_len\": {},\n",
                        run.stderr.len()
                    ));
                    if fs::read(&stage2_path).is_ok() {
                        let a = fs::read(&stage1_path).map_err(|e| e.to_string())?;
                        let b = fs::read(&stage2_path).map_err(|e| e.to_string())?;
                        if a == b {
                            receipt.push_str("  \"fixed_point_diff\": \"ok\",\n");
                        } else {
                            receipt.push_str("  \"fixed_point_diff\": \"mismatch\",\n");
                            self_host_slice_failed = Some(
                                "fixed-point: stage1.rs bytes != stage2.rs (Invariant D-1)"
                                    .to_string(),
                            );
                        }
                    } else {
                        receipt
                            .push_str("  \"fixed_point_diff\": \"skipped_stage2_not_written\",\n");
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&run.stderr);
                    receipt.push_str(&format!(
                        "  \"self_host_run\": {},\n",
                        json_string(&format!("failed: {stderr}"))
                    ));
                    self_host_slice_failed =
                        Some(format!("self_host_run: stage1_bin failed: {stderr}"));
                }
            } else {
                let stderr = String::from_utf8_lossy(&rustc_inv.stderr);
                receipt.push_str(&format!(
                    "  \"stage1_rustc\": {},\n",
                    json_string(&format!("failed: {stderr}"))
                ));
                self_host_slice_failed = Some(format!("rustc failed on stage1.rs: {stderr}"));
            }
        }
        Err(msg) => {
            receipt.push_str(&format!(
                "  \"{}\": {},\n",
                receipt_p0::K_COMPILER_DAG_V3_PARSE,
                json_string(&msg)
            ));
        }
    }

    let exit_status = if self_host_slice_failed.is_some() {
        "failed_self_host_slice"
    } else {
        "completed"
    };
    receipt.push_str(&format!(
        "  \"{}\": {}\n}}\n",
        receipt_p0::K_STATUS,
        json_string(exit_status)
    ));

    // Must match the three anchors documented on `self_host_receipt_p0::top_level_property_needle`.
    receipt_p0::validate_receipt_json_always_emitted_keys(&receipt).map_err(|e| {
        format!("self_host_fixed_point: receipt contract (P0 always-emitted keys): {e}")
    })?;

    write_receipt(&receipt_path, &receipt);
    writeln!(
        io::stdout(),
        "self_host_fixed_point: receipt -> {}",
        receipt_path.display()
    )
    .map_err(|e| e.to_string())?;

    if let Some(detail) = self_host_slice_failed {
        let _ = writeln!(io::stderr(), "{detail}");
        return Err(detail);
    }
    Ok(())
}

fn json_string(s: &str) -> String {
    let mut out = String::new();
    out.push('"');
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
