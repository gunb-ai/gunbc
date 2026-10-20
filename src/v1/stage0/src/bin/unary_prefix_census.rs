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

// The operand node's own source text, exactly as far as its span reaches.
fn operand_slice(source: &str, node: &Rc<Node>) -> String {
    let a = (node.span.start.max(0) as usize).min(source.len());
    let b = (node.span.end.max(0) as usize).min(source.len());
    source[a..b.max(a)].to_string()
}

// A |> at bracket/string depth zero: absorbed by the prefix, not nested inside it.
fn has_top_level_pipe(s: &str) -> bool {
    let bs = s.as_bytes();
    let mut depth = 0i32;
    let mut i = 0;
    let mut in_str = false;
    let mut esc = false;
    while i < bs.len() {
        let c = bs[i];
        if in_str {
            if esc {
                esc = false;
            } else if c == b'\\' {
                esc = true;
            } else if c == b'"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        match c {
            b'"' => in_str = true,
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth -= 1,
            b'|' if depth == 0 && i + 1 < bs.len() && bs[i + 1] == b'>' => return true,
            _ => {}
        }
        i += 1;
    }
    false
}

fn one_line(s: &str) -> String {
    let t: String = s
        .chars()
        .map(|c| if c == '\n' || c == '\r' { ' ' } else { c })
        .collect();
    let t: String = t.split_whitespace().collect::<Vec<_>>().join(" ");
    if t.len() > 96 {
        format!("{}..", &t[..96])
    } else {
        t
    }
}

fn visit(node: &Rc<Node>, file: &str, source: &str, hits: &mut Vec<String>) {
    if let Some(p) = prefix_label(&node.expr_data) {
        if let Some(child) = node.children.iter().next() {
            let root = match absorbing_binop(&child.expr_data) {
                Some(op) => Some(format!("ABSORBS[{}]", op)),
                None => match &*child.expr_data {
                    // Pipe (|>) builds ExprMethodCall at the infix level, but try_postfix also
                    // builds one for ordinary method access; the OPERAND'S OWN SLICE decides:
                    // a top-level |> in it (bracket/string depth 0) means the pipe was absorbed
                    // by the prefix -- the meaning changes when the declared power lands.
                    ExprData::ExprMethodCall { .. } => {
                        let slice = operand_slice(source, child);
                        if has_top_level_pipe(&slice) {
                            Some(format!("PIPE-ABSORBED[{}]", one_line(&slice)))
                        } else {
                            Some(format!("postfix-method[{}]", one_line(&slice)))
                        }
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
