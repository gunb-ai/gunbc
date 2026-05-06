# R2 Impossible-Bugs Manager Brief

**Status:** PROPOSAL (per [`docs/r2-structure.md`](../r2-structure.md), LIVE 2026-04-26 via PR #827; refreshed 2026-04-28 post-#1078 merge to reference structural-acceptance-per-lane-close discipline + R2-close archival per Director cascade). Eligible to spawn pre-R1-close per `r2-structure.md` Transition mechanics step 4 (no technical R1 dependency). NEW manager.

## Orient before reading

- **R2 structure authority:** [`docs/r2-structure.md`](../r2-structure.md). Names this manager one of **7** standing R2 managers (count rose from 6 to 7 with Evaluator added 2026-04-28 per #1078).
- **Program scope source:** [`docs/r2-structure.md` §"Goals" item 4](../r2-structure.md) + [`THESIS.md §"Enumerable impossible-bug classes"`](../../THESIS.md) (R2+ tags authority).
- **Cross-program coordination:** classes that surface substrate gaps escalate to Substrate Manager rather than expanding T-ImpossibleBugs scope.
- **Demo coordination:** signal class-close to R2 Release Manager (closure ledger; per the **structural-acceptance-per-lane-close discipline** locked in `r2-structure.md` — the demo IS the structural gate, not a separate artifact).
- **Lifecycle:** **Manager archives at R2 close** per Director cascade ratified 2026-04-28 (`r3-structure.md` §"Manager structure"): "Modeling/Impossible-Bugs Managers archive at R2 close." No R3 continuation work; post-R2 emergent impossible-bug classes are absorbed by Substrate Manager continuation per closed-system principle (new classes = enumeration was wrong = substrate gap).
- **Substrate-fact-introduction procedure** ([`INVARIANTS.md`](../../INVARIANTS.md) §P1): if implementation surfaces a substrate gap, self-serve through the 3-step decision procedure or signal Substrate Manager. Don't expand T-ImpossibleBugs scope to author substrate locally.

## Program scope (T-ImpossibleBugs)

R2's Goal 4 — **Remaining R2+ impossible-bug classes**. Three classes currently tagged `[R2+]` in `ROADMAP.md §"Lane acceptance — .dag gates"` (T-Demo row); THESIS §"Enumerable impossible-bug classes" is authority on scheduling tags.

| Class | Design authority + implementation worker (post PR #836 merge) | Implementation status | Substrate gating |
|---|---|---|---|
| Nested-optional flatten | Design: `t-impossiblebugs-nested-optional-flatten-design.md` (PR #798) — closed in scope; Worker brief: [`r2-impossible-bugs-nested-optional-flatten-worker.md`](r2-impossible-bugs-nested-optional-flatten-worker.md) (PR #836) | **IMPLEMENTATION LANDED via #890** (substrate-constructor invariant: `cardinality_idempotent_target`, `Dag::alloc_cardinality_decl`, TypeConnective-level enforcement). **Follow-ups landed via #962** (alias peel, resolve_decl idempotence, int literals). Class-close manager work: verify structural test coverage + audit residual integration paths. | UNGATED |
| Unhandled diagnostic paths | Design: `t-impossiblebugs-unhandled-diagnostic-paths-design.md` (PR #801) — closed in scope, §4 Director-actionable recommends totality-by-omission; Worker brief: [`r2-impossible-bugs-unhandled-diagnostic-paths-worker.md`](r2-impossible-bugs-unhandled-diagnostic-paths-worker.md) (PR #836) | **`Int / Int` FIRST SLICE LANDED via #969** (totalized via `Result<Int, DivError>` + `std.error_primitives` algebraic carrier + target prelude support; subsumes prior PR #931). Manager work: dispatch siblings (indexing / quotient / remainder) per design doc §4 — each queues separately as totality-by-omission slices. | UNGATED per design doc §4 (totality-by-omission via algebra retype, not predicate-entailment substrate) |
| Unenumerated effects | Design: `t-impossiblebugs-unenumerated-effects-design.md` (PR #808) — **canonical authority**, closed-system effects model + 5-behavior fold; Worker brief: [`r2-impossible-bugs-unenumerated-effects-worker.md`](r2-impossible-bugs-unenumerated-effects-worker.md) (PR #836) | **IMPLEMENTATION LANDED via #971** (closed-system structural derivation; lens landing; audit verdict path (ii) — retire `OperationEffect` as parallel representation per design doc Q5.5 / worker req 2). Class-close manager work: verify lens-landing acceptance + drive parallel-representation retirement of `OperationEffect`. | UNGATED — closed-system structural derivation does not require new substrate; prior worker briefs SUPERSEDED 2026-04-25 |

Each class becomes an impossible-bug-by-construction at R2 close — not a runtime check, but a structural fact carried by the substrate that the lens / type checker reads.

## Owned deliverables (through R2 close)

For each class:
- Implementation worker brief **landed on main via PR #836 merge** (see Program scope table above for canonical filenames + UNGATED status). The earlier DESIGN/SCOPING workers (`t-impossiblebugs-nested-optional-flatten-worker.md`, `t-impossiblebugs-unhandled-diagnostic-paths-worker.md`) and the SUPERSEDED effects workers were absorbed by the design docs + new implementation workers — do not re-dispatch the older workers.
- Manager dispatches the implementation worker for each class (each ungated per design-doc audit; substrate gaps surface during execution as STOP-AND-ESCALATE → Substrate Manager).
- Substrate-gap discoveries (if any class's implementation surfaces a real substrate need beyond what design docs accounted for) become a Substrate Manager hand-off (escalate per cross-program rule below). This is the exception path; the expected path is direct implementation per design-doc Director-actionable recommendation.
- Structural test demonstrating the impossible-bug status (e.g., synthetic input → compile-time diagnostic, not runtime panic) — included in each implementation worker brief's acceptance.
- Class-close signal to R2 Release Manager (per **structural-acceptance-per-lane-close discipline** — the structural test IS the demo).

## Cross-program dependencies

**Produces (none).**

**Consumes (potentially):**
- Substrate Manager — cardinality refinement (for nested-optional flatten; if substrate gap surfaces beyond what's in T-Substrate sub-lanes)
- Substrate Manager — Tier 2 runtime-safety substrate (for unhandled diagnostic paths; Tier 2 substrate work is post-R1 territory)

**Adjacent designs (already landed):**
- `#808` — closed-system effects model + 5-behavior fold framing (canonical for unenumerated effects work)
- `#805` — Fn→Arrow refactor pre-prereq (may gate unenumerated effects worker)

**Escalation rule:** if a class turns out to need substrate work outside what's already scoped in T-Substrate sub-lanes, **STOP and escalate to Substrate Manager** rather than expanding T-ImpossibleBugs scope. Substrate Manager re-scopes; T-ImpossibleBugs class waits on substrate.

## Locked design decisions consumed (per #1078 dialogue)

Worker briefs MUST consume these without re-litigation:

- **Q6 + Q6.5 (LANDED via #1129)**: two-layer diagnostic-kind authority per [`docs/design-lens-framework.md` §"Q6.5 — Two-layer authority for diagnostic kinds"](../design-lens-framework.md). Layer 1 = `CompilerDiagnosticKind` Substrate-owned closed sum; Layer 2 = lens-instance kinds in lens's own `.dag`. Relevant for unhandled-diagnostic-paths + unenumerated-effects dispatch — these classes consume Layer 1 (compiler-primitive diagnostics); they do not author Layer-2 lens-instance kinds.
- **Closed-system effects model** (PR #808): canonical authority for unenumerated-effects class; 5-behavior structural fold; no annotation surface.

Full disposition: [`docs/r2-structure.md`](../r2-structure.md) §4.

## Pre-spawn vs post-spawn authority

- **Pre-spawn (post-#1078-merge):** brief authoring complete (this skeleton + 3 implementation workers landed via PR #836). Manager spawns once R2 promotion fires; pre-R1-close spawn allowed (no technical R1 dependency).
- **Post-spawn (R2 promotion onward):** Manager owns all worker-brief authoring autonomously per "Autonomous dispatch authority" below. Director's role narrows to cross-program conflict resolution + scope-change escalation.
- **R2 close:** Manager archives. No R3 continuation. Post-R2 emergent classes route to Substrate Manager continuation.

## Autonomous dispatch authority

- Authors all T-ImpossibleBugs worker briefs without Director.
- Dispatches workers against worker briefs (subject to substrate-gating coordination with Substrate Manager).
- Resolves T-ImpossibleBugs-internal scope refinements; escalates substrate gaps to Substrate Manager and blockers/scope changes to Director.
- Per `docs/r2-structure.md` P5 dispatch-discipline: every T-ImpossibleBugs worker brief names dissolution trigger + adjacent ROADMAP debt row + contributes-or-defers stance; every PR introducing hand-Rust under `src/v3/` fills the per-PR gate.
- **Cross-program signal authority:** substrate-gap signals → Substrate Manager via cross-manager queue; per-class closure → R2 Release Manager (closure ledger).

## Reporting cadence

- Class-close → R2 Release Manager (closure ledger; per **structural-acceptance-per-lane-close discipline**).
- Substrate-gap discoveries → Substrate Manager (cross-manager queue).
- Blockers + scope changes → Director.
- **Weekly health surfacing to Director:** which classes within 1 step of close, which workers fill vs. ready, any substrate-gap escalations in flight.

## Acceptance — `.dag` gates

Each class closes under a structural acceptance gate authored as a `.dag` `TestClaim`:

- `nested_optional_flatten_compile_error` — `Some(Some(x))` is structurally inexpressible (substrate-constructor invariant) — synthetic `T??` input produces compile-time diagnostic
- `unhandled_diagnostic_paths_int_div_first_slice` — `Int / Int` divide-by-zero produces compile-time diagnostic via totality-by-omission (algebra retype)
- `unenumerated_effects_closed_system_complete` — 5-behavior fold over substrate primitives demonstrates effect enumeration is exhaustive; new effect = substrate gap, not new effect kind

## Sub-briefs (authored / pending)

**Design docs landed (closed in scope — next-step recommendations are authority for implementation worker briefs):**
- `docs/briefs/t-impossiblebugs-nested-optional-flatten-design.md` (PR #798) — design + scoping closed; implementation path: substrate-constructor invariant per design doc §Director-actionable.
- `docs/briefs/t-impossiblebugs-unhandled-diagnostic-paths-design.md` (PR #801) — design + scoping closed; implementation path: totality-by-omission per design doc §4 Director-actionable recommendation.
- `docs/briefs/t-impossiblebugs-unenumerated-effects-design.md` (PR #808) — canonical authority; implementation path: closed-system structural derivation per design doc §Q1, §Q2, §Q3 + §Q6.

**Implementation worker briefs landed on main via PR #836 merge — manager dispatches against these:**
- [`docs/briefs/r2-impossible-bugs-nested-optional-flatten-worker.md`](r2-impossible-bugs-nested-optional-flatten-worker.md) — M; ungated per design doc (substrate-constructor invariant; cardinality bridge already past per Director audit).
- [`docs/briefs/r2-impossible-bugs-unhandled-diagnostic-paths-worker.md`](r2-impossible-bugs-unhandled-diagnostic-paths-worker.md) — M; per-class totality-by-omission (Int / Int divide-by-zero first; siblings queue separately).
- [`docs/briefs/r2-impossible-bugs-unenumerated-effects-worker.md`](r2-impossible-bugs-unenumerated-effects-worker.md) — M; closed-system structural derivation; no substrate gate.

**SUPERSEDED — do not dispatch (implementation workers above replace these):**
- `docs/briefs/t-impossiblebugs-nested-optional-flatten-worker.md` — SUPERSEDED by `r2-impossible-bugs-nested-optional-flatten-worker.md`; the older DESIGN/SCOPING worker's role was absorbed into the design doc + implementation worker pair.
- `docs/briefs/t-impossiblebugs-unhandled-diagnostic-paths-worker.md` — SUPERSEDED by `r2-impossible-bugs-unhandled-diagnostic-paths-worker.md`; same pattern.
- `docs/briefs/t-impossiblebugs-unenumerated-effects-worker.md` (SUPERSEDED 2026-04-25 by design doc PR #808; further dissolved by `r2-impossible-bugs-unenumerated-effects-worker.md` now landing as the implementation authority).
- `docs/briefs/t-impossiblebugs-unenumerated-effects-parser-worker.md` (SUPERSEDED 2026-04-25 by design doc; closed-system framing dissolves the proposed parser surface).

**Pending — Manager dispatches residuals (3 main implementations already landed via #890/#969/#971; manager drives class-close completion):**
- Nested-optional-flatten: residuals after #890 + #962 (verify structural test coverage; audit integration paths; close class). **Queued follow-up:** [`r2-impossible-bugs-nested-optional-codegen-bypass-closure-worker.md`](r2-impossible-bugs-nested-optional-codegen-bypass-closure-worker.md) narrows the remaining scope to `CardinalityPayload::new_unchecked` and generated construction sites beyond the original std-only audit; authored from a `git-metadata-unavailable` checkout.
- Unhandled-diagnostic-paths: dispatch indexing / quotient / remainder slices (per-class totality-by-omission, modeled on the #969 `Int/Int` first slice).
- Unenumerated-effects: drive `OperationEffect` parallel-representation retirement per #971's path (ii) verdict.
- PR #805 Fn→Arrow refactor stays dispatchable as independent vestigial-syntax cleanup (not a thesis-claim closure dependency).

## Working state (fill on spawn)

Spawn refresh, 2026-04-28 (post-#1078, status-refresh against landed PRs):

- **All 3 main implementations landed pre-spawn:** nested-optional flatten (#890 + #962 follow-ups); `Int / Int` totality-by-omission first slice (#969 — subsumes prior #931); unenumerated effects (#971 — closed-system structural derivation; lens landing).
- Manager day-1 work is **class-close completion** (residual coverage + integration audit + parallel-representation retirement) plus **dispatching siblings** of the unhandled-diagnostic-paths totalization pattern (indexing / quotient / remainder).
- Manager archives at R2 close; no R3 continuation work.
- Post-R2 emergent classes route to Substrate Manager continuation per closed-system principle.

## Cross-refs

- Parent: `docs/r2-structure.md` §"Impossible-Bugs Manager"
- Goal authority: `THESIS.md §"Enumerable impossible-bug classes"` (R2+ tags)
- Effects framing: PR #808 (closed-system effects model + 5-behavior fold)
- Effects prereq: PR #805 (Fn→Arrow refactor)
- Adjacent designs: PRs #798 (nested-optional), #801 (unhandled diagnostics)
- INVARIANTS substrate-fact-introduction procedure: `INVARIANTS.md` §P1
- R2-close archival authority: `docs/r3-structure.md` §"Manager structure"
- Thesis-claim disposition: `docs/thesis/r2-r3-thesis-mapping.md`
