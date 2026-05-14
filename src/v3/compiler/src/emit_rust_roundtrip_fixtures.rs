//! Emit Rust / omni round-trip fixture tables shared by the host `#[test]`
//! harnesses (`m1_3_*`, `m1_5_*`) and R1C-E `r1c_e_gates::check_*` (the
//! `r1c_e_emit_gates` `ExecuteCommand` bin). **Single source of truth** for
//! `PROGRAM_FIXTURES` / `REFLECTED_FIXTURES` (was `tests/.../determinism_fixtures`
//! + `m1_3` locals).

/// Self-contained program sources (compile as full programs).
pub struct ProgramFixture {
    pub name: &'static str,
    pub source: &'static str,
    /// Expected stdout from the compiled binary (exact string equality).
    pub expected_stdout: &'static str,
}

pub const PROGRAM_FIXTURES: &[ProgramFixture] = &[
    ProgramFixture {
        name: "list_fold_six",
        source: "let total: Int = fold(cons(1, cons(2, singleton(3))), 0, |acc, x| acc + x)",
        expected_stdout: "6",
    },
    ProgramFixture {
        name: "generic_list_fold_one",
        source: "let total: Int = fold(singleton(1), 0, |acc, x| acc + x)",
        expected_stdout: "1",
    },
    ProgramFixture {
        name: "list_map_then_fold_twelve",
        source: "let total: Int = fold(map(cons(1, cons(2, singleton(3))), |x| x * 2), 0, |acc, x| acc + x)",
        expected_stdout: "12",
    },
    ProgramFixture {
        name: "list_filter_then_fold_seven",
        source: "let total: Int = fold(filter(cons(1, cons(2, cons(3, singleton(4)))), |x| x > 2), 0, |acc, x| acc + x)",
        expected_stdout: "7",
    },
    ProgramFixture {
        name: "nested_list_builtins_inside_lambda_six",
        source: "let total: Int = fold(cons(1, singleton(2)), 0, |acc, x| acc + fold(map(singleton(x), |y| y * 2), 0, |n, y| n + y))",
        expected_stdout: "6",
    },
    ProgramFixture {
        name: "user_function_call_three",
        source: "fn add(a: Int, b: Int) -> Int = a + b\nlet total: Int = add(1, 2)",
        expected_stdout: "3",
    },
    ProgramFixture {
        name: "recursive_function_call_six",
        source: "fn count_down(n: Int) -> Int = if n == 0 then 0 else n + count_down(n - 1)\nlet total: Int = count_down(3)",
        expected_stdout: "6",
    },
    ProgramFixture {
        name: "record_literal_through_function_one",
        source: "type Point { x: Int y: Int }\nfn x_of(p: Point) -> Int = p.x\nlet total: Int = x_of({ x: 1, y: 2 })",
        expected_stdout: "1",
    },
    ProgramFixture {
        name: "user_sum_match_zero",
        source: "type Sign = Plus | Minus\nfn classify(s: Sign) -> Int = match s { Plus => 0, Minus => 1 }\nlet total: Int = classify(Plus)",
        expected_stdout: "0",
    },
];

/// Module-shaped sources for `emit_module` determinism (reflected harness matrix).
pub struct ModuleFixture {
    pub name: &'static str,
    /// Surface module body (lowers to a module fragment; consumed by `emit_rust_module`).
    pub source: &'static str,
}

// --- DB-8 module surface: one `const` per row so `REFLECTED_FIXTURES` can borrow it. ---

const MODULE_NODE_COUNT: ModuleFixture = ModuleFixture {
    name: "node_count",
    source: "fn node_count(d: Dag) -> Int = fold(d.nodes, 0, |n, node| n + 1)",
};

const MODULE_BIND_COUNT: ModuleFixture = ModuleFixture {
    name: "bind_count",
    source: "fn bind_count(d: Dag) -> Int = fold(d.nodes, 0, |n, behavior| match behavior { Value(v) => n, Transform(t) => n, Branch(b) => n, Loop(l) => n, Bind(bind) => n + 1 })",
};

const MODULE_SINGLETON_SPAN: ModuleFixture = ModuleFixture {
    name: "singleton_span",
    source: "fn singleton_span(bind: BindNode) -> List<SourceSpan> = [bind.span]",
};

const MODULE_RESULT_PORT_IS_PARAM: ModuleFixture = ModuleFixture {
    name: "result_port_is_param",
    source:
        "fn result_port_is_param(bind: BindNode) -> Bool = contains(bind.params, bind.result_port)",
};

const MODULE_BIND_NAMES: ModuleFixture = ModuleFixture {
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
};

