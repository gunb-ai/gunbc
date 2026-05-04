# R3 Row-85 PB-Zero Method-Template Projection Packet

> **Status:** docs-only worker packet (planning + dispatch-shape).
> **Mode:** PROPOSAL → DECISION-GATED. Per the merged STOP matrix this
> packet does **not** authorize a PB-Zero worker to start implementation
> today; it defines (a) the substrate/Director decision that must land
> first and (b) the dispatchable worker shape that becomes valid only
> after that decision is recorded.
> **Tracks:** R3 Debt-Paydown row 85 — *Method-template consumer
> migration* (`docs/debt/r3-debt-paydown-ledger-2026-05-02.md:85`,
> `ROADMAP.md:512`).

## 1. Packet scope

This packet turns Grounding #1133's "Phase-2 retirement is blocked on
Gap 4 + Gap 5" signal into a concrete dispatch-shape grounded in the
existing audits on `main`. It does **not** invent a substrate snapshot
schema, does **not** choose a canonical read surface, and does **not**
re-author the row-authority list. Both audits already concluded those
decisions are Substrate/Director-owned (see §3 below).

**Gap 4** — bootstrap-Dag row-consumer projection for v2 emit.
**Gap 5** — map-shaped `LanguageSpec.method_templates` rewrite in
`src/v2/languages.dag`, sequenced strictly after Gap 4.

## 2. Authoritative inputs (verified at HEAD)

- `docs/briefs/method-template-consumer-migration-audit.md` (PR #1549,
  merged) — Phase-1 consumer inventory + retirement sequence; §"Substrate
  Gaps Blocking Complete Migration" enumerates the same Gap 4 / Gap 5
  shape.
- `docs/audit/pb-zero-v2-method-template-row-authority-consumer-gap.md`
  — architectural boundary (`v2 ∌ v3.std.*`); routing split between
  Substrate/Grounding and PB-Zero; §"Scope clarification needed before
  implementation dispatch" lists the still-open canonical-read-surface
  question.
- `docs/audit/pb-zero-v2-canonical-read-surface-options-stop-matrix.md`
  — STOP matrix over six candidate surfaces; explicit conclusion: *no*
  canonical surface has been chosen; active Substrate manager + Director own
  the next decision per `INVARIANTS.md` `## P1` / `## P2`.
- `src/v3/std/rust_method_template_contracts.dag:6-17` (header) +
  `python_method_template_contracts.dag` + `go_method_template_contracts.dag`
  (sibling deferrals) — operational blocker recorded in row authorities
  themselves.
- `src/v2/languages.dag:390-400, 544, 688, 832, 979` — map-shaped
  `LanguageSpec.method_templates: Map<String, String>?` and four
  per-target assignments (Rust, Python, Go, RustTest).
- `ROADMAP.md:512` — *consumer migration over more row population* is
  the immediate-priority signal; row population is parallel-authority
  drift until consumers retire.
- `docs/debt/r3-debt-paydown-ledger-2026-05-02.md:85` — debt row,
  acceptance "Migrate consumers off old runtime/emit tables before
  adding more rows."

## 3. Pre-dispatch determination — is this PB-worker work or Director/Substrate decision?

**Determination: Director/Substrate decision first; PB-worker dispatch
second.**

