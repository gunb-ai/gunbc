//! Evaluate `spice_rc_passive_deck_cir_text`, write the netlist to `deck.cir`, run `ngspice -b`.
//!
//! E-10 consumer for the P4 SPICE/analog ngspice oracle lane (SP-M1).

use std::process::ExitCode;

use emit_host_runner::{run_ngspice_batch_netlist, unique_work_dir};
use v2_compiler::cli_run::{
    build_multi_entry_index, make_eval_context, resolve_entry_with_index, run_value,
};
use v2_compiler::v2_interpreter::{free_monoid_to_vec, with_active_context, Value};

const FIXTURE_ENTRY: &str = "src/v4/test/fixture/spice_rc_passive_deck.dag";
const EMIT_FN: &str = "spice_rc_passive_deck_cir_text";

fn decode_freemonoid_string(val: &Value) -> Result<String, String> {
    let items =
        free_monoid_to_vec(val).ok_or_else(|| "expected String FreeMonoid netlist text".to_string())?;
    let mut out = String::new();
    for item in items {
        match item {
            Value::Int(codepoint) => {
                let ch = char::from_u32(codepoint as u32)
                    .ok_or_else(|| format!("invalid String codepoint {codepoint}"))?;
                out.push(ch);
            }
            _ => return Err("expected Int codepoints in String FreeMonoid".into()),
        }
    }
    Ok(out)
}

fn main() -> ExitCode {
    match run_oracle() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run_oracle() -> Result<(), String> {
    let roots = vec!["src/v4".to_string()];
    let index = build_multi_entry_index(&roots);
    let (graph, source_indices) =
        resolve_entry_with_index(&index, FIXTURE_ENTRY).map_err(|e| e.to_string())?;
    let ctx = make_eval_context(&graph, source_indices);
    let value = run_value(&ctx, EMIT_FN)?;
    let netlist_text = with_active_context(&ctx, || decode_freemonoid_string(&value))?;
    if netlist_text.is_empty() {
        return Err("emitted netlist text is empty".into());
    }

    let work_dir = unique_work_dir("gunbc_spice_ngspice_oracle");
    let exit = run_ngspice_batch_netlist(&netlist_text, &work_dir)
        .map_err(|e| format!("ngspice setup failed: {e:?}"))?;
    if exit.exit_holds() {
        Ok(())
    } else {
        Err(format!("ngspice -b failed: {:?}", exit.outcome))
    }
}
