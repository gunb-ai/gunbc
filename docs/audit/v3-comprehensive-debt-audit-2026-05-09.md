# v3 Comprehensive Debt Audit — 2026-05-09

**Author**: deep-wolf-155 (PM)
**Operator authorization**: Brian directive 2026-05-09 (~20:30Z): *"can we please do a one time audit to get EVERYTHING, not just R3 - all debt paydown- might as well use that lane for all v3 paydown now, even if its past R3 - additionally, we need to get that lane working on the debt thats gone in ASAP"*
**Authority scope**: PM-tier audit. Inventory across all v3 debt sources (not just R3). Input to Debt-Paydown Mgr (gentle-newt-665 / #2062) for organize-dispatch-cadence under expanded lane scope.

**Companion artifacts**:
- Lane scope amendment in same PR: `docs/r3-structure.md` Debt-Paydown lane definition + `docs/r3-program-plan.md` §10 cadence
- Mgr dispatch to inbox #2062 (separate from this PR; operator-authorized active posture)

---

## §0. Operator-authorized posture change

Per Brian directive 2026-05-09:

1. **Lane scope expands from R3-only to all v3 debt** — including post-R3-close items (parking lot for future horizons; not deletion, just routing into the lane's organize-track).
2. **Lane posture changes from cadence-only to active** — Mgr proactively polls debt sources, organizes inventory into Class A-G framework, surfaces to PM/Director, dispatches workers on highest-priority items. Prior Director ratification at gunbc#828 c#4411537717 ("no eager-action between cadence checkpoints") supersedes via direct operator authorization.

This audit is the **initial inventory dump** the new posture consumes. Mgr re-organizes per their judgment; this is input, not authority.

---

## §1. Debt source inventory — sample / not exhaustive

### Source 1: ROADMAP "Post-merge debt" sections (10 sections; ~40 still-open rows estimated)

Sampling representative sources (Mgr does full enumeration during organize phase):

- **`src/v3/compiler/parse_parser_body.txt` 1350 LOC hand-authored recursive-descent parse algorithm** — SG-2b dispatch behind-queue
- **Class 5 Gap 1 — `Bool` inhabits `BooleanAlgebra<Bool>` not wired** — 1e-2b lane Path A
- **Class 5 Gap 3 — `data` body shape boundary (substrate capability gap)** — substrate-capability lane; `ValueBody` extension required
- **`emit_rust_module` SurfaceLiteral → LiteralBits rename** — blocks SG-3g slices
- **`emit_rust_module` `render_variant_constructor` fails on external tuple variants** — blocks SG-3g
- **File-preference rank scaffold (P2 violation)** — `dag.rs` + `lower.rs` mirrors; ratified parallel-authority pending convergence
- **`container_template_algebra_rows` string table duplicate authority** — `dsl/std/types.dag:133-146`
- **Emitter render-helper consolidation** — `named_variant_id` / `render_named_template` 3-5× duplicated
- **SG-2c growth-discipline** — pivot vs tiny-extractions decision pending
- **Multiple thesis-doc reference cleanup waves** — partially retired via #1795/#1801/#1820

Full enumeration: `awk '/^### Post-merge debt/,/^---/{if(/^- \*\*/) print}' ROADMAP.md` returns ~40 still-open rows across 10 sections (2026-04-20 through ongoing).

### Source 2: docs/audit/r3-debt-sweep-2026-05-06.md Class A-G framework (placeholders unpopulated)

Per `docs/r3-program-plan.md:18`: *"NOT yet authoritative for per-row inventory: Class A/B/C/D/E/F/G sections carry `[Mgr canvas populates rows]` placeholders"*.

The framework is solid. The rows are blank. Class breakdown per Director's framework:
- **Class A — Substrate-gap-blocked** (~15-20 expected)
- **Class B — Pattern-A NYI predicates** (~7 expected)
- **Class C — Pattern-C typed-carrier + Rust-mirror** (~6+ expected)
- **Class D — Generated bridges with freshness gates** (~3-5 expected)
- **Class E — v2 ↔ v3 transition bridges** (~10+ expected; **now in R3 scope per fold-in**)
- **Class F — Operator/algebra ontology duplication** (~3 surfaces expected)
- **Class G — Local/small bridges** (~5-10 expected)

Total expected rows: 49-71 across the 7 classes. Mgr populates per existing framework.

**NEW class proposal (Brian directive 2026-05-09)**: **Class P — Past-R3 debt** (substrate-capability gaps that don't block R3 close but do block R4+ ambition). Examples: TC1 #11 canvas-deferred work; full T-LBP at non-R3-load-bearing horizons; ProgramShape richer-shape variants per design §8.2. Mgr proposes Class P entries during organize phase.

### Source 3: §10.3 RED items in `docs/r3-program-plan.md` (~151 RATIFIED entries; ~5-10 still RED-pending)

Most §10.3 entries are RATIFIED (Q-decisions resolved). Outstanding RED:
- Q-PB0-Trajectory-Risk5 (RATIFIED 2026-05-09 — but remediation in progress; SG-0 census still GROWING)
- Q-PB0-ClusterM-Cold-Risk6 (RATIFIED 2026-05-09 — Phase 1 dispatch authorized; not yet executed at HEAD)
- Q-Cluster-M-Reclassification (RATIFIED 2026-05-09 — §2/§6/§1.8 status update pending)

These are ratification-level RED → operationally tracked. Lane-active work item: monitor execution against ratification.

### Source 4: Today's gpt-5-5-pro reflective analyses (6 novel findings; not yet tracked)

From PR #2358 §8 meta-finding cycle (3 sha windows: b09e0c8 / 1211e453 / cf1d523):

| # | Finding | Class candidate |
|---|---|---|
| 1 | `MissingEmissionPath` stringifies typed axes (`connective: String, behavior: String, target: String`) | Class C — typed-carrier regression at diagnostic boundary |
| 2 | `ShapeATarget = Rust \| Python \| Go` closed enum vs `LanguageSpec` data extensibility | Class F — ontology duplication |
| 3 | `Map<String, Bool>` as set across graph/syntax/node files (`set_has` ignores stored bool) | Class F — missed algebraic structure (Set<A> declared in std but bypassed) |
| 4 | `PartitionResult` bypassed by anonymous return type | Class G — small duplicate-authority cleanup |
| 5 | `ComposedEffect { idempotent, breaking_operation }` illegal product — **REMEDIATED** (`dsl/std/effects.dag` → `CompositionVerdict`; PR #2491 / #2469) | Class C — illegal-state-representable (closed) |
| 6 | `derive_op_effect(method_str, path_str)` string parser at structural boundary | Class C — string-keyed dispatch over typed carriers |

### Source 5: 4 dispatched bug-fix briefs (PR #2373 merged; durable in docs/briefs/)

stern-ram-58 archived; tasks await re-dispatch:

| Brief | File | Class | Status |
|---|---|---|---|
| u128 grounding-pilot Rust mirror sync | `r3-bug-u128-grounding-pilot-mirror-sync-worker.md` | Class C — concrete drift bug (Rust mirror diverged from .dag) | HIGHEST priority; thesis-validating |
| FieldProject dual-authority dissolution | `r3-bug-fieldproject-dual-authority-dissolution-worker.md` | Class C — illegal-state-representable | HIGH priority; soundness bug |
| `resolve_producer_opt` typed return | `r3-bug-resolve-producer-opt-typed-return-worker.md` | Class B — P3 fail-closed violation | HIGH priority; concrete bug |
| CallGraph forward-only authority | `r3-bug-callgraph-forward-only-authority-worker.md` | Class C — illegal-state-representable within product | MEDIUM priority |

### Source 6: SG-0 census untracked entries (113 entries; per PR #2358 §1)

Live count: 159 entries (104 TEST + 53 NON_TEST + 2 FRAGMENTS). 113 (72%) lack explicit dissolution-trigger comments. **Class A** candidates if substrate-gap-blocked; **Class B** if Pattern-A NYI; **Class E** if v2↔v3 transition; **Class G** if local/small.

PM proposed earlier: per-entry §1.8 gate-id linkage script + PR-window ratchet. Awaiting authorization. Mgr-tier work: classify per-entry against Class A-G + dispatch workers per class.

### Source 7: §1.8 ledger Status drift (PR #2399 in flight; 10 candidates)

PR #2399 surfaces 10 promotion candidates (DECLARED → CONSUMER_LANDED). One self-corrected post-Grounding-Mgr-dispatch (#25 backend half pending). Lane-active work: each Mgr review their lane's rows; promote on review.

### Source 8: Drift-class anti-patterns (ratchet generalization candidates)

Today's R4-carve dissolution ratchet (`scripts/check-r4-carve-dissolution-discipline.sh`) catches one drift class. Candidates for generalization (per gpt-5-5-pro reviews + my drift sweep):

| Anti-pattern | Detection | Class |
|---|---|---|
| Stale "94 R3-load-bearing" / "94 gates" framings without supersession marker | Regex on docs/ | Drift discipline |
| "Director ratification" near locked-design citations (canvas-tier anti-pattern) | Regex + locked-design-doc proximity | Drift discipline |
| `Map<String, Bool>` as field type in `.dag` declarations | grep on type declarations | Class F |
| Anonymous-return records matching named declared types | AST analysis | Class G |
| Comment patterns "validity is enforced today by [Rust constructor]" (Track 9 forgeable witnesses) | Regex on `.dag` files | Class C |
| `// TODO` / `// FIXME` markers in src/v3/ without explicit dissolution path | grep + dissolution-trigger check | Class G |

### Source 9: TODOs / FIXMEs in src/v3/ (4 markers)

`grep -rn "TODO\|FIXME\|XXX" src/v3/` yields ~4 hits (filtered for non-test, non-comment). Mgr inventories during organize phase.

### Source 10: 5 PM structural recommendations (awaiting authorization; from earlier audit)

| # | Recommendation | Status |
|---|---|---|
| 1 | Per-entry §1.8 gate-id linkage in `sg0_census_test.rs` | Highest-leverage; awaiting authorization |
| 2 | C0 "residual" class in `docs/design-tests-as-data-completeness.md` §3 | High-leverage; locked-design extension |
| 3 | SG-0 option-(c) gate-id citation tightening | Medium; mirrors URL-tightening fix |
| 4 | Cluster M sequencing plan §5.2 per-class enumeration | Medium |
| 5 | Generalize ratchet pattern to other drift classes | High-leverage going forward |

---

## §2. Initial classification (Mgr re-organizes per their judgment)

**Approximate inventory size**: 50-80 distinct items across all sources. Many overlap (e.g., the same drift item may surface in multiple sources).

**Priority sizing** (PM proposal; Mgr finalizes):

### P0 — block R3 close
- gpt-5-5-pro Finding 1 (u128 mirror drift) — concrete drift bug
- gpt-5-5-pro Finding 2 (FieldProject dual-authority) — concrete soundness bug
- gpt-5-5-pro Finding 3 (resolve_producer_opt) — P3 fail-closed violation
- 113 SG-0 census untracked entries — block #84 / #8 strict-zero close
- §10.3 ratification → execution gap (Cluster M Phase 1 / Cluster F dispatch)

### P1 — accelerate R3 close
- §1.8 ledger Status promotion (PR #2399)
- ROADMAP open Post-merge debt rows (sample: 5-Gap-1 Bool inhabits; ValueBody extension; emit_rust_module gaps)
- Cluster A-G framework population (49-71 expected rows)

### P2 — post-R3 ambition / R4 prep
- TC1 #11 canvas-deferred (post-R3 substrate)
- C4/C5/C6 substrate-axis defers in r4-carve-out-routing.md
- ProgramShape richer-shape variants
- gpt-5-5-pro Findings 4–6 (CallGraph; ComposedEffect illegal-product Finding 5 remediated PR #2491; derive_op_effect)

### P3 — discipline / drift prevention
- Generalize R4-carve ratchet pattern to other drift classes
- SG-0 option-(c) gate-id tightening
- Per-entry §1.8 gate-id linkage script

---

## §3. Mgr action plan (proposed; gentle-newt-665 finalizes)

**Phase 1 (next ~24h)**: organize this audit's inventory into `docs/audit/r3-debt-sweep-2026-05-06.md` Class A-G framework. Extend with Class P (past-R3) per Brian directive. Update inventory rows with concrete owners + sizing.

**Phase 2 (next ~48h)**: dispatch workers on P0 items. Substrate Mgr / Verification Mgr / Grounding Mgr partner per item's lane. Initial dispatch candidates:
- 4 stranded bug-fix briefs (re-dispatch under new worker spawn)
- 6 gpt-5-5-pro novel findings (author worker briefs)
- 113 SG-0 census per-entry classification (script + sweep)

**Phase 3 (continuous)**: cadence reporting:
- Daily debt-paydown progress snapshot to PM inbox (#846)
- Weekly velocity-tripwire reading
- Per-PR debt-receipt rule enforcement (CI-driven; existing posture preserved)

**Phase 4 (ongoing organize-and-track)**: maintain inventory under continuous polling — new debt items routed through Mgr inbox; classified + queued + surfaced.

---

## §4. Cadence + posture amendment

Per Brian directive, lane scope and posture change from prior Director-ratified `c#4411537717` "no eager-action between cadence checkpoints" to:

1. **Active organize cadence**: Mgr proactively polls debt sources continuously; no specific cadence-checkpoint constraint
2. **Active dispatch authority**: Mgr dispatches workers on P0/P1 items under standing authority; surfaces to PM/Director only on shape questions
3. **Continuous reporting**: daily snapshot to PM inbox

Prior trigger model retained as fallback (CI per-PR; cadence velocity-tripwire; Director-dispatched audits) but no longer the only triggers — proactive work expected.

---

## §5. Receipt

When this audit + amendment land:
- Lane scope expanded to all v3 debt (not R3-only) per Brian directive
- Lane posture changed to active per Brian directive
- gentle-newt-665 receives this audit as inventory input + new posture
- gentle-newt-665 begins Phase 1 organize cycle
- PM (deep-wolf-155) tracks Mgr velocity + surfaces blockers to Brian

---

## §6. Provenance

- Source 1: `awk` over ROADMAP.md "Post-merge debt" sections
- Source 2: `docs/audit/r3-debt-sweep-2026-05-06.md` framework + placeholders
- Source 3: `docs/r3-program-plan.md` §10.3 grep
- Source 4: gpt-5-5-pro reflective analyses (3 sha windows; PR #2358 §8 + my consolidation)
- Source 5: PR #2373 (4 stranded bug-fix briefs)
- Source 6: PR #2358 §1 (SG-0 census untracked entries)
- Source 7: PR #2399 (10 §1.8 Status promotion candidates)
- Source 8: Today's drift sweep ratchet generalization candidates (my analysis)
- Source 9: `grep` on src/v3/ for TODO/FIXME/XXX
- Source 10: PM structural recommendations from earlier audits (this session)

Authority chain: Brian directive (operator) 2026-05-09 ~20:30Z → PM authoring this audit + lane scope amendment → gentle-newt-665 receives + organizes → PM tracks velocity.

---

**End of audit.**