The STOP-matrix audit (§"Do existing docs already pick one canonical
surface?") concluded **No** at merge. None of the six candidate read
surfaces (committed generated snapshot, bootstrap-Dag/(γ) staged load,
PB-process-owned hook, `v3.std.*` import bridge [non-option],
`collect_dag_sources` source-root expansion, cross-binary extract) has
been adopted as canonical. Three substrate-named questions remain open
verbatim from that audit:

1. Which artifact is canonical for template-row facts consumed outside
   the v3 std module graph (snapshot vs bootstrap-declared load vs
   hybrid), with versioning/ratchet so consumers cannot drift.
2. Who authors the typed contract for that artifact (Substrate-first
   per P1, vs PB-bootstrap-owned slice with substrate-named targets).
3. Whether v2 emit retirement attaches to build-step consumption,
   test-oracle consumption, or both — coordinated post-R3 per
   `docs/r2-structure.md`.

Per `feedback_no_textual_enforcement_bridges.md` and
`feedback_audit_adjacent_authority_first.md`: a PB worker that picks
any of those answers in code unilaterally would create exactly the
parallel-authority hazard both audits flagged. **The first PR
gating row 85 implementation is therefore not the implementation
itself; it is a Substrate/Director-owned decision-routing PR
(distinct from row 85's R3-Grounding-owned implementation work
named in `ROADMAP.md:512` and the ledger).** This packet routes that
decision and only describes the implementation packet that becomes
dispatchable afterward.

## 4. Decision-routing artifact (this packet's first deliverable)

**Owner of this §4 decision-routing artifact:** active Substrate
manager + Director (gate). This is a **distinct deliverable from row
85 itself**: `ROADMAP.md:512` and
`docs/debt/r3-debt-paydown-ledger-2026-05-02.md:85` name **R3
Grounding** as the owner of row 85 (the consumer-migration
implementation work). The two audits cited in §2 route the *canonical
read-surface decision* — which precedes implementation — to Substrate
+ Director per `INVARIANTS.md` `## P1` / `## P2`. Sequencing: the
Substrate/Director §4 artifact lands first; the row-85 Grounding
implementation work (Gap 4 + Gap 5 + leaf-emit migrations) consumes
that decision and remains R3 Grounding-owned per the ledger. Older
audits cited in §2 say "R2 Substrate Manager" because they predated
R3 routing — read those references as "active Substrate manager"; row
85's ledger ownership is unchanged.
**Surface:** new short note under `docs/decisions/` (or extension of
the active Substrate manager's brief) — Substrate manager chooses.
**Required content:**

- Names the canonical read surface for `MethodTemplateContract` row
  facts crossing the `v2 ∌ v3.std.*` boundary, picked from the STOP
  matrix's six candidates (or an explicitly-named seventh that the
  matrix did not list, with justification under `INVARIANTS.md` `## P1`
  procedure).
- Names the typed contract owner for that surface (which `.dag` file
  declares the carrier; whether a new carrier is needed beyond
  `MethodTemplateContract` itself).
- States the single-authority / non-fork rule: row text lives in
  `src/v3/std/{rust,python,go}_method_template_contracts.dag` only; the
  read surface is a **projection**, not a copy.
- States the versioning/ratchet that keeps the projection from drifting
  from the row authority (regen-equality test, isomorphism test, or
  generation-from-rows pipeline).
- Records the v2-retirement coupling: whether projection lands as
  build-step consumption, test-oracle consumption, or both, and that
  full v2 emit retirement remains post-R3.

**Until that artifact lands, no PB-Zero worker on row 85 is
dispatchable.** This packet itself does not satisfy that artifact —
authoring it is an active-Substrate-manager act, not a director-mode
act.

## 5. PB-Zero Gap-4 worker shape (dispatchable only after §4 lands)

### 5.1 Landing surface (named without choosing the schema)

The worker lands the **read path** required by the §4 decision. Concrete
form depends on §4:

- If §4 picks "committed generated snapshot": worker lands the v2-side
  consumer that ingests bytes from the generated artifact and exposes a
  typed `MethodTemplateContract` projection to the v2 emit pipeline,
  via the carrier named in §4. No template strings live in v2 sources.
- If §4 picks "bootstrap-Dag / (γ) staged load": worker lands the
  evaluator/build-step hook that loads the bootstrap slice declared in
  §4, exposing the same typed projection. The slice does not duplicate
  row text in `dsl/` or `src/v2/`.
- If §4 picks "PB-process-owned hook" (consumer-only): worker lands the
  hook that consumes the substrate-named artifact from §4. Hook does
  not redefine row semantics.

In all three cases the worker's **public API to v2 emit** is the same
shape: a target-keyed projection that, given a `MethodRef`, returns a
`MethodTemplateContract` row carrying all five fields required by A4:
`dag_method`, `runtime_template`, `emit_template`, `wraps_result`, and
`placeholder_convention`.

### 5.2 Acceptance criteria (Gap 4)

A1. **Boundary preserved.** No new `use v3.std.*` import in the v2
crate graph. Verified by an existing or newly-added structural ratchet
(audited against `src/v2/tests/src/source_audit.rs` patterns).

A2. **No second map authority.** No new `Map<String, String>` table of
template strings is introduced anywhere in `src/v2/`, `dsl/`, or the
new projection surface. Verified structurally — not by grep — via a
test that asserts the projection consumes from row authorities (or the
§4 named artifact) and exposes typed `MethodTemplateContract` only.

A3. **No copied template text.** Row strings do not appear in any v2
source. The projection's only template-text source is the §4 named
read surface, which itself derives from
`src/v3/std/{rust,python,go}_method_template_contracts.dag`.

A4. **Five-field preservation.** For each `(target, MethodRef)` row
present in the substrate authorities, the v2-side projection exposes
a `MethodTemplateContract` with all five fields: `dag_method`,
`runtime_template`, `emit_template`, `wraps_result`,
`placeholder_convention`. Tested by a parity claim against the row
authority (test shape per §4's chosen ratchet — equality, isomorphism,
or generation).

A5. **Diagnostics-empty bootstrap gate.** Acceptance includes the
ledger-row-82 gate (`diagnostics_empty_after_bootstrap`) being green
for the three contract-row authorities. If that gate is still open at
dispatch, Gap-4 worker is **STOP-blocked** on Substrate row 82 (see
`docs/debt/r3-debt-paydown-ledger-2026-05-02.md:81-82`).

A6. **Source-level ratchet preserved.** `LEGACY_METHOD_TEMPLATE_AUTHORITIES`
and the non-v2 deferral ratchet in `src/v2/tests/src/source_audit.rs`
remain live until §6 (Gap 5) and the leaf-emit migrations land. Worker
does **not** weaken the ratchet.

A7. **No row population.** Worker adds zero rows to the contract
authorities. Substrate row-parity work (today: `fold` row disposition
for Python/Go — see §6.2 residue correction) remains Substrate-owned
and out of scope. *Earlier revisions of this clause cited
`string_contains` Python/Go and Go `chars` as residue; that list is
stale post-#1549/#1598 and superseded by §6.2.*

### 5.3 Non-goals (Gap 4)

- No `LanguageSpec` shape change (that is Gap 5).
- No deletion of `dsl/extdeps/languages/{rust,python,go}/emit.dag` legacy
  authorities.
- No leaf-emit migration in `src/v2/05_emit*.dag` (those land *after*
  the projection exists and after Gap 5).
- No new substrate carrier beyond `MethodTemplateContract` unless §4
  explicitly authorizes one.
- No source-root expansion of `collect_dag_sources` treated as a
  semantic row consumer (STOP-matrix row 5).

## 6. Gap-5 worker shape (sequenced strictly after Gap 4 merges)

### 6.0 Post-#1598 readiness delta (2026-05-04)

PR #1598 (`a04ab525c`) landed Gap 4 as a **bounded `Map<String, String>`
build-step adapter** for `MethodEmitTemplate::Single` rows only —
**not** a typed `MethodTemplateContract` projection. The producer
(`src/v3/compiler/src/pb_method_template_projection_dag_emit.rs`) emits
an ephemeral `generated.method_template_projection` module with
per-target `data <target>_method_template_emit: Map<String, String>`
declarations; v2 consumes it via the source-root mechanism from PR #1575
(see `src/v2/tests/src/pb_method_template_projection_consumability.rs`).
The decision artifact (`docs/decisions/r3-row85-method-template-read-surface.md`)
is committed.

What that means for §6 below:

- **Gap-4 acceptance, retroactive read.** A1 (no `v3.std.*` import) and
  A2 (no second map authority) are satisfied by the producer's typed-
  rows-only source-of-text discipline. A3 (no copied template text in
  v2) holds: the generated `.dag` is ephemeral, never tracked. A7 (no
  row population) holds. **A4 (five-field preservation) is intentionally
  not satisfied** — the merged adapter preserves only `emit_template`
  (Single arm) and per-target identity; `dag_method`/`MethodRef`,
  `runtime_template`, `wraps_result`, and `placeholder_convention` are
  **not** carried through the Map shape. This is the `MethodEmitTemplate::Single`
  scope clamp that #1598 explicitly named. **A4 in its original form is
  parked on Gap 5b.**
- **P3 is not satisfied by #1598.** P3 below requires a *typed*
  `MethodTemplateContract` projection at the four `LanguageSpec`
  assignment sites; the merged adapter is `Map<String, String>`. P3 is
  reframed (see 6.1) and the Gap-5 work splits.

### 6.1 Preconditions (all must hold before dispatch)

P1. §4 decision-routing artifact merged. **Satisfied** by
`docs/decisions/r3-row85-method-template-read-surface.md`.

P2. Gap-4 projection PR merged with the **scope-clamped** subset of
A1–A7 (A1, A2, A3, A7 satisfied; A4 explicitly partial — Single-arm
`emit_template` only — and parked on Gap 5b; A5 / A6 status carried
forward unchanged). **Satisfied for the Single-row Map adapter** by
PR #1598.

P3. **Split into P3a + P3b** as of #1598:

  - **P3a** (Map-shape Single-row consumability): v2 can import the
    generated `Map<String, String>` per target via the ephemeral
    source-root mechanism. **Satisfied** by PR #1575 + PR #1598.
  - **P3b** (typed `MethodTemplateContract` projection at the four
    `LanguageSpec` assignment sites — i.e. `src/v2/languages.dag:544,
    688, 832, 979` reading a row carrying all five fields):
    **NOT satisfied.** Remains gating for any structural `LanguageSpec`
    rewrite. Substrate/Director must approve a typed read shape for
    higher-order rows + non-`emit_template` fields before this
    precondition can flip.

### 6.2 Landing surface — split into 5a (enabled) and 5b (parked)

**Gap 5a — structurally degenerate / no-op post-#1598 (re-classified
2026-05-04).** Earlier revisions of this section (PR #1603) framed 5a
as an overlay-merge rewrite of legacy `dsl/extdeps/languages/{rust,
python,go}/emit.dag::*_method_templates` maps. **That framing is
obsolete.** Re-audit on `origin/main` post-#1598 (`a04ab525c` plus
follow-ups) shows the consumer migration the overlay was meant to
enable already happened in #1598's wider footprint:

