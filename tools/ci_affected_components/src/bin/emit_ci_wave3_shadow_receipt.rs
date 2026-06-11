// Wave 3 §11.7.2 — live CI shadow receipt host transport (Phase 2 queued on bootstrap eval).
// Authority: `.github/workflows/ci.yml` (emit-ci-wave3-shadow-receipt step); does NOT call
// `ci_selection_receipt_shadow_from_git_diff`
// until `node://adhoc-331899f9-19a` lands. Shadow Class C: always exits 0.
#![allow(clippy::disallowed_macros)]

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use ci_affected_components::git_diff_transport::git_read_changed_paths_for_event;
use ci_affected_components::wave3_shadow_receipt::{
    build_shadow_emit, shadow_emit_to_json, shadow_status_log_line, write_receipt_json,
    EMIT_STEP_NAME,
};

fn usage() -> ! {
    eprintln!("usage: {EMIT_STEP_NAME} <event_name>\nevent_name: pull_request | push");
    std::process::exit(2);
}

fn receipt_output_path() -> PathBuf {
    env::var("RUNNER_TEMP")
        .map(|t| PathBuf::from(t).join("wave3-shadow-receipt.json"))
        .unwrap_or_else(|_| PathBuf::from("wave3-shadow-receipt.json"))
}

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let event_name = args.next().unwrap_or_else(|| usage());
    if args.next().is_some() {
        usage();
    }

    let git_read = git_read_changed_paths_for_event(event_name.as_str());
    if let ci_affected_components::git_diff_transport::GitChangedPathsRead::Ok {
        ref range,
        ref paths,
    } = git_read
    {
        eprintln!(
            "Wave 3 shadow receipt: git diff {range} ({} paths)",
            paths.len()
        );
        for path in paths {
            eprintln!("  {path}");
        }
    } else if let ci_affected_components::git_diff_transport::GitChangedPathsRead::FailClosed {
        ref detail,
        ..
    } = git_read
    {
        eprintln!("Wave 3 shadow receipt: {detail} — fail-closed component flags");
    }

    let emit = build_shadow_emit(event_name.as_str(), git_read);
    let out_path = receipt_output_path();
    if let Err(e) = write_receipt_json(
        out_path.to_str().unwrap_or("wave3-shadow-receipt.json"),
        &shadow_emit_to_json(&emit),
    ) {
        eprintln!("error: write receipt: {e}");
        return ExitCode::from(1);
    }
    eprintln!("wrote {}", out_path.display());
    println!("{}", shadow_status_log_line(&emit));
    ExitCode::SUCCESS
}
