//! CLI for generic seed-shim closure assembly (curated self-host harness).

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
    #[arg(long, default_value = "dag/tools/self_host_std_bridge_shims")]
    std_bridge_dir: PathBuf,
}

fn main() {
    let args = Args::parse();
    let std_bridge = if args.std_bridge_dir.is_absolute() {
        args.std_bridge_dir
    } else {
        args.root.join(args.std_bridge_dir)
    };
    let entry_dag = if args.entry_dag.is_absolute() {
        args.entry_dag
    } else {
        args.root.join(args.entry_dag)
    };
    match assemble_seed_linked_closure(&args.out_dir, &entry_dag, &args.root, &std_bridge) {
        Ok(()) => {
            println!("CSSL_ASSEMBLE: PASS");
        }
        Err(e) => {
            eprintln!("CSSL_ASSEMBLE: REFUSED {e}");
            std::process::exit(match e {
                AssemblyError::RefusedDep { .. } => 2,
                _ => 1,
            });
        }
    }
}
