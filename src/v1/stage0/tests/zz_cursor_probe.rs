use v1_compiler::v1_compiler_artifact::RenderTarget;
use v1_compiler::v1_compiler_compile::{compile_sources, SourceFile};

fn module_path(root: &std::path::Path, m: &str) -> std::path::PathBuf {
    let mut parts = m.split('.');
    let top = parts.next().unwrap();
    let rest: Vec<&str> = parts.collect();
    let base = if top == "v2" {
        root.join("src/v2")
    } else {
        root.join("dag").join(top)
    };
    let mut p = base;
    for r in &rest {
        p = p.join(r);
    }
    p.set_extension("dag");
    p
}

fn closure(root: &std::path::Path, entry: &str) -> Vec<std::rc::Rc<SourceFile>> {
    let mut seen = std::collections::BTreeSet::new();
    let mut todo = vec![entry.to_string()];
    let mut out = Vec::new();
    while let Some(m) = todo.pop() {
        if !seen.insert(m.clone()) {
            continue;
        }
        let path = module_path(root, &m);
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        for line in content.lines() {
            if let Some(rest) = line.strip_prefix("import ") {
                let name: String = rest
                    .chars()
                    .take_while(|c| !c.is_whitespace() && *c != '{')
                    .collect();
                todo.push(name);
            }
        }
        out.push(std::rc::Rc::new(SourceFile {
            path: path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .into_owned(),
            content,
        }));
    }
    out
}

fn emit(body: &str) -> String {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let mut files = closure(&root, "std.algebra");
    files.push(std::rc::Rc::new(SourceFile {
        path: "cursor_fixture.dag".to_string(),
        content: format!(
            "module cursor_fixture\nimport std.algebra {{ Cons, Empty }}\n{}",
            body
        ),
    }));
    let c = compile_sources(
        std::rc::Rc::new(files.into_iter().collect()),
        RenderTarget::Rust,
    );
    format!(
        "DIAG={:?}\n{}",
        c.diagnostics
            .iter()
            .map(|d| format!("{:?}", d).chars().take(300).collect::<String>())
            .collect::<Vec<_>>(),
        c.files
            .iter()
            .find(|f| f.path.ends_with("cursor_fixture.rs"))
            .map(|f| f.content.clone())
            .unwrap_or_default()
    )
}

#[test]
fn probe() {
    println!("=====pos\n{}", emit("fn walk(xs: List<Int>, acc: Int) -> Int {\n  match xs {\n    Empty => acc\n    Cons { head: h, tail: t } => walk(t, acc + h)\n  }\n}\n"));
    println!("=====escape\n{}", emit("fn walk(xs: List<Int>, acc: Int) -> List<Int> {\n  match xs {\n    Empty => []\n    Cons { head: h, tail: t } => if h == 0 { t } else { walk(t, acc + h) }\n  }\n}\n"));
    println!("=====xs_after\n{}", emit("fn walk(xs: List<Int>, acc: Int) -> Int {\n  match xs {\n    Empty => acc + list_len(xs)\n    Cons { head: h, tail: t } => walk(t, acc + h)\n  }\n}\n"));
    println!("=====other_pos\n{}", emit("fn walk(xs: List<Int>, ys: List<Int>, acc: Int) -> Int {\n  match xs {\n    Empty => acc\n    Cons { head: h, tail: t } => walk(ys, t, acc + h)\n  }\n}\n"));
    println!("=====depth2\n{}", emit("fn walk(xs: List<Int>, acc: Int) -> Int {\n  match xs {\n    Empty => acc\n    Cons { head: a, tail: Cons { head: b, tail: r } } => walk(r, acc + a + b)\n    Cons { head: a, tail: Empty } => acc + a\n  }\n}\n"));
    panic!("probe");
}
