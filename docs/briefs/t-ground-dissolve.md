# T-Ground-Dissolve — Track-13 table-coercion retirement

**Status:** PROPOSAL — implementation dispatches only after Coercion-Fold consumer wiring proves the structural authorities carry the load. Brief authored 2026-04-29.

**Lane:** T-Ground-Dissolve (S) — item **11** of 11 in [`r2-grounding-manager.md`](r2-grounding-manager.md) (lane description line 39, lane row line 70, acceptance gate line 130, pending list line 148).

**Manager:** R2 Grounding Manager ([`r2-grounding-manager.md`](r2-grounding-manager.md)).

**Lineage / authorities consumed (no re-litigation):**
- R2 manager lane row + acceptance gate: [`r2-grounding-manager.md`](r2-grounding-manager.md) lines 39, 70, 130, 148 — Track-13 dissolution deletes `TypeCheckpoint` / `InhabitantDecl` / `carrier: String` once Coercion-Fold carries the load. This is the **final critical-path step** of R2 Grounding.
- Engine-reframe spec: [`docs/design-emission-model.md`](../design-emission-model.md) — Modeling problem 6 names `dsl/std/coercion.dag` + `dsl/extdeps/languages/{rust,python,go}/types.dag` as table-driven bootstrap scaffolding to dissolve once `LanguageSpec` is consumed; lane table line 384 assigns the dissolve to this lane.
- Current Track-13 schema: [`dsl/std/coercion.dag`](../../dsl/std/coercion.dag) — `TypeCheckpoint` and `InhabitantDecl` declarations plus target-independent helpers (`CallableRepr`, `CastSyntax`, `CastRule`).
- Current Track-13 data consumers: `dsl/extdeps/languages/{rust,python,go}/types.dag` and `dsl/extdeps/languages/dag/types.dag`.
- Structural successors: [`t-ground-languagespec.md`](t-ground-languagespec.md), [`t-ground-diagnostic.md`](t-ground-diagnostic.md), [`t-ground-cross-target-meta.md`](t-ground-cross-target-meta.md), [`t-ground-tests.md`](t-ground-tests.md), plus Coercion-Fold consumer wiring.
- Fail-closed and P1 discipline: [`INVARIANTS.md`](../../INVARIANTS.md) C-8 and §P1.

---

## Framing question this lane answers

Can the repo delete the last table-driven Track-13 coercion scaffolds because every semantic fact they carried is now consumed structurally by `LanguageSpec` + Coercion-Fold + typed diagnostics?

A "yes" means `TypeCheckpoint`, `InhabitantDecl`, and string `carrier` fields have no remaining semantic consumer. Their facts have been absorbed by structural per-target primitive / inhabitance / realization rows, and failure modes surface as typed diagnostics. A "no" means at least one old table row is still doing semantic work; this lane must stop and route that gap to the owning upstream lane rather than preserving the scaffold under a new name.

---

## Scope

### A. Delete `TypeCheckpoint` and its data tables

Retire the name-keyed fast path:

- `dsl/std/coercion.dag` declaration: `type TypeCheckpoint`.
- Rust table: `dsl/extdeps/languages/rust/types.dag` `rust_type_checkpoints`.
- Python table: `dsl/extdeps/languages/python/types.dag` `python_type_checkpoints`.
- Go table: `dsl/extdeps/languages/go/types.dag` `go_type_checkpoints`.
- Dag debug target table: `dsl/extdeps/languages/dag/types.dag` `dag_type_checkpoints`.

Absorption target: `LanguageSpec` per-target primitive / realization rows. Each row previously carried as `{ dag_name, target_type, default_expr, is_copy, literal_suffix }` must either:

- be represented structurally by the target's `LanguageSpec` primitive / construction / default / ownership facts, or
- have an explicit dissolution receipt explaining why the old row was dead data.

No string-keyed `.dag type name -> target type` table remains on the emission path.

### B. Delete `InhabitantDecl` and its data tables

