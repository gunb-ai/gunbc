#![allow(clippy::disallowed_macros)]

use std::process::ExitCode;

use v1_compiler::cli_run::{build_multi_entry_index, resolve_entry_with_index};

fn usage() -> ! {
    eprintln!("cache_walk: measure MultiEntryIndex cache structure after resolving an entry");
    eprintln!("usage: cache_walk --source-root <dir> [--source-root <dir>...] --entry <file>");
    std::process::exit(1);
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut source_roots: Vec<String> = Vec::new();
    let mut entry: Option<String> = None;
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
                entry = Some(args[i].clone());
            }
            _ => {
                eprintln!("unknown arg: {}", args[i]);
                usage();
            }
        }
        i += 1;
    }
    if source_roots.is_empty() || entry.is_none() {
        usage();
    }
    let entry = entry.unwrap();

    eprintln!("cache_walk: building index for {:?}", source_roots);
    let index = build_multi_entry_index(&source_roots);

    eprintln!("cache_walk: resolving entry {entry}");
    match resolve_entry_with_index(&index, &entry) {
        Err(e) => {
            eprintln!("cache_walk: resolve failed: {e}");
            return ExitCode::FAILURE;
        }
        Ok(_) => {
            eprintln!("cache_walk: resolve OK");
        }
    }

    let report = index.measure_caches();
    println!("{}", report.format());

    ExitCode::SUCCESS
}
