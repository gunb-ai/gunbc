# R3 Debt-Paydown Baseline

Status: DISPATCH-READY baseline audit for the R3 Debt-Paydown program.
Date: 2026-05-02.
Scope: current `ROADMAP.md` tracked-debt rows, with recent landed PR cross-reference through PR #1515.

## Purpose

This audit gives the standing R3 Debt-Paydown program a first operating baseline:

1. catalogue live tracked-debt rows by lane/owner and introduction window;
2. identify rows that may be de facto advanced or retired by recent landed PRs but still need an explicit closure receipt;
3. rank the highest-leverage retirement targets for the next cycle;
4. seed velocity-tripwire reporting for the introduction:dissolution ratio discipline in `INVARIANTS.md` P5(c).

This is an audit/spec artifact. It does not retire any ROADMAP row by itself.

## Baseline Sources

- `ROADMAP.md` `Tracked debts -- 2026-04 analyses`, post-merge debt waves, and 2026-05-01 R3 analysis ingestion.
- PR #1430, which folded the 2026-05-01 paired exploratory + reflective analyses into ROADMAP.
- PR #1480, which added the R3 Debt-Paydown standing program and expanded R3 scope to 16 lanes.
- Recent merged PRs #1431-#1515, used only as evidence for possible row advancement.

## Active Rows By Owner

| Owner / lane | Live rows | Introduction window | Notes |
|---|---:|---|---|
| R3 Substrate | 18 | 2026-04-20 through 2026-05-01 | Dominant bucket. Includes parser/data fail-closed bugs, substrate mirror drift, operator ontology, algebra-law violations, `Json`/`Bytes` disposition, and numeric-adjacent substrate rows. |
| R3 Verification | 9 | 2026-04-30 through 2026-05-01 | Mostly witness/execution debt: `BinaryDimensionReportEquals`, BridgeLedgerZero decreasing ratchet, algebra-law coverage, diagnostics-empty bootstrap gates, generated TestClaim replacement pressure. |
| R3 Grounding | 8 | 2026-04-21 through 2026-05-01 | LanguageSpec / services / method-template consumer migration, provider/API mirror multiplication, parser-grammar syntax authority drift, Track-13 table retirement. |
| R3 Pure Bootstrap | 7 | 2026-04-20 through 2026-05-01 | SG-0 and v2-retirement surfaces: `parse_parser_body.txt`, test-runner arms, hand-Rust acceptance, `dsl/v3/std/emit_model.dag` facade, BinShim/regen-lens retirement chain. |
| R3 Evaluator | 4 | 2026-04-25 through 2026-05-01 | Lens fold execution bridge, E7 symbolic-cost-only handoff, runner output-producer scaffolds, complexity/evaluator execution gates. |
| Unassigned / manager-disposition | 6 | 2026-04-20 through 2026-05-01 | Old P0/P1/P2 rows with no current named R3 owner, plus `Json`/`Bytes` lane-disposition placeholder. These should be assigned or explicitly closed as obsolete. |

The count is intentionally coarse: consolidated rows such as "B4 bridge-retirement queue" and "test_runner.rs becoming a parallel predicate authority" represent multiple concrete sites but one dispatch surface.

## Rows With Multiple Unretired Entries

### Substrate: highest concentration

- Parser/data fail-closed: duplicate record-literal fields, top-level `ValueBody` shape/mirror drift, `FieldMap` duplicate-free invariant drift, `ValueBody` Rust-vs-`.dag` isomorphism gap.
- Operator/algebra: `resolve_operator_arrow` synthetic fallback, `??`/`%` syntax/parser/operator drift, SymbolicCost semiring annihilation violation, SubValueRelation false `BoundedLattice` claim, hand-rolled lattice instances.
- Numeric/modeling: surface int-literal host narrowing, Peano witness carrier bridge, `PRELUDE_BARE_RHS_ALIAS_IDENTS` drift, `Json`/`Bytes` decomposition placeholder.

### Verification: executable-gate backlog

- `BinaryDimensionReportEquals` still has author-now/fire-later consumers.
- BridgeLedgerZero reports known-open rows but does not yet enforce decreasing count.
- Algebra-law witness coverage is not exhaustive across declared inhabitants.
- Bootstrap fixture shape tests can pass while the Dag still carries diagnostics.

