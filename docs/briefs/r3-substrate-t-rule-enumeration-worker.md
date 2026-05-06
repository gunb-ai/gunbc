---
status: draft (worker brief; pre-authored per pre-authored-brief-queue discipline; dispatch fires on Director Q2 ratification of emission-provenance canvas at gunbc#828 #issuecomment-4392519713)
authority parent: R3 Substrate Manager (#1739)
ratification: pending — gates on Director Q2 ratification of `r3-substrate-emission-provenance-shape-canvas.md` (Q2: T-Rule-Enumeration parallel-dispatch authorization)
roadmap row: §1.8 ledger row TBD (gate: `langspec_emission_rules_enumerable`); cluster pending Q1 disposition (likely T-LangSpec or new cluster)
authority docs:
  - docs/briefs/r3-substrate-emission-provenance-shape-canvas.md (parent canvas — Q2 names this brief as prerequisite to every Q1 path)
  - src/v3/compiler/src/emit/rust_target.rs (current emission code; uses inline `&str` template names via `render_named_template(template: &str, ...)`)
  - gunbc#1739 #issuecomment-4392477594 (Substrate Mgr fold-rule-enumerability finding — 0 hits on RuleName/FoldRule/EmitRule/EmissionRule across src/v3/)
  - gunbc#846 #issuecomment-4392510255 / #issuecomment-4392543633 (PM concur on Path 1)
gates:
  - `langspec_emission_rules_enumerable` (proposed §1.8 row; slot pending)
worker pin: TBD — smart-ram-167 (S12-retirement-context-fresh) OR valiant-ibex-312 (substrate-authoring discipline fresh post-#1842) per Mgr discretion at dispatch
---

# R3 Substrate T-Rule-Enumeration — LangSpec emission-rule names as enumerable substrate worker brief

## Context

Substrate-state-grep at HEAD (gunbc#1739 #issuecomment-4392477594):
- `RuleName` / `FoldRule` / `EmitRule` / `EmissionRule` → 0 hits in `src/v3/`
- `derive_for_disj` / `predicate_per_variant` / `constructor_per_conj` → 0 hits anywhere
- `src/v3/compiler/src/emit/rust_target.rs` (6611 LOC) is template-driven: `render_named_template(template: &str, bindings: &[(&str, &str)]) -> String` is the central rendering primitive; template names exist as **inline `&str` literals at each call site**, NOT as enumerable identifiers / data declarations

This is structurally a substrate-fact-introduction (P1 procedure):
- **Why now**: PR #1902 emission-provenance lane surfaced that any
  carrier naming `EmissionOrigin::FoldRuleAutoEmit` (whether Lens<C>
  or per-line instrumentation per Q1 disposition) cannot faithfully
  name the rule until the rule set is enumerable as data
- **Why not bundle**: per Director bundled-scope discipline at
  gunbc#1739 #issuecomment-4392225548, rule-name enumeration is an
  independent substrate-fact-introduction; bundling = DISALLOWED
  parallel infrastructure
- **Why prerequisite to every Q1 path**: (a) per-Behavior lens / (b)
  per-line instrumentation / (d) both — all three carrier-naming
  paths consume the rule-name set. Path (c) withdraw renders this
  unblocked-but-unconsumed; still useful for any future
  emission-introspection consumer

## Scope

### Deliverable 1 — Rule-name carrier authoring

Author rule-name carrier in `src/v3/std/` (worker greps for canonical
location at dispatch — likely `langspec.dag` adjacent if such file
exists, or new file `langspec_emission_rules.dag`). Two shape options
worker chooses via DFS:

- **Option α (sum type)**: `type EmissionRule = DeriveForDisj | PredicatePerVariant | ConstructorPerConj | ...` — enumerated variants per current rule. **🟢 PRIMITIVE** if rule set is small (<20) and stable; closed sum.
- **Option β (named-string list)**: `data emission_rules: List<RuleName>` where `RuleName = String` with discipline that emission code looks up by name. **🟡 SCAFFOLD** with named dissolution trigger (rule-name typo'd at usage site won't fail closed; closed-sum form is structural).

**Mgr recommendation: α (sum type) if rule set is enumerable closed**. Worker DFS-catalogs current `render_named_template(...)` call sites at HEAD to count actual rule names; if count is reasonable (<20-30) and rules are structurally distinguished (not just template variants), α is the right shape. If rules turn out to be open-ended (template-name-as-data per LangSpec authoring convention), β with named dissolution trigger.

### Deliverable 2 — Emission code refactor: dispatch via named-rule lookup

Refactor `src/v3/compiler/src/emit/rust_target.rs` (and any sibling
emission files surfaced via grep) to:

- Replace inline `&str` template names at `render_named_template` call sites with named-rule references (e.g., `EmissionRule::DeriveForDisj` enum variant under α; `EMISSION_RULES["derive_for_disj"]` lookup under β)
- Keep `render_named_template(template: &str, ...)` signature unchanged at the leaf (the rendering primitive doesn't need to know about rule names; the caller resolves the rule reference to its template string before calling)
- Bootstrap snapshot + parse corpus manifest must hold post-refactor (no semantic drift; refactor is cosmetic-but-structural — names become enumerable, semantics unchanged)

### Deliverable 3 — Practice 4 checkpoint

Per `docs/modeling-discipline.md#4-coproduct-dissolution`:
- α (sum type with N≥2 variants): 🟢 PRIMITIVE checkpoint comment naming the classification + ledger entry
- β (named-string list): 🟡 SCAFFOLD checkpoint with named dissolution trigger (e.g., `rule_name_typo_fail_closed_landed` — closed-sum graduation gate)

Worker authors in-source checkpoint comment on the live declaration
(not just PR-body summary) per modeling-discipline.md.

### Deliverable 4 — §1.8 ledger row receipt

Add `langspec_emission_rules_enumerable` to §1.8 ledger; advance
DECLARED → CONSUMER_LANDED on merge IF emission code refactor
(Deliverable 2) lands same-PR (consumer + producer co-located —
allowed per bundled-scope discipline as "necessary structural fix").
Otherwise PRODUCER_LANDED only.

## Slice — single PR

Phase ordering (PR-internal):
1. DFS-catalog `render_named_template(...)` call sites at HEAD; enumerate distinct template-name strings
2. Choose α vs β shape based on enumeration result; author Practice 4 checkpoint
3. Author rule-name carrier (Deliverable 1)
4. Refactor emission code to dispatch via named-rule lookup (Deliverable 2)
5. Verify bootstrap snapshot + parse corpus manifest hold (semantic-equivalence verification)
6. §1.8 ledger row receipt (Deliverable 4)

## Acceptance

- Rule-name carrier landed in `src/v3/std/` per α or β shape with
  Practice 4 checkpoint comment + ledger entry / dissolution trigger
- Emission code (`rust_target.rs` + any siblings) refactored to
  dispatch via named-rule lookup; inline `&str` template names at
  `render_named_template` call sites replaced
- Substrate-state-grep verification: post-refactor, `grep -E "RuleName|FoldRule|EmitRule|EmissionRule"` returns ≥ N hits (N = number of distinct rules enumerated); previously 0
- Bootstrap snapshot + parse corpus manifest hold (semantic equivalence)
- §1.8 ledger row `langspec_emission_rules_enumerable` advances
  DECLARED → CONSUMER_LANDED (or PRODUCER_LANDED if refactor splits)
- `cargo test --workspace --exclude v2-compiler-tests` green (3 pre-existing v2-compiler --lib failures verified unrelated)
- `cargo test -p v2-compiler-tests` green; strict-compile diagnostic ratchet at 0
- `cargo clippy --all-targets -- -D warnings` clean
- `cargo fmt --all --check` clean
- Citation discipline per `docs/briefs/brief-authoring-checklist.md`
- 5-question authority audit in PR body
- P1 substrate-fact-introduction receipt:
  - DFS-of-concept-DAG (no parallel rule-name carrier already exists)
  - Named consumer demand (Lens<EmissionProvenance> / per-line
    instrumentation per Q1 disposition; both consume this substrate)
  - Carrier-shape rationale (α vs β with reasoning)

## STOP-AND-ESCALATE

- **Rule set is unbounded / open** (e.g., LangSpec authors define new templates per-target-language and the set is expected to grow significantly): STOP — α closed-sum form may be wrong; surface to Substrate Mgr; β with named dissolution trigger may be the only shape, OR a different decomposition (template-name-as-data per LangSpec carrier) is needed
- **Refactor surfaces semantic drift** (e.g., a rule's template binding can't be cleanly extracted to a named lookup without changing emission output): STOP — bootstrap snapshot fail is the canonical detection; root-cause; do NOT bridge with `// TODO match prior output` placeholder
- **Refactor scope expands beyond emission/rust_target.rs** (e.g., other emit/* files have parallel template-name conventions that ALSO need refactor): STOP — surface scope expansion; bundled-scope check (necessary-structural-fix vs parallel-infrastructure); Mgr disposes
- **DSL-side rule-name dispatch needs new substrate not yet landed** (e.g., enum-variant-as-runtime-key requires reflection substrate that doesn't exist): STOP — substrate-fact-introduction cascade; surface
- **Bundled-scope drift on the consumer side**: do NOT bundle `Lens<EmissionProvenance>` or per-line instrumentation authoring into this PR. Q1 disposition is downstream; this brief is upstream substrate. Per Director ratification at gunbc#1739 #issuecomment-4392225548 — parallel infrastructure DISALLOWED

## Authority audit receipt

1. **Substrate exists?** Substrate-state-grep at HEAD (verified by Substrate Mgr at gunbc#1739 #issuecomment-4392477594):
   - `RuleName` / `FoldRule` / `EmitRule` / `EmissionRule` → 0 hits across `src/v3/`
   - Specific rule names (`derive_for_disj` / `predicate_per_variant` / `constructor_per_conj`) → 0 hits anywhere
   - Worker re-greps at dispatch to confirm nothing has been authored in the interim
2. **Existing brief?** No standalone brief at HEAD. Parent canvas `r3-substrate-emission-provenance-shape-canvas.md` Q2 names this prerequisite; this is the dispatch packet
3. **Design-doc match?** `feedback_no_textual_enforcement_bridges` + `feedback_compositional_not_templating`: rule-name-as-data structurally enables faithful provenance carriers; current inline-`&str` form is the textual-enforcement anti-pattern. INVARIANTS P1 substrate-fact-introduction procedure binds
4. **Citations live?** Worker re-verifies at dispatch; canvas + program plan rows are the authority
5. **Carrier dissolves the bridge?** Yes — rule-name enumeration carrier dissolves the gap between "templates are rendered with named identifiers" and "those identifiers are not data the type system can refer to." Once enumerable, every downstream consumer (Lens<EmissionProvenance>, per-line instrumentation, future emission-introspection consumers) can name rules faithfully

## Provenance

Drafted 2026-05-06 by Substrate Mgr (quick-crab-830) per Mgr authoring authority on substrate-fact-introduction (PM concur at gunbc#846 #issuecomment-4392510255 / #issuecomment-4392543633). Pre-authored per pre-authored-brief-queue discipline; dispatch fires on Director Q2 ratification of canvas (gunbc#828 #issuecomment-4392519713). Worker pin assigned at dispatch.

Cross-references:
- Canvas `r3-substrate-emission-provenance-shape-canvas.md` (parent — Q2 prerequisite)
- PR #1902 (PM-authored Lens<EmissionProvenance> proposal — superseded by canvas; closes per PM disposition at #issuecomment-4392528502)
- Q1 disposition (a/b/c/d) — independent of this brief; this brief is prerequisite to every consuming path
