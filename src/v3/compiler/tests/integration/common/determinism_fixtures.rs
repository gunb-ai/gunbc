//! Shared emit determinism matrix (DB-8 / Lane D).
//!
//! Keep in sync with `m1_3_emit_rust_test.rs` program harness — this is the
//! single list of program fixtures used for rustc round-trip batching.
//!
//! Multiple integration-test binaries import this module; each uses a different
//! subset of exports — `dead_code` is allowed at module scope.
#![allow(dead_code)]

/// Self-contained program sources (compile as full programs).
pub struct ProgramFixture {
    pub name: &'static str,
    pub source: &'static str,
}

pub const PROGRAM_FIXTURES: &[ProgramFixture] = &[
    ProgramFixture {
        name: "list_fold_six",
        source: "let total: Int = fold(cons(1, cons(2, singleton(3))), 0, |acc, x| acc + x)",
    },
    ProgramFixture {
        name: "generic_list_fold_one",
        source: "let total: Int = fold(singleton(1), 0, |acc, x| acc + x)",
    },
    ProgramFixture {
        name: "list_map_then_fold_twelve",
        source: "let total: Int = fold(map(cons(1, cons(2, singleton(3))), |x| x * 2), 0, |acc, x| acc + x)",
    },
    ProgramFixture {
        name: "list_filter_then_fold_seven",
        source: "let total: Int = fold(filter(cons(1, cons(2, cons(3, singleton(4)))), |x| x > 2), 0, |acc, x| acc + x)",
    },
    ProgramFixture {
        name: "nested_list_builtins_inside_lambda_six",
        source: "let total: Int = fold(cons(1, singleton(2)), 0, |acc, x| acc + fold(map(singleton(x), |y| y * 2), 0, |n, y| n + y))",
    },
    ProgramFixture {
        name: "user_function_call_three",
        source: "fn add(a: Int, b: Int) -> Int = a + b\nlet total: Int = add(1, 2)",
    },
    ProgramFixture {
        name: "recursive_function_call_six",
        source: "fn count_down(n: Int) -> Int = if n == 0 then 0 else n + count_down(n - 1)\nlet total: Int = count_down(3)",
    },
    ProgramFixture {
        name: "record_literal_through_function_one",
        source: "type Point { x: Int y: Int }\nfn x_of(p: Point) -> Int = p.x\nlet total: Int = x_of({ x: 1, y: 2 })",
    },
    ProgramFixture {
        name: "user_sum_match_zero",
        source: "type Sign = Plus | Minus\nfn classify(s: Sign) -> Int = match s { Plus => 0, Minus => 1 }\nlet total: Int = classify(Plus)",
    },
];

/// Module-shaped sources for `emit_module` determinism (reflected harness matrix).
pub struct ModuleFixture {
    pub name: &'static str,
    /// Surface module body (lowers to a module fragment; consumed by `emit_rust_module`).
    pub source: &'static str,
}

pub const MODULE_FIXTURES: &[ModuleFixture] = &[
    ModuleFixture {
        name: "node_count",
        source: "fn node_count(d: Dag) -> Int = fold(d.nodes, 0, |n, node| n + 1)",
    },
    ModuleFixture {
        name: "bind_count",
        source: "fn bind_count(d: Dag) -> Int = fold(d.nodes, 0, |n, behavior| match behavior { Value(v) => n, Transform(t) => n, Branch(b) => n, Loop(l) => n, Bind(bind) => n + 1 })",
    },
    ModuleFixture {
        name: "singleton_span",
        source: "fn singleton_span(bind: BindNode) -> List<SourceSpan> = [bind.span]",
    },
    ModuleFixture {
        name: "result_port_is_param",
        source: "fn result_port_is_param(bind: BindNode) -> Bool = contains(bind.params, bind.result_port)",
    },
    ModuleFixture {
        name: "bind_names",
        source: "type FoundBind { name: String }\n\
             fn bind_names(d: Dag) -> List<FoundBind> = \
               fold(d.nodes, empty(), |acc, behavior| \
                 match behavior { \
                   Value(v) => acc, \
                   Transform(t) => acc, \
                   Branch(b) => acc, \
                   Loop(l) => acc, \
                   Bind(bind) => cons({ name: bind.name }, acc) \
                 })",
    },
];

/// On-disk four-fixture pressure suite (Lane 6 / dependency design).
pub const FOUR_FIXTURE_FILES: &[&str] = &["id.v3", "drop.v3", "wrap.v3", "is_empty.v3"];

/// `PROGRAM_FIXTURES` entries excluded from per-target 5× determinism.
/// Empty: Go `Behavior::Loop` emission landed in PR #692.
pub const GO_EMIT_EXCLUDE: &[&str] = &[];

/// Program fixtures excluded from Python 5× determinism.
/// Empty: Python `Behavior::Loop` emission landed in PR #692.
pub const PYTHON_EMIT_EXCLUDE: &[&str] = &[];