### Grounding: parallel authority backlog

- LanguageSpec rows landed faster than old method-template consumers retired.
- `CollectionOps` / `StringOps` / `MapOps` duplicate algebra operation identity as target-string templates.
- Service/provider schemas risk accumulating per-provider mirrors before shared service ingestion lands.
- Track-13 tables and `carrier: String` remain pending final dissolve.

### Pure Bootstrap: SG-0/test-runner pressure

- `parse_parser_body.txt` remains a large hand-authored parser scaffold.
- `test_runner.rs` keeps accumulating bespoke predicate arms.
- Hand-Rust test files are growing faster than generated TestClaim replacement.
- BinShim/regen-lens retirement is actively briefed but not yet retired.

## Row Catalogue

| Row | Owner / lane | Introduced | Status |
|---|---|---:|---|
| `render.dag` `repeat_string` ignores `n` | Unassigned / legacy P0 | 2026-04 analyses | Open unless superseded by P0 receipt. |
| REST_OPS `CreateComment` test-table drift | Unassigned / legacy P0 | 2026-04 analyses | Open unless extdep-consuming test receipt exists. |
| `__BUG_NO_PROFILE_...` fabrication sentinel | Unassigned / legacy P0 | 2026-04 analyses | Open unless Option/Result rewrite receipt exists. |
| `http_path.dag` `None => ""` fabrication | R3 Substrate | 2026-04 analyses | Open. |
| `effects.dag` reconstructs typed method/path from strings | R3 Substrate | 2026-04 analyses | Open; related to transport typing. |
| Forgeable `ResourceHandle` opacity claim | R3 Substrate | 2026-04 analyses | Open. |
| Four hand-rolled lattice instances | R3 Substrate + Verification | 2026-04 analyses | Open; sharpened by 2026-05-01 SubValueRelation finding. |
| `languages.dag` vs per-target `emit.dag` language facts | R3 Grounding | 2026-04 analyses | Open; overlaps LanguageSpec and services parser-grammar. |
| Triple `MethodTranslation` schema | R3 Grounding / PB | 2026-04 analyses | Retired by PR #1210; do not count as live except downstream method-template consumers. |
| `effects.dag` dual authority (`dsl/std` vs `src/v3/std`) | R3 Substrate | 2026-04 analyses | Open. |
| `container_template_algebra_rows` duplicate aliases | R3 Substrate | 2026-04 analyses | Open; blocked on alias reflection. |
| Parallel string-keyed authorities in `types.dag` + `coercion.dag` | R3 Grounding | 2026-04 analyses | Open; Track-13 / LanguageSpec dissolve. |
| `declaration_by_name(...)` in emit | R3 Substrate / PB | 2026-04 analyses | Open; consume cached structural ids. |
| `pipeline_authority.rs` stage-order vs `fn compile` drift | R3 Substrate / PB | 2026-04 analyses | Open; source-text bridge retired, structural body still pending. |
| LLM typed carriers not consumed by service ops | R3 Grounding | 2026-04 analyses | Open; includes typed output and wire-serde closure triggers. |
| GitHub auth model bypass | R3 Grounding | 2026-04 analyses | Open. |
| `errors.dag` dead generic layer | R3 Substrate / Grounding | 2026-04 analyses | Open; wire or delete. |
| Fixed-width types lack structural cardinality/refinement | R3 Substrate | 2026-04 analyses | Open; adjacent to DB-11/cardinality. |
| Surface int-literal host narrowing | R3 Substrate | 2026-04 analyses | Partially advanced by T-Numeric work; tokenizer boundary still needs receipt. |
| Peano witness carriers ratchet | R3 Substrate / Evaluator | 2026-04 analyses | Open; routes through E-P / unified positive-int refinement. |
| `parse_parser_body.txt` hand-authored parser scaffold | R3 PB | 2026-04-20 | Open; SG-2b proper trigger. |
| Bool `BooleanAlgebra<Bool>` grounding not wired | R3 Substrate | 2026-04-21 | Possibly partly advanced by Bool identity PRs; needs explicit closure receipt. |
| Top-level `ValueBody` data-body boundary | R3 Substrate | 2026-04-21 | Original framing stale after List/Map; live issue is mirror/isomorphism. |
| File-preference rank scaffold / duplicate std-file pairs | R3 PB / Release legacy | 2026-04-21 / 2026-04-25 | Open; routes through B6 / PB-zero. |
| Go branch `UnknownVariant` fabrication | R3 Substrate / PB | 2026-04-25 | Maybe partially advanced by #1467; needs targeted receipt. |
| Lens fold file-path semantics and callable fallback | R3 Evaluator / Substrate | 2026-04-25 | Open; E6 readiness docs active. |
| Defensive Arrow re-derive row | R3 Substrate | 2026-04-25 | Resolved/stale; do not count live. |
| Exact-string lower-helper patching | R3 PB | 2026-04-25 | Resolved by PR #1014; ensure bridge ledgers stop counting it. |
| Filename / sentinel bridges in `test_runner.rs` | R3 PB / Verification | 2026-04-25 | Open; B4 identity-carrier queue. |
| B4 file/name bridge-retirement queue | R3 PB / Substrate | 2026-04-25 | Open, but stale exact-string item should be removed. |
| Duplicate record-literal fields silently accepted | R3 Substrate | 2026-04-30 | Open; high-priority S target. |
| `ValueBody` Rust-vs-`.dag` mirror drift | R3 Substrate / Verification | 2026-04-30 | Open; high-priority M target. |
| `FieldMap` duplicate-free invariant lost in `.dag` mirror | R3 Substrate / Verification | 2026-04-30 | Open; co-locate with mirror drift. |
| Operator inference synthetic-arrow fallback | R3 Substrate | 2026-04-30 | Open; sharpened again 2026-05-01. |
| `go_method_template_contracts` live diagnostic mismatch | R3 Substrate / Grounding | 2026-04-30 | Open unless diagnostics-empty fixture gate lands. |
| Missing `diagnostics_empty_after_bootstrap` gate | R3 Verification / Substrate | 2026-04-30 | Open; high-priority S target. |
| `test_runner.rs` second predicate language | R3 Evaluator / PB | 2026-04-30 | Open; reinforced 2026-05-01. |
| `dsl/v3/std/emit_model.dag` facade duplicate authority | R3 Substrate / Grounding | 2026-04-30 | Open; v2/CI compatibility bridge. |
| Method-template consumer migration priority | R3 Grounding / PB | 2026-04-30 | Open; row population no longer enough. |
| Reflection completeness over-trusted | R3 Verification | 2026-04-30 | Open; generated conformance walker or equivalent. |
| Provider/API mirror multiplication risk | R3 Grounding | 2026-04-30 | Open; prioritize shared service ingestion path. |
| Hand-Rust acceptance growing faster than generated TestClaim | R3 PB / Verification | 2026-04-30 | Open; pair new hand-Rust with decreases/replacements. |
| `Json` + `Bytes` opaque kernel decomposition placeholder | R3 Substrate | 2026-05-01 | Open disposition placeholder; not T-Numeric scope. |
| SymbolicCost product normalization violates semiring annihilation | R3 Substrate / Verification | 2026-05-01 | Open; high-priority S-M target. |
| SubValueRelation false `BoundedLattice` claim | R3 Substrate / Verification | 2026-05-01 | Open; high-priority S-M target. |
| Emitter `as_bind().expect()` panic paths | R3 Substrate / PB | 2026-05-01 | Open; high-priority S target. |
| `??` and `%` syntax/parser/operator drift | R3 Substrate / Grounding | 2026-05-01 | Open; operator ontology / parser-grammar. |
| `CollectionOps` / `StringOps` / `MapOps` duplicate operation surfaces | R3 Grounding | 2026-05-01 | Open; audit PR #1441 gives dissolution path. |
| `BinaryDimensionReportEquals` consumers not executable | R3 Verification / Evaluator | 2026-05-01 | Partially advanced by W1 Int slice; broader path open. |
| `PRELUDE_BARE_RHS_ALIAS_IDENTS` table drift | R3 Substrate | 2026-05-01 | Open; scope-resolution over name table. |
| BridgeLedgerZero known-open reporting only | R3 Verification | 2026-05-01 | Open; high-priority decreasing-count ratchet. |
| New hand-Rust test PR pairing rule | R3 PB / Verification | 2026-05-01 | Open policy/gate. |
| CI timeout/cost signal | R3 Debt-Paydown / Director cadence | 2026-05-01 | Open signal; feed velocity-tripwire cadence. |

