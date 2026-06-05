# v4 Claim-Corpus Execution Map — snapshot 2026-06-04

**Dated measurement snapshot, not a maintained ledger.** Regenerate from source
with `scripts/v4-claim-corpus-execution-map.sh`; the marks in the witness `.dag`
files remain authoritative. This records *what executed* when the whole claim
corpus was run on 2026-06-04, so the green/red/error shape can guide where
interpreter/substrate effort pays off.

## Method

Every Bool witness in `src/v4/test/claim/**` was *run* via the existing
single-witness CLI (`gunbc run --claim-run --entry <file> --function <name>`),
looped — no new batch layer. Two witness classes, **deduplicated**:

- **fn** — `fn name() -> Bool` (nullary predicate functions): all 291 run.
- **data** — `data name: Bool = expr` (the runner evaluates these as zero-arg
  thunks): run **only when distinct**. A `data w: Bool = foo()` that just rebinds a
  same-file nullary fn `foo` is skipped — running it re-evaluates a fn already
  counted under **fn**. Of 137 total data witnesses, 51 are such rebindings
  (skipped) and **86 are distinct** (58 in files with no fn + 28 in fn-bearing
  files). 291 fn + 86 distinct data = **377 distinct witnesses**.

Discipline that keeps the map honest:

- **Classify by output text, not exit code** — a missing-function run also exits 1,
  so `true`/`false`/error is read from stdout, not `$?`.
- **Mechanical perf cap (60 s wall + 6 GiB vmem), not a category allowlist** — any
  witness over the cap self-classifies `PERF` (keystone perf track), so the
  boundary cannot drift and the sweep never fights the perf wall by curation.
- **claim vs helper tag** — a RED *helper* (e.g. `compose_assoc_lhs`) is a law
  *component* that legitimately returns false; only RED *claims* are findings.
- Sequential execution respects the container pids-cgroup fork ceiling.

This is E-10 applied corpus-wide: most of these witnesses had **never executed**.
Running them is the disease-detection. **The map is the deliverable — not
"fix every red."**

## Headline — 377 distinct witnesses executed

| status | meaning | fn (291) | data: fn-less files (58) | data: distinct in fn files (28) | **total** |
|--------|---------|------:|------:|------:|------:|
| **GREEN**  | executed and holds | 151 | 44 | 25 | **220** |
| **RED**    | executed and **false** | 34 | 4 | 1 | **39** |
| **ERROR**  | never executed (infra/interp) | 96 | 9 | 2 | **107** |
| **PERF**   | over 60 s/6 GiB cap (keystone) | 10 | 1 | 0 | **11** |
| no-witness | pure library/roster, no Bool claim | — | — | — | **109** |

~58% green, ~28% error, ~10% red, ~3% perf across everything that asserts a Bool.
The data class closes what a per-fn-only sweep would miss: of the 129 files with no
nullary `()->Bool`, **20 held 58 runnable data witnesses**; a further **28 distinct
data witnesses live in fn-bearing files** (not rebindings of their fns); the
remaining **109 files are genuinely claim-free** (rosters, manifests, helpers).

claim-vs-helper split (fn pass): GREEN 140 claim / 11 helper; RED 29 claim / 5
helper; ERROR 82 claim / 14 helper; PERF 9 / 1.

## The big finding: 107 ERRORs collapse to ~9 root causes

The error bucket is not 107 problems. It is a short list of shared
interpreter/builtin gaps, each blocking many witnesses. Fix the gap, unblock the
fan-out (the table below totals ~103; the 28 distinct-data witnesses added 2 more
errors that fall into the existing `ClassifiedDependencyView` and coercion classes,
plus 1 new RED — `infer_bounded_lattice_completeness_anchor`):