- `src/v2/05_emit.dag:84-86` and `src/v2/05_emit_rust.dag:67` import
  `generated.method_template_projection { rust_method_template_emit,
  python_method_template_emit, go_method_template_emit }` directly
  and consume them at `src/v2/05_emit.dag:2573-2584` and
  `src/v2/05_emit_rust.dag:3889`.
- `LanguageSpec.method_templates: Map<String, String>?` at
  `src/v2/languages.dag:400` has **zero live readers** anywhere in
  `src/v2/` or `src/v3/` (verified by `grep '\.method_templates\b'`
  excluding stage0 mirrors and the projection module). The four
  assignments at `src/v2/languages.dag:544, 688, 832, 979` write a
  field nothing reads.
- The legacy `*_method_templates` declarations in
  `dsl/extdeps/languages/{rust,python,go}/emit.dag` are kept alive
  only by `src/v2/tests/src/source_audit.rs::LEGACY_METHOD_TEMPLATE_AUTHORITIES`
  (lines 12–15), not by any data flow.

A 5a overlay-merge in `dsl/extdeps/languages/*/emit.dag` would
therefore have nothing live to overlay for. None of the four feasible
implementation paths fits this packet's STOP boundary:

1. Rewrite legacy `*_method_templates` to import + overlay the
   generated map → either requires uncertain `data` decl with
   non-literal RHS or converts the data decl to `fn`, forcing
   call-site edits at `src/v2/languages.dag:688, 832` (forbidden
   per §6.4 / dispatch STOP).