## Recent PR Cross-Reference

The following landed PRs may affect open rows. They should be treated as "needs receipt check," not automatic closure:

| PR | Evidence | Possible affected debt row | Closure status |
|---|---|---|---|
| #1459 | Added substrate carriers including tokenize/int literal/range related changes and `src/v3/std/substrate.dag` updates. | Surface int-literal host narrowing; ValueBody/substrate mirror surfaces. | Partial advancement likely. Needs row-specific receipt before ROADMAP retirement. |
| #1466 | `Int = AbelianGroup<GroupCompletion<Nat>>`. | T-Numeric construction / width-baked integer modeling. | Advances reframe; does not close old int-literal tokenizer boundary alone. |
| #1467 | Reified Bool branch scrutinees through shared variant identity. | Go/variant identity and branch emission bridge class. | May reduce one identity site; not enough to retire B4/file-name queue. |
| #1472, #1498, #1508 | Added CommutativeSemiring, FieldOfFractions, Rational alias surfaces. | Algebra-law witness coverage; numeric construction. | Introduces substrate. Pair with law-witness coverage to avoid adding unchecked algebra claims. |
| #1480, #1488, #1500 | R3 scope/design coherence updates. | Debt-paydown authority; lens application/configuration rows. | Establishes authority; not debt retirement. |
| #1493 | L4/L7 exhaustive algebra-inhabitant-law coverage matrix. | Algebra-law witness coverage gap. | Dispatch-ready research. No executable witness closure yet. |
| #1499, #1504 | W1 DifferentialEquals rust emit vs dag eval Int slice and activation. | Author-now/fire-later / one executable path. | Important progress toward course correction #1, but current W1 remains Int-slice transitional runner debt. |
| #1506 | Sharpened `Nat` through full bootstrap. | Numeric construction / algebra substrate. | Progress, no standalone debt retirement. |
| #1514 | T-E-P-Producer-Broadening single-lookup-authority ratchet. | E-P producer broadening and single-authority lookup. | Ratchet progress. Needs pairing with concrete broadening deletion/closure. |
| #1515 | E7 symbolic-cost-only closure handoff. | SymbolicCost / evaluator E7 row. | Handoff/brief closure, not SymbolicCost semiring bug fix. |

