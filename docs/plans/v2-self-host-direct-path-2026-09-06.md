# The direct path to a clean v2 self-host — orientation, sequencing, autonomy (2026-09-06)

**What this file is.** An orientation and sequencing read, written at one point in time, plus the autonomy contract under which an agent executes it. It is **not** a work-item roster and not a second authority: every fact about what remains lives in the carriers named below and is re-derived by running the instruments named below, never by reading a number off this page (DESIGN §6 — name the instrument, never transcribe its output). When this file and a carrier disagree, the carrier is right and this file is stale.

**The authorities this file defers to:**

- DESIGN §7 — the goal itself, and the honesty boundary around it.
- [v2-self-hosting.md](v2-self-hosting.md) — the operator-signed (2026-07-11) Weak → Strong Self Host program: four waves, dependency-gated exits, `src/v1` deleted only at the end of Wave 4. That is the plan from here to there; this file does not replace it.
- `dag/gunbc/roadmap/roadmap_authority.dag` — the node chain with per-node boundary, first slice, RED control, exclusions, handback. Projected to `ROADMAP.md`; every node requires explicit manual acceptance.
- The three self-host frontier carriers: `v2.compiler.self_host.seed_retention_frontier`, `v2.compiler.self_host.emit_coverage_frontier`, `v2.compiler.self_host.native_routing_frontier`.
- `dag/gunbc/guarantee_stall/` — the typed stall roster; `self_host_candidate_generation_add_slice_stall` is the grounding-frontier row.
- gunbc#9664 ("XL-N") — the native-closure burn-down program with go/no-go gates.

## 1. What you actually get at the end

In daily terms: **you edit `.dag` files, and the compiler that compiles them was itself compiled from `.dag` files.** The hand-written Rust seed is gone except a pinned, content-addressed bootstrap kernel (the wave doc sizes it at roughly 8–15k LOC); everything else the compiler does — reading source, resolving names, deriving types, translating, emitting — is emitted from the graph it compiles.

The proof shape, per the roadmap nodes' own RED controls:

- The generator builds itself **twice**; the second build is shown never to have reached the old compiler — run with the old one taken away, or with a receipt proving it was never called. Behaving the same is not accepted as proof of independence.
- Every self-emitted module passes the same behavioral corpus as the seed it replaced, including the deliberately-broken controls that must still fail. Byte-identity with the seed is explicitly **not** the goal (operator, 2026-07-08) — the seed's warts are not reproduced.
- Required CI runs `v2.test.*` through the native emitted crate; the interpreter survives only for `src/v1` regen until that too is gone.

The payoff is DESIGN §7's: **language design opens up.** A new guarantee is a row, not a compiler fork; and because the substrate reads one grammar in both directions over many media, that row applies on top of an existing language through ingest — no adoption problem. This is the economic point of the whole exercise: today every improvement routes through the generator we are trying to delete.

