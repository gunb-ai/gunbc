# Branch C TargetModel Audit — C.1–C.5 Findings (Class 2 preflight)

> **Status:** PREFLIGHT ARCHIVE + **live residual scope** (P1 live-state, 2026-06-02).
> **Author:** Cross-target Emission Mgr (`silent-bear-54`), 2026-06-02.
> **Evidence:** Preflight at main pre-#4297; **live state** verified at main post-#4297 (`bf5f7102`).
> **Worksheet:** `v4-cross-target-emission-rr-c-worksheet-2026-06-02.md`.

## Live state (post-#4297 on `main`)

| Row | Scope | Status |
|-----|--------|--------|
| C.1 | `ConcreteSyntaxToken` in `target_model.dag` | ✅ Landed #4297 |
| C.2 | Per-language `*ConcreteSyntaxToken` coproducts | ✅ Dissolved #4297 (`rg 'type .*ConcreteSyntaxToken' src/v4/extdeps/languages` → no matches) |
| C.3a | `dag.dag` `emit_fixed_token` / `emit_bound_token` | ✅ Delegate `concrete_syntax_token_to_node` |
| C.3b | `grammar.dag` `grammar_concrete_*_token_node` | **OPEN** — parallel wire author at `:410` / `:429` |
| C.4 | Thin `05_emit` | ✅ Unchanged |
| C.5 | `06_translate` structural decode/match | ✅ Landed #4297 (`concrete_syntax_token_from_node`; no `concrete_token_kind_is_declared`) |
| C.6–C.10 | RCA target realization | Dispatch after G.0 + interface freeze (§6.7) |

**Active implementation guidance:** close **C.3b only** (substrate PR). Do not re-run C.1/C.2/C.5
dissolution — that work is on `main`. Sections below document **preflight findings** for audit trail.

## Executive summary (preflight, pre-#4297)

Branch C was blocked on **split authority** for concrete syntax tokens: per-language
`*ConcreteSyntaxToken` coproducts, a parallel **std grammar** constructor surface, and
Symbol-tagged `kind` edges in `06_translate`. #4297 closed all but **grammar.dag C.3b**.

## C.1 — Shared carrier

> **Preflight finding (historical):** carrier not yet declared. **Live:** declared at
> `target_model.dag:101` with 🟢 terminal-sum disposition and `concrete_syntax_token_to_node`.

```101:103:src/v4/std/target_model.dag
type ConcreteSyntaxToken
  = FixedToken { token_class: Symbol }
  | BoundToken { token_class: Symbol, binding: Symbol }
```

## C.2 — Per-language duplicate coproducts

> **Preflight finding (historical):** eleven parallel `*ConcreteSyntaxToken` type authorities.
> **Live:** dissolved — per-language `*_concrete_token_node` import shared carrier and delegate to
> `concrete_syntax_token_to_node` (no `type .*ConcreteSyntaxToken` under `extdeps/languages`).

## C.3 — Constructor alignment (`dag.dag` + `grammar.dag`)

### C.3a — `extdeps/languages/dag.dag` ✅ on main

**Preflight:** `emit_fixed_token` / `emit_bound_token` were untyped Conj builders.

**Live:** Typed `FixedToken` / `BoundToken` + `concrete_syntax_token_to_node` (#4297).

### C.3b — `std/grammar.dag` — **OPEN (active scope)**

**Live finding:** `grammar_concrete_fixed_token_node` and `grammar_concrete_bound_token_node`
hand-build the `concrete_syntax_token_field_*` + `concrete_syntax_token_kind_*` Conj shape — a
**second substrate constructor authority** (P2/P5):

```410:426:src/v4/std/grammar.dag
fn grammar_concrete_fixed_token_node(token_class: Symbol) -> Node {
  Node {
    kind: TypeNode { connective: Conj },
    children: [
      grammar_named_edge(
        name: concrete_syntax_token_field_kind,
        target: grammar_atom(identity: concrete_syntax_token_kind_fixed)
      ),
      grammar_named_edge(
        name: concrete_syntax_token_field_class,
        target: grammar_atom(identity: token_class)
      )
    ]
  }
}
```

(`grammar_concrete_bound_token_node` at `:429`; call sites `:457` / `:462`.)

**Action (dispatch):** Import `FixedToken`, `BoundToken`, `concrete_syntax_token_to_node`;
delegate both helpers through the typed carrier (mirror post-#4297 `dag.dag`). Substrate PR;
gate with M1 v4 emit + smoke rows touching grammar relations.

## C.4 — `05_emit` thin orchestrator ✅

Unchanged on main — `emit = translate ∘ serialize_target` (§2.2.1 option ii). No action.

## C.5 — `06_translate` structural match ✅ on main

> **Preflight finding (historical):** Symbol-equality via `concrete_token_kind_is_declared`.
> **Live:** `concrete_syntax_token_from_node` decode boundary + structural dispatch (#4297).

## Active dispatch checklist (live)

- [ ] **C.3b** `grammar.dag` `grammar_concrete_fixed_token_node` / `grammar_concrete_bound_token_node` → `concrete_syntax_token_to_node`
- [x] C.1–C.2, C.3a, C.5 — on `main` (#4297); do not re-dispatch
- [x] C.4 — thin `05_emit` preserved
- [ ] C.6–C.10 — RCA mgr lane after G.0 + C.3b close (§6.7 interface freeze sent)

## Handoffs (§6.7)

C.1 row shapes stable on `main` (#4297). RCA mgrs consume shared `ConcreteSyntaxToken`; no
per-language coproduct revival. Remaining substrate: **C.3b** before declaring C.3 complete.