2. Hand-author the merged map literally → second template-text
   authority (forbidden per A2 / §7 S5).
3. Keep legacy maps as inert literals → no-op; nothing changes.
4. Delete legacy `*_method_templates` + dead `LanguageSpec.method_templates`
   field + four assignments → wholesale legacy deletion + structural
   `LanguageSpec` change; that is Gap 5b territory and crosses §6.4
   STOP.

**Residue correction.** Earlier revisions cited Python/Go
`string_contains` and Go `chars` as legacy-only residue blocking
wholesale replacement. **That list is stale.** On current `main`:

- `string_contains` is a row in `src/v3/std/python_method_template_contracts.dag`
  and `src/v3/std/go_method_template_contracts.dag` (#1549 Gap 1
  classified it as a target-only emit-shortcut name in
  `dsl/std/methods.dag`, with `string_contains_method` rows in both
  contract files).
- Go `chars` was never in legacy `go_method_templates` — verify with
  `grep '"chars"' dsl/extdeps/languages/go/emit.dag`; no hit.
- The actual current residue is **`fold` (Python + Go)**: legacy
  `python_method_templates` and `go_method_templates` carry `fold`
  entries (`functools.reduce({arg}, {recv})` and `v2rt.Fold(...)`
  respectively); the v3 substrate carries `fold_method` through a
  separate `*_language_spec_free_monoid_fold_contract` standalone
  declaration with the `__v3_fold(...)` shape — deliberately *not*
  inside the per-target `<target>_method_template_contracts` list,
  because it is a different arity/contract per
  `docs/briefs/collectionops-algebra-reframe.md`. Rust legacy
  `rust_method_templates()` has `∅` residue (the 9 Single specs
  match the 9 `SingleTemplate` rows in
  `src/v3/std/rust_method_template_contracts.dag`).

