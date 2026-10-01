// THE CENSUS OF RECURSIVE VALUE WALKERS, ONE FUNCTION WITH TWO EXECUTORS. `build.rs` includes this
// file and REFUSES THE BUILD when it finds a walker, so every lane that compiles v1-compiler -- the
// required lint step among them -- enforces it on every merge; `v1_interpreter`
// `value_depth_walker_tests` includes it again to assert the same thing as a test. It is `include!`d
// rather than declared as a module because the crate root is generated and a build script cannot
// link the crate it builds. Std only, no allocation beyond the scan.
//
// A function whose parameters name the interpreter's `Value` and whose body calls itself recurses
// once per level of a value, and a value's depth is bounded by the heap, not by any call limit
// (gunbc.recurring_failure_mode recursion_over_value_depth_uncounted_by_the_call_limit). Such a
// function must run under `value_depth_guarded`. REACH, stated rather than implied: DIRECT
// self-recursion only. A mutually recursive pair or a walk through a trait impl is not seen; the
// Value trait impls (PartialEq, Display, Debug) are guarded by hand and Drop is iterative.

/// `path::fn` for every recursive Value walker under `root` that does not run under the guard.
fn unguarded_recursive_value_walkers(root: &std::path::Path) -> Vec<String> {
    let mut unguarded = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            unguarded.push(format!("{}: unreadable source directory", dir.display()));
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                let Ok(text) = std::fs::read_to_string(&path) else {
                    unguarded.push(format!("{}: unreadable source file", path.display()));
                    continue;
                };
                for (name, body) in self_recursive_value_fns(&text) {
                    if !body.contains("value_depth_guarded") {
                        unguarded.push(format!("{}::{name}", path.display()));
                    }
                }
            }
        }
    }
    unguarded.sort();
    unguarded
}

/// (name, body) of each `fn` whose parameter list names `Value` (the interpreter's, not
/// serde_json's) and whose body calls it by name.
fn self_recursive_value_fns(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(at) = rest.find("fn ") {
        let after = &rest[at + 3..];
        rest = after;
        let name: String = after
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if name.is_empty() {
            continue;
        }
        let Some(open) = after.find('(') else {
            continue;
        };
        let Some(close) = after[open..].find(')') else {
            continue;
        };
        let params = &after[open..open + close];
        let names_value = params
            .split(&[',', '(', ' ', '&', '\n'][..])
            .any(|tok| tok == "Value" || (tok.ends_with("::Value") && !tok.contains("serde_json")));
        if !names_value {
            continue;
        }
        let Some(body_open) = after[open + close..].find('{') else {
            continue;
        };
        let start = open + close + body_open;
        let mut depth = 0usize;
        let mut end = None;
        for (i, c) in after[start..].char_indices() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(start + i);
                        break;
                    }
                }
                _ => {}
            }
        }
        let Some(end) = end else { continue };
        let body = &after[start + 1..end];
        if body.contains(&format!("{name}(")) {
            out.push((name, body.to_string()));
        }
    }
    out
}
