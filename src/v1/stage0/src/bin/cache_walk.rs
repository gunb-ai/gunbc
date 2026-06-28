// SCAFFOLD: diagnostic bin for typed_module_cache retention measurement; delete once func_env.sigs
// Rc-sharing dedup lands. Dissolution trigger = func_env.sig sharing > 1.0× in corpus.
#![allow(clippy::disallowed_macros)]

use std::process::ExitCode;

use v1_compiler::cli_run::{build_multi_entry_index, resolve_entry_with_index};

fn usage() -> ! {
    eprintln!("cache_walk: measure MultiEntryIndex cache structure after resolving entries");
    eprintln!(
        "usage: cache_walk --source-root <dir> [--source-root <dir>...] --entry <file> [--entry <file>...]"
    );
    std::process::exit(1);
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut source_roots: Vec<String> = Vec::new();
    let mut entries: Vec<String> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--source-root" => {
                i += 1;
                if i >= args.len() {
                    usage();
                }
                source_roots.push(args[i].clone());
            }
            "--entry" => {
                i += 1;
                if i >= args.len() {
                    usage();
                }
                entries.push(args[i].clone());
            }
            _ => {
                eprintln!("unknown arg: {}", args[i]);
                usage();
            }
        }
        i += 1;
    }
    if source_roots.is_empty() || entries.is_empty() {
        usage();
    }

    eprintln!("cache_walk: building index for {:?}", source_roots);
    let index = build_multi_entry_index(&source_roots);

    for entry in &entries {
        eprintln!("cache_walk: resolving entry {entry}");
        match resolve_entry_with_index(&index, entry) {
            Err(e) => {
                eprintln!("cache_walk: resolve failed for {entry}: {e}");
                return ExitCode::FAILURE;
            }
            Ok(_) => {
                eprintln!("cache_walk: resolve OK ({} modules cached so far)", index.typed_module_count());
            }
        }
    }

    let report = index.measure_caches();
    println!("{}", report.format());

    ExitCode::SUCCESS
}
