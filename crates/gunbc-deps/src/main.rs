use std::env;

use gunbc_exec::{execute, ExecError, Value};
use gunbc_ir::viz::dag_to_svg;
use gunbc_validate::{validate_acyclic, validate_port_saturation, validate_types};

use gunbc_deps::{build_graph, subdag_for_entry, Mode};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_usage();
        return;
    }

    let list = args.iter().any(|a| a == "--list");
    let svg = args.iter().any(|a| a == "--svg");

    let mode = match value_for(&args, "--mode") {
        Some(v) => match v.as_str() {
            "check" => Mode::Check,
            "upsert" => Mode::Upsert,
            other => exit_err(&format!("unknown mode '{other}'")),
        },
        None => Mode::Check,
    };

    let graph = build_graph(mode);

    if list {
        for entry in &graph.entries {
            println!("{entry}");
        }
        return;
    }

    let entry = match value_for(&args, "--entry") {
        Some(v) => v,
        None => exit_err("missing --entry"),
    };

    let dag = match subdag_for_entry(&graph.dag, &entry) {
        Ok(dag) => dag,
        Err(e) => exit_err(&e),
    };

    if svg {
        println!("{}", dag_to_svg(&dag, true));
        return;
    }

    if let Err(e) = validate_acyclic(&dag) {
        exit_err(&format!("acyclic validation failed: {e}"));
    }
    if let Err(e) = validate_types(&dag) {
        exit_err(&format!("type validation failed: {e}"));
    }
    if let Err(e) = validate_port_saturation(&dag) {
        exit_err(&format!("port saturation failed: {e}"));
    }

    match execute(&dag) {
        Ok(log) => {
            let ok = log
                .entries
                .iter()
                .find(|entry_log| entry_log.node_id == entry)
                .and_then(|entry_log| entry_log.outputs.get("ok"));
            match ok {
                Some(Value::Bool(true)) => {
                    println!("{entry}: ok");
                }
                Some(other) => {
                    exit_err(&format!("{entry}: unexpected result {other}"));
                }
                None => {
                    exit_err(&format!("{entry}: missing ok output"));
                }
            }
        }
        Err(ExecError(msg)) => {
            exit_err(&msg);
        }
    }
}

fn value_for(args: &[String], flag: &str) -> Option<String> {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == flag {
            return iter.next().cloned();
        }
    }
    None
}

fn print_usage() {
    println!("usage: gunbc-deps --entry <name> [--mode check|upsert] [--svg] [--list]");
}

fn exit_err(msg: &str) -> ! {
    eprintln!("{msg}");
    std::process::exit(1);
}