Retire the algebra-keyed fallback:

- `dsl/std/coercion.dag` declaration: `type InhabitantDecl`.
- Rust table: `dsl/extdeps/languages/rust/types.dag` `rust_algebra_inhabitants`.
- Python table: `dsl/extdeps/languages/python/types.dag` `python_algebra_inhabitants`.
- Go table: `dsl/extdeps/languages/go/types.dag` `go_algebra_inhabitants`.

Absorption target: Coercion-Fold over `LanguageSpec` algebra-inhabitance facts with Q1/Q2 refinement axes. Each old `{ algebra, template, arity, identity_expr, import_path, is_copy }` row must map to:

- a structural inhabitance / construction pattern / import-or-external-realization fact on the target language spec, and
- a typed fail-closed outcome (`EmissionDiagnostic::NoInhabitant` or narrower variant) when no structural inhabitance exists.

No fallback that matches an algebra by string and substitutes a template remains.

### C. Delete `carrier: String` from v3 emission substrate rows

Retire residual rendered-carrier strings in `src/v3/std/emit_model.dag`:

- `TypeRealization.carrier: String`.
- `OperatorRealization.carrier: String`.
- `BehaviorRealization.carrier: String`.
- `TypeInstantiationRealization.carrier: String`.

Absorption target: typed `DeclarationRef` / target-language structural facts already carried by the row's `language`, `target`, construction pattern, operator dispatch, and `LanguageSpec` realization entries. If a consumer still needs display text for diagnostics or target syntax, that text must be derived from the target's structural realization, not stored as a parallel string authority.

### D. Delete or split `dsl/std/coercion.dag`

If A and B remove the last semantic declarations from `dsl/std/coercion.dag`, delete the file and remove imports from every target file.

If non-Track-13 helpers remain live (`CallableRepr`, `CastSyntax`, `CastRule`, `dag_cast_rules`, `dag_can_cast`, `is_dag_cast_domain_type`), this lane must make an explicit split decision in the PR:

- move live non-Track-13 helpers to their owning file with a receipt, then delete `coercion.dag`; or
- leave a reduced file only for those non-Track-13 helpers, with `TypeCheckpoint` / `InhabitantDecl` removed and a named follow-up owner for the remaining file.

Do not keep `coercion.dag` merely because deleting it is inconvenient. The Track-13 dissolution claim is about the table-driven type-realization surface, not about preserving a filename.

### E. Delete consumers

Remove or rewrite every read path that still depends on:

- `TypeCheckpoint`.
- `InhabitantDecl`.
- the `*_type_checkpoints` / `*_algebra_inhabitants` data tables.
- `TypeRealization.carrier` / `OperatorRealization.carrier` / `BehaviorRealization.carrier` / `TypeInstantiationRealization.carrier` strings.

The replacement consumer is Coercion-Fold reading structural substrate facts directly. If a consumer cannot be migrated because the structural fact is missing, stop and route to the owning prerequisite lane.

---

## Dependencies / gates

| Gate | Status | Lane impact |
|---|---|---|
| **T-Ground-LanguageSpec** | predecessor | Supplies per-target primitive rows, construction patterns, operator dispatch, realization cost, and MethodTemplateContract consumption. This lane deletes only after those rows are live. |
| **T-Ground-Coercion-Fold consumer wiring** | direct gate | Proves emission reads substrate facts directly. This lane is the closer, not the opener. |
| **T-Ground-Diagnostic** | predecessor / sibling | Supplies typed `EmissionDiagnostic` variants for under-determinism and no-inhabitant cases. Deletion must not replace typed failure with panic / `None` / warning. |
| **T-Ground-Tests** | predecessor | Supplies `routing_correctness_l4_verified`; Track-13 deletion depends on routing stability against structural facts. |
| **T-Ground-CrossTarget-Meta** | companion | L6 load-completeness catches missing cross-target structural rows before old tables are deleted. |
| **MethodTemplateContract rows** | predecessor | Method-template parallel authorities are not Track-13, but LanguageSpec close depends on them. This lane should verify they are already structural and avoid re-opening them. |