### 6.2.1 Remaining real work (out of this packet's PB scope)

The unfinished portion of row-85 consumer migration after #1598 is
not an overlay-merge. It splits into two ordered batches by current
liveness:

**Batch A — dead-authority deletions, gated on `fold` row disposition.**
These authorities have zero live readers outside the source-audit
ratchet and assignments to a dead field.

- **Delete `LanguageSpec.method_templates: Map<String, String>?`
  field** (`src/v2/languages.dag:400`) and the four dead assignments
  (`:544, :688, :832, :979`) — structural change to a Substrate-shaped
  type. Editing `src/v2/languages.dag` is therefore **Gap 5b
  territory**, not 5a.
- **Delete legacy `dsl/extdeps/languages/{rust,python,go}/emit.dag::*_method_templates`**
  (Rust `rust_method_templates()`, Python `python_method_templates`,
  Go `go_method_templates`). Their only readers are the four dead
  `LanguageSpec.method_templates` assignments above; deletion happens
  in lockstep.
- **Shrink the corresponding entries in
  `src/v2/tests/src/source_audit.rs::LEGACY_METHOD_TEMPLATE_AUTHORITIES`**
  (lines 12–15) for the deleted authorities.

Batch A gating: Substrate either folds the standalone
`*_language_spec_free_monoid_fold_contract` rows into the per-target
`<target>_method_template_contracts` list (extending the contract list
shape to admit the differently-shaped fold contract or accepting a
different arity row), or explicitly classifies `fold` as target-only
legacy text with no v3 row.

**Batch B — `wraps_result` consumer migration; P3b-gated.** The
following authority is **live** and must not be deleted with Batch A:

- `dsl/extdeps/languages/rust/emit.dag::rust_method_wraps_result()`
  is read at `src/v2/05_emit_rust.dag:2843`
  (`map_contains_key(rust_method_wraps_result(), function_name)` —
  Rust Rc-wrapping decision). The current Map-shape adapter from
  #1598 deliberately does **not** carry `wraps_result` (A4 partial,
  parked on Gap 5b); the consumer therefore cannot migrate to a
  generated-map read today.
- `dsl/extdeps/languages/rust/emit.dag::rust_simple_method_specs`
  is the substrate that derives both `rust_method_templates()` (dead,
  Batch A) and `rust_method_wraps_result()` (live, Batch B). It can
  only be deleted after Batch B retires `rust_method_wraps_result()`.

Batch B gating: the **P3b typed-read carrier decision** (Gap 5b) must
land first, exposing `MethodTemplateContract.wraps_result` to v2 emit
through a typed projection. Then `src/v2/05_emit_rust.dag:2843`
migrates to read `wraps_result` from the typed row, and only then can
`rust_method_wraps_result()` + `rust_simple_method_specs` be deleted.
Their entries in `LEGACY_METHOD_TEMPLATE_AUTHORITIES` (lines 11, 13)
shrink in that same step.

