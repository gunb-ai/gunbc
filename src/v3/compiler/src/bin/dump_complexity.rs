// Exploratory dump: apply the complexity lens to every named bind in the
// bootstrap Dag (the entire compiler pipeline + std + lenses) and print a
// summary report.
//
// Usage:
//   cargo run -p v3-compiler --bin dump_complexity
//     → class-distribution histogram across the whole bootstrap Dag.
//   cargo run -p v3-compiler --bin dump_complexity -- --file pipeline.dag
//     → per-bind detail for any file whose path ends with the given suffix.
//   cargo run -p v3-compiler --bin dump_complexity -- --file lenses/complexity.dag --limit 30

use std::collections::BTreeMap;
use std::env;
use std::process::ExitCode;

use v3_compiler::dag::{AsymptoticClass, Behavior, Dag, SymbolicCost};
use v3_compiler::lens_cost::{complexity_of, Certainty, ComplexityLookup, ComplexitySummary};

fn main() -> ExitCode {
    let mut filter: Option<String> = None;
    let mut limit: Option<usize> = None;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--file" => filter = args.next(),
            "--limit" => {
                limit = args.next().and_then(|v| v.parse::<usize>().ok());
            }
            other => {
                eprintln!("unknown argument `{other}`");
                return ExitCode::FAILURE;
            }
        }
    }

    let dag = Dag::new();
    println!(
        "bootstrap Dag: {} nodes, {} declarations, {} diagnostics",
        dag.nodes().len(),
        dag.declarations().len(),
        dag.diagnostics().len()
    );

    // File histogram across all binds (so caller can see what's actually present
    // in the nodes table and pick a useful --file filter).
    if filter.is_none() {
        let mut file_hist: BTreeMap<String, usize> = BTreeMap::new();
        for node in dag.nodes() {
            if let Some(bind) = node.as_bind() {
                let short = bind
                    .span
                    .file
                    .rsplit_once("/src/v3/")
                    .map(|(_, s)| format!("v3/{s}"))
                    .unwrap_or_else(|| bind.span.file.clone());
                *file_hist.entry(short).or_default() += 1;
            }
        }
        println!("\nbind-count by source file (top 20):");
        let mut by_count: Vec<(&String, &usize)> = file_hist.iter().collect();
        by_count.sort_by_key(|(_, c)| std::cmp::Reverse(**c));
        for (file, count) in by_count.iter().take(20) {
            println!("  {:>5}  {}", count, file);
        }
        println!();
    }

    let mut class_hist: BTreeMap<String, usize> = BTreeMap::new();
    let mut certainty_hist: BTreeMap<String, usize> = BTreeMap::new();
    let mut miss_count = 0usize;
    let mut hit_count = 0usize;
    let mut shown = 0usize;

    if filter.is_none() {
        println!("(no --file filter — skipping per-bind complexity_of; pass --file <suffix> to dump)");
        return ExitCode::SUCCESS;
    }
    for node in dag.nodes() {
        let Some(bind) = node.as_bind() else { continue };
        if let Some(ref suffix) = filter {
            if !bind.span.file.ends_with(suffix) {
                continue;
            }
        }
        match complexity_of(&dag, &bind.value) {
            ComplexityLookup::Hit(summary) => {
                hit_count += 1;
                *class_hist
                    .entry(format!("{:?}", summary.asymptotic_class))
                    .or_default() += 1;
                let cert_key = format!(
                    "work={} span={}",
                    cert_str(&summary.work_certainty),
                    cert_str(&summary.span_certainty)
                );
                *certainty_hist.entry(cert_key).or_default() += 1;
                if filter.is_some() {
                    if let Some(cap) = limit {
                        if shown >= cap {
                            continue;
                        }
                    }
                    print_bind(&bind.span.file, &bind.name, &summary);
                    shown += 1;
                }
            }
            ComplexityLookup::Miss => {
                miss_count += 1;
            }
        }
    }

    println!();
    println!(
        "=== summary ({}{}) ===",
        if let Some(f) = filter.as_ref() {
            format!("filter=**/{f} ")
        } else {
            "all binds ".into()
        },
        format!("hits={hit_count} miss={miss_count}")
    );
    println!("\nasymptotic class distribution:");
    for (class, count) in &class_hist {
        println!("  {:<18} {}", class, count);
    }
    println!("\ncertainty distribution:");
    for (cert, count) in &certainty_hist {
        println!("  {:<32} {}", cert, count);
    }

    ExitCode::SUCCESS
}

fn cert_str(c: &Certainty) -> &'static str {
    match c {
        Certainty::Proven => "Proven",
        Certainty::Conservative => "Conservative",
    }
}

fn print_bind(file: &str, name: &str, summary: &ComplexitySummary) {
    let short_file = file.rsplit_once('/').map(|(_, s)| s).unwrap_or(file);
    println!(
        "  {:<28} {:<48} class={:<16} work={:<32} span={:<28} cert={}/{}",
        short_file,
        truncate(name, 48),
        class_label(&summary.asymptotic_class),
        truncate(&cost_label(&summary.work), 32),
        truncate(&cost_label(&summary.span), 28),
        cert_str(&summary.work_certainty),
        cert_str(&summary.span_certainty)
    );
}

fn truncate(s: &str, n: usize) -> String {
    if s.len() <= n {
        s.to_string()
    } else {
        format!("{}...", &s[..n.saturating_sub(3)])
    }
}

fn class_label(c: &AsymptoticClass) -> String {
    match c {
        AsymptoticClass::ClassConstant => "Constant".into(),
        AsymptoticClass::ClassLog => "Log".into(),
        AsymptoticClass::ClassLinear => "Linear".into(),
        AsymptoticClass::ClassLinearithmic => "Linearithmic".into(),
        AsymptoticClass::ClassQuadratic => "Quadratic".into(),
        AsymptoticClass::ClassPolynomial { degree } => format!("Poly^{:?}", degree),
        AsymptoticClass::ClassExponential => "Exponential".into(),
        AsymptoticClass::ClassUnknown => "Unknown".into(),
    }
}

fn cost_label(c: &SymbolicCost) -> String {
    match c {
        SymbolicCost::ConstantCost { _0 } => format!("Const({})", _0),
        SymbolicCost::LinearCost { _0 } => format!(
            "Linear({})",
            _0.display_name.clone().unwrap_or_else(|| "?".into())
        ),
        SymbolicCost::PolynomialCost { var, degree } => format!(
            "Poly({}^{:?})",
            var.display_name.clone().unwrap_or_else(|| "?".into()),
            degree
        ),
        SymbolicCost::LogCost { _0 } => format!(
            "Log({})",
            _0.display_name.clone().unwrap_or_else(|| "?".into())
        ),
        SymbolicCost::ProductCost { _0 } => format!("Product[len={}]", _0.len()),
        SymbolicCost::SumCost { _0 } => format!("Sum[len={}]", _0.len()),
        SymbolicCost::UnknownCost { _0 } => format!("Unknown({})", _0),
    }
}