**Critical-path framing:** this is the final R2 Grounding step. It closes only after the upstream structural authorities are consumed and verified. The PR-N / closure cadence for Grounding cannot complete while any Track-13 table or `carrier: String` semantic consumer remains.

---

## P1 / dissolution receipts

This lane should not introduce new substrate facts. It primarily deletes old facts and records which structural authority absorbed each one.

The PR body MUST include a dissolution receipt table:

| Deleted surface | Deleted data / consumers | Absorbing authority | Test / receipt |
|---|---|---|---|
| `TypeCheckpoint` | per-target `*_type_checkpoints` + name-keyed readers | `LanguageSpec` primitive / default / ownership / construction rows | routing parity + no string-keyed lookup |
| `InhabitantDecl` | per-target `*_algebra_inhabitants` + algebra-template readers | Coercion-Fold over `LanguageSpec` inhabitance facts | no-inhabitant and unique-inhabitant tests |
| `carrier: String` | v3 emit-model carrier fields + readers | typed target / realization refs and construction/operator facts | compile-time schema load + no carrier-string read |

If implementation discovers a genuinely new type / variant / field is required, the worker must stop and run `INVARIANTS.md` §P1 before adding it. A deletion lane that needs new substrate facts is probably uncovering an upstream gap, not closing Track-13.

---

## Out of scope (do NOT do)

- **Coercion-Fold body implementation.** This lane consumes the finished fold and deletes old tables after it carries the load.
- **Re-modeling `TypeCheckpoint` / `InhabitantDecl` under new names.** If the same shape survives, Track-13 did not dissolve.
- **Per-target primitive modeling.** Rust / Python / Go primitive declarations and axes belong to their target lanes and `LanguageSpec`.
- **Per-target diagnostic localization or UX copy.** T-Ground-Diagnostic owns typed carriers; renderers own wording.
- **Runtime equivalence / R3 verification.** This lane closes R2 table deletion; R3 proves emitted artifacts behave equivalently.
- **Pure-Bootstrap-Zero method retirement.** v2-side `MethodTranslation` / `SimpleMethodSpec` / Python-Go template-map retirement belongs to LanguageSpec / Pure Bootstrap Zero surfaces, not Track-13 deletion unless a remaining consumer directly blocks the Track-13 surface.
- **Deleting unrelated coercion helpers without a receipt.** `CallableRepr`, `CastSyntax`, `CastRule`, and `dag_cast_rules` are only in scope if their current home prevents deleting the table-driven Track-13 schema; their semantic retirement is not implied by this brief.
- **Touching `src/v3/compiler/` for convenience.** If compiler code still reads the old surfaces, migrate it to structural facts or escalate if the structural fact is missing.

---

## Sizing

**S** per `r2-grounding-manager.md:70`. This is deletion-only once predecessors land:

- remove declarations and data tables,
- migrate final consumers,
- add dissolution receipts,
- run structural routing / fail-closed tests.

If implementation grows beyond deletion and consumer rewiring, the worker should stop and report which upstream lane failed to absorb a fact.

---

## Test plan

Per `TESTING.md` — hermetic, behavior-driven, unit-first.