Owner: **R3 Grounding** for both batches' deletion / ledger sequencing
per `ROADMAP.md:512` and ledger row 85. **R3 Substrate sign-off**
required on `fold` row disposition for Batch A and on the typed-read
carrier shape for Batch B. PB-Bootstrap-Process role ends with #1598's
producer + the docs lineage in this packet; PB does not dispatch
either batch.

### 6.2.2 Implication for §9 sequencing diagram

§9 has been updated directly: the former "Gap 5a — Grounding leaf
re-export migration" node is marked **SUPERSEDED 2026-05-04** with
inline reasoning, and the next live nodes split into **Batch A —
Grounding dead-authority deletion** (gated on `fold` row disposition)
and **Batch B — Grounding `wraps_result` consumer migration +
deletion** (P3b-gated, separated because `rust_method_wraps_result()`
is still a live consumer at `src/v2/05_emit_rust.dag:2843` and the
current Map adapter does not carry `wraps_result`). Owners +
deliverables are named in the diagram body itself. The diagram is
therefore self-authoritative; this subsection serves only as a
lineage breadcrumb back to the §6.2 audit that drove the
re-classification.

**Gap 5b — typed `MethodTemplateContract` projection + structural
`LanguageSpec` rewrite (parked).** Structural rewrite of
`src/v2/languages.dag:390-400`'s `LanguageSpec.method_templates:
Map<String, String>?` to a typed `MethodTemplateContract`-projection
field carrying all five A4 fields, plus higher-order
`MethodEmitTemplate` arms. Specific shape (target-keyed row table vs
per-target lookup function vs full replacement) authored by the Gap-5b
worker against the §4-named carrier extended for typed reads. This
remains parked on a Substrate/Director decision: today no typed
read-shape contract exists for non-Single rows or for the four
`MethodTemplateContract` fields the current adapter drops. STOP+PING
S2 (new-carrier or snapshot-schema decision) applies.

### 6.3 Acceptance criteria (Gap 5b only — 5a is out of this packet's scope)

B1. **No `Map<String, String>` template field on `LanguageSpec`.** Field
either deleted or replaced with a typed projection. State-space audit
per `feedback_state_space_vs_behavioral_invariants.md`.

B2. **All four target spec sites updated.** Rust (line 544), Python
(688), Go (832), RustTest (979) read the typed projection. No site
retains `method_templates: <legacy-map-fn>()`.

B3. **Stage0 regen clean.** Generated mirrors at
`src/v2/stage0/src/v2_compiler_languages.rs:23-55, 379, 522, 703, 886,
1053` regenerate without hand-editing.

B4. **Row-parity gaps preserved as ledger-tracked debt, not silently
deleted.** Current residue (per §6.2 re-audit) is `fold` row
disposition for Python and Go — Substrate-owned; Gap 5b does not
delete legacy `*_method_templates` maps yet — that is a later
authority-deletion PR per audit §"Closing
PR Shapes" row 6.

B5. **Per-PR debt receipt.** PR description explicitly cites
`ROADMAP.md:512` and `docs/debt/r3-debt-paydown-ledger-2026-05-02.md:85`
and reports the new ratchet state (e.g., consumer count remaining).

B6. **Higher-order + non-Single arms covered.** The typed projection
preserves all `MethodEmitTemplate` arms, not only `Single`. Higher-order
rows that #1598 deliberately omitted from the Map adapter route through
the typed read.

B7. **A4 five-field preservation reinstated.** The typed read carries
`dag_method`/`MethodRef`, `runtime_template`, `emit_template` (all arms),
`wraps_result`, and `placeholder_convention`. Verified by parity claim
against the row authority.

### 6.4 Non-goals (Gap 5b)

- No deletion of `dsl/extdeps/languages/{rust,python,go}/emit.dag`
  legacy declarations (5a may convert them to re-exports of the
  generated map; full deletion is a later authority-deletion PR).
- No leaf `05_emit*.dag` migration (separate PR per audit §"Closing
  PR Shapes").
- No retirement of the source-level audit ratchet.
- No `src/v2/languages.dag` edit until P3b flips (typed-read carrier
  approved by Substrate/Director). The merged Map-shape adapter does
  **not** unblock a `LanguageSpec` rewrite — flipping field shape today
  would either drop fields the Map cannot carry or reintroduce a
  parallel typed authority alongside the Map adapter.

## 7. STOP+PING conditions (apply to both Gap-4 and Gap-5 workers)

Each of the following triggers immediate STOP and a comment on the
dispatching manager's inbox issue with the trigger named:

S1. **Canonical-artifact ownership ambiguous.** The worker discovers
    the §4 decision-routing artifact is silent, contradictory, or
    missing on a question the implementation must answer. Do not pick.
S2. **New carrier or snapshot-schema decision required.** Implementation
    cannot proceed without naming a substrate field, variant, or
    carrier shape that §4 did not authorize. Route to Substrate
    Manager per `INVARIANTS.md` `## P1` procedure.
