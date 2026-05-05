# R3 Debt Sweep — 2026-05-06

**Status:** **Framework draft** — Phase 1 of 3-phase sweep per Director dispatch at [gunbc#828 #issuecomment-4383739792](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4383739792). PM-authored §1 schema + §3 eligibility rubric + §4 anticipation discipline; awaiting Director ratification before Phase 2 (parallel Mgr canvas dispatch).

**Authority:** Brian-ratified scope at gunbc#828 (Director-relayed):
1. **v2 retirement folded into R3** (was separate Pure-Bootstrap-Zero / T-PB-A + T-PB-B program)
2. **Staffing not bottleneck**; comprehensive sweep (no partial); audit covers full v3 codebase + R2/R3-era PR history + ROADMAP + recent analyses

**R3 closure criteria (Brian-ratified)**: all 5 substrate-gap classes closed + v2 fully retired + Pattern A NYI predicates executable + BridgeLedgerZero ratcheting at zero.

**Methodology:** PM-coordinated, Mgr-parallelized canvas + compilation. Per `feedback_corrections_must_grep_verify_source`: all claims grep-verified; per `feedback_section_anchors_over_line_numbers`: cross-references use section/symbol anchors; per `feedback_modeling_inversion_and_paydown_flow`: RED items get inversion-test before flagging out-of-scope.

---

## §1 Comprehensive bridge inventory

Per Director's bridge-class framework (A-G). Populated post-Mgr-canvas (Phase 2-3).

### Schema

Each bridge has one row in the inventory:

| Bridge | Class | File / location | Dissolution trigger | R3-close eligibility | Source PR | Owner Mgr |
|---|---|---|---|---|---|---|
| `<bridge name or symbol>` | A-G | `<file path or symbol anchor>` | `<named trigger that fires at R3 close>` | GREEN / YELLOW / RED | `#NNNN` | Substrate / Verification / Evaluator / PB / Debt-Paydown / Grounding |

Column rules (load-bearing):

- **Bridge**: name or canonical symbol; not a free-text description. Examples: `CardinalityPayload::new_unchecked`, `SourceSpan.file participation checks`, `EmissionDiagnostic Rust mirror`.
- **Class**: single letter A-G per Director's framework (below). No "multi-class" entries — pick the load-bearing class; cross-reference others in the trigger column if structurally coupled.
- **File / location**: section/symbol anchor preferred; line numbers permitted only when no symbol name exists at the cited point.
- **Dissolution trigger**: named structural event that ends the bridge's existence (e.g., "T-V2-Retirement G-2 deletion landing", "R3 Substrate substrate-gap-class-3 closing", "PR #XXXX merging"). Not heuristic; not "eventually."
- **R3-close eligibility**: GREEN / YELLOW / RED per §3 rubric below.
- **Source PR**: PR that introduced the bridge, if traceable; "pre-R2" if older than R2 lane structure.
- **Owner Mgr**: which R3 Mgr's lane the bridge sits in (or coordinates-to). Single owner; cross-program coupling noted in trigger column.

### Class A — Substrate-gap-blocked (~15-20 expected)

Bridges that exist because a substrate carrier or grammar surface isn't yet modeled. Wait for parser/grammar surface, function-valued data, file-ingestion, workflow/scheduling, reflection-closure.

[Mgr canvas populates rows]

### Class B — Pattern-A NYI predicates (~7 expected)

Test predicates declared but `NotYetImplemented`-shaped runtime: TC1, TC2, TC3, free-consequences, RustDagIsomorphism, BridgeLedgerZero, SymbolicCostExprEquals.

[Mgr canvas populates rows; some entries pre-named per Director framework]

### Class C — Pattern-C typed-carrier + Rust-mirror (~6+ expected)

Rust mirrors of `.dag` typed carriers that should dissolve when v3 evaluator authoritatively executes the .dag side: EmissionDiagnostic, Value, EvalStrategy, BridgeLedgerRef, target-primitive routing.

[Mgr canvas populates rows]

### Class D — Generated bridges with freshness gates (~3-5 expected)

Generated artifacts that would be debt without freshness ratchets but are bounded scaffolds with explicit regeneration gates: `render_repeat_string_bootstrap.dag`, `generated_method_template_projection.rs`. **Not debt by intent**; freshness gate prevents drift. May still be GREEN at R3 close if R3-close includes v2 retirement (which it now does per Brian-ratified fold-in).

[Mgr canvas populates rows]

### Class E — v2 ↔ v3 transition bridges (~10+ expected; **now in R3 scope per fold-in**)

Bridges that exist solely because v3 isn't yet authoritative for some surface that v2 still owns: `emit_model.dag` facade, method-template legacy adapters, v2 transport infrastructure. Per Brian's R3-fold-in ratification, these are R3-close eligible (GREEN if v2 retirement closes; previously would have been "post-R3 / Pure Bootstrap").

[Mgr canvas populates rows]

### Class F — Operator/algebra ontology duplication (~3 surfaces expected)

Duplicate representations of operator/algebra concepts: OperatorSpec / OperatorKind / BinaryOpRow. Should consolidate to single ontology.

[Mgr canvas populates rows]

### Class G — Local/small bridges (~5-10 expected)

Smaller scope bridges: F10 install_hint, ad-hoc fold sentinels, etc.

[Mgr canvas populates rows]

---

## §2 Hidden debt audit

Items NOT currently in ROADMAP that should be tracked. Sources:
- SG-0 census per-entry justification (live count via `EXPECTED_HAND_AUTHORED_NON_TEST` + `EXPECTED_HAND_AUTHORED_TEST` + `EXPECTED_HAND_AUTHORED_FRAGMENTS` arrays in `src/v3/compiler/tests/integration/sg0_census_test.rs`; per-entry: is it bridge or pre-existing STRUCTURAL apparatus? if bridge, dissolution trigger?)
- Worker-inbox findings not yet escalated (Mgr canvas surfaces from per-Mgr inbox histories)
- Recent analyses items not yet rowed (cross-reference gpt-5-5-pro Exploratory main@41019a8 + Reflective main@2b7b21b + earlier paired analyses 2026-04-25 / 2026-04-30 / 2026-05-01)

[Populated post-Mgr-canvas + PM compile pass]

---

## §3 R3 closure-eligibility rubric

Three valid statuses per item. Each item in §1 has a defined status; the count of RED is the load-bearing signal for Director review.

### GREEN — dissolves at R3 close per [trigger]; in scope

**Criteria:**
- Item has a **named structural dissolution trigger** that fires at R3 close (e.g., specific lane landing, substrate carrier landing, PR merging)
- No upstream prerequisite that's RED
- Trigger is **structurally** verifiable, not heuristic ("when X lands" verifiable; "eventually" not)
- Source PR known + owner Mgr identified

**Example shape** (post-population): `CardinalityPayload::new_unchecked` → trigger: "elimination per Path B (idempotence-inline rename); landing at R3 Substrate Mgr dispatch under SG-0-delta discipline"

### YELLOW — dissolves at R3 close IF [structural prerequisite] lands; tracked

**Criteria:**
- Item dissolves IF a **named structural prerequisite** lands in R3
- Prerequisite is itself in R3 scope (GREEN or YELLOW with a clear chain back to GREEN)
- Tracked with the prerequisite chain explicit; no "we'll figure it out" prerequisites
- Source PR + owner Mgr identified

**Example shape** (post-population): `EmissionDiagnostic` Rust mirror → trigger: "v3 evaluator authoritatively executes std diagnostic constructors"; prerequisite: "T-V2-Retirement G-2 deletion landing (which gates on T-FixedPoint + T-LensProducer-Retirement)"

### RED — no clear path / explicitly post-R3 / unknown; flagged for Director review

**Criteria** (any of):
- No clear dissolution path identified
- Explicitly post-R3 / out-of-current-scope by design
- Unknown / unclassified — flagged for Director investigation

**RED count is the load-bearing signal.** Each RED needs Director-level investigation per Brian's framing: "are these honestly out-of-scope, or are they 'we don't know how to close this'?"

### Inversion-test discipline (per `feedback_modeling_inversion_and_paydown_flow`)

Before flagging an item RED, apply inversion test:
- Forward framing: "what dissolves this bridge?" → if no answer, RED candidate
- Inverse framing: "what would let this bridge persist?" → if the inverse forces a structural fix, the bridge is actually GREEN/YELLOW with a reframed dissolution shape

If inversion dissolves the case → status updates to GREEN/YELLOW. If inversion doesn't dissolve it → RED is structurally honest.

---

## §4 Anticipated debt — preventive lane

Going-forward discipline encoded as PR-authoring contract. Folds in Director's prior PR-template SG-0-delta extension routing to quiet-otter (R3 Debt-Paydown, gunbc#1744 #issuecomment-4383628247).

### PR-authoring contract

**Every PR adding hand-Rust must same-PR answer (in PR description or commit message):**

(a) **Bridge class** — A-G per §1 framework above. SG-0 zero-floor discipline: hand-authored v3 Rust must dissolve same-PR, cite a named structural trigger (with R3-close eligibility GREEN/YELLOW per §3), or carry explicit Director RED allocation. There is no permitted "intentional permanent" exception in the going-forward contract — pre-existing STRUCTURAL infrastructure (the SG-0 measurement apparatus itself; see §3.A grandfathering pending Director ratification of the new STRUCTURAL status) is the only category exempt, and new STRUCTURAL additions require explicit Director allocation citing the SG-0 program shape.

(b) **Substrate-gap-class blocking dissolution** (if any) — name the gap that prevents same-PR dissolution. If no gap, the contract's options collapse to "dissolve same-PR" or "explicit Director RED allocation"; "intentional permanent" without Director allocation is not allowed.

(c) **Same-PR dissolution OR named-trigger-with-R3-close-eligibility** — if dissolution doesn't land same-PR, name the trigger + R3-close eligibility (GREEN/YELLOW/RED per §3 rubric). RED requires Director-level allocation.

(d) **Net SG-0 delta** — positive only with explicit Director allocation citing gap-closure. Per Brian-ratified discipline: SG-0 ratchet is only-down except for Director-allocated additions tied to specific gap-closure.

### CI gate

[Routed to R3 Debt-Paydown (quiet-otter-416, gunbc#1744) per Director #issuecomment-4383628247 — the lightweight CI check that PR description carries (a)-(d) for PRs adding hand-Rust]

### PR-template extension

[Specifies (a)-(d) as required-fields for PRs adding hand-Rust; quiet-otter authors the template change]

---

## §5 Sources audited

- **v3 compiler**: ~69 `.rs` files in `src/v3/compiler/src/`
- **v3 std**: ~35 `.dag` files in `src/v3/std/`
- **SG-0 census**: live entries via `EXPECTED_HAND_AUTHORED_NON_TEST` + `EXPECTED_HAND_AUTHORED_TEST` + `EXPECTED_HAND_AUTHORED_FRAGMENTS` arrays in `src/v3/compiler/tests/integration/sg0_census_test.rs` (the SG-0 partition authority; do not snapshot the count here — derive at audit time from the live arrays per `feedback_corrections_must_grep_verify_source`)
- **R2/R3-era PR history**: ~500+ PRs (R2 close at #1275 → current HEAD ~#1803). Method: per-Mgr-lane parallel canvas (each Mgr surveys merged PRs in their lane subset for debt-introduction patterns + dissolution status). Pattern-audit, not per-PR-deep-audit.
- **ROADMAP**: open tracked items + DB-1 through DB-20 + W-C1 + scheduled deletions + Tracked debts 2026-04 + 2026-05 sub-sections + Course Corrections #1-#4 (ROADMAP §"Reflective course corrections" section anchor; per `feedback_section_anchors_over_line_numbers` — do not cite line numbers since ROADMAP edits drift them)
- **Recent analyses**: gpt-5-5-pro Exploratory main@41019a8 (F1-F11) + Reflective main@2b7b21b (Risks 1-5) + earlier paired analyses 2026-04-25 / 2026-04-30 / 2026-05-01

---

## Methodology notes (per session-discipline patterns)

- All claims grep-verified before encoding (per `feedback_corrections_must_grep_verify_source`)
- Cross-references use section/symbol anchors not line numbers (per `feedback_section_anchors_over_line_numbers`)
- Mgr canvases request grep-verified sourcing per item
- RED items get inversion-test before flagging out-of-scope (per `feedback_modeling_inversion_and_paydown_flow`)
- 8 cross-relay timing instances this session — apply sha-style timestamp pointers in routing claims

---

## Phase plan

**Phase 1 (current — framework draft)**: PM authors §1 schema + §3 rubric + §4 anticipation discipline. Surface to Director for ratification.

**Phase 2 (post-ratification)**: parallel Mgr canvas dispatch to all 6 R3 Mgr inboxes simultaneously; identical message format. Each Mgr surfaces lane debt with grep-verified sourcing.

**Phase 3 (post-canvas)**: PM compiles Mgr responses + adds §2 hidden-debt audit + cross-references ROADMAP review + recent analyses. Surfaces compiled sweep for Director final ratification + R3 closure-criteria explicit list.

[Phase 1 ratification gates Phase 2; Phase 2 completion gates Phase 3]