| n | root cause | class | where |
|--:|------------|-------|-------|
| 22 | **`contains()` / `contains sub` is String/Set-only** — called on List/Variant/Record | builtin gap | lens_affected_set, workflow/ci_*, manual/bootstrap, grammar fixtures |
| 14 | **`ClassifiedDependencyView` fold non-exhaustive** — missing arms (BindingResolved, FactsLookupMiss, DataDependent, RequiresAccessWitness, …) | non-exhaustive | lens_effect/idempotency/ownership/parallelism/structural_resolution/unused_parameters, workflow |
| 13 | **`undefined variable: left`** — one TS type-projection helper references an unbound var | bug | grounding_typescript, manual/sg2_typescript |
| 10 | **component-path classifier non-exhaustive** (`src/v2/…`) | non-exhaustive | workflow/ci_component_affected (whole file) |
| 10 | misc variant folds non-exhaustive (Diagnostic, Generator, UnusedParameterFact, null) | non-exhaustive | manual, name_resolve, round_trip, lens_* |
| 9 | **`error type cascade`** — downstream of a primary type error (not independent) | cascade | manual/sg_rc_layering, manual/sg2 |
| 8 | **`eq: fn(x,y){…}` callbacks non-exhaustive** on `prefix_sym_*` / `zip_eq_sym_*` | non-exhaustive | algebra_laws/is_prefix_of, zip_eq_list_equality |
| 5 | **TestClaim/`EqualsClaim` variant fold non-exhaustive** | non-exhaustive | workflow/affected_set_ci_runner, ci_consumer_node_precise |
| 5 | **`method 'lookup'` not implemented** (interpreter gap, T-22 family) | interp gap | lens_affected_set, grounding_go, manual/sg_rc |
| 4 | `undefined variable: p` / `node` (unbound lambda vars) | bug | lens_application/subterm, manual/sg_rc round-trip |
| 3 | grammar parse match missing go/py/kotlin source surface | non-exhaustive | manual/*_mvp1_grammar_claim |

**Materiality read:** the top three rows (`contains` String/Set-only,
`ClassifiedDependencyView` fold, TS `left`) account for ~49 of the 107 errors and
are exactly what E-10 should surface — load-bearing lens families (affected_set,
ownership, structural_resolution) whose witnesses are specification-only because the
predicate they call doesn't yet run. The three have *different* fix locations,
detailed in the routing section below — `contains` is an interpreter builtin,
`ClassifiedDependencyView` is a `.dag` missing match-arm, TS `left` is in the
translate serialize path — so "top three by fan-out" is not "three fixes in one
place." The two interpreter-side gaps are concrete:

- `contains`/`has` only handles String (substring) and Set (membership)
  receivers — `src/v2/stage0/src/v2_interpreter.rs:1723`; a List/Variant/Record
  receiver falls through to the String branch and `expect_str` rejects it.
- the `error type cascade` (9) and `null`/empty matches are downstream — fix the
  primaries and re-run; they likely auto-resolve.

### Routing the error fan-out by WHERE the fix lives

The fan-out is the gold, but the fixes split by location — and the split decides
keystone-lane vs clean-parallel. One check routes each cause:
`InterpError::PatternMatchFailure` ("non-exhaustive pattern match") is raised when
the interpreter evaluates a **`.dag` match with no covering arm** — so every
non-exhaustive cluster is a `.dag` missing-arm fix, *not* a Rust `match` gap.
`TypeError`/`Unimplemented` (`contains`, `lookup`, `atom_identity_hash`) are
interpreter builtins. `undefined variable` is checked at its source.

**Keystone-lane (~54) — on the round-grind's path; prioritize within that lane by
this menu, do NOT spawn a second editor on these files:**

- `contains` List/Variant/Record support (22) + `method 'lookup'` (5) +
  `atom_identity_hash` arity (1) — interpreter builtins in `v2_interpreter.rs`
  (the file the keystone round-grind edits).
- `error type cascade` (9) — downstream of a primary type error; re-run after the above.
- `undefined variable: left`/`p`/`node` (17) — the failing TS witnesses call
  `target_serialize_source_from_model`, which lives in
  **`src/v4/compiler/06_translate.dag`** (a load-bearing pipeline stage the keystone
  owns). On the keystone's path regardless of whether the precise cause is
  interpreter capture or `.dag` scoping — so it's the keystone's to pin. (Exact
  `left` origin not yet localized; flagged for that owner.)

**Clean-parallel (~50) — `.dag` missing arms in lens/witness files, off every
load-bearing pipeline stage (confirmed: none of these matches live in
emit/lower/infer/parse/translate):**

- `ClassifiedDependencyView` fold (14) — missing classification arms in
  `src/v4/lens/{effect,ownership,parallelism,idempotency,structural_resolution,unused_parameters}.dag`.
- component-path classifier `src/v2/…` (10), `EqualsClaim` fold (5),
  `prefix_sym`/`zip_eq` eq-callbacks (8), grammar go/py/kotlin surfaces (3), misc
  variant folds (10) — all `.dag` matches in lens or witness files.

This **inverts the naive read**: it is not "fix the interpreter and everything
unblocks." Roughly half the error fan-out is `.dag`-side missing arms in lens/test
files — genuinely parallelizable work off the keystone's path. The other half is
keystone-lane (interpreter builtins + the translate serialize path) and should be
sequenced within that lane, not double-edited.

## RED claims (39) — behavioral, executed-and-false

Clusters dominate; triage by materiality:

- **8 — `manual/test_claim_cache_digest_sensitivity.dag`**: a family of
  `*_digest_differs` claims asserting two cache digests differ, but they compare
  **equal** → either a digest-collision / cache-key-sensitivity bug or the witnesses
  encode the wrong expectation. One file, one root cause — highest-value RED.
- **4 — `manual/model_core_wave1_anchor.dag`**: wave-1 model-core contract claims
  (bool encoding, unbound-axes fail-closed, primitives, payload) all false.
- **4 (pass 2) — v4 evaluator / find_witness runtime**: `eval_runtime_mvp ::
  witness_eval_mvp2_add_accepts_five`, `find_witness_distinct_node_zip_fold ::
  witness_coercion_fold_distinct_node_accepts`, `find_witness_identity_mvp ::
  witness_find_witness_other_rule_rejects`, `v4_evaluator_runtime_anchor ::
  witness_v4_evaluator_model_core_wires_wave1` — touch the eval/coercion-witness
  path; worth a look alongside the keystone's eval work.
- Singletons: `claim_pipeline/infer.dag :: spine_infer_accepts_mvp_resolved_tree`
  (infer spine rejecting an MVP tree it should accept — its sibling `translate` is
  PERF, so this whole pipeline lane is the keystone's), `lens_affected_set/irt1_*`
  and `lens_application/substitute_at_depth_*` receipts, `lens_fact_density`
  hollow-alias rejections.

RED **helpers** (5: field_patch_monoid lhs/rhs, infer spine fixture, sg2 golden,
apply_lens rhs) are law components — benign unless their parent `_holds` is also red.

## PERF (11) — keystone track, do NOT fix here

`claim_pipeline/translate.dag` (the keystone's own file — self-classified off, as
predicted), `manual/refinement_authoritative_constants` (6 witnesses),
`manual/sg2_type_expression_projection :: mvp1_translate_fallback`,
`manual/typescript_derive_grammar_relation_row_round_trip` (2),
`manual/rust_wave2_grammar_structure`. These hit the import-closure perf wall —
they belong to the keystone's perf track, not this map's findings.

## Recommended sequence (materiality ordering; lanes per the routing section above)

This is a *priority ordering*, not a re-classification — lane assignment is the
"Routing by WHERE the fix lives" section above; do not re-route here.

1. **Keystone-lane interpreter builtins (~28)** — `contains` List/Variant/Record
   support (`v2_interpreter.rs:1723`) + `method 'lookup'` (5) + `atom_identity_hash`
   arity (1). Highest single-fix fan-out; load-bearing interpreter, so
   model-before-implement, not a spot patch.
2. **Clean-parallel `.dag` missing arms (~50)** — `ClassifiedDependencyView` fold
   arms (14) in `src/v4/lens/*.dag`, plus component-path (10), `EqualsClaim` (5),
   `prefix_sym`/`zip_eq` (8), grammar surfaces (3), misc folds (10). Off the
   keystone path → parallelizable now, independent of lane 1.
3. **Keystone-lane translate path (17)** — TS `left`/`p`/`node` via
   `06_translate.dag` serialize path (exact origin to be pinned by that owner).
4. **`error type cascade` (9)** — re-run after lane 1; likely auto-resolves.
5. **RED investigation** — start with `test_claim_cache_digest_sensitivity` (8,
   possible digest bug) and `model_core_wave1_anchor` (4).
6. **PERF (11)** → keystone perf track.

## Reproduce

```bash
cargo build --release -p v2-compiler --bin gunbc
scripts/v4-claim-corpus-execution-map.sh          # writes .claim-map/results.tsv (gitignored)
```

Output is a TSV (`status, class, file, witness, secs, exit, lastline`); the
trailing `status x class` summary prints the headline tally.
