// Host transport for `v4.workflow.ci` `ci_component_affected_from_git_diff_read` (T-24).
// Replaces scripts/detect-affected-components.sh. Modeled authority is `src/v4/workflow/ci.dag`;
// this bin executes the Rust mirror in `ci_affected_components` (parity-ratcheted to ci.dag).
// Crate lives outside v3-compiler so affected-set gating does not require compiling v3 first.
#![allow(clippy::disallowed_macros)]

use std::io::Write;
use std::process::ExitCode;
use std::{env, fs, io};

use ci_affected_components::git_diff_transport::{
    git_read_changed_paths_for_event, GitChangedPathsRead,
};
use ci_affected_components::{
    ci_component_affected_fail_closed, ci_component_affected_from_changed_paths,
    CiComponentAffected,
};

fn usage() -> ! {
    eprintln!(
        "usage: detect-ci-affected-components <event_name> <output_file>\n\
         event_name: pull_request | push\n\
         output_file: GitHub Actions GITHUB_OUTPUT path"
    );
    std::process::exit(2);
}

fn write_github_output(path: &str, flags: CiComponentAffected) -> io::Result<()> {
    let mut file = fs::OpenOptions::new().append(true).open(path)?;
    writeln!(file, "v2={}", flags.v2)?;
    writeln!(file, "v3={}", flags.v3)?;
    writeln!(file, "v4={}", flags.v4)?;
    writeln!(file, "testclaim_corpus={}", flags.testclaim_corpus)?;
    writeln!(file, "workflow_policy={}", flags.workflow_policy)?;
    writeln!(file, "release_distribution={}", flags.release_distribution)?;
    writeln!(
        file,
        "release_distribution_only={}",
        flags.release_distribution_only
    )?;
    Ok(())
}

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let event_name = args.next().unwrap_or_else(|| usage());
    let output_file = args.next().unwrap_or_else(|| usage());
    if args.next().is_some() {
        usage();
    }

    // Single git-diff transport authority shared with the Wave 3 shadow receipt emitter
    // (`git_diff_transport`) so the gate output and the receipt it shadows cannot drift.
    let flags = match git_read_changed_paths_for_event(event_name.as_str()) {
        GitChangedPathsRead::Ok { range, paths } => {
            eprintln!("Changed files in {range}:");
            if paths.is_empty() {
                eprintln!("  (none detected)");
            } else {
                for path in &paths {
                    eprintln!("  {path}");
                }
            }
            eprintln!();
            ci_component_affected_from_changed_paths(paths.iter().map(String::as_str))
        }
        GitChangedPathsRead::FailClosed { range, detail } => {
            eprintln!("Changed files in {range}: (read failed — fail-closed superset)");
            eprintln!("  {detail}");
            eprintln!();
            ci_component_affected_fail_closed()
        }
    };

    eprintln!(
        "v2 affected: {}",
        if flags.v2 {
            "yes"
        } else {
            "no (skipping v2 fixed-point)"
        }
    );
    eprintln!(
        "v3 affected: {}",
        if flags.v3 {
            "yes"
        } else {
            "no (skipping v3 CI per freeze 2026-05-15)"
        }
    );
    eprintln!(
        "v4 affected: {}",
        if flags.v4 {
            "yes (running v2→v4 bootstrap viability test)"
        } else {
            "no (skipping v4 bootstrap test)"
        }
    );
    eprintln!(
        "testclaim_corpus (T-22 corpus bridge): {}",
        if flags.testclaim_corpus { "yes" } else { "no" }
    );
    eprintln!(
        "workflow_policy (Gate #103 surface): {}",
        if flags.workflow_policy { "yes" } else { "no" }
    );
    eprintln!(
        "release_distribution (RELEASE §5 parity smoke): {}",
        if flags.release_distribution {
            "yes"
        } else {
            "no"
        }
    );
    eprintln!(
        "release_distribution_only (RELEASE §5 skip phase1 rung): {}",
        if flags.release_distribution_only {
            "yes"
        } else {
            "no"
        }
    );

    if let Err(e) = write_github_output(&output_file, flags) {
        eprintln!("error: write {output_file}: {e}");
        return ExitCode::from(1);
    }

    ExitCode::SUCCESS
}
