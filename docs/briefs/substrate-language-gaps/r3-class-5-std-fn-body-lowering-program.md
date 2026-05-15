# R3 — class-5 STD `fn` body lowering program (substrate receipts)

Brief anchor: PATH X POC — make std-owned predicates such as `dsl/std/unicode.dag::char_in_class` executable in the bootstrap Dag by authoring **`fn ... -> T = <expr>`** (expression-bodied surfaces) rather than relying on host **`is_ascii_*`** bridges.

## Bootstrap snapshot compass (counts drift across regen)

At HEAD after regen bootstrap, **`bootstrap_generated.rs`** order-of-magnitude counts:

| `ArrowBody` variant | ~count |
| --- | --- |
| `Unparsed` | 159 |
| `UserDefined` | 117 |

`char_in_class` is **`UserDefined`**; long-form brace bodies in staged `.dag` still parse-skip unless opted in elsewhere (see sibling note `dag-fn-brace-body-parse-opt-in.md`).

## Architectural bridges (existing design)

- **DB-16 / external-body reconciliation:** `docs/design-fn-external-body-reconciliation.md` — how authoritative bodies reconcile with tooling surfaces.
- **Parse-time brace choice:** `src/v3/compiler/src/parse_generated.rs` — **`fn_brace_body_parse_as_expression`** and related scaffolding control whether a brace-bodied `fn` is retained as **`Unparsed`** vs lowered as real surface AST.
- **Substrate corpus lock:** **`m17_dag_corpus_brace_fn_stays_fn_external_body_at_parse_time`** (`src/v3/compiler/tests/integration/m1_substrate_test.rs`) — brace `fn` in the corpus remains external-body shaped at parse time until explicitly migrated.
- **Compile-time proofs lane:** `docs/lane2-compile-time-proofs.md` (~§200 vicinity) — proof obligations interacting with authoritative std bodies.
- **Dimensional abstention framing:** `docs/design-dimension-abstraction.md` (~§311 vicinity) — when nominal brands / dimensions interact with lexical classes.
- **Lens fold prerequisites:** `docs/design-lens-fold-prerequisites.md` — downstream consumers assuming particular body shapes during lens traversal.

This folder exists so **tokenize** and **substrate scaffolding** prose can cite one stable location for character-class/std-body work without duplicating long ROADMAP threads.