S3. **Row parity gap blocks acceptance.** A row authority is missing
    a `MethodRef` or row needed to satisfy A4 / B-criteria. Current
    residue (per §6.2 re-audit) is `fold` for Python/Go: legacy
    `python_method_templates` and `go_method_templates` carry `fold`
    via `functools.reduce(...)` / `v2rt.Fold(...)`, but the v3
    substrate routes `fold_method` through standalone
    `*_language_spec_free_monoid_fold_contract` rows with a different
    `__v3_fold(...)` arity rather than inside the per-target
    `<target>_method_template_contracts` list. Route to Substrate; do
    not paper over with placeholder rows in v2. *Earlier revisions of
    this clause cited Python/Go `string_contains` and Go `chars`;
    that list is stale and superseded by §6.2.*
S4. **Source-root / `collect_dag_sources` shortcut tempts.** Any
    consideration of expanding v2 source roots as the *semantic* row
    consumer hits STOP-matrix row 5 — escalate, do not implement.
S5. **`v3.std.*` import bridge tempts.** Hard STOP. Architectural
    boundary violation; non-option in the STOP matrix.
S6. **Diagnostics-empty bootstrap gate (ledger row 82) still open at
    dispatch time.** Hard STOP, matching A5. Gap-4 cannot land green
    until Substrate closes ledger row 82
    (`docs/debt/r3-debt-paydown-ledger-2026-05-02.md:81-82`). No
    implicit acceptance path: any deviation requires a named,
    bounded `INVARIANTS.md` `## P5` responsible-bridge artifact
    (named trigger, scoped surface, ratchet, closure condition)
    authored by Substrate + Director — not a §4 hand-wave. Without
    that explicit P5 bridge, S6 STOP stands and dispatch waits.
S7. **Phase 2 emit retirement creep.** Either worker is asked to delete
    `dsl/extdeps/.../emit.dag` legacy authorities or migrate
    `src/v2/05_emit*.dag` leaves. That work is later PRs per the
    audit's PR-shape table; STOP and re-scope.

## 8. Per-PR debt receipt (mandatory in every PR description)

Each PR (the §4 decision-routing PR, the Gap-4 PR, the Gap-5 PR, and
any later authority-deletion PR on this row) must include:

- Reference: `ROADMAP.md:512` and
  `docs/debt/r3-debt-paydown-ledger-2026-05-02.md:85`.
- New ratchet state: count of remaining old-authority readers in
  `LEGACY_METHOD_TEMPLATE_AUTHORITIES`, count of remaining
  `Map<String, String>` template fields on `LanguageSpec`, and any
  delta to the source-level audit allow-list.
- Explicit statement of which audit STOP condition (S1–S7) was
  *not* triggered, with one-line justification per non-trivial choice.
- Confirmation that no new row was added to
  `src/v3/std/{rust,python,go}_method_template_contracts.dag` as part
  of the consumer-migration PR (row population is a separate Substrate
  PR class).

## 9. Sequencing summary

