# Pure Bootstrap to Zero Manager Brief

**Status:** `PROPOSAL` — pending cascade promotion of
[`docs/design-pure-bootstrap-zero.md`](../design-pure-bootstrap-zero.md)
(in PROPOSAL on main as of #762). Until that cascade lands, this
brief is the manager's scope-in-preparation; it does not dispatch
work formally. Pre-promotion deliverables (audit table + PB-1
brief amendment + first prototyped lane closure) are the manager's
first work and gate the cascade.

**Naming clarity.** This is the **Pure Bootstrap to Zero Program
Manager**. The program is **parallel to R2**, not part of R2. Per
the locked Strong Post-R2 stance ([`docs/r2-structure.md`](../r2-structure.md)
Decisions locked), R2 = thesis close with Grounding Completeness
as single co-anchor. Zero-floor is implementation completeness —
making the codebase match the thesis at the file-count level. Two
distinct concerns; two parallel programs; do not conflate.

The release-ledger placement of this program (R2 absorption /
separately-named program / under Director closure) is **deferred
to the cascade promotion PR** per the design doc's Promotion
mechanism. Until then: program runs structurally without
committing to a release-ledger label.

**Cross-program coordination routing.** Cross-program signals
between Zero-Floor and R2 (Grounding + R2 ad-hoc lanes) route
through Director, who is the only entity with both program
contexts. Manager-to-manager direct coordination is for
substrate-shape only (per Cross-manager notifications queued
below); broader cross-program concerns (scope, ledger placement,
sequencing decisions affecting both programs) escalate to
Director.

## Orient before reading

- **Program scope authority:** [`docs/design-pure-bootstrap-zero.md`](../design-pure-bootstrap-zero.md)
  (PROPOSAL). Names goals (γ shape, bootstrap-as-data, 0 hand-Rust
  in v3 source tree), lane structure, dependency DAG,
  acceptance gates, STOP-AND-ESCALATE conditions, pre-promotion
  deliverables. Single source for what 0-floor means and how to
  verify reaching it.
- **Legacy authority being superseded:** [`docs/design-pure-bootstrap.md`](../design-pure-bootstrap.md)
  (≤5-floor framing) + [PR #756](https://github.com/gunb-ai/gunbc/pull/756)
  (2-3 principled-floor framing). Both retract atomically in the
  cascade promotion PR; until then, they remain live authority.
- **Thesis claim tracked:** [`THESIS.md`](../../THESIS.md) §"Thesis
  claims — complete list" + [`docs/thesis/self-inspection.md`](../thesis/self-inspection.md).
  Hand-Rust in v3's source tree contradicts "the substrate is its
  own subject" at the implementation layer; this program closes
  that contradiction.
- **Subsumed brief:** [`docs/briefs/pb-1-data-driven-bootstrap.md`](pb-1-data-driven-bootstrap.md).
  PB-1 (XXL, 5 sub-lanes) was authored under R1's ≤5-floor framing;
  its non-goals invert under 0-floor and the brief gets amended in
  the cascade promotion PR. Lane-owner reports here going forward —
  see "Hand-off points / PB-1 lineage transition" below.
- **Coordination context:** Director session (currently
  `zesty-bear-812`); per
  [`docs/briefs/r1-director-brief.md`](r1-director-brief.md)
  Staffing set + [`docs/r2-structure.md`](../r2-structure.md)
  Manager structure (R2 has 1 standing manager + Director;
  Zero-Floor adds a second standing manager parallel to R2).
- **R1 closure transition:** R1 Self-hosting Manager archives at
  R1 close. PB-1 lineage transition (currently reporting through
  R1 Self-hosting) routes here at archival. See "Hand-off points"
  below for the explicit channel.

## Slice

This manager owns the full Pure Bootstrap to Zero program per
[`design-pure-bootstrap-zero.md`](../design-pure-bootstrap-zero.md)
§"Subsumed lanes" + §"New lanes":

**Subsumed (existing lanes folding under this manager):**

- **PB-1** ([brief](pb-1-data-driven-bootstrap.md), XXL) —
  data-driven bootstrap loader. 5 sub-lanes (PB-1-a through
  PB-1-e). Lineage transition from R1 Self-hosting at R1 close.
- **PB-4** — `lower.rs` migration. Blocked on SG-3 series + lower.dag
  authority.
- **PB-5** — `infer.rs` migration. SG-4 dispatch.
- **PB-6** — `emit.rs` + emit-target cluster (`emit/python_target.rs`,
  `emit/rust_target.rs`, `emit_rust.rs`). Lane 1e dependency.

**New (introduced by this program):**

- **PB-Substrate** (M-L) — generate `dag.rs`, `dag/ports.rs`,
  `dag/effects.rs` from `src/v3/std/substrate.dag`. Cementing test:
  generated Rust matches structural facts.
- **PB-Lib + PB-Build** (M, subsumes PB-8 aspiration) — `lib.rs` and
  `build.rs` generated; trampolines or vanish.
- **PB-Runtime** (M-L) — `test_runner.rs`, `lens_apply.rs`,
  `lens_testgen.rs`, `post_emit_verifier.rs` from `.dag` authorities.
- **PB-Bootstrap-Process** (M, conceptual core) — `bootstrap.dag`
  declares the bootstrap workflow as data; `bootstrap.rs` becomes
  generated trampoline. Includes N=0 runtime boundary verification
  per design doc (operation-set bounded, size budget, substrate-
  runner equivalence test).
- **PB-Workflow** (continuation) — `workflow_idempotency.rs` and
  `workflow_parallelism.rs` migrate as Lane 2 dissolution lands.
- **PB-Tier1-Sweep** (per-file fast-retire, S each) — 13 Tier-1
  files (regen binaries + bin helpers) retire as their backing
  migrations land.

## Framing question this manager answers

**Does v3's source tree reach zero hand-authored files — substrate
generated from `.dag` substrate model, evaluator generated from
primitive operations only, bootstrap process declared as data,
test residual dissolved — such that "the substrate is its own
subject" holds at the implementation layer rather than just the
thesis-claim layer?**

Today (pre-dispatch state):
- **Live count authority:** `src/v3/compiler/tests/integration/sg0_census_test.rs`
  `EXPECTED_HAND_AUTHORED_NON_TEST` and `EXPECTED_HAND_AUTHORED_TEST`
  arrays. Read at dispatch time; brief does not restate the count
  inline (drift risk). Reference snapshot: 35 NON_TEST files at
  brief authoring (post #763 `lens_depth.rs` retirement); use the
  live array as the source-of-truth for any audit work.
- TESTING.md "Post-R2 shape" residual carves out two Rust-authored
  test categories permanently.
- `bootstrap.rs` is hand-Rust (~470 LOC) with the chicken-egg
  framing: Dag::new() needs the compiler pipeline.
- Zero-floor design doc (#762) lays out the program; pre-promotion
  deliverables not yet authored.

The ask: close all program lanes per design doc acceptance gates
(`EXPECTED_HAND_AUTHORED_NON_TEST = 0`, `EXPECTED_HAND_AUTHORED_TEST = 0`,
TESTING.md rewritten, DB-8 fixed-point converges, all `[ext]`
predicates evaluate against generated authorities, `bootstrap.dag`
declares workflow including N=0 resolution).

## Sequence + dispatch

**Critical path** is **PB-Bootstrap-Process** — depends on PB-1
(data-bootstrap) + PB-4/5/6 (compiler in .dag) + PB-Substrate
(substrate types in .dag) being usable at minimum. The other lanes
are fill queues operating in parallel; any available worker picks
top-priority unblocked work.

**Pre-promotion phase (Day-1 dispatch).** Cascade promotion requires
**four** pre-promotion items per
[`design-pure-bootstrap-zero.md`](../design-pure-bootstrap-zero.md)
"Pre-promotion deliverables" + "What gate" sections:

- **Deliverable 1: 35-file audit table.** Author the file-by-file
  mapping (file → why-currently-hand-authored →
  migration-path-into-PB-lane) inline for the cascade promotion PR.
  Director already authored a starting categorization in
  conversation with PM; this manager refines and authorities it.
- **Deliverable 2: PB-1 brief amendment.** Amend
  [`pb-1-data-driven-bootstrap.md`](pb-1-data-driven-bootstrap.md)
  non-goals to align with 0-floor (currently they explicitly
  reject deletion of tokenize/parse/lower/infer/emit and reject
  binary-blob format; under 0-floor those non-goals invert as
  scope absorbed into PB-* program lanes).
- **Deliverable 3: TESTING.md "Post-R2 shape" rewrite.** Per
  design doc Promotion mechanism — TESTING.md residual carve-outs
  (compiler-internal unit tests for Rust-only helpers; external-
  toolchain boundary tests) dissolve under 0-floor. **Authoring
  ownership = Director-call at promotion time** per design doc:
  bundle into cascade PR, or sibling PR landed first. This manager
  surfaces the readiness signal (when PB-Runtime and ExecuteCommand-
  based TestClaim migration work is sufficient evidence to author
  the rewrite); Director picks bundle-vs-sibling and either authors
  or directs authoring.
- **Gate item 4: first prototyped lane closure.** Per design doc
  Promotion mechanism's "What gate" — cascade PR includes at least
  one prototyped lane closure proving an existing hand-Rust file
  can be retired via `.dag` migration without regression. This
  manager picks the lane (e.g., PB-Substrate pilot or PB-1-a
  continuation) and executes end-to-end before the cascade locks
  the framing.

Items 1, 2, 4 are this manager's authoring; item 3's authoring is
Director-call. All four gate cascade promotion.

**Post-promotion phase (parallel-capable):**

- **Day-1 (post-promotion).** PB-Substrate dispatches; pattern
  proves substrate-type generation. PB-1 sub-lanes continue;
  PB-1-a should be done or near-done from R1 era — verify and
  re-baseline. PB-4/5/6 dispatch as substrate evolves.
- **Day-1.** PB-Runtime dispatches. Per-file parallelism over the
  4 runtime files (test_runner / lens_apply / lens_testgen /
  post_emit_verifier).
- **As PB-Substrate + PB-1 reach usability.** PB-Lib + PB-Build
  dispatches; `lib.rs` + `build.rs` migrate to trampolines.
- **As PB-1 + PB-4/5/6 reach usability.** PB-Bootstrap-Process
  dispatches. N=0 runtime boundary verification gates land per
  design doc.
- **Continuous, gated on backing migrations.** PB-Tier1-Sweep
  retires regen binaries + bin helpers as their backing files
  retire. Per-file PRs.
- **Continuous, gated on Lane 2 dissolution.** PB-Workflow handles
  workflow_idempotency / workflow_parallelism as Lane 2 dissolves.

Lane ordering is a **fill-queue model**, not strict sequential. PB-
Bootstrap-Process is the sole critical-path spine. Cross-lane
dispatch failures surface as blocked workers on specific
sub-lanes; manager reassigns to unblocked queues.

## Hand-off points

This program is **parallel to R2**; coordination crosses two
manager boundaries (Grounding + Director) and the R1-closure
transition.

### PB-1 lineage transition (load-bearing)

PB-1 was authored under R1 era. Its lane-owner (whoever drives
sub-lane PRs against `pb-1-data-driven-bootstrap.md`) currently
reports through R1 Self-hosting Manager. R1 Self-hosting Manager
archives at R1 close.

**Transition mechanic:** at R1 close, PB-1 lineage explicitly
transitions to this manager. The cascade promotion PR for the
zero-floor design doc names this transition in its acceptance
record, alongside the PB-1 brief amendment. PB-1 sub-lane PRs
authored after R1 close cite this manager as the reporting
channel; pre-R1-close PRs continue under R1 Self-hosting until
archival.

If R1 closure happens before this manager's brief promotes, PB-1
lineage transitions to Director ad-hoc (interim) until brief
promotes; then transitions here.

### Cross-manager notifications queued

- **Grounding Manager** (substrate-shape coordination). Bidirectional:
  - **Zero-Floor → Grounding:** when PB-Substrate lands shape-affecting
    changes to `dag.rs` or `dag/*` (substrate type definitions
    generated from `substrate.dag`), notify Grounding so T-Ground-
    Engine work can adapt. Substrate types are load-bearing for
    Grounding's inhabitance-search engine.
  - **Grounding → Zero-Floor:** when T-Ground-Engine work needs
    new substrate fields or operations beyond what the current
    `substrate.dag` declares, notify Zero-Floor before authoring
    against shape that's mid-migration. Avoid double-edits to
    `dag.rs` from both sides.
- **R1 Self-hosting Manager** (PB-1 lineage; transitional, until R1
  close). Coordinate the lineage transition timing — Self-hosting
  signals when ready to hand off; Zero-Floor confirms ownership.
  This entry retires when R1 Self-hosting archives.
- **Director** (escalation hub for scope changes, cross-program
  conflicts, release-ledger placement decisions, cascade
  promotion PR authoring).
  - **Specific coordination case: `tokenize_char_class.rs` retirement.**
    PB-Tier1-Sweep retirement of `tokenize_char_class.rs` (per
    Working state) closes Class 5 Gap 3 from the substrate side,
    which unblocks the R2 T-Substrate 4th sub-lane (charclass phase-2
    consumer = tokenizer). Zero-Floor Manager signals Director when
    this retirement lands; Director coordinates dispatch of the
    R2 T-Substrate sub-lane (Director-ad-hoc per `r2-structure.md`,
    no standing R2 manager owns it). One-sided note from this brief
    by design: the consumer side is Director-dispatched, so the
    bidirectional pairing collapses to Zero-Floor → Director.

### Up to director

- **Cascade promotion PR.** This manager prepares three of the
  four pre-promotion items (audit table + PB-1 brief amendment +
  first prototyped lane closure); the fourth (TESTING.md rewrite)
  is Director-call on authoring per design doc Promotion mechanism.
  Director authors the cascade PR itself (atomic across multiple
  authority docs) using the manager's deliverables as inputs.
- **Release-ledger placement.** Whether the program lands as R2
  absorption / separately-named program / under Director closure
  is a Director call at promotion time per the design doc.
- **N=0 resolution shape pick.** Three options scoped in the
  design doc (shipped binary / `gunbc-runtime` crate / rustc
  proc-macro). Director picks at or before PB-Bootstrap-Process
  acceptance; doesn't gate earlier lane work.
- **Scope-creep flags.** If a lane surfaces work broader than its
  scoped acceptance criterion, escalate rather than absorb. The
  design doc commits to bounded subsets; scope drift would re-open
  framing decisions the cascade PR locks.

## Working state

Lane-owner dispatch status (update as sub-deliverables close).
Section populates on cascade promotion + first dispatches; entries
below are scaffolding.

**Pre-promotion items (all four gate the cascade promotion PR):**
- [ ] 35-file audit table authored (file → why-current-hand-authored
      → migration-path-into-PB-lane). Inline in cascade PR. **Owner:** this manager.
- [ ] PB-1 brief amendment ([`pb-1-data-driven-bootstrap.md`](pb-1-data-driven-bootstrap.md))
      non-goals revised to align with 0-floor. Amends in cascade PR. **Owner:** this manager.
- [ ] TESTING.md "Post-R2 shape" rewrite (residual carve-outs dissolve).
      **Owner: Director-call** at promotion time per design doc — bundle into
      cascade PR or sibling PR landed first; this manager surfaces the
      readiness signal when PB-Runtime + ExecuteCommand-based TestClaim
      migration is sufficient evidence to author the rewrite.
- [ ] First prototyped lane closure (PB-Substrate pilot or PB-1-a
      continuation) proving migration pattern works. Cited in
      cascade PR.

**PB-1 (XXL, subsumed; lineage transition pending R1 close):**
- [ ] PB-1-a (std fixtures generated constructors) — verify
      current state from R1 era; re-baseline.
- [ ] PB-1-b (STAGED_FILES generated constructors)
- [ ] PB-1-c (V3_SPECS generated constructors)
- [ ] PB-1-d (COMPILER_FILES generated constructors)
- [ ] PB-1-e (runtime-path retirement + measurement)

**PB-Substrate (M-L):**
- [ ] `dag.rs` generated from `src/v3/std/substrate.dag`
- [ ] `dag/ports.rs` generated
- [ ] `dag/effects.rs` generated
- [ ] Cementing test: generated Rust matches substrate model

**PB-4 (lower):** *(blocked on SG-3 series + `lower.dag` authority)*
- [ ] `lower.dag` authoring
- [ ] `regen_lower` binary
- [ ] `lower_generated.rs`
- [ ] `lower.rs` retired

**PB-5 (infer):** *(SG-4 dispatch)*
- [ ] `infer.dag` + dispatch
- [ ] `infer_generated.rs`
- [ ] `infer.rs` retired

**PB-6 (emit cluster):** *(Lane 1e dependency)*
- [ ] `emit.rs` migrated
- [ ] `emit/python_target.rs` migrated
- [ ] `emit/rust_target.rs` migrated
- [ ] `emit_rust.rs` retired (re-export shim collapses)

**PB-Lib + PB-Build (M):**
- [ ] `lib.rs` generated trampoline
- [ ] `build.rs` generated trampoline (or eliminated)

**PB-Runtime (M-L):**
- [ ] `test_runner.rs` migrated
- [ ] `lens_apply.rs` migrated
- [ ] `lens_testgen.rs` migrated
- [ ] `post_emit_verifier.rs` migrated

**PB-Bootstrap-Process (M, conceptual core):**
- [ ] `bootstrap.dag` authored (load order + target state +
      invariants + entry point)
- [ ] Universal evaluator generated from primitive-operations
      authority
- [ ] `bootstrap.rs` retired or trampoline only
- [ ] N=0 runtime boundary verification:
  - [ ] Operation set bounded (Node/Conj/Disj/Cardinality/Bit only)
  - [ ] Size budget met (<500 LOC outside v3 tree)
  - [ ] Substrate-runner equivalence test passes
- [ ] N=0 resolution shape picked (Director call)

**PB-Workflow (continuation):**
- [ ] `workflow_idempotency.rs` migrated (Lane 2 dissolution)
- [ ] `workflow_parallelism.rs` migrated (Stage 2e .dag surface
      lands first)

**PB-Tier1-Sweep (per-file fast-retire; depends on backing migrations):**
- [ ] regen_bootstrap.rs
- [ ] regen_lens.rs
- [ ] regen_parse.rs
- [ ] regen_parse_tables.rs
- [ ] regen_tokenize.rs
- [ ] regen_v3.rs
- [ ] regen_bootstrap_emit.rs
- [ ] regen_parse_emit.rs
- [ ] regen_parse_tables_emit.rs
- [ ] dag/builder.rs
- [ ] lens_unused_parameters.rs
- [ ] dimension.rs
- [ ] tokenize_char_class.rs (also unblocks R2 T-Substrate
      4th sub-lane via Class 5 Gap 3 closure — coordination point
      with Director-dispatched R2 work)

**Acceptance gates (per design doc):**
- [ ] `EXPECTED_HAND_AUTHORED_NON_TEST` count = 0
- [ ] `EXPECTED_HAND_AUTHORED_TEST` count = 0
- [ ] TESTING.md "Post-R2 shape" residual rewritten to 0-residual
- [ ] DB-8 `self_host_fixed_point` converges bit-identically
- [ ] All `[ext]` test predicates evaluate against generated
      authorities
- [ ] `bootstrap.dag` declares workflow + N=0 resolution

Decisions log (append as they happen):

- **2026-04-25** — Brief authored per Director request after #762
  merge (charclass reclassification + zero-floor design doc
  PROPOSAL).
- **2026-04-25** — **PB-1 lineage transition: automatic at R1
  Self-hosting archive.** No explicit handoff PR required. Verification
  on results: PB-1 sub-lane PRs authored after R1 close should cite
  Zero-Floor Manager as reporting channel; if they cite R1 Self-hosting
  (which has archived), the transition didn't take and Director
  surfaces it. Manager confirms first post-R1-archive PB-1 PR routes
  here as the verification artifact.
- **2026-04-25** — **Cascade promotion PR authoring split.** Manager
  authors the three pre-promotion deliverables as separate PRs (audit
  table inline-able into cascade; PB-1 brief amendment; first
  prototyped lane closure). Director authors the cascade promotion PR
  itself using those as inputs. Pattern matches "what worked" from R1
  brief refreshes: managers own their scope surfaces; director owns
  cross-cutting authority changes (the cascade is cross-cutting across
  4 authority docs).

Open questions for director:

- _(none — both resolved 2026-04-25; see Decisions log below)_

Cross-manager notifications queued:

- **Grounding Manager**: brief authored 2026-04-25; coordination
  channel established for substrate-shape changes (bidirectional
  per Hand-off points). No active signal yet — pre-promotion.
- **R1 Self-hosting Manager** (transitional): PB-1 lineage
  transition queued for R1 close. Self-hosting Manager should
  acknowledge in their working state when ready to hand off.
- **Director**: brief in PROPOSAL pending cascade promotion;
  three pre-promotion deliverables are first work and gate the
  cascade.
