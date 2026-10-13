use v1_compiler::v1_compiler_artifact::RenderTarget;
use v1_compiler::v1_compiler_compile::{compile_sources, SourceFile};

fn emit(body: &str) -> String {
    let src = format!(
        "module cursor_fixture\nimport std.algebra {{ Cons, Empty }}\n{}",
        body
    );
    let c = compile_sources(
        std::rc::Rc::new(im::vector![std::rc::Rc::new(SourceFile {
            path: "cursor_fixture.dag".to_string(),
            content: src
        })]),
        RenderTarget::Rust,
    );
    format!(
        "DIAG={:?}\n{}",
        c.diagnostics,
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