## De Facto Retired Or Stale Candidates

These rows deserve a focused receipt sweep before starting new implementation:

1. `patch_lower_helpers_generated_type_alias_refinement` is already marked resolved in ROADMAP via PR #1014. No action except ensuring BridgeLedger rows no longer count it as open.
2. `Triple MethodTranslation schema` is already marked retired via PR #1210. Remaining method-template authority rows should be tracked under LanguageSpec/PB-Zero, not as the old triple-schema row.
3. `lower_fn_body_into_existing_decl` defensive Arrow re-derive is already marked resolved/stale. Ensure no downstream audit continues citing it as active.
4. `Class 5 Gap 3` top-level `ValueBody` gap has been overtaken by `List` and `Map` landing in Rust; the live unresolved piece is mirror/isomorphism, not the original missing-variant framing.
5. B4 bridge queue still lists exact-string generated-helper patching as an item, but ROADMAP says the patch helper retired via PR #1014. That queue should be updated to avoid overcounting an already-closed site.

## Highest-Leverage Retirement Targets

### 1. Duplicate record-literal fields

Owner: R3 Substrate.
Size: S.
Reason: sharp correctness bug, localized parser/lowering failure, clear sibling pattern in `lower_string_map_entries`.
Closure evidence: diagnostic on repeated record field label at the repeated field span; regression test proving `{ a: 1, a: 2 }` fails closed.

### 2. Emitter `as_bind().expect()` panic paths

Owner: R3 Substrate / R3 PB.
Size: S.
Reason: six known panic sites, direct fail-closed violation, easy to verify with typed emitter error tests.
Closure evidence: replace expects with typed `EmitError` / target-specific error; malformed UserDefined arrow body returns diagnostic instead of panic.

### 3. SymbolicCost product-zero normalization

