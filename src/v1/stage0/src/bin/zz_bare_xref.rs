//! BRANCH-LOCAL EXPERIMENT — deleted before merge.
//!
//! THE MIGRATION DENOMINATOR, which unresolved-name diagnostics cannot supply.
//!
//! The cut's terminal rule is that every cross-module reference is authored as
//! `container.member`. Diagnostics only surface references that FAIL to
//! resolve; today's resolver is whole-pool, so a bare cross-module name
//! resolves silently whenever the pool holds exactly one declarer. Those sites
//! are unqualified, invisible to the diagnostic census, and still owed.
//!
//! For each module: every referenced name that is NOT declared locally and is
//! not a kernel type, classified by how many modules declare it.
//!
//!   BARE_UNIQUE   exactly one other declarer -> resolves today, still owed
//!   BARE_MULTI    several declarers -> ambiguous
//!   BARE_NONE     no declarer anywhere -> broken reference
//!
//! Prints one TSV row per site-name plus a summary.
#![allow(clippy::disallowed_macros)]

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::ExitCode;

use v1_compiler::cli_run::workspace_root;
use v1_compiler::source_closure::{
    build_declaration_index, dag_files_under, declared_names, parse_file, referenced_names,
};

const KERNEL: &[&str] = &[
    "String", "Int", "Bool", "Float", "Secret", "Json", "Unit", "Bytes", "None",
];

fn main() -> ExitCode {
    let ws = workspace_root();
    let roots: Vec<PathBuf> = vec![ws.join("dag"), ws.join("src/v2")];
    let (index, unparsed) = build_declaration_index(&roots, &ws);
    if !unparsed.is_empty() {
        eprintln!("UNPARSED {} {:?}", unparsed.len(), unparsed);
    }

    let mut uniq = 0usize;
    let mut multi = 0usize;
    let mut none = 0usize;
    let mut by_name: BTreeMap<String, usize> = BTreeMap::new();

    for file_path in dag_files_under(&roots) {
        let Ok(content) = std::fs::read_to_string(&file_path) else {
            continue;
        };
        let rel = file_path
            .strip_prefix(&ws)
            .unwrap_or(&file_path)
            .to_string_lossy()
            .to_string();
        let Some(module) = parse_file(&rel, &content) else {
            continue;
        };
        let local: std::collections::HashSet<String> =
            declared_names(&module).into_iter().collect();
        for name in referenced_names(&module) {
            if local.contains(&name) || KERNEL.contains(&name.as_str()) || name.contains('.') {
                continue;
            }
            match index.modules_declaring(&name) {
                Some(mods) => {
                    let others: Vec<&String> = mods.iter().collect();
                    if others.len() == 1 {
                        uniq += 1;
                        *by_name.entry(name.clone()).or_default() += 1;
                        println!("BARE_UNIQUE\t{rel}\t{name}\t{}", others[0]);
                    } else {
                        multi += 1;
                        println!("BARE_MULTI\t{rel}\t{name}\t{}", others.len());
                    }
                }
                None => {
                    none += 1;
                    println!("BARE_NONE\t{rel}\t{name}");
                }
            }
        }
    }

    println!("SUMMARY_BARE_UNIQUE {uniq}");
    println!("SUMMARY_BARE_MULTI {multi}");
    println!("SUMMARY_BARE_NONE {none}");
    println!("SUMMARY_TOTAL {}", uniq + multi + none);
    let mut ranked: Vec<(&String, &usize)> = by_name.iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(a.1));
    for (n, c) in ranked.iter().take(25) {
        println!("TOPNAME\t{c}\t{n}");
    }
    ExitCode::SUCCESS
}