1. **No Track-13 schema loads** — bootstrap / parser no longer loads `TypeCheckpoint` or `InhabitantDecl`; any import of those names fails the test.
2. **No old data tables** — `rust_type_checkpoints`, `python_type_checkpoints`, `go_type_checkpoints`, `dag_type_checkpoints`, and the `*_algebra_inhabitants` tables are absent or reduced to explicitly dead historical fixtures outside the build.
3. **No `carrier: String` semantic reads** — v3 emit-model schema has no `carrier: String` field on the named realization rows; consumers read typed structural facts.
4. **Routing parity after deletion** — programs formerly covered by checkpoints (`Int`, `Float`, `Bool`, `Unit`, `String`, `Bytes`, `Secret`, `Json`) still route through `LanguageSpec` / Coercion-Fold or fail with typed diagnostics where no structural target exists.
5. **Fallback parity after deletion** — algebra cases formerly covered by `InhabitantDecl` (`FreeMonoid`, `BooleanAlgebra`, `PartialFunction`, `OrderedRing`, `ApproximateField`) route structurally with the same target intent.
6. **Fail-closed missing-inhabitant test** — removing a structural inhabitance row produces `EmissionDiagnostic::NoInhabitant` (or successor typed variant), not a panic, warning, silent default, or old fallback table lookup.
7. **Dissolution receipt check** — the PR includes the per-declaration deletion table and cites the absorbing authority for each deleted surface.

Run:

- `cargo test --workspace --exclude v2-compiler-tests`
- `cargo test -p v2-compiler-tests`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`

Add narrower tests for any migrated consumer that previously read Track-13 tables.

---

## Dissolution claim

Lane closes under the `r2-grounding-manager.md:130` acceptance gate:

> `track_13_dissolution_complete` — `TypeCheckpoint` / `InhabitantDecl` / `carrier: String` deleted

Concrete close criteria:

- `TypeCheckpoint` declaration deleted.
- `InhabitantDecl` declaration deleted.
- per-target checkpoint and algebra-inhabitant data tables deleted or removed from build authority.
- all consumers of those tables migrated to Coercion-Fold / `LanguageSpec`.
- `carrier: String` fields deleted from the named v3 realization rows and semantic consumers migrated to typed structural facts.
- dissolution receipt names the absorbing authority for each deleted fact.
- fail-closed diagnostics prove missing structural facts no longer fall back to table-driven coercion.

Per the **structural-acceptance-per-lane-close discipline**, the `.dag` `TestClaim` is the demo; no separate artifact is required.

---

## Hand-off discipline

Escalate to manager (#1133, do not absorb in lane) if:

- a `TypeCheckpoint` row has no structural home in `LanguageSpec`;
- an `InhabitantDecl` row requires a new refinement axis not already in the target / LanguageSpec lanes;
- a `carrier: String` consumer cannot be rewritten without adding a new target syntax carrier;
- deleting `coercion.dag` would delete live non-Track-13 helpers whose owner is unclear;
- implementation requires a new substrate type rather than deleting the old one;
- routing parity fails after deletion;
- any referenced authority doc has drifted materially from this brief.

Per `feedback_root_causes_over_quick_fixes.md`: no quick fixes. Per `feedback_no_textual_enforcement_bridges.md`: no grep/regex bridges standing in for structural consumption.

---

## What unblocks on merge

- R2 Grounding's final critical-path lane closes.
- R2 Release Manager can mark `track_13_dissolution_complete` in the closure ledger.
- The old "coercion engine" / table-driven realization vocabulary stops existing as a parallel authority.
- Downstream verification can rely on structural substrate facts and typed diagnostics without a fallback table path.

---

## Cross-refs

- Parent: [`r2-grounding-manager.md`](r2-grounding-manager.md) (lane 11 of 11)
- Engine-reframe spec: [`docs/design-emission-model.md`](../design-emission-model.md)
- Track-13 schema: [`dsl/std/coercion.dag`](../../dsl/std/coercion.dag)
- Current per-target Track-13 data: `dsl/extdeps/languages/{rust,python,go,dag}/types.dag`
- V3 carrier strings: [`src/v3/std/emit_model.dag`](../../src/v3/std/emit_model.dag)
- Predecessor / sibling briefs: [`t-ground-languagespec.md`](t-ground-languagespec.md), [`t-ground-diagnostic.md`](t-ground-diagnostic.md), [`t-ground-cross-target-meta.md`](t-ground-cross-target-meta.md), [`t-ground-tests.md`](t-ground-tests.md)
- Substrate-fact procedure: [`INVARIANTS.md`](../../INVARIANTS.md) §P1
