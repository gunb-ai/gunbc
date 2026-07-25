// Scratch census: shell-transport operations, their declared inputs, and the argv
// expression shapes the seed materializer can bind today. Not shipped.
use std::collections::BTreeMap;
use std::rc::Rc;
use v1_compiler::cli_run::parse_extdeps_module_items;
use v1_compiler::v1_std_core::{expr_var_name_at, param_node_name_at, ExprData, LiteralValue, Node};

fn walk_dag_files(root: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(rd) = std::fs::read_dir(root) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            walk_dag_files(&p, out);
        } else if p.extension().and_then(|s| s.to_str()) == Some("dag") {
            out.push(p);
        }
    }
}

#[derive(Default, Debug)]
struct ExprShapes {
    literal: usize,
    var: usize,
    interp: usize,
    other: BTreeMap<String, usize>,
}

fn classify(
    node: &Rc<Node>,
    si: &Rc<im::HashMap<String, Rc<v1_compiler::v1_std_core::NewlineIndex>>>,
    refs: &mut Vec<String>,
    shapes: &mut ExprShapes,
) {
    match node.expr_data.as_ref() {
        ExprData::ExprLiteral { value } => match value.as_ref() {
            LiteralValue::LitStr { .. } => shapes.literal += 1,
            other => {
                *shapes
                    .other
                    .entry(format!("non-str-literal {other:?}"))
                    .or_default() += 1
            }
        },
        ExprData::ExprVar { .. } => {
            shapes.var += 1;
            refs.push(expr_var_name_at(node.clone(), si.clone()));
        }
        ExprData::ExprStringInterp => {
            shapes.interp += 1;
            for part in
                v1_compiler::v1_compiler_emit::extract_string_interp_parts(node.clone()).iter()
            {
                if let v1_compiler::v1_std_core::StringPart::Interpolation { expr } = part.as_ref() {
                    classify(expr, si, refs, shapes);
                }
            }
        }
        other => {
            let tag = format!("{other:?}");
            let tag = tag.split(&[' ', '{'][..]).next().unwrap_or("?").to_string();
            *shapes.other.entry(tag).or_default() += 1;
        }
    }
}

fn main() {
    let root = std::path::Path::new(".");
    let mut files = Vec::new();
    for sub in ["dag", "src/v2"] {
        walk_dag_files(&root.join(sub), &mut files);
    }
    files.sort();

    let hardcoded: Vec<&str> = vec!["package", "bin", "args", "unit", "property"];

    let mut total_ops = 0usize;
    let mut shell_ops = 0usize;
    let mut bindable_today = 0usize;
    let mut unbindable: Vec<(String, String, String, Vec<String>)> = Vec::new();
    let mut unsupported_expr: Vec<(String, String, String, ExprShapes)> = Vec::new();
    let mut argv0_nonliteral: Vec<(String, String, String)> = Vec::new();
    let mut all_shapes = ExprShapes::default();

    for f in &files {
        let rel = f.strip_prefix(root).unwrap().to_string_lossy().to_string();
        let rel = rel.trim_start_matches("./").to_string();
        let text = std::fs::read_to_string(f).unwrap_or_default();
        if !text.contains("transport shell") && !text.contains("transport shell") {
            if !text.contains("argv:") {
                continue;
            }
        }
        let res = std::panic::catch_unwind(|| parse_extdeps_module_items(&rel));
        let Ok((items, si)) = res else {
            continue;
        };
        for item in items.iter() {
            // service node: children are operations
            if item.children.is_empty() {
                continue;
            }
            let service_name = item.name.clone();
            for op in item.children.iter() {
                let Some(transport) = op.transport.clone().or_else(|| item.transport.clone()) else {
                    continue;
                };
                // shell transport: children are argv nodes and body marker present
                if transport.children.is_empty() {
                    continue;
                }
                total_ops += 1;
                let argv: Vec<Rc<Node>> = transport.children.iter().cloned().collect();
                // does it look like a shell argv? first child literal string
                let is_shell = true;
                if !is_shell {
                    continue;
                }
                shell_ops += 1;
                let declared: Vec<String> = op
                    .params
                    .iter()
                    .map(|p| param_node_name_at(p.clone(), si.clone()))
                    .collect();
                let mut refs: Vec<String> = Vec::new();
                let mut shapes = ExprShapes::default();
                for a in argv.iter() {
                    classify(a, &si, &mut refs, &mut shapes);
                }
                // argv0 literal?
                if let Some(a0) = argv.first() {
                    let lit = matches!(a0.expr_data.as_ref(), ExprData::ExprLiteral { value } if matches!(value.as_ref(), LiteralValue::LitStr{..}));
                    if !lit {
                        argv0_nonliteral.push((
                            rel.clone(),
                            service_name.clone(),
                            op.name.clone(),
                        ));
                    }
                }
                all_shapes.literal += shapes.literal;
                all_shapes.var += shapes.var;
                all_shapes.interp += shapes.interp;
                for (k, v) in shapes.other.iter() {
                    *all_shapes.other.entry(k.clone()).or_default() += v;
                }
                if !shapes.other.is_empty() {
                    unsupported_expr.push((
                        rel.clone(),
                        service_name.clone(),
                        op.name.clone(),
                        shapes,
                    ));
                }
                let not_declared: Vec<String> = refs
                    .iter()
                    .filter(|r| !declared.contains(r))
                    .cloned()
                    .collect();
                if !not_declared.is_empty() {
                    println!(
                        "REF-NOT-DECLARED\t{rel}\t{service_name}\t{}\t{}\tdeclared={}",
                        op.name,
                        not_declared.join(","),
                        declared.join(",")
                    );
                }
                let outside: Vec<String> = refs
                    .iter()
                    .filter(|r| !hardcoded.contains(&r.as_str()))
                    .cloned()
                    .collect();
                if outside.is_empty() {
                    bindable_today += 1;
                } else {
                    unbindable.push((
                        rel.clone(),
                        service_name.clone(),
                        op.name.clone(),
                        outside,
                    ));
                }
                let _ = declared;
            }
        }
    }

    println!("files scanned: {}", files.len());
    println!("operations with a transport: {total_ops}");
    println!("shell-ish ops: {shell_ops}");
    println!("bindable with today's 5-name vocabulary: {bindable_today}");
    println!("UNBINDABLE today: {}", unbindable.len());
    println!("ops with unsupported argv expr shapes: {}", unsupported_expr.len());
    println!("argv[0] non-literal: {}", argv0_nonliteral.len());
    println!("argv expr shape totals: literal={} var={} interp={} other={:?}", all_shapes.literal, all_shapes.var, all_shapes.interp, all_shapes.other);
    println!("\n--- UNBINDABLE (path service op | outside-vocab refs) ---");
    for (p, s, o, refs) in &unbindable {
        println!("{p}\t{s}\t{o}\t{}", refs.join(","));
    }
    println!("\n--- UNSUPPORTED EXPR SHAPES ---");
    for (p, s, o, sh) in &unsupported_expr {
        println!("{p}\t{s}\t{o}\t{:?}", sh.other);
    }
    println!("\n--- ARGV0 NON-LITERAL ---");
    for (p, s, o) in &argv0_nonliteral {
        println!("{p}\t{s}\t{o}");
    }
}
