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
authorities. Row-parity gaps (`string_contains` Python/Go, Go `chars`)
remain Substrate-owned and out of scope.

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

**Gap 5a — Single-row leaf-emit migration (enabled by #1598; R3
Grounding owns dispatch, not this packet).** The legacy `dsl/extdeps/
languages/{rust,python,go}/emit.dag::*_method_templates` map declarations
re-source their entries from the generated
`generated.method_template_projection.<target>_method_template_emit`
maps **only for the keys actually present in the generated map**. Keys
that exist in the legacy map but are not yet projected — verified at
authoring time per the consumer-migration audit (`docs/briefs/method-template-consumer-migration-audit.md`)
and the v3-row-file headers — must remain carried in the legacy
declaration until substrate parity lands. Dropping any legacy key
during 5a is a P2 boundary violation (facts must flow forward, not
silently disappear). Concretely the keys still legacy-only at
authoring time include Python `string_contains`
(`src/v3/std/python_method_template_contracts.dag:12-18`), Go
`string_contains` and Go `chars`
(`src/v3/std/go_method_template_contracts.dag:12-29`); a Gap-5a worker
must verify that list at HEAD before flipping any key.

The 5a target shape is therefore **overlay-merge**, not wholesale
replacement: legacy declaration becomes "generated rows ∪ legacy-only
residue," with a structural ratchet that fails the build if any
legacy-only key disappears without an explicit Substrate row landing.
The existing `LanguageSpec.method_templates: Map<String, String>?`
field shape and assignments stay untouched; no `src/v2/languages.dag`
edit; no second authority introduced (the generated map is the only
source of text for the keys it owns; the residue is text that already
lives in the legacy `.dag` until Substrate adopts it). Legacy
`rust_simple_method_specs` and `rust_method_wraps_result` and the
higher-order rows stay on the existing path until 5b.

5a closure (full deletion of legacy `*_method_templates`) is gated on
Substrate landing the missing rows (`string_contains` Python/Go, Go
`chars`) **and** Gap 5b approval, not on 5a alone. This packet does
**not** dispatch 5a — it is R3-Grounding-owned per ledger row 85; this
section only records that #1598 unblocks the bounded overlay form.

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
deleted.** `string_contains` (Python/Go) and Go `chars` remain
substrate-owned; Gap 5b does not delete legacy `*_method_templates`
maps yet — that is a later authority-deletion PR per audit §"Closing
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
S3. **Row parity gap blocks acceptance.** A row authority is missing a
    `MethodRef` (e.g., Python/Go `string_contains`) or a row
    (`go chars`) needed to satisfy A4 / B-criteria. Route to Substrate;
    do not paper over with placeholder rows in v2.
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
[Gap 5a — Grounding leaf re-export migration]   (R3 Grounding-owned;
        │   not dispatched by this packet)
        │   Flip dsl/extdeps/languages/{rust,python,go}/emit.dag::*_method_templates
        │   to re-export generated.method_template_projection.<target>_method_template_emit;
        │   no src/v2/languages.dag edit; LanguageSpec field shape unchanged.
        ▼
[Substrate/Director typed-read carrier decision]   ← P3b GATE (parked)
        │   Approve typed MethodTemplateContract read shape covering
        │   higher-order rows + non-Single arms + five-field preservation.
        ▼
[Gap 5b — typed LanguageSpec rewrite PR]   ← acceptance B1–B7
        │
        ▼
[Grounding leaf-emit migration PRs in src/v2/05_emit*.dag]   (out of scope)
        │
        ▼
[Authority-deletion PR — dsl/extdeps/.../emit.dag]   (out of scope)
```

## 10. References

- `docs/decisions/r3-row85-method-template-read-surface.md` (§4 decision artifact, landed)
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
