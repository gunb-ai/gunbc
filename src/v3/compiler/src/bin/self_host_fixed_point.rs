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

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::process::{Command, Stdio};

use v3_compiler::compile_to_dag;
use v3_compiler::emit::{emit, EmitTarget};
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

    let mut receipt = String::new();
    receipt.push_str("{\n");
    receipt.push_str("  \"pipeline_fixed_point_default_source\": \"ok\",\n");

    match compiler_parse {
        Ok(dag) => {
            receipt.push_str("  \"compiler_dag_v3_parse\": \"ok\",\n");
            let stage1 =
                emit(&dag, EmitTarget::Rust).map_err(|e| format!("emit compiler.dag: {e:?}"))?;
            let stage1_path = out_dir.join("stage1.rs");
            fs::write(&stage1_path, &stage1.text)
                .map_err(|e| format!("write {}: {e}", stage1_path.display()))?;
            receipt.push_str(&format!("  \"stage1_rs_bytes\": {},\n", stage1.text.len()));

            let bin_path = out_dir.join("stage1_bin");
            let rustc_status = Command::new("rustc")
                .arg("--edition=2021")
                .arg(&stage1_path)
                .arg("-o")
                .arg(&bin_path)
                .stderr(Stdio::piped())
                .status()
                .map_err(|e| format!("rustc: {e}"))?;

            if rustc_status.success() {
                receipt.push_str("  \"stage1_rustc\": \"ok\",\n");
                let run = Command::new(&bin_path)
                    .arg(&compiler_abs)
                    .output()
                    .map_err(|e| format!("run stage1_bin: {e}"))?;
                if run.status.success() {
                    let stage2_path = out_dir.join("stage2.rs");
                    // If the binary wrote nowhere predictable, record stderr only.
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
                        }
                    } else {
                        receipt
                            .push_str("  \"fixed_point_diff\": \"skipped_stage2_not_written\",\n");
                    }
                } else {
                    receipt.push_str(&format!(
                        "  \"self_host_run\": \"failed: {}\",\n",
                        String::from_utf8_lossy(&run.stderr).escape_default()
                    ));
                }
            } else {
                receipt.push_str("  \"stage1_rustc\": \"failed\",\n");
            }
        }
        Err(msg) => {
            receipt.push_str(&format!(
                "  \"compiler_dag_v3_parse\": {},\n",
                json_string(&msg)
            ));
        }
    }

    receipt.push_str("  \"status\": \"completed\"\n}\n");
    write_receipt(&receipt_path, &receipt);
    writeln!(
        io::stdout(),
        "self_host_fixed_point: receipt -> {}",
        receipt_path.display()
    )
    .map_err(|e| e.to_string())?;
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
