//! CLI for generic seed-shim closure assembly (curated self-host harness).
#![allow(clippy::disallowed_macros)]

use clap::Parser;
use std::path::PathBuf;

#[path = "../cssl_seed_linked_closure_assembly.rs"]
mod cssl_seed_linked_closure_assembly;

use cssl_seed_linked_closure_assembly::{assemble_seed_linked_closure, AssemblyError};

#[derive(Parser)]
#[command(name = "cssl_assemble")]
struct Args {
    #[arg(long)]
    out_dir: PathBuf,
    #[arg(long)]
    entry_dag: PathBuf,
    #[arg(long)]
    root: PathBuf,
}

fn main() {
    let args = Args::parse();
    let entry_dag = if args.entry_dag.is_absolute() {
        args.entry_dag
    } else {
        args.root.join(args.entry_dag)
    };
    match assemble_seed_linked_closure(&args.out_dir, &entry_dag) {
        Ok(()) => {
            println!("CSSL_ASSEMBLE: PASS");
        }
        Err(e) => {
            eprintln!("CSSL_ASSEMBLE: REFUSED {e}");
            std::process::exit(match e {
                AssemblyError::RefusedDeclaredMember { .. } => 2,
                _ => 1,
            });
        }
    }
}
