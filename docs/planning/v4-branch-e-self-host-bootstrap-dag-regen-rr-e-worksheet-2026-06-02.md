# v4 Branch E Self-Host / Bootstrap / DAG-Regen Worksheet (RR-E)

> **Status:** RATIFIED FOR W2 DESIGN ONLY — no Class 2 substrate or self-host implementation.
> **Work item:** `node://adhoc-ed410c90-7cb` — Branch E Mgr (`royal-badger-408`).
> **Gate:** E.1–E.4 implementation waits for W3–W6 consumers and Branch H canonical `.dag` source authority.

## §10.0-adapted worksheet

```text
Migration class:        E-SELF-HOST-BOOTSTRAP-DAG-REGEN (bootstrap-as-data + H.1 canonical .dag source regen)
Representative failure:  Treating `gunbc compile --target dag` as a canonical source regenerator when
                         it currently emits a JSON IR dump; or regenerating `.dag` from emitted Rust,
                         which is Rust→DAG decompilation and belongs to R5, not Branch E W2.
Immediate local patch:   Add bootstrap/script glue that diffs emitted Rust or JSON dumps, then stamps
                         success as "DAG regen" without a canonical `.dag` serializer.
Why forbidden:           INVARIANTS P2/P5 — source authority would split between `.dag`, JSON IR, and
                         emitted Rust. Branch E must consume canonical H.1 `.dag` source AST facts
                         from Branch H, not invent a parallel serializer or decompiler.
DFS path:
  E.1 stage0 self-host rows:
    - Model the stage0 compiler-of-record chain as data; implementation waits until stage rows
      consume canonical source artifacts, not emitted Rust.
  E.2 bootstrap-as-data rows:
    - `src/v4/workflow/bootstrap.dag` remains the owner for stage ordering, content hashes,
      fixed-point checks, generated artifacts, and v2-as-seed status.
  E.3 DAG-regen rows:
    - E3.1 is the foundational gap: canonical H.1 `.dag` source serialization is not live.
    - E3.4/E3.5/E3.6 are blocked until E3.1 lands through Branch H source authority.
    - Minimal vertical slice: ONE small compiler module proves
      `source.dag -> H.1 source AST/IR -> canonical_source.dag -> H.1 source AST/IR`
      with normalized H.1 AST equality.
  E.4 hand-maintained dissolution rows:
    - Dissolve hand-maintained Rust only after the corresponding `.dag` source and bootstrap
      receipts are single-authority; census pressure is downstream, not the migration path.
Deepest unsound boundary:
  A JSON IR dump or post-infer semantic IR can be mechanically stable while still not preserving
  canonical `.dag` source text spelling, module imports, comments/headers, or source-authority paths.
  Calling it regen would hide the missing H.1 source serializer behind a successful fixed-point hash.
Systemic fix:
  Branch H defines canonical `.dag` serializer contract and first vertical slice; Branch E consumes
  that contract for bootstrap-as-data and DAG-source regeneration. Branch C C.1-C.5 supplies the
  Shape-A TargetModel + `06_translate` structural-match / grammar-inverse substrate needed for
  scaling DAG regen after C.5 is green; Branch E is a downstream consumer, not a translate owner.
Non-goals:
  - No Rust→DAG decompilation in Branch E W2 (R5 only).
  - No full self-host implementation or stage0 replacement in W2.
  - No new bootstrap substrate rows before the Class 1 worksheet is accepted.
  - No parallel `.dag` source authority outside Branch H.
Falsification probe:
  Given one small compiler module, canonical DAG regen must prove normalized H.1 AST equality after
  source.dag -> H.1 source AST/IR -> canonical_source.dag -> H.1 source AST/IR and must fail closed
  if the current output is only JSON IR. Semantic IR equality is secondary and cannot replace the
  source-law receipt.
Metric allowed only as secondary:
  Count of hand-maintained bootstrap/script files or regenerated artifacts; secondary to the
  normalized H.1 AST source round-trip proof and fixed-point receipt.
```

## Branch E Row Map

| Row | Design closure | W2 disposition |
|-----|----------------|----------------|
| E.1 | Stage0 self-host rows | Design only; waits for canonical H.1 `.dag` source artifacts |
| E.2 | Bootstrap-as-data rows | `bootstrap.dag` is authority; no new W2 substrate rows |
| E.3 | DAG-regen rows | E3.1 canonical H.1 source serializer blocks E3.4/E3.5/E3.6 |
| E.4 | Hand-maintained dissolution rows | Downstream of source + bootstrap receipts |

## Dependency Boundaries

- **Branch H / Source authority (`swift-lark-66`):** owns canonical `.dag` serializer contract and
  H.7.2 vertical slice, gated on H.7.1/RR-H worksheet ratification in PR #4291. The serializer is
  over the H.1 source AST and prints deterministic parser-accepted source text; it is not
  `dag-artifact.json`, post-infer semantic IR, or a JSON pretty-printer. Branch E must consume that
  contract rather than parallel-authoring source.
- **Cross-target Emission (`silent-bear-54`):** owns C.1-C.5: shared `ConcreteSyntaxToken`,
  `06_translate` structural match, and C.4's `05_emit = translate ∘ serialize` boundary.
  Branch E consumes C.5 green for Shape-A DAG-regen scaling and must not expand `05_emit` or
  implement `06_translate` in W2.
- **RR-D Shape-B guarded boundary:** orthogonal to Branch E. DAG regen must not pull OpenAPI,
  SQL, React, or other Shape-B formats/frameworks into compiler emit paths.
- **T-15 fixed point (W6.1):** consumes this design once canonical source regen and bootstrap
  implementation receipts exist.

## W2 Acceptance Checklist

- [x] RR-E records DAG regen from `.dag` source/IR, not emitted Rust.
- [x] E3.1 canonical H.1 `.dag` source serialization is named as the blocker.
- [x] Minimal E3 vertical slice is one small compiler module plus normalized H.1 AST round-trip test.
- [x] No Class 2 substrate, Rust implementation, or self-host stage replacement in this PR.
