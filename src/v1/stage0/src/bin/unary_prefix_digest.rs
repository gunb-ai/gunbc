// SCRATCH TOOL (lane adhoc-9102e2d6-92f, not landed): the AUTHORITATIVE census. Prints one line
// per authored item: path, item name, and a CANONICAL TREE DIGEST that excludes spans and
// occurrence identities -- exactly what the parser's precedence decisions produce. Run the
// old-binding binary (built at main) and the new-binding binary (built at the fix) over the SAME
// corpus text; the line diff is the set of expressions whose meaning the fix changes. An empty
// diff is the ruling-out the brief demands: no authored expression changes meaning.
#![allow(clippy::disallowed_macros)]

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use v1_compiler::module_path_index::parsed_dag_file::parse_dag_file;
use v1_compiler::v1_std_core::Node;

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

// Drop identity/span-bearing keys from a serialized expr_data; meaning lives in the rest,
// including literal values, which must NOT be scrubbed (that is what the diff measures).
fn scrub(v: &serde_json::Value) -> serde_json::Value {
    match v {
        serde_json::Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (k, val) in map {
                if k == "occurrence_identity"
                    || k == "span"
                    || k == "name_span"
                    || k == "type_span"
                    || k == "descent_evidence"
                {
                    continue;
                }
                out.insert(k.clone(), scrub(val));
            }
            serde_json::Value::Object(out)
        }
        serde_json::Value::Array(xs) => serde_json::Value::Array(xs.iter().map(scrub).collect()),
        other => other.clone(),
    }
}

// Canonical, span-free, occurrence-free digest of one node subtree.
fn digest_node(node: &Rc<Node>, h: &mut DefaultHasher) {
    // The discriminant + payload of expr_data, with identity/span-bearing keys dropped (the
    // payload keeps LITERAL VALUES -- those are meaning; the dropped keys are parse-order churn
    // that would mask the real shape diff by making every digest differ).
    let value = serde_json::to_value(&node.expr_data).unwrap_or(serde_json::Value::Null);
    let scrubbed = scrub(&value);
    serde_json::to_string(&scrubbed)
        .unwrap_or_else(|_| "?".to_string())
        .hash(h);
    node.name.hash(h);
    for c in node.children.iter() {
        digest_node(c, h);
    }
    if let Some(b) = &node.body {
        "body".hash(h);
        digest_node(b, h);
    }
    for c in node.params.iter() {
        "param".hash(h);
        digest_node(c, h);
    }
    for c in node.uses.iter() {
        "use".hash(h);
        digest_node(c, h);
    }
    for c in node.properties.iter() {
        "prop".hash(h);
        digest_node(c, h);
    }
    if let Some(t) = &node.type_annotation {
        "ty".hash(h);
        digest_node(t, h);
    }
    if let Some(t) = &node.transport {
        "tp".hash(h);
        digest_node(t, h);
    }
}

fn digest_of(node: &Rc<Node>) -> String {
    let mut h = DefaultHasher::new();
    digest_node(node, &mut h);
    format!("{:016x}", h.finish())
}

fn main() {
    let roots: Vec<String> = std::env::args().skip(1).collect();
    let mut files = Vec::new();
    for r in &roots {
        walk(Path::new(r), &mut files);
    }
    files.sort();
    let mut count = 0usize;
    let mut skipped: Vec<String> = Vec::new();
    for f in &files {
        let Some(parsed) = parse_dag_file(f) else {
            skipped.push(f.display().to_string());
            continue;
        };
        for (i, item) in parsed.items.iter().enumerate() {
            println!("{}#{} {} {}", f.display(), i, item.name, digest_of(item));
            count += 1;
        }
    }
    // FAIL-OPEN MADE LOUD (manager ruling, lane adhoc-9102e2d6-92f): a silent skip on
    // parse failure let the two #14559-broken grammar files vanish from the first census.
    // Every skip is counted and named; the census is only citable with skipped=0.
    for s in &skipped {
        println!("SKIPPED {}", s);
    }
    eprintln!(
        "ITEMS {count} FILES {} SKIPPED {}",
        files.len(),
        skipped.len()
    );
}
