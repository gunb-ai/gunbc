#!/usr/bin/env python3

from __future__ import annotations

from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[1]


def read(relative_path: str) -> str:
    return (ROOT / relative_path).read_text(encoding="utf-8")


def live_source(source: str) -> str:
    return "\n".join(
        line for line in source.splitlines() if not line.lstrip().startswith("//")
    )


def require_contains(
    failures: list[str],
    relative_path: str,
    needle: str,
    message: str,
    *,
    live_only: bool = False,
) -> None:
    source = read(relative_path)
    haystack = live_source(source) if live_only else source
    if needle not in haystack:
        failures.append(f"{relative_path}: {message}")


def require_not_contains(
    failures: list[str],
    relative_path: str,
    needle: str,
    message: str,
    *,
    live_only: bool = False,
) -> None:
    source = read(relative_path)
    haystack = live_source(source) if live_only else source
    if needle in haystack:
        failures.append(f"{relative_path}: {message}")


def main() -> int:
    failures: list[str] = []

    require_contains(
        failures,
        "src/v2/04_infer.dag",
        "detect_type_cycles",
        "should retain explicit type-cycle detection",
    )
    require_contains(
        failures,
        "src/v2/04_infer.dag",
        "recursive_types",
        "should retain recursive type tracking",
    )

    require_contains(
        failures,
        "src/v2/03_resolve.dag",
        "acyclic_resolved",
        "should keep the acyclic resolved module filter",
    )
    require_contains(
        failures,
        "src/v2/03_resolve.dag",
        "r.resolved.target_module != none",
        "should filter failed imports before downstream passes",
    )
    require_contains(
        failures,
        "src/v2/03_resolve.dag",
        "r.diagnostics |> count == 0",
        "should filter cyclic/diagnostic-bearing imports",
    )

    require_contains(
        failures,
        "src/v2/04_resolve.dag",
        "fn resolve_expr_types(",
        "should define resolve_expr_types in the resolve pass",
        live_only=True,
    )
    require_not_contains(
        failures,
        "src/v2/04_infer.dag",
        "fn resolve_expr_types(",
        "should not define resolve_expr_types in infer anymore",
        live_only=True,
    )
    require_contains(
        failures,
        "src/v2/04_infer.dag",
        "if env_errors |> count > 0 {",
        "should gate inference on env_errors before infer_items",
        live_only=True,
    )

    require_contains(
        failures,
        "src/v2/00_core.dag",
        "fn expr_children",
        "should retain the shared expr_children walk",
    )
    require_contains(
        failures,
        "src/v2/00_core.dag",
        "ExprReturn",
        "expr_children coverage should include return nodes",
    )
    require_contains(
        failures,
        "src/v2/00_core.dag",
        "ExprForEach",
        "expr_children coverage should include for-each nodes",
    )
    require_contains(
        failures,
        "src/v2/00_core.dag",
        "ExprIndex",
        "expr_children coverage should include index nodes",
    )
    require_contains(
        failures,
        "src/v2/00_core.dag",
        "ExprSlice",
        "expr_children coverage should include slice nodes",
    )

    require_not_contains(
        failures,
        "src/v2/05_emit.dag",
        "classify_intrinsic_method",
        "should not regress to intrinsic string classifier fallback",
    )
    require_not_contains(
        failures,
        "src/v2/05_emit_rust.dag",
        "extend_scope_for_lambda",
        "should not restore lambda scope fallback",
    )
    require_not_contains(
        failures,
        "src/v2/05_emit_rust.dag",
        "let needs_wrap = false",
        "should not restore the legacy lambda wrap fallback",
    )

    require_contains(
        failures,
        "src/v2/02_parse.dag",
        "parse_recovery_placeholder()",
        "should keep parse recovery placeholder instead of fabricating a null literal",
    )
    require_not_contains(
        failures,
        "src/v2/02_parse.dag",
        (
            "make_expr_node(expr_data: ExprLiteral { value: LitNull }, "
            "return_type: none, span: SourceSpan { start: 0, end: 0 })"
        ),
        "should not fabricate null literals in parse recovery",
    )
    require_not_contains(
        failures,
        "src/v2/05_emit_rust.dag",
        'LitNull => ""',
        "should not emit empty-string null fabrication in Rust",
    )
    require_not_contains(
        failures,
        "src/v2/05_emit_rust.dag",
        'None => Node { name: ""',
        "should not fabricate empty Node fallbacks in Rust emit",
    )

    require_not_contains(
        failures,
        "src/v2/05_emit_rust.dag",
        'todo!("unsupported simple expr',
        "testgen should fail loud, not leave todo! placeholders",
    )
    rust_emit = read("src/v2/05_emit_rust.dag")
    shared_emit = read("src/v2/05_emit.dag")
    if not (
        'compile_error!("unsupported simple expr' in rust_emit
        or 'compile_error!(\\"unsupported simple expr' in rust_emit
        or 'emit_error_expr(message: "unsupported simple expr' in rust_emit
        or 'emit_error_expr(message: "unsupported simple expr' in shared_emit
    ):
        failures.append(
            "src/v2/05_emit*.dag: simple expr testgen path should fail loud"
        )
    require_not_contains(
        failures,
        "src/v2/05_emit_rust.dag",
        "Ok(Default::default())",
        "Rust testgen should not default-success empty projections",
    )
    require_contains(
        failures,
        "src/v2/05_emit_rust.dag",
        "has_mock_prefix",
        "Rust testgen should still gate on mock-prefixed fields",
    )
    require_not_contains(
        failures,
        "src/v2/05_emit_rust.dag",
        "starts_with_prefix",
        "legacy mock prefix helper should stay deleted",
    )
    require_contains(
        failures,
        "src/v2/05_emit.dag",
        "extract_test_projections",
        "shared emit should own test projection extraction",
    )
    require_contains(
        failures,
        "src/v2/05_emit.dag",
        "TestProjection",
        "shared emit should retain the TestProjection contract",
    )
    require_contains(
        failures,
        "src/v2/05_emit.dag",
        'Rust => concat("Rc<dyn Fn(',
        "shared emit should render Rust callable aliases as Rc<dyn Fn(...)>",
    )
    require_not_contains(
        failures,
        "src/v2/05_emit.dag",
        'Rust => concat("impl Fn(',
        "shared emit should not render Rust callable aliases as impl Fn",
    )
    require_contains(
        failures,
        "src/v2/05_emit_rust.dag",
        "emit_test_file",
        "Rust emit should still own test-file emission",
    )
    require_contains(
        failures,
        "src/v2/05_emit.dag",
        "has_mock_prefix",
        "shared emit should still filter mock-only source fields",
    )
    require_not_contains(
        failures,
        "src/v2/05_emit.dag",
        "Ok(Default::default())",
        "shared emit should not default-success empty projections",
    )
    require_not_contains(
        failures,
        "src/v2/05_emit.dag",
        "Value::Object(Default::default())",
        "shared emit should not fabricate empty objects",
    )
    require_not_contains(
        failures,
        "src/v2/05_emit.dag",
        '_ => "null"',
        "shared emit should not collapse unsupported cases to null",
    )
    require_not_contains(
        failures,
        "src/v2/05_emit.dag",
        "extract_mock_props",
        "legacy mock-prop walker should stay deleted",
    )
    require_not_contains(
        failures,
        "src/v2/05_emit.dag",
        "starts_with_prefix",
        "legacy prefix helper should stay deleted in shared emit",
    )

    if failures:
        for failure in failures:
            print(f"FAIL: {failure}", file=sys.stderr)
        return 1

    print("source audit passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
