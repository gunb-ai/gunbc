# R2 PB Tier 3 — Worker 1 termination mirror audit

**Status:** Implementation-ready audit (R2 PB Manager dispatch from
`docs/briefs/r2-pb-tier3-mirror-dissolution-workers.md`, Worker 1).

**Authority:** `src/v3/std/termination.dag` (v3 staged mirror of `dsl/std/termination.dag`).

**Rust scope:** `src/v3/compiler/src/dag.rs`, block annotated as the Rust execution mirror for
`src/v3/std/termination.dag` (from `DescentEvidence` through `map_evidence_merge_at`, immediately
before the computation mirror block).

## 1. Executive summary

- **Carrier types** (`DescentEvidence`, `RankingDimension`, `PositiveDescentAmount`,
  `ProportionalDivisor`, `DescentSource`, `TerminationProof`, `ProofEdge`) are **terminal typed
  spellings**: they mirror the `.dag` coproduct/record shapes and are already reflected in the
  bootstrap `Dag` (`m2_substrate_inhabitance_test::termination_carriers_bootstrap_from_v3_std`).
- **Lattice and numeric-bridge functions** are **executable scaffold**: every authoritative body in
  `std.termination` still lowers as `ArrowBody::Unparsed` in bootstrap (`bootstrap_generated*.rs`).
  The Rust copies in `dag.rs` exist because those bodies are not evaluated from `.dag` yet.
- **Dissolution trigger** (per `dag.rs` comment and `docs/r2-structure.md` / worker pack): std
  block bodies for `module std.termination` lower to an evaluable form, then lattice helpers can
  delegate to evaluated `.dag` or disappear behind a single evaluated boundary—without weakening
  fail-closed contracts.
- **No STOP** on this read: no new substrate shapes are required for the audit conclusions; Peano
  cap alignment is already explicit in Rust (`MAX_PEANO_MATERIALIZATION`) and `.dag`
  (`peano_literal_materialization_cap`).

## 2. Classification (terminal spelling vs executable scaffold)

| Rust symbol (in `dag.rs`) | Class | Rationale |
| --- | --- | --- |
| `DescentEvidence` | Terminal spelling | Same three-variant lattice as `.dag`; consumed downstream as durable evidence. |
| `RankingDimension` | Terminal spelling + **String bridge** | Matches `.dag`; `param: String` is the documented bootstrap bridge until structural param refs exist (worker pack STOP axis). |
| `PositiveDescentAmount`, `ProportionalDivisor` | Terminal spelling | Proof-grade witnesses; mirror `.dag`. |
| `DescentSource`, `TerminationProof`, `ProofEdge` | Terminal spelling + **String bridges** where `.dag` uses `String` for names | Mirror `.dag`; dissolution of name-like payloads is structural-ref work, not termination-only. |
| `positive_descent_count` | Scaffold (authority **not** `std.termination`) | Declared in `src/v3/std/computation.dag`; Rust mirror lives next to Peano types for call-site ergonomics. Dissolution tracks **Worker 2** (computation mirror) and shared inhabittance tests—not a `std.termination` body. |
| `proportional_divisor_to_int` | Scaffold | `.dag` `fn proportional_divisor_to_int`; bootstrap body `Unparsed`. |
| `MAX_PEANO_MATERIALIZATION`, `positive_amount_from_i64`, `proportional_divisor_from_i64` | Scaffold | Mirror `peano_literal_materialization_cap`, `positive_descent_amount_from_positive_int`, `proportional_divisor_from_int_at_least_two`; iterative Rust matches fail-closed cap semantics. |
| `evidence_rank`, `merge_evidence`, `join_evidence`, `promote_to_strict`, `optional_evidence_meet`, `map_evidence_merge_at` | Scaffold | Full `.dag` function bodies; all `ArrowBody::Unparsed` at bootstrap. |

`promote_to_strict`: `.dag` uses an explicit `match` that preserves the three variants; Rust uses
`evidence` passthrough—same observable behavior (fail-closed, no promotion to `Strict`).

## 3. Smallest path to evaluate or reflect lattice helpers

 Preconditions (already named elsewhere in the repo; this audit does not expand them):

1. **Lower** `std.termination` arrow bodies from `ArrowBody::Unparsed` to a representation the
   compiler can interpret (user-defined / lowered body), **or** invoke an existing evaluator on a
   parsed std surface for those decls.
2. **Wire** one host entry point (e.g. “evaluate `std.termination::merge_evidence` on two
   `DescentEvidence` values”) that returns the same `DescentEvidence` the Rust mirror returns today.
3. **Replace** Rust bodies with thin wrappers that marshal native `DescentEvidence` ↔ evaluated
   value, **or** delete Rust if all call sites can consume evaluated values only.

 Until step 1 exists for these decls, removing the Rust mirror would **not** dissolve the bridge
 (worker pack: deleting mirrors without an evaluated consumer path is invalid).

 Optional hardening (not blocking the plan): add a parity test row for
 `positive_amount_from_i64` / `proportional_divisor_from_i64` against symbolic expectations derived
 from `.dag` (today: lattice-only + carrier variant tests in `m2_substrate_inhabitance_test`).

## 4. Implementation-ready slice ordering (suggested queue)

1. **Receipt-preserving lattice swap** — First implementation PR should keep
   `termination_lattice_rust_mirror_matches_dag_authority` behavior identical while swapping the
   implementation source of truth to evaluated `.dag` once bodies lower (or add a shadow eval path
   and assert Rust == eval for the property matrix in that test).
2. **Peano bridges** — Second: delegate `MAX_PEANO_MATERIALIZATION` / materializers to evaluated
   `peano_literal_materialization_cap` and friends so numeric authority cannot drift (STOP if caps
   diverge during wiring).
3. **Type mirrors** — Leave in place until consumers read reflected types only; removing Rust enums
   before consumers migrate is out of scope for Tier 3 dissolution.

## 5. Test receipts (run / extend; no SG-0 census change for this audit doc)

| Receipt | Role |
| --- | --- |
| `m2_substrate_inhabitance_test::termination_carriers_bootstrap_from_v3_std` | Carrier shape parity bootstrap ↔ `.dag`. |
| `m2_substrate_inhabitance_test::termination_lattice_functions_preserve_std_body_spans` | Documents that lattice + `proportional_divisor_to_int` are still unparsed std bodies (dissolution trigger). |
| `m2_substrate_inhabitance_test::termination_lattice_rust_mirror_matches_dag_authority` | Behavioral ratchet for lattice helpers + `promote_to_strict`. |
| `m2_substrate_inhabitance_test::e_p_per_call_descent_evidence_side_table_reads_recursive_call` | Integration: `DescentEvidence` / shrink carriers flow through E-P side table (downstream consumer). |

**SG-0:** This audit adds documentation only; no change to
`src/v3/compiler/tests/integration/sg0_census_test.rs` expected. Implementation PRs in this slice
must report SG-0 deltas per worker pack rules.

## 6. STOP boundaries (unchanged from worker pack)

Escalate to Pure Bootstrap Manager if dissolution planning requires:

- New `ValueBody` / connective / evaluation surface not already covered by “lower unparsed std
  bodies” work,
- Structural parameter refs before the team accepts a narrower first slice,
- Any change to Peano materialization caps that is not a straight import of `.dag` authority.