```
[Substrate/Director §4 decision-routing PR]   ← LANDED
        │  (docs/decisions/r3-row85-method-template-read-surface.md)
        ▼
[PB-Zero Gap-4 build-step producer]   ← LANDED scope-clamped (PR #1598)
        │   Map<String, String> Single-row adapter only;
        │   A1+A2+A3+A7 satisfied; A4 partial (parked on 5b);
        │   producer = src/v3/compiler/src/pb_method_template_projection_dag_emit.rs;
        │   v2 ratchet = src/v2/tests/src/pb_method_template_projection_consumability.rs.
        ▼
[~~Gap 5a — Grounding leaf re-export migration~~]   ← SUPERSEDED 2026-05-04
        │   Re-classified in §6.2 as structurally degenerate / no-op
        │   post-#1598: src/v2/05_emit*.dag already migrated; legacy
        │   *_method_templates + LanguageSpec.method_templates field
        │   are dead carriers held alive by source_audit.rs ratchet
        │   only. No PB-implementable overlay-merge shape exists
        │   inside this packet's STOP boundary.
        ▼
[Batch A — Grounding dead-authority deletion]   ← OUT OF PB SCOPE
        │   Owner: R3 Grounding (per ROADMAP.md:512 / ledger row 85)
        │   + R3 Substrate sign-off on `fold` row disposition.
        │   Gated on Substrate decision: fold standalone contract into
        │   per-target list (with shape extension), or accept `fold`
        │   as target-only legacy text without v3 row.
        │   Deliverable (DEAD authorities only): delete
        │   LanguageSpec.method_templates field + four assignments in
        │   src/v2/languages.dag (also Gap 5b territory) + delete
        │   dsl/extdeps/languages/*/emit.dag::*_method_templates
        │   (rust_method_templates(), python_method_templates,
        │   go_method_templates) + shrink corresponding
        │   LEGACY_METHOD_TEMPLATE_AUTHORITIES entries.
        │   NOT in this batch: rust_method_wraps_result() and
        │   rust_simple_method_specs — both have a live consumer at
        │   src/v2/05_emit_rust.dag:2843; see Batch B.
        ▼
[Substrate/Director typed-read carrier decision]   ← P3b GATE (parked)
        │   Approve typed MethodTemplateContract read shape covering
        │   higher-order rows + non-Single arms + five-field preservation
        │   (notably `wraps_result`, which the current Map adapter
        │   does not carry).
        ▼
[Gap 5b — typed LanguageSpec rewrite PR]   ← acceptance B1–B7
        │   May absorb Batch A above if Grounding/Substrate sequence
        │   them together.
        ▼
[Batch B — Grounding wraps_result consumer migration + deletion]
        │   Owner: R3 Grounding. Gated on P3b above.
        │   Deliverable: migrate src/v2/05_emit_rust.dag:2843 from
        │   rust_method_wraps_result() to a typed
        │   MethodTemplateContract.wraps_result read; then delete
        │   rust_method_wraps_result() + rust_simple_method_specs +
        │   shrink remaining LEGACY_METHOD_TEMPLATE_AUTHORITIES entries.
        ▼
[Grounding leaf-emit migration PRs in src/v2/05_emit*.dag]   ← LARGELY DONE (#1598)
        │   Direct generated-map consumption already on main; this
        │   slot remains for any non-method-template leaf migrations
        │   if discovered.
        ▼
[Authority-deletion PR — dsl/extdeps/.../emit.dag]   (folded into
        the Grounding/Substrate node above on this row; kept in the
        diagram as a separate slot for any non-method-template legacy
        authorities, none currently identified)
```

## 10. References

- `docs/decisions/r3-row85-method-template-read-surface.md` (§4 decision artifact, landed)
- `docs/briefs/collectionops-algebra-reframe.md` (`fold_method` standalone-contract rationale; relevant to §6.2 residue correction)
- `src/v3/compiler/src/pb_method_template_projection_dag_emit.rs` (Gap-4 producer, PR #1598)
- `src/v2/tests/src/pb_method_template_projection_consumability.rs` (v2 consumer ratchet)
- `docs/briefs/method-template-consumer-migration-audit.md`
- `docs/audit/pb-zero-v2-method-template-row-authority-consumer-gap.md`
- `docs/audit/pb-zero-v2-canonical-read-surface-options-stop-matrix.md`
- `docs/design-pure-bootstrap-zero.md`
- `docs/briefs/r2-pure-bootstrap-manager.md`
- `docs/briefs/r2-substrate-manager.md`
- `docs/r2-structure.md`
- `INVARIANTS.md` `## P1`, `## P2`
- `ROADMAP.md:512`
- `docs/debt/r3-debt-paydown-ledger-2026-05-02.md:85`
