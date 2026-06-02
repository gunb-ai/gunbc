# v4 Cross-Target Emission Round-Robin Worksheet (RR-C)

> **Status:** RATIFIED FOR W2 DISPATCH — Branch C design closure (ctrl#1425 §6.8, 2026-06-02).
> **Work item:** `node://adhoc-b4e8b554-bae` — Cross-target Emission Mgr (`silent-bear-54`).
> **Gate:** RR-C + C.1–C.5 audit before C.6–C.10 RCA handoffs; `05_emit` stays thin per §2.2.1 option ii.

## §10.0-adapted worksheet

```text
Migration class:        C-OMNI-EMISSION (Shape-A cross-target: TargetModel + translate + serialize)
Representative failure:  Ten parallel *ConcreteSyntaxToken coproducts (Rust/Python/Go/…) while
                         06_translate discriminates token kind via Symbol equality on
                         concrete_syntax_token_kind_{fixed,bound}; 05_emit grows ad-hoc render
                         paths instead of translate ∘ serialize_target.
Immediate local patch:   Per-language symbol-equality branches in 06_translate; duplicate emit
                         helpers in 05_emit; RCA mgrs fork template strings beside target_model.
Why forbidden:           P2 single-authority — target vocabulary lives in target_model.dag;
                         emission authority is 06_translate + grammar-inverse serialize (C.4);
                         cross-target parity requires one structural token carrier (C.1–C.3).
DFS path:
  C.1 substrate:
    - src/v4/std/target_model.dag — declare ConcreteSyntaxToken = FixedToken | BoundToken
      (dissolve concrete_syntax_token_kind_* Symbol discriminators)
  C.2–C.3 dissolution bar (sunny-owl-821 / §2.8.3 pull-forward):
    - src/v4/extdeps/languages/*.dag — retire per-target *ConcreteSyntaxToken coproducts when
      C.1 lands; grammar rows consume shared carrier via target-local wrappers only if
      MODELING M9 requires a thin alias, not a second coproduct authority
  C.4 orchestration:
    - src/v4/compiler/05_emit.dag — translate ∘ serialize_target ONLY (no new substantive paths)
  C.5 consumer:
    - src/v4/compiler/06_translate.dag — structural match on typed token carrier;
      delete concrete_token_kind_is_declared Symbol gate (~624–629, ~917–959)
  C.6–C.10 realization (W3.2 — RCA mgr lane):
    - vivid-lynx (Rust) / vivid-eagle (Python) / gentle-lynx (Go) / swift-fox (TS)
    - Per-target SG rows + host receipts; coordinate on PR open (§6.7)
Deepest unsound boundary:
  Symbol-kind equality on token nodes passes shape checks while per-language coproducts drift
  (fixed vs bound payload shape) — cross-target serialize inverts the wrong grammar row.
Systemic fix:
  Single typed ConcreteSyntaxToken in target_model; dag.dag emit_fixed_token/emit_bound_token
  construct that carrier; 06_translate + grammar.dag share structural morphisms; RCA population
  reads TargetModel edges only.
Non-goals:
  - Shape B compiler emitters (Branch D / §2.3.1 GUARDED).
  - Duplicate per-language 05_emit_* modules or v2-style template tables in compiler/.
  - Conflating TargetSource, TargetNodeTree, and JSON receipts (§2.7.4).
Falsification probe:
  CompilesClaim on mvp1_*_add_translate for rust/python/go/typescript after C.5;
  translate rejects tokens whose kind edge is not a FixedToken/BoundToken variant (fail-closed).
Metric allowed only as secondary:
  Count of per-language *ConcreteSyntaxToken type declarations (target: 0 post-C.2); not a merge gate.
```

## §1 Branch C row map (ctrl#1425 §3)

| Row | Deliverable | Authority / owner |
|-----|-------------|-------------------|
| C.1 | Shared `ConcreteSyntaxToken` coproduct | `v4.std.target_model` |
| C.2 | Per-language coproduct dissolution bar | extdeps/languages (sunny-owl spec) |
| C.3 | `emit_fixed_token` / `emit_bound_token` typed constructors | `v4.extdeps.languages.dag` |
| C.4 | Thin `05_emit` (translate ∘ serialize) | `v4.compiler.emit` — **no expansion** |
| C.5 | `06_translate` structural token match | `v4.compiler.translate` |
| C.6–C.8 | Rust / Python / Go realization receipts | RCA mgrs |
| C.9–C.10 | TypeScript + extended targets | swift-fox + follow-on |

## §2 Classifier (§2.9.4)

| Dispatch | Class | Item |
|----------|-------|------|
| This worksheet | 1 | RR-C |
| `v4-branch-c-targetmodel-audit-c1-c5-2026-06-02.md` | 2 | C.1–C.5 audit → implementation worker |
| C.6–C.10 | 2/3 | After C.5 green + RCA preflight |

## §3 Cross-branch consumers

- **Branch G:** `PerTargetGroundingReceipt` host verification (RR-G landed #4287).
- **Branch D:** D.1–D.2 format contract after C.1 + G.1 (W4.6).
- **MW-D3:** Rust + Python + Go release-minimum cross-target set for L5/L6 proofs.

## §4 Test plan

- M1 v4 emit — full `src/v4` compile after substrate edits
- Existing `test/claim/manual/mvp1_*_add_translate.dag` rows stay green
- Add C.5 regression claim if translate diagnostic surface changes (worker scope)
