# Branch C TargetModel Audit — C.1–C.5 Findings (Class 2 preflight)

> **Status:** AUDIT COMPLETE — ready for implementation worker dispatch.
> **Author:** Cross-target Emission Mgr (`silent-bear-54`), 2026-06-02.
> **Evidence:** HEAD main; sunny-owl-821 spec cited at ctrl#1425 §3 Branch C (dashboard brief).
> **Worksheet:** `v4-cross-target-emission-rr-c-worksheet-2026-06-02.md`.

## Executive summary

Branch C blocked on a **split authority** for concrete syntax tokens: ten per-language
`*ConcreteSyntaxToken` coproducts in `src/v4/extdeps/languages/` vs Symbol-tagged `kind` edges
consumed by `06_translate`. C.4 (`05_emit`) already matches the ratified thin-orchestrator shape.
C.1–C.3 are one substrate PR; C.5 is the compiler consumer PR (or same PR if reviewable size).

## C.1 — Shared carrier (not yet declared)

**Finding:** `target_model.dag` names `concrete_syntax_token_kind_fixed` / `_bound` Symbols and
documents dissolution, but does **not** declare `ConcreteSyntaxToken = FixedToken | BoundToken`.

```81:86:src/v4/std/target_model.dag
// dissolve-on: target_model.dag declares a typed ConcreteSyntaxToken coproduct (FixedToken |
// BoundToken) and emit_fixed_token/emit_bound_token in dag.dag produce typed values; the
// Symbol-equality discriminators below dissolve into structural pattern matching.
data concrete_syntax_token_kind_fixed: Symbol = concrete_syntax_token_kind_fixed
data concrete_syntax_token_kind_bound: Symbol = concrete_syntax_token_kind_bound
```

**Action:** Add terminal coproduct + field symbols on grammar relation rows; keep edge names
(`concrete_syntax_token_field_*`) stable for one release to limit churn.

## C.2 — Per-language duplicate coproducts (dissolution bar)

**Finding:** Eleven parallel type authorities (grep `type .*ConcreteSyntaxToken`):

| File | Type |
|------|------|
| `rust.dag` | `RustConcreteSyntaxToken` |
| `python.dag` | `PythonConcreteSyntaxToken` |
| `go.dag` | `GoConcreteSyntaxToken` |
| `typescript.dag` | `TsConcreteSyntaxToken` |
| `cpp.dag`, `java.dag`, `kotlin.dag`, `swift.dag`, `wasm.dag`, `ecmascript.dag` | `*ConcreteSyntaxToken` |

Kotlin documents the dissolution bar explicitly (pull-forward §2.8.3):

```208:215:src/v4/extdeps/languages/kotlin.dag
// dissolve-on: v4.std.target_model declares shared ConcreteSyntaxToken (FixedToken |
// BoundToken); this per-target coproduct dissolves when that std carrier lands.
type KotlinConcreteSyntaxToken
  = KotlinFixedToken { token_class: Symbol }
  | KotlinBoundToken { token_class: Symbol, binding: Symbol }
```

**Action:** After C.1, replace per-target coproducts with `type X = ConcreteSyntaxToken` aliases
or row functions taking `ConcreteSyntaxToken` — **no second sum** with distinct variant names.

## C.3 — Constructor alignment (`dag.dag`)

**Finding:** `emit_fixed_token` / `emit_bound_token` already build Conj nodes with
`concrete_syntax_token_field_*` edges but are untyped Node producers:

```416:433:src/v4/extdeps/languages/dag.dag
fn emit_fixed_token(token_class: Symbol) -> Node {
  ...
}
fn emit_bound_token(token_class: Symbol, binding: Symbol) -> Node {
  ...
```

**Action:** Retarget constructors to emit `FixedToken` / `BoundToken` shapes once C.1 exists;
update grammar relation row builders in release-minimum languages first (rust, python, go, ts).

## C.4 — `05_emit` thin orchestrator ✅

**Finding:** `05_emit.dag` is already `emit = translate ∘ serialize_target` with no target-specific
logic — matches §2.2.1 option ii.

```33:42:src/v4/compiler/05_emit.dag
fn emit(
  tree: InferredTree,
  target: TargetModel
) -> Outcome<TargetSource> {
  bind_outcome(
    o: translate(tree: tree, target: target),
    f: fn(target_tree) {
      serialize_target(tree: target_tree, target: target)
    }
  )
}
```

**Action:** None for C.4; reject PRs that add substantive emit paths (anti-scope).

## C.5 — `06_translate` Symbol-equality match migration

**Finding:** Three gated sites use Symbol comparison instead of structural variants:

1. `concrete_token_kind_is_declared` — lines 624–629
2. `token_spelling_from_model` — `kind == concrete_syntax_token_kind_fixed/bound` — lines 917–959
3. `token_sequence_item_kind` — indirect via (1)

**Action:** Replace with `match` on typed `ConcreteSyntaxToken` once token nodes carry the C.1
carrier (may require decode helper from `Node` → `ConcreteSyntaxToken` at bundle boundary).

**Risk:** MVP-1 translate claims must stay green; worker runs M1 + `mvp1_*_add_translate` claims.

## Preflight checklist for implementation worker

- [ ] C.1 types compile under M1 v4 emit gate
- [ ] C.2 migration ordered: rust → python → go → typescript (MW-D3 minimum), then remainder
- [ ] C.3 `dag.dag` constructors updated in same PR as C.1 or immediately after
- [ ] C.5 no remaining `concrete_syntax_token_kind_*` equality in `06_translate.dag`
- [ ] SG-0: no new hand-authored emit templates in `src/v3/compiler/src/emit*` (v4 authority)

## Handoffs (§6.7)

Notify RCA mgrs when C.5 PR opens — C.6–C.10 target realization can proceed in parallel once
C.1 row shapes are stable (interface freeze after PR review, not before).