pub const MODULE_FIXTURES: &[ModuleFixture] = &[
    MODULE_NODE_COUNT,
    MODULE_BIND_COUNT,
    MODULE_SINGLETON_SPAN,
    MODULE_RESULT_PORT_IS_PARAM,
    MODULE_BIND_NAMES,
];

/// On-disk four-fixture pressure suite (Lane 6 / dependency design).
pub const FOUR_FIXTURE_FILES: &[&str] = &["id.v3", "drop.v3", "wrap.v3", "is_empty.v3"];

/// `PROGRAM_FIXTURES` entries excluded from per-target 5× determinism.
/// Empty: Go `Behavior::Loop` emission landed in PR #692.
pub const GO_EMIT_EXCLUDE: &[&str] = &[];

/// Program fixtures excluded from Python 5× determinism.
/// Empty: Python `Behavior::Loop` emission landed in PR #692.
pub const PYTHON_EMIT_EXCLUDE: &[&str] = &[];

// ── Reflected batched-harness matrix (m1_3 + R1C-E) ─────────────────────────

/// Expected output shape for a reflected-module roundtrip fixture.
pub enum ReflectedExpected {
    /// Exact stdout string (trimmed).
    Exact(&'static str),
    /// Any positive integer (used for `node_count`, whose exact value is not pinned).
    PositiveInt,
}

/// One reflected-module `rustc` roundtrip: the module surface is **only** in
/// [`ModuleFixture::source`]; this row adds the wrapper that imports `v3_compiler`
/// at runtime and the expected stdout. Points at the same `const` as
/// [`MODULE_FIXTURES`] (no duplicate `source` text).
pub struct ReflectedFixture {
    pub module: &'static ModuleFixture,
    pub wrapper_body: &'static str,
    pub expected_stdout: ReflectedExpected,
}

pub const REFLECTED_FIXTURES: &[ReflectedFixture] = &[
    ReflectedFixture {
        module: &MODULE_NODE_COUNT,
        wrapper_body: "let dag = v3_compiler::compile_to_dag(\"let x: Int = 1\\nlet y: Int = x + 2\", \"runtime_reflection.v3\").expect(\"compiles\"); node_count(&dag)",
        expected_stdout: ReflectedExpected::PositiveInt,
    },
    ReflectedFixture {
        module: &MODULE_BIND_COUNT,
        // Subtract the bootstrap baseline so the test still pins the
        // user-program bind count after Lane 2 Stage 2d std-module binds.
        wrapper_body: "let dag = v3_compiler::compile_to_dag(\"let x: Int = 1\\nlet y: Int = x + 2\", \"runtime_reflection.v3\").expect(\"compiles\"); let baseline = v3_compiler::dag::Dag::new(); bind_count(&dag) - bind_count(&baseline)",
        expected_stdout: ReflectedExpected::Exact("2"),
    },
    ReflectedFixture {
        module: &MODULE_SINGLETON_SPAN,
        wrapper_body: "let dag = v3_compiler::compile_to_dag(\"let x: Int = 1\\nlet y: Int = x + 2\", \"runtime_reflection.v3\").expect(\"compiles\"); let bind = dag.nodes().iter().find_map(|node| match node { v3_compiler::dag::Behavior::Bind(bind) => Some(bind.clone()), _ => None }).expect(\"bind\"); singleton_span(&bind).len() as i64",
        expected_stdout: ReflectedExpected::Exact("1"),
    },
    ReflectedFixture {
        module: &MODULE_RESULT_PORT_IS_PARAM,
        wrapper_body: "let dag = v3_compiler::compile_to_dag(\"fn id(x: Int) -> Int = x\", \"runtime_reflection.v3\").expect(\"compiles\"); let bind = dag.nodes().iter().find_map(|node| match node { v3_compiler::dag::Behavior::Bind(bind) if bind.name == \"id\" && bind.emit_participation() == Some(v3_compiler::dag::BindEmitParticipation::UserCallable) => Some(bind.clone()), _ => None }).expect(\"function bind\"); if result_port_is_param(&bind) { 1 } else { 0 }",
        expected_stdout: ReflectedExpected::Exact("1"),
    },
    ReflectedFixture {
        module: &MODULE_BIND_NAMES,
        // Same baseline subtraction as `bind_count`.
        wrapper_body: "let dag = v3_compiler::compile_to_dag(\"let x: Int = 1\\nlet y: Int = x + 2\", \"runtime_reflection.v3\").expect(\"compiles\"); let baseline = v3_compiler::dag::Dag::new(); (bind_names(&dag).len() as i64) - (bind_names(&baseline).len() as i64)",
        expected_stdout: ReflectedExpected::Exact("2"),
    },
];
