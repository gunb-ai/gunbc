// SCRATCH TOOL (lane adhoc-9102e2d6-92f, not landed): enumerate every authored prefix (- / !) node
// whose operand root is an infix row that the pre-fix min_bp 12 absorbed (+,-,*,/,%,|>), or a
// method-call root (pipe builds one) -- i.e. every site whose tree CHANGES when parse_prefix stops
// handing the operand min_bp 12. Run at main and at the fix head; the diff of the two reports is
// the census the brief demands: a silent change of meaning in the corpus is exactly what is ruled
// out. Prints one line per exposed site.
#![allow(clippy::disallowed_macros)]

use std::path::{Path, PathBuf};
use std::rc::Rc;

use v1_compiler::module_path_index::parsed_dag_file::parse_dag_file;
use v1_compiler::v1_std_core::{BinOp, ExprData, Node, UnaryOpKind};

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            if p.file_name()
                .map(|n| n == "target" || n == ".git")
                .unwrap_or(false)
            {
                continue;
            }
            walk(&p, out);
        } else if p.extension().map(|x| x == "dag").unwrap_or(false) {
            out.push(p);
        }
    }
}

// The infix roots a prefix operand must not have: exactly the rows left_bp >= 12 absorbed on main.
fn absorbing_binop(d: &ExprData) -> Option<String> {
    match d {
        ExprData::ExprBinOp { op, .. } => match op {
            BinOp::Add => Some("+".to_string()),
            BinOp::Sub => Some("-".to_string()),
            BinOp::Mul => Some("*".to_string()),
            BinOp::Div => Some("/".to_string()),
            BinOp::Mod => Some("%".to_string()),
            _ => None,
        },
        _ => None,
    }
}

fn prefix_label(d: &ExprData) -> Option<String> {
    match d {
        ExprData::ExprUnaryOp { op, .. } => match op {
            UnaryOpKind::Neg => Some("-".to_string()),
            UnaryOpKind::Not => Some("!".to_string()),
        },
        _ => None,
    }
}

fn line_of(source: &str, start: i64) -> usize {
    let start = start.max(0) as usize;
    let end = start.min(source.len());
    source.as_bytes()[..end]
        .iter()
        .filter(|b| **b == b'\n')
        .count()
        + 1
}

fn visit(node: &Rc<Node>, file: &str, source: &str, hits: &mut Vec<String>) {
    if let Some(p) = prefix_label(&node.expr_data) {
        if let Some(child) = node.children.iter().next() {
            let root = match absorbing_binop(&child.expr_data) {
                Some(op) => Some(format!("ABSORBS[{}]", op)),
                None => match &*child.expr_data {
                    // Pipe (|>) builds ExprMethodCall at the infix level, but try_postfix also
                    // builds one for ordinary method access; the report line carries source text
                    // around the prefix node for hand classification.
                    ExprData::ExprMethodCall { .. } => {
                        Some("MAYBE-PIPE(method-call root)".to_string())
                    }
                    _ => None,
                },
            };
            if let Some(root) = root {
                hits.push(format!(
                    "{}:{} PREFIX[{}] {}",
                    file,
                    line_of(source, node.span.start),
                    p,
                    root
                ));
            }
        }
    }
    for c in node.children.iter() {
        visit(c, file, source, hits);
    }
    if let Some(b) = &node.body {
        visit(b, file, source, hits);
    }
    for c in node.params.iter() {
        visit(c, file, source, hits);
    }
    for c in node.uses.iter() {
        visit(c, file, source, hits);
    }
    for c in node.properties.iter() {
        visit(c, file, source, hits);
    }
    if let Some(t) = &node.type_annotation {
        visit(t, file, source, hits);
    }
    if let Some(t) = &node.transport {
        visit(t, file, source, hits);
    }
}

fn main() {
    let roots: Vec<String> = std::env::args().skip(1).collect();
    let mut files = Vec::new();
    for r in &roots {
        walk(Path::new(r), &mut files);
    }
    files.sort();
    let mut total_files = 0usize;
    let mut parse_failures = 0usize;
    let mut all: Vec<String> = Vec::new();
    for f in &files {
        total_files += 1;
        let Some(parsed) = parse_dag_file(f) else {
            parse_failures += 1;
            println!("PARSE-FAIL {}", f.display());
            continue;
        };
        let path = f.display().to_string();
        let Ok(source) = std::fs::read_to_string(f) else {
            println!("READ-FAIL {}", path);
            continue;
        };
        let mut hits = Vec::new();
        for item in parsed.items.iter() {
            visit(item, &path, &source, &mut hits);
        }
        visit(&parsed.module, &path, &source, &mut hits);
        all.extend(hits);
    }
    for h in &all {
        println!("{}", h);
    }
    println!(
        "CENSUS files={} parse_failures={} exposed_sites={}",
        total_files,
        parse_failures,
        all.len()
    );
}
