# Glob-discovery D1–D2: roster ≠ TestClaimRun — measured contradiction + System-B census

Status: REPORT / read-only census. No consumer repoint, no hand-roster delete, no marker
switch. Mgr-C (witty-pike-248) confirmed hold 2026-06-08.

## TL;DR

The work item asked to "discover TestClaim/TestClaimRun by resolved type, prove
**TestClaimRun discovered == v4_roster_pilot hand run-roster**." Measured against the real
corpus and the real resolver, that equality is **impossible as written**: the two sides are
disjoint *kinds*.

| side | kind | count |
|------|------|------:|
| `v4_roster_pilot` run-roster | `fn() -> Bool` witness functions (run via `gunbc run --claim-run`) | **38** rows |
| discovered `TestClaimRun` | `data x: TestClaimRun<..> = ..` declarations | **95** decls / 26 files |
| discovered `TestClaim` | `data x: TestClaim = <Variant>Claim { .. }` declarations | **540** decls |
| roster ∩ TestClaimRun decl-names | — | **0** |

All four figures are **resolved-type measurements** produced by the executable probe
`src/v2/tests/src/glob_discovery_testclaim_census_test.rs` (one `compile_to_resolved` over
the whole `src/v4` corpus, walking the resolved typed-module items; census restricted to
`src/v4/test/claim/`). Raw `grep` over `src/v4/test` reports ~97/547 — slightly higher
because it also counts type-def lines and decls in `src/v4/test` paths outside the
`claim/` census scope; the resolved-type figures above are authoritative.

This is an **architectural fork**, not a bug to patch:

* **System A** = `v4_roster_pilot.dag` — 38 hand-listed Bool witness functions, transported by
  `scripts/v4-testclaim-smoke-roster.sh`. (The file restates each row twice: a
  `V4RosterPilotClaimRunRow { .. }` data literal **and** a `v4_roster_pilot_row_matches(..)`
  composition-guard call — 76 entry/function text pairs, 38 distinct rows.)
* **System B** = the `TestClaimRun` / `TestClaim` *data-decl* corpus (95 + 540), evaluated by
  `run_test_claim*` and folded by `workflow/testclaim_corpus_runner.dag` over
  `manual_corpus_node_subject_rows`.

Discovering by resolved type `== TestClaimRun` recovers **none** of System A's witnesses;
discovering by resolved type `== TestClaim(Run)` finds 95/540 data decls System A never lists.

## Why D2 cannot be satisfied as written

`v4_roster_pilot` rows point at **functions**:

```
data v4_roster_pilot_row_mvp1_rust_emit_add_fn: V4RosterPilotClaimRunRow = V4RosterPilotClaimRunRow {
  label: "...", entry: "src/v4/test/claim/manual/mvp1_rust_add_translate.dag",
  function: "mvp1_rust_emit_add_fn_accepts_holds"   // <- a `fn() -> Bool`
}
```

`TestClaimRun` is a **data type** carrying a cache receipt + a verdict:

```
data run_mvp1_emit_add_fn_accepts: TestClaimRun<Node, Node> = TestClaimRun {
  cache: test_claim_cache_receipt(subject: subject_mvp1_emit_add_fn_accepts),
  verdict: run_test_claim_assert(claim: claim_mvp1_emit_add_fn_accepts, actual: .., at: ..)
}
```

The set of roster function names and the set of `TestClaimRun` decl names are disjoint
(intersection 0). No marker that discovers `TestClaimRun` declarations can reproduce the
function-witness roster.

## The marker-form fork (held for gate)

Two coherent re-scopings; **neither is implemented here** — this report is the decision input.

* **(A) Dissolve System A** — switch discovery to resolved **return type** (`fn() -> Bool`
  under `claim/**`) and prove `roster ⊆ discovered` (completeness: glob loses no hand-listed
  witness). This is what actually dissolves `v4_roster_pilot`, **but** it switches the
  discovery marker from the operator-marked TestClaim/TestClaimRun decls to Bool witness
  functions — a marker-form/design decision that contradicts the parent brief. **Requires
  Mgr-C/operator gate** (this is exactly the §7 marker-form escalation point in the glob
  design doc, ctrl#1480).
* **(B) Build the System-B census** (this PR) — the structural `TestClaim`/`TestClaimRun`
  data-decl census by resolved type, as the glob input to `testclaim_corpus_runner`, NOT tied
  to `v4_roster_pilot`. Stays within the operator marker. Touches no consumer.

## Rename-vs-wrap mapping for the 38 roster rows

The key migration-cost input: for each roster row, does its **entry file already carry a
`TestClaimRun` decl** ("ALIGN/RENAME" — a wrapper exists in-family, a future migration could
repoint discovery at it) or **none** ("WRAP" — a `TestClaimRun` wrapper would have to be
authored)?

| verdict | rows | meaning |
|---------|-----:|---------|
| ALIGN/RENAME | **3** | entry file already has ≥1 `TestClaimRun` decl |
| WRAP | **35** | entry file has no `TestClaimRun` decl |

ALIGN rows (entry file already carries TestClaimRun):

* `mvp1_rust_emit_add_fn_accepts_holds` — `manual/mvp1_rust_add_translate.dag`
* `ts_g2_sg1_symbol_carrier_holds` — `grounding_typescript/sg_claims.dag`
* `ts_g2_sg5_absence_fail_closed_holds` — `grounding_typescript/sg_claims.dag`

The other 35 rows (lens `sg_claims`, `extdeps_react/structural_receipts`, `std_text`,
`parse`, etc.) are WRAP: their witnesses are direct `fn() -> Bool` predicates with no
`TestClaimRun` data wrapper in the file.

> **Caveat — file-level vs name-level.** "ALIGN" is the *file-level* presence signal (the
> entry file contains some `TestClaimRun`). For a file with multiple `TestClaimRun` decls the
> exact witness↔decl pairing still needs author confirmation; ALIGN means "a wrapper exists
> to align to," not "this specific witness is already wrapped." The dominant finding stands
> regardless: **the vast majority (35/38) of System-A witnesses have no TestClaimRun wrapper
> at all** — so any move from System A to TestClaimRun-decl discovery is overwhelmingly a
> *wrap* migration, not a rename. That is the cost the gate decision turns on.

## What this PR contains (read-only)

* `docs/planning/glob-discovery-d2-roster-testclaimrun-mismatch-2026-06-08.md` — this report.
* `src/v2/tests/src/glob_discovery_testclaim_census_test.rs` — the executable census/probe:
  discovers `TestClaim`/`TestClaimRun` by resolved type across `src/v4/test/claim/**`
  (one real `compile_to_resolved` over the whole `src/v4` corpus, ~130s), pins the empty
  roster∩TestClaimRun intersection, and prints the rename-vs-wrap mapping. Standalone
  `#[test]` (dormant in CI by
  design — see [[project_v2_tests_not_run_broadly_in_ci]]); run with
  `cargo test -p v2-compiler-tests glob_discovery_testclaim_census -- --nocapture`.

No `compiler/` edits, no consumer repoint, no `v4_roster_pilot` / smoke-roster delete, no
marker-switch implementation. Affected-set rows wait on the same gate decision.