What you do **not** get (§7's honesty boundary): the compiler does not invent unstated intent, does not prove unmodeled external reality, and does not lift arbitrary predicates to proof. Runtime mechanisms — typed refusal, totality, rollback, budgets — remain real at honest boundaries.

## 2. Where "here" is — by instrument

Each line names the producer that re-derives the state. Run it; do not believe this sentence.

- **Axis A — the real path, slice by slice.** The add slice (the `add` fn of `std.integer`) is green end-to-end on the dag path as of #10673: infer derives grounding by declared-inhabitant roster membership, and the ingest→compile round trip is byte-faithful. Instrument: `gunbc run --source-root dag --source-root src/v2 --entry src/v2/test/claim/execution/self_host_candidate_generation_stage_verdicts_test.dag --function add_slice_stage_verdicts_entry`. The **direct-rust-door** (roadmap `v2-emitter-direct-rust-door`) is **green by execution as of 2026-09-06**: the production walk reaches `ArtifactProduced`, all five leaf projections establish (source fidelity byte-equal to `direct_rust_door_expected_source`, producer identity, seed-emitter absence, `SourceProduced` qualification, missing-rule refusal), and the known-red admission row deleted per its own dissolution, promoting the group root to the lane's permanent regression wall. Instrument: `claim_batch --source-root dag --source-root src/v2 --entry src/v2/test/claim/long/direct_rust_door_production_group_test.dag --functions direct_rust_door_production_group_closing_expectation_holds` (a `long/` row — it reconstructs the pipeline per evaluation; the cost defect is declared in the test module's header).
- **The grounding frontier.** The door specimen's frontier is closed: six derivation rules now cover declared-inhabitants roster membership, binding denotation, Conj/Arrow product introduction, canonical-operations and grammar-productions roster membership, and binding references from the enclosing arrow's domain. The stall carrier `self_host_candidate_generation_add_slice_stall` holds the live population and its `next_rung_trigger` names the parse-product family (grammar parse-tree nodes of the python/typescript fixtures). The add-slice rule and its blast-radius handling (#10673) are the template.
- **Axis B — the native closure (XL-N).** The XL-0 gate (`cargo check` = 0 on the emitted compiler closure) is **not satisfied**; the issue's own receipt table records that the closure reached complete rustc type checking for the first time at XL-0A and carries the honest error population there. gunbc#10266 (fabricated runtime panics on representation-identical casts) is filed as an independent compiler slice on this path. Instrument (the issue's own): `gunbc compile --source-root dag --source-root src/v2 --entry src/v2/compiler/00_compile.dag --target rust --output-dir /tmp/v2emit && cd /tmp/v2emit && cargo check --message-format short`.
- **Axis C — the seed dissolves.** `v2.compiler.self_host.seed_retention_frontier` holds every retained seed file as a declared row; `undeclared_reason_rows()` is the prioritizable debt, and the census refuses unrostered retention in three directions. `v2.compiler.self_host.emit_coverage_frontier`'s `interpreter_retained_rows()` and `v2.compiler.self_host.native_routing_frontier`'s roster cover the witness-execution axes.
- **Wave position.** Per the wave doc, Wave 1's four exit items all carry landed receipts; Wave 2 (first real flips — emit-surface tracks self-emit, behaviorally verify, and **replace** their v1 counterparts) is the open wave. The wave doc's header records that its roster carrier was deleted and per-module dispositions must be re-derived from the emitter — which is what the door-first sequence below does.

## 3. The sequence from here

Ordered by the roadmap chain; each step names its green condition as the node's own contract states it.

1. **The door** (`v2-emitter-direct-rust-door`) — **LANDED 2026-09-06.** The loop's actual findings, for the record: not missing `TargetModel` rows. (a) The grounding frontier closed by six real derivation rules in `04_infer.dag` (see §2's frontier line). (b) Emission needed the right composition, not more rules: a resolved module is a module shell over named declarations, so the production path is `generate_rust_module_emission_candidate` — collect the produced-decl-shaped nodes, admit exactly one, emit through `v2.compiler.emit_produced` — while the raw whole-tree `generate_rust_emission_candidate` remains the fixture lane's composition with its own dissolution trigger. (c) One representation decode gap: resolution canonicalizes surface operators into canonical-operation wire nodes, so the translate value-expression projection decodes the operator position with `canonical_operation_from_wire_node` first and falls to the surface-token table only on a wire miss. Green condition met: the production observation reaches `ArtifactProduced`, all five leaf projections establish, the known-red expectation row deleted in the same change, and the group root promotes to permanent regression evidence (`v2_emitter_direct_rust_door_contract_dissolution_note`).
2. **The parse-product grounding family** (the stall's named trigger). Generalize the roster-membership derivation from the dag language roster to the grammar parse-tree rosters of python/typescript. Greens the stall's remaining population and the same-grammar parse ingest bridge; the stall retires by its trigger.
3. **First behavioral module** (`v2-emitter-first-behavioral-module`). One already-tested module regenerated by the new generator; its existing tests — including the deliberately-broken one — pass unchanged, and the run is shown never to have reached the old generator.
4. **The XL-N milestones, in the issue's order** — arms → 0; emitter defects C, B → 0; crate compiles; native = interpreted with every divergence typed; swap. This is Wave 4's "pipeline runs on emitted Rust" executed as a burn-down, and its rules bind: fixes land in `.dag` and reach Rust via regen, and each closed class gets a `test.claim` fixture under the required gate. Its no-go gates are real: M2 stops if a fix needs a fact the inferred tree doesn't carry (an `04_infer` modeling gap — file it, don't grind); M3 re-plans if a class needs a language capability we don't have.
5. **Native bootstrap, then fixed point** (`v2-emitter-native-bootstrap`, `v1-emitter-fixed-point`). The two-round self-build for the generator, then the same two-round test extended stage by stage to the rest of the compiler. Then `dag/gunbc/v1/v1_deletion_plan.dag` executes; the seed-retention roster drains as modules flip, and Wave 3's first obligation — a derived denominator for what the compiler consists of — is named in the wave doc.
6. **Parallel track, not a wave gate:** the import-deletion ladder (wave doc §3) — B3's migration alongside Waves 2–3, B4's grammar deletion at Wave 4.

## 4. The autonomy contract

An agent can execute, end to end and without supervision, the per-slice loop: run the instrument → diagnose from the typed diagnostics → land the `.dag` fix → flip each affected witness per §4b(4) with per-witness justification → narrow the roster/stall rows in the same change → commit, push, open the PR, and iterate to green CI. #10673 is the worked example of that loop. Declaring seed-retention reasons, landing derivation rules, and burning XL-N error classes are all inside it.

The operator gates that remain, each with its repo citation:

- **Merges.** The agent never merges; every slice pauses at a green PR for human review.
- **Roadmap node acceptance.** ROADMAP.md's header states every active row requires explicit manual acceptance; a green node is handed back, not self-accepted.
- **CI job roster growth.** Closed without operator sign-off (DESIGN *Building & checks*, 2026-09-04 ruling). XL-N's M5 swap changes what required CI runs on — it is an operator decision by construction.
- **Scaffold admission.** §5: approval is external to the diff; an author cannot self-approve a scaffold.
- **Declared rung drops.** §4b(3): the agent can author the full declaration (previous rung, temporary rung, reason, population, restoration trigger); landing it is review-gated.
- **Language-semantics rulings** that the tree cannot decide. The Atom-authority question is the template: surfaced de-jargoned, decided by the operator (2026-09-06: the namespacing rules answer it), recorded in the carrier. These are rare but real, and the agent's obligation is to stop and raise them rather than guess.

## 5. What would make this plan wrong

- An XL-N no-go gate firing (M2's modeling gap, M3's missing language capability) — re-plan at that gate, per the issue's own rule.
- ~~The door's emission diagnostics turning out to be an `04_infer` modeling gap rather than `TargetModel` rows~~ — discharged 2026-09-06: the door is green; the gaps were derivation rules, one emission composition, and one operator-representation decode, none of them a modeling wall.
- A Wave-2 flip surfacing behavioral divergence that re-sequences the module order — typed as emitter bug or interpreter bug either way, but the sequence absorbs it.
- This file aging. It is a position read, not a carrier; the instruments in §2 are the standing truth.
