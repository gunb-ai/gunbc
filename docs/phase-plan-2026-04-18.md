> Parent: [post-l15-phase-plan.md](./post-l15-phase-plan.md) | [ROADMAP.md](../ROADMAP.md) (root, authoritative) | [Session relay](./session-relay-queue.md) (pointer to PR #530 / GitHub — not a mutable ledger)

# Phase plan — Post-merge-wave coordination (snapshot 2026-04-18)

**Author:** director chat session `clever-lark-108`
**As of:** 2026-04-18
**Refresh cadence:** session start + after every merge wave (never more than ~1 day stale)
**Since last refresh:** §4.1 trimmed to **pointer index only** (single-authority hygiene per ChatGPT review 2026-04-18); Lane 1e + determinism mirror added to [post-l15-phase-plan.md](./post-l15-phase-plan.md). **Relay:** PR [#530](https://github.com/gunb-ai/gunbc/pull/530) — [session-relay-queue.md](./session-relay-queue.md) pointer-only; ChatGPT `sha:10bd34e…` **APPROVE_WITH_COMMENTS** @ 2026-04-18T16:10:15Z; **pending** on tips `sha:210381a…` (meta-review) and `sha:4014aa…` (full review) — see relay.

---

## Snapshot discipline

**This doc is a thin read-model over ROADMAP and the design docs ROADMAP references** (DB-N design docs, lane docs, `post-l15-phase-plan.md`). It originates coordination state (who's working what, dispatch order, next-batch briefs) but restates facts from those authorities only where essential. When an authority doc and this doc conflict, the authority wins and this doc must be fixed — delete the restated fact, replace with a line reference.

Authority order on conflict:
1. A specific DB design doc (`docs/design-*.md`) for anything it locks — shape, acceptance gate, rejected alternative.
2. ROADMAP (root `ROADMAP.md` for cross-cutting tracks; `src/v3/ROADMAP.md` for lane/milestone active-deferrals).
3. Lane / phase docs (`post-l15-phase-plan.md`, `lane3-self-hosting-cycle.md`, etc.) for sequencing and dependency statements.

Do NOT argue this doc into agreement with an outdated fact; that reintroduces the parallel-tracker debt this doc exists to reduce.

When reading:
1. Cross-check §1 merges against `git log --oneline --since=2026-04-17` before dispatching from §2/§3.
2. For any DB reference, confirm the DB design doc exists at the cited path.
3. For any restated authority fact (primitive lists, tracked debt, DB numbering), verify the line reference resolves — treat mismatch as a bug in this doc.

---

## 1. Where we are

**Session window:** 2026-04-17 → 2026-04-18. 10 merges this session (9 PRs + 1 post-merge-wave hotfix), recovering from a rough early-day state into coordinated multi-chat fan-out.

### Merges (chronological)

| PR | Stage | Outcome |
|---|---|---|
| #514 | Lane 1 Stage 1c PR 1 — Rust pilot | Rust E-5 clean-emission contract shipped |
| #511 | DB-9 R2 docs | Mutual recursion lowering design landed |
| #515 | DB-11 — refinement consumer wiring + composite-canonical conjunction | 3a.3 partial closure (see ROADMAP Lane 3 Stage 3a.3 row) |
| #516 | Track 9 substrate primitives graduation | Canonical list in [ROADMAP Track 9](../ROADMAP.md) lines 765-768: `NonEmptyList<T>`, `NonSingletonList<T>`, `ParamRef`, `TransformRef`. `ArityIndex` was explicitly *rejected* as a standalone primitive (ROADMAP:772). |
| #517 | Lane 2 Stage 2a — effects port (decompressed substrate) | `EffectShape`, `KeySource`, `IdempotencyEvidence`, etc. |
| #518 | W3 — `lens_structural_resolution` + emit fix + `NoBody` substrate | E-5 reconciliation; `ArrowBody::NoBody` for type-alias arrows |
| #521 | Lane 2 Stage 2a-followup — collapse `DerivedOpEffect` | Single-authority restored; Stage 2b unblocked except `ComposedEffect` |
| #519 | Lane 3 Stage 3a.1 — mutual recursion impl | Track 9 primitives consumed; substrate enforces ≥2 cluster members structurally |
| #520 | Lane 1 Stage 1c PR 2 — Go pilot | Second-target proof; contract held under Go's stricter unused-rule |
| #523 | post-merge-wave hotfix — `emit_go.rs` `bound.count` → `bound.count_port()` | Mechanical 3-line fix; merged 2026-04-18 as `977224ac7` |

### Structural rule banked

`feedback_substrate_principle_audit` (memo, 2026-04-18) — 6-question audit before any new substrate field/variant; structural recovery patterns named (Track 9 vocabulary). Currently cited from dispatch briefs (see §2, §3). Graduation to INVARIANTS.md is a separate follow-up, not tracked from this snapshot — counting citations inside a read-model doc would itself make the doc act as a second authority for a rule not yet in a canonical authority file.

### Coordination meta-observation

The cross-PR "lossy compression at substrate" class that surfaced in #511, #515, #517, #518 was dissolved structurally rather than patched per-instance — Track 9 graduation + composite-canonical refinements + decompressed effects + NoBody distinction + the audit memo. The pattern that bit the early session has been retired.

Multi-chat fan-out (4 children + this director chat) ran 4 parallel dispatches with structural pre-clearance from director chat. Validated as the operating model — though throughput constraint identified (see §2 footer).

---

## 2. In flight (currently dispatched)

| Chat | Lane | PR | Status | Unblocks |
|---|---|---|---|---|
| **D** (loyal-otter-908) | DB-16 Part 1 — refined-generic substitution design | #522 | Iterating on R3+ reviews; design-only | DB-16 Part 2 implementation |
| **A'** (sleek-crab-150, cursor) | DB-16 Part 2 — implementation | TBD | Working against R3 design as contract | ROADMAP Lane 3 Stage 3a.3 → ✅ Shipped (closes refined-generic-parameters remaining-item) |
| **B'** (warm-newt-750, claude) | ComposedEffect reshape | TBD | Working | Last Stage 2a debt → Stage 2b consumer dispatch unblocked |
| **C'** (loyal-koi-680, claude) | PR 2.5 — dissolve `PatternBindingRule` name-keyed recovery | TBD | Working | Stage 1c PR 3 (Python pilot) cleanly |

*(Hotfix #523 was in-flight at time of draft; already merged as `977224ac7` during this director-chat session. Removed from the table.)*

### Acceptance for in-flight items

- **#522 (DB-16 Part 1):** R3 design contract locked at `40f95806c`; non-blocking nits ignored per meta-review prescription.
- **DB-16 Part 2 (A'):** all `test_3a4_*` cases pass; `is_retryable_generic_decl` retry-then-succeed path locked; ROADMAP Lane 3 Stage 3a.3 row's "Remaining — Refined generic parameters" item clears.
- **ComposedEffect reshape (B'):** `effects.dag` shape change + ROADMAP Lane 2 Stage 2a/Track 17a boundary entry updated from Deferral → Cleared; principle-audit Q3 cited.
- **PR 2.5 (C'):** zero `named_variant_id(_, "PatternBindingRule", _)` remain in any emitter; typed `PatternBindingRuleVariants` cache hoisted to `Dag`; principle-audit Q5 cited. (Clears ROADMAP Lane 1 Stage 1c "Deferral: 1c PR 2.5" entry.)

### Throughput reality check

Pre-clearance in director chat takes real director-time. With 4 child chats active, director throughput gates the whole operation. This doc's §6 commits explicit time-budgets for the next director session so pre-clearance keeps pace with dispatch. If it doesn't, children start dispatching without pre-clearance (bad) or sitting idle (also bad).

---

## 3. Next-batch dispatches (paste-ready briefs)

When current batch lands, dispatch in this order (heavy + light batching mirrors tonight's shape):

### Brief — Stage 1c PR 3: Python pilot

**Status:** gated on PR 2.5 landing.

**Scope:** Python pilot for the clean-emission contract. Python is the structurally interesting target — it doesn't emit the binding as part of the pattern at all (per `phase1-lane2-clean-emission-invariant.md` §Scope discussion; see ROADMAP Lane 1 Stage 1c "Deferral: 1c Python pilot" for full rationale). Adds `python_clean_emission` to `CleanEmissionContract`; wires `emit_python.rs` to consume via the typed cache PR 2.5 installs; port-liveness walk handles Python's pattern-bind elision case.

**Read first:**
- `src/v3/std/clean_emission.dag` (the contract)
- `src/v3/spec/rust.dag`, `spec/go.dag` (now-existing instantiations from PRs 1–2)
- `src/v3/compiler/src/emit_python.rs` (target file)
- `docs/phase1-lane2-clean-emission-invariant.md` §Scope
- `feedback_substrate_principle_audit` — this brief cites Q5 (construction authority)

**Acceptance:**
- `spec/python.dag` declares `python_clean_emission` mirroring the contract shape, consumed via the typed `PatternBindingRuleVariants` cache (NOT via `named_variant_id` — Q5 compliance is load-bearing)
- `emit_python.rs` consumes via cached accessor pattern
- Pilot Python emission passes `python3 -m py_compile` (and `ruff` if wired)
- ROADMAP "Deferral: 1c Python pilot" row cleared

**STOP-AND-ESCALATE rule (load-bearing):**

If Python forces a `CleanEmissionContract` SHAPE change (not just a new instantiation), **HALT the dispatch and report to director chat.** Do not extend the contract in-flight. The whole point of the pilot sequence is to prove the contract generalizes; a shape change at Python is a structural finding that means the contract over-fit Rust+Go. The director chat decides whether to reshape upstream or keep Python on a separate surface rule. Silently patching forward destroys the signal.

**Size:** M.

---

### Brief — Lens-name-filter dissolution (#518 follow-up)

**Status:** dispatchable now (independent of in-flight items).

**Scope:** broader migration of `ArrowBody::Pending` writes from anonymous sites (`lower.rs:872` nested Arrow, `infer.rs:1884` variant constructor, `infer.rs:2807` operator fallback) to `NoBody` where appropriate. Once migrated, `lens_structural_resolution` predicate drops the `name: Some(_)` filter and becomes purely structural.

**Read first:**
- `src/v3/lenses/structural_resolution.dag`
- Migration sites: `lower.rs:872`, `infer.rs:1884`, `infer.rs:2807`
- #518 commit `816d536` for the NoBody pattern

**Acceptance:**
- Sites 2 and 4 (anonymous nested Arrow + variant constructor) migrated to `NoBody`
- Site 5 (operator fallback) — explicit classification: NoBody if "no body by construction"; documented Pending-with-rationale otherwise
- Lens predicate drops `name: Some(_)` filter where safe
- New regression tests: anonymous arrows previously silent-via-name-filter now silent-via-NoBody
- ROADMAP entry added and closed in the same PR

**STOP-AND-ESCALATE rule:** if site 5 forces a NEW substrate variant (neither Pending nor NoBody fits), escalate — don't introduce variants inside an anonymous dispatch.

**Size:** S-M depending on site 5.

---

### Brief — Substrate-asymmetry + planner alignment (combined XS)

**Status:** dispatchable now.

**Scope:** two tiny items folded into one chat:

1. **ParamRef/TransformRef asymmetry — substrate comment only.** ROADMAP already tracks this debt at [Track 9 lines 777-786](../ROADMAP.md) ("Tracked debt — substrate constructor-validation asymmetry"), including the Lane 3c graduation trigger. **Do NOT add a new ROADMAP entry — that would duplicate an existing one.** The remaining work is a cross-reference comment at `src/v3/std/substrate.dag` near `type ParamRef` / `type TransformRef` pointing readers to the ROADMAP entry so a reader of the substrate finds the asymmetry documentation.

2. **Mutual-recursion planner vs `is_first` authority alignment.** Planner walks raw `module.items`; lowering applies the `is_first` duplicate filter. Filter `compute_mutually_recursive` to the same first-authority set lowering uses. Add regression test: duplicate fn declarations where the first is part of an SCC and the second isn't — planner sees only the first. (Not currently in ROADMAP; add a Lane 3 Stage 3a.1 follow-up row in the same PR.)

**Acceptance:**
- Substrate comment references ROADMAP Track 9 tracked-debt entry (no new ROADMAP entry created).
- Planner filter applied + regression test passes; Lane 3 Stage 3a.1 follow-up ROADMAP row added.

**Size:** XS combined. Fold both into one dispatch.

---

## 4. Medium-horizon design pre-clearance

Items flagged for director-chat pre-clearance before dispatch. **Audit finding from this review:** several items the draft called "un-pre-cleared" are already pre-cleared by existing DB docs. Remaining work is reduced accordingly.

### Stage 2c — Test obligation materialization

**Status:** **pre-cleared by DB-15** (`design-test-infra.md`, R2 draft).

ROADMAP Lane 2 Stage 2c names it: *"DB-15 tests-as-declarations extensions (M, blocks Lane 2 Stage 2c). R2 consumes the compiler-as-dependency-analyzer thesis: tests are declarations (extending `src/v3/std/verification.dag` `TestClaim`/`TestSuite` authority), resources are references to `dsl/std/resources.dag`."*

**Remaining director-chat work (not new design):**
- Read DB-15 R2 and confirm it locks the open questions the draft raised: obligation shape (✅ extends `TestClaim`), target test framework interaction (✅ via declared resources), composition rules (N obligations vs workflow obligation vs both — confirm R2 answers or escalate to R3).
- Confirm the prerequisite `dsl/std/resources.dag` → v3 reconciliation (S) deferral plan in ROADMAP.

**Output:** confirm against the DB doc; add a **pointer row** in §4.1 only — do not restate gates here.

### Stage 2d — Symbolic cost

**Status:** **pre-cleared by DB-7** (`design-symbolic-cost-algebra.md`).

DB-7 locks: `SymbolicCost` carrier (7 variants: Constant/Linear/Polynomial/Product/Sum/Log/Unknown), dominance + normalization, sequential/iterate/max_path composition, per-Behavior lowering (including Loop with `recursion_depth_bound` that walks mutual-recursion cluster descent), nested-fold O(n²) diagnostic, dead-work detection, `Dimension<SymbolicCost>` wiring, display format, acceptance gates.

**Remaining director-chat work:**
- Confirm DB-7's 4 open questions map onto the draft's open questions (they do; DB-7 Q4 on Branch with different asymptotic costs is answered, DB-7 Q3 on fail-compile bounds is a legitimate downstream extension).
- Confirm the Stage 2b → 2d handoff: DB-7 composes through `Loop`'s `LoopBound::Descent` for mutual-recursion clusters (post-#519 substrate) — no gap.
- Decide whether `WorkflowEffect` integration (Stage 2b) needs any DB-7 extension. If yes, write a specific ask; if no, Stage 2d dispatches directly against DB-7.

**Output:** confirm against the DB doc; add a **pointer row** in §4.1 only — do not restate gates here.

### Stage 3b — Diagnostics-as-corrections

**Status:** **pre-cleared by DB-1** (`design-correction-shape.md`) + lane3-self-hosting-cycle.md §Stage 3b.

Per lane3 doc: *"Type shapes are locked in DB-1. Lane 3 does not restate them here — see DB-1 for the `Correction` record, the `Diagnostic.fixes` field (plural `fixes`, a `List`), `CorrectionStyle` per-target style."*

**Remaining director-chat work:** none. Stage 3b is execution-blocked (gates on Lane 1c close — CleanEmissionContract surface is the target-spec mechanism), not design-blocked. Dispatch when Lane 1c PR 3 lands.

### Stage 3c — Self-hosting cycle

**Status:** **pre-cleared by DB-8** (`design-fixed-point-ratchet.md`) + lane3-self-hosting-cycle.md §Stage 3c.

DB-8 locks: the cycle (4 mandatory steps + 1 optional), CI gate binary implementation, CI job YAML, 8 sources of non-determinism with fixes, INVARIANT D-1 (determinism), per-fixture 5x determinism test, rationale, rejected alternatives, workspace layout, debug output format, performance targets.

**Remaining director-chat work for 3c is NOT rediscovering DB-8.** It's the substrate-readiness checklist + the genuinely-open downstream questions about what compiler.dag carries today vs what 3c needs. See §6.

### DB numbering discipline

The draft reserved DB-18/19/20/21/22 speculatively. This review dropped those references because:
- DB-15 (test-infra) already covers Stage 2c → "DB-18 candidate" is redundant.
- DB-7 (symbolic cost) already covers Stage 2d → "DB-19 candidate" is redundant.
- DB-8 (fixed-point) already covers Stage 3c mechanics → "DB-20 candidate" is redundant.
- **DB-17 is already allocated to reference-resolution provenance** — allocating authority is [`design-reference-resolution-provenance.md`](./design-reference-resolution-provenance.md); also referenced from `src/v3/ROADMAP.md` §Scheduled deletions (the "Needs DB-17" enforcement marker). Using DB-17 for WorkflowEffect would collide.

**Rule for this doc:** do not pre-reserve DB numbers. A DB number gets assigned at the moment a DB design doc is opened, not at the moment of speculation.

### 4.1 Director session — audit index (still-deer-308, 2026-04-18)

§4 above asked for director-chat **read-and-confirm** of DB docs, not new design. This subsection is an **index only**: it records *where* the 2026-04-18 session looked. **Gates, acceptance criteria, and pre-clearance verdicts live in the cited files** — not here. (Keeps this doc from becoming a second authority for stage decisions; see ChatGPT review on #530.)

| Topic | Authority to read (single source of truth) |
|---|---|
| DB-15 / Lane 2 Stage 2c | [`design-test-infra.md`](./design-test-infra.md) R2; deferrals [`src/v3/ROADMAP.md`](../src/v3/ROADMAP.md) §Lane 2 Stage 2c |
| DB-7 / Lane 2 Stage 2d | [`design-symbolic-cost-algebra.md`](./design-symbolic-cost-algebra.md) |
| Stage 3b | [`design-correction-shape.md`](./design-correction-shape.md) (DB-1), [`lane3-self-hosting-cycle.md`](./lane3-self-hosting-cycle.md) §Stage 3b; Lane 1c blockers [`src/v3/ROADMAP.md`](../src/v3/ROADMAP.md) §Lane 1 Stage 1c |
| Stage 3c mechanics | [`design-fixed-point-ratchet.md`](./design-fixed-point-ratchet.md) (DB-8) |
| Lane 1 Stage 1e scope, emit determinism, `tests/determinism_test.rs` | [`post-l15-phase-plan.md`](./post-l15-phase-plan.md) Lane 1 summary (incl. DB-8 pointer), [`design-generic-walker-api.md`](./design-generic-walker-api.md) (DB-2) |
| Open program questions (what `compiler.dag` is today) | **§6** below — not a design substitute for DB-8 |

---

## 5. Tracked debt

Split into **already in ROADMAP** (point to authority) and **migration candidates** (new from this session).

### 5a. Already in ROADMAP active-deferrals

| Item | ROADMAP location |
|---|---|
| `ComposedEffect` post-reshape verification | Lane 2 Stage 2a / Track 17a boundary |
| 1c post_emit_verifier CI gate | Lane 1 Stage 1c |
| 1c Python pilot | Lane 1 Stage 1c |
| 1c PR 2.5 dissolve PatternBindingRule | Lane 1 Stage 1c |
| Stage 2b → 2c handoff (via DB-15) | Lane 2 Stage 2c |
| Self-compile perf ratchet investigation | Cross-cutting — performance |
| ParamRef/TransformRef constructor-validation asymmetry | Track 9 lines 777-786 (authoritative; §3 XS brief adds only a substrate.dag cross-reference comment — no new ROADMAP row) |
| Track 9 second consumer (`IndexedElement<T>.index` → `ElementRef<T>`) | Track 9 line 770 — deliberately NOT pre-declared; graduates when a concrete consumer arrives. No migration needed. |

No migration needed for any row above — §3 briefs or ROADMAP's existing stance covers them.

### 5b. Migration candidates (new from this session)

| Debt | Source | Suggested ROADMAP placement |
|---|---|---|
| Mutual-recursion planner vs `is_first` alignment | #519 ChatGPT non-blocking | Lane 3 Stage 3a.1 follow-up — §3 combined XS brief lands this |
| Variant-payload field-access general model | #518 (multi-field) + #519 chat A (single-field) | New "infra debt" subsection OR Lane 1 follow-ups — needs a dedicated hygiene chat |
| Lens-name-filter dissolution (#518 sites 2/4/5) | #518 broader migration | §3 brief handles; ROADMAP row added when that PR opens |
| compiler.dag v2-path carryover (§6 Q3) | `hand_maintained_src` references `src/v2/stage0/src`, `cli_run.rs`, `v2_interpreter.rs` — v2 paths in a doc that should eventually key off v3 source | Lane 3 Stage 3c prerequisite OR downgrade to "open question until 3c starts" — needs director pre-clearance before migration |

**Dispatch decision:** §3 briefs cover 2 rows — the combined XS brief lands the planner alignment; the lens-name-filter brief lands that dissolution. The **variant-payload field-access general model** row needs a dedicated ROADMAP-hygiene chat (not folded into the XS brief — it won't get attention inside an XS dispatch). The **compiler.dag v2-path carryover** row needs director-chat pre-clearance first (is it a real 3c blocker, or a v2→v3 bridging detail that self-resolves?) before ROADMAP migration.

---

## 6. Stage 3c readiness exploration (director-chat output)

This section fills what the draft marked "intentionally sparse." Grounded in reads of DB-8, lane3-self-hosting-cycle.md, `dsl/gunbc/compiler.dag`, and ROADMAP's Post-A/B Lane Plan.

### What compiler.dag currently IS

`dsl/gunbc/compiler.dag` (310 lines) **models the self-hosting cycle, not the compiler internals.**

It declares:
- `SourceRoot`, `GeneratedCrate`, `SelfHostingCycle` — structural model of the source→generated-crate relationship
- `ResolvedCompilerTools` — typed record of absolute tool paths (cargo, grep, diff, cp, rm, find)
- Command-derivation functions: `build_command`, `check_command`, `compile_command`, `clean_stage0_generated_command`, `copy_generated_command`, `diff_exclude_args`, `lint_command`, `test_command`, `ignored_test_command`
- Tool registry with install-source enumeration (brew / apt / rustup / coreutils)

It does NOT contain:
- Parser / lowerer / inference / emission logic (those live in `src/v3/compiler/src/*.rs`)
- Substrate declarations (those live in `src/v3/std/*.dag`)
- The `self_host_fixed_point` binary (DB-8 prescribes `src/v3/compiler/src/bin/self_host_fixed_point.rs`)

**Implication:** compiler.dag today is the meta-model — "what the self-hosting cycle is made of." Stage 3c wires the cycle USING compiler.dag as the structural authority for which files get regenerated, what tools are needed, etc. The compiler's own LOGIC being expressed in .dag is the further horizon (M3 in the original roadmap, now absorbed into Lane 3's self-hosting cycle framing).

### Substrate-readiness checklist for Stage 3c kickoff

After current batch closes + §3 next-batch lands:

| Readiness item | State |
|---|---|
| Clean emission contract (Lane 1 Stage 1c) | 🟡 Rust ✅ + Go ✅; Python gated on PR 2.5 |
| Mutual recursion lowering (Lane 3 Stage 3a.1) | ✅ Shipped (#519) |
| Refined types (Lane 3 Stage 3a.3) | 🟡 DB-16 Part 2 in flight closes it |
| `data` value semantics (3a.2), surface generics (3a.4), Disj dotted-path (3a.5) | ✅ Shipped (#496) |
| Substrate primitives (Track 9) | ✅ Shipped (#516) |
| `NoBody` substrate distinction (#518) | ✅ Shipped |
| Effect substrate (Lane 2 Stage 2a) | 🟡 ComposedEffect reshape in flight |
| Workflow effect carrier (Lane 2 Stage 2b) | ❌ Not started (gated on 2a close) |
| Test obligation materialization (Lane 2 Stage 2c) | ❌ DB-15 R2 drafted; implementation not started |
| Symbolic cost (Lane 2 Stage 2d) | ❌ DB-7 locked; implementation not started |
| Diagnostics-as-corrections (Lane 3 Stage 3b) | ❌ DB-1 locked; gated on Lane 1c close |
| **Lane 1 Stage 1e — single generic walker** | ❌ **HARD BLOCKER** (see below) |

### The Lane 1 Stage 1e gate

**Stage 3c cannot start until Lane 1 Stage 1e lands.** Per lane3-self-hosting-cycle.md dependencies: *"Requires Lane 1 Stage 1e complete — self-hosting through fragmented per-target emitters is worthless. The dissolved single-emitter is what gets re-emitted in 3c."*

Critical path from post-l15-phase-plan.md: `1a → 1b → 1c → 1d → 1e → 3c`.

Current critical-path status:
- **1a** ✅ PR #495
- **1b** 🟡 DB-14 wired; ROADMAP has "Deferral: 1b full implementation (M)"
- **1c** 🟡 Rust+Go pilots shipped; Python pending; post_emit_verifier CI gate pending
- **1d** ❌ not started
- **1e** ❌ not started (design written just before execution, informed by 1a–1d)
- **3c** ❌ gated on 1e

**Stage 3c is several stages away from dispatch.** Everything in §2 (in flight) and §3 (next batch) is UPSTREAM of 3c. The director-chat role for 3c in the current window is substrate-readiness tracking, NOT pre-clearance of 3c itself (DB-8 already did that).

### Genuinely open 3c questions (not in DB-8 or lane3 doc)

1. **Does 3c fire on compiler.dag AS-IS, or on an expanded compiler.dag?** compiler.dag today is 310 lines of cycle meta-model. If 3c fires the cycle using the existing hand-written Rust compiler, compiler.dag doesn't have to grow. If the intent is "compiler.dag EXPRESSES the compiler so the cycle is non-trivial," that's the horizon beyond 3c (M3 per original roadmap). **Open for next director session.** Candidate acceptance for 3c: "cycle fires on compiler.dag as it stands — trivially bit-identical because generated Rust depends on no field of compiler.dag that varies." Distinct from M3 acceptance: "compiler.dag EXPRESSES the compiler and the cycle re-derives the Rust emitter." Pick one or declare both.

2. **Bootstrap sequencing for the first self-hosted run.** DB-8 assumes the v3-Rust-compiler can produce a working binary from compiler.dag. If compiler.dag doesn't include parser/lowerer/emitter logic, the binary doesn't actually DO the compiler's work — it runs the cycle-runner. Question for next director session: is there an intermediate "self-hosting of the cycle-runner" milestone before "self-hosting of the compiler proper"?

3. **compiler.dag's `hand_maintained_src` references v2 paths** (`src/v2/stage0/src`, `cli_run.rs`, `v2_interpreter.rs`). Stage 3c operates on v3's compiler source, not v2. Does 3c require compiler.dag itself to be rewritten to v3-centric first, or is it a v2→v3 bridging detail that self-resolves? Tracked as a §5b migration candidate pending director pre-clearance — don't speculate on the answer here.

4. **Determinism test precedes 3c.** DB-8 prescribes `tests/determinism_test.rs` (per-fixture 5x re-run). This should land BEFORE 3c as part of Lane 1 Stage 1e acceptance — it's a unit-level prerequisite that catches non-determinism at emit-level before the full cycle runs. **Recommendation:** add `tests/determinism_test.rs` to Lane 1 Stage 1e acceptance list when 1e design is written.

### What the next director session should do

Explicit time-budgeted work (first 60 min of the next session, regardless of tactical surprises — triage surprises in <10 min then return to this block):

1. **30 min** — read DB-15 (test-infra R2) + `lane2-compile-time-proofs.md` Stage 2c. Confirm pre-clearance is R2-read + handoff, not new design. Output: dispatch brief for Stage 2c when Stage 2b lands.
2. **20 min** — trace DB-7's WorkflowEffect integration points. Confirm whether Stage 2d needs a DB-7 addendum for WorkflowEffect or dispatches directly. Output: either a brief for Stage 2d dispatch, or an explicit "2d needs DB-7 addendum" finding with scope.
3. **Remaining time** — open questions 1–4 above. Output: updates to this doc's §6 in-place; may produce 1-2 new Lane 3 follow-up entries in ROADMAP.

---

## 7. Operating model — how to use this document

### Director chat

- Reads §1 (where we are) + §2 (in flight) at session start AND cross-checks §1 against `git log` for staleness (this review caught PR #523 already-merged during director review).
- Updates §3 when current batch settles.
- Pre-clears items in §4 — **with the discipline that an existing DB doc counts as pre-clearance, not "un-pre-cleared."** This review caught the draft's "DB-18/19/20 candidate" framing re-deriving work DB-7/DB-8/DB-15 already locked.
- Migrates §5b to ROADMAP when entries stabilize.
- Owns §6 Stage 3c readiness exploration.

### Child chats

- Read this doc to understand their lane's place.
- §3 briefs are paste-ready into fresh chats.
- §4 items get briefs drafted here before dispatch.

### Structural-finding escalation rule (load-bearing)

When a child chat discovers that director-chat pre-clearance was wrong — the design contract needs reshape, a DB doc contradicts the dispatch brief, a substrate primitive doesn't fit — the child chat **HALTS and reports back** rather than patching forward. Silently absorbing a structural finding destroys the signal that pre-clearance exists to capture.

Every §3 dispatch brief names this explicitly (STOP-AND-ESCALATE rule). Children should treat it as load-bearing, not a formality.

### Status update discipline

- Each merge → update §1 + §2 tables with the exact commit SHA.
- Each design pre-clearance → move §4 item to §3 or directly to dispatched.
- Each new tracked-debt finding → add to §5b.

### Refresh cadence

Session start + after every merge wave. Never more than ~1 day stale. When stale, add a "since last refresh" note at the top and cross-check §1 against `git log`.

### DB-numbering discipline

Do NOT pre-reserve DB numbers. Assign at the moment a DB doc is opened, not at the moment of speculation. This draft's review caught collision pressure on DB-17 (reference-resolution provenance vs WorkflowEffect) and redundancy with DB-7/8/15.

---

End of phase plan.