Owner: R3 Substrate + R3 Verification.
Size: S-M.
Reason: correctness bug in cost facts; high thesis value because it exposes algebra-law witness coverage gap.
Closure evidence: product normalization returns zero if any factor is zero; law tests for additive identity, multiplicative identity, annihilation, and `iterate(0, body)`.

### 4. SubValueRelation false BoundedLattice claim

Owner: R3 Substrate + R3 Verification.
Size: S-M.
Reason: declared algebra law currently false; same verification surface as SymbolicCost.
Closure evidence: either fix meet/join/top semantics or stop claiming `BoundedLattice<SubValueRelation>`; add law witness for the selected claim.

### 5. `ValueBody` / `FieldMap` mirror drift receipt pair

Owner: R3 Substrate + R3 Verification.
Size: M.
Reason: two adjacent rows share one root: hand-maintained Rust-vs-`.dag` shape drift and duplicate-free map invariant loss.
Closure evidence: `.dag` mirror includes all live Rust `ValueBody` variants and a duplicate-free map carrier, or a generated conformance walker/generation path makes drift impossible.

### 6. `diagnostics_empty_after_bootstrap` gate

Owner: R3 Verification + R3 Substrate.
Size: S.
Reason: turns several shape-only ratchets into semantic gates and catches method-template-contract drift class.
Closure evidence: shared test helper/ratchet asserting new bootstrap fixture authorities lower with empty diagnostics.

### 7. BridgeLedgerZero decreasing-open-count ratchet

Owner: R3 Verification.
Size: S-M.
Reason: converts known-open reporting into debt-paydown mechanics.
Closure evidence: ratchet pins current open count and requires each bridge-owning PR to reduce count or cite blocker in receipt.

### 8. B4 queue stale-site cleanup

Owner: R3 PB / R3 Substrate.
Size: S.
Reason: cheap baseline hygiene; prevents already-retired exact-string patching from inflating open debt.
Closure evidence: update B4 queue and BridgeLedger rows to remove PR #1014-retired site and align remaining sites with live bridge ledger.

### 9. Method-template consumer migration plan

Owner: R3 Grounding + R3 PB.
Size: M.
Reason: row population has advanced; retirement value now comes from deleting old consumer authorities.
Closure evidence: dispatch brief or implementation PR identifying first old table consumer to migrate, with receipt naming deleted or de-authorized table.

### 10. `test_runner.rs` new-arm freeze + receipt rule

Owner: R3 Evaluator + R3 PB.
Size: S policy, M implementation.
Reason: stops debt growth while generated TestClaim capability catches up.
Closure evidence: PR discipline note or test-runner ratchet that new bespoke arms must name dissolution hook and owning lane.

## Velocity-Tripwire Seed

Recent window sampled: merged PRs #1430-#1515.

Observed pattern:

- Many PRs are docs/audit/briefs that introduce or refine dispatch surfaces.
- Several implementation PRs add substrate or runner capability.
- Few PR titles explicitly signal `retire`, `dissolve`, `delete`, `remove`, or `collapse`.

Initial reading: introduction pressure is still above dissolution pressure. The ROADMAP's previous 2026-04-18 to 2026-04-25 baseline was about 1.6:1; the 2026-05-01 to 2026-05-02 burst likely trends higher unless manual sweep credits feature PRs that delete bridges inside implementation diffs.

Recommended cadence rule for the next report:

1. Start with title heuristic: introduction-shaped vs dissolution-shaped PRs.
2. Manually inspect feature PRs with large deletions or bridge-file touchpoints.
3. Count closure receipts only when a PR deletes/de-authorizes a named row or updates the ledger with an explicit closure.
4. Escalate to Director only after manual sweep confirms a sustained ratio at or above 3:1.

## Next Dispatch Packet

For the next retirement cycle, dispatch three small implementation closures plus one hygiene sweep:

1. Substrate worker: duplicate record-literal rejection.
2. Substrate/PB worker: emitter `as_bind().expect()` typed errors.
3. Substrate/Verification worker: SymbolicCost zero-product law fix plus witness.
4. PB/Verification worker: B4/BridgeLedger stale-site cleanup and open-count baseline.

Those four produce visible retirement receipts without waiting on larger design lanes, and they exercise the per-PR debt-receipt rule immediately.
