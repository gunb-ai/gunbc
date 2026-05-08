# Canvas — Q-Complexity-Composition-Layering (substrate-shape question)

**Authority**: Director authorization at gunb-ai/gunbc#828 #issuecomment-4402669133 — canvas-tier scope question for complexity-lens BEHAVIORAL COMPLETION; parallel structure to Q-Cost-Composition-Layering (RATIFIED 2026-05-07 ε path) but **factoring claim differs per substrate-grep**.

**Status**: **canvas — DRAFT 2026-05-08**; PROPOSAL maturation pending Director ratification.

**Sibling canvases (already RATIFIED)**: `q-cost-composition-layering-canvas.md` (ε path; cost-lens specific factoring); `q-lens-target-context-canvas.md` (β-extended; DEFERRED to N=2 trigger).

## The factoring claim under test

The cost-lens canvas RATIFIED ε factoring:

> **(target-agnostic-shape) × (target-specific-values)** — abstract `SymbolicCost` algebra (Layer 1 objective concepts pure) × per-primitive realization-cost values (Layer 2 LanguageSpec) → composed Rust-side at emit time (Layer 3 emit composition).

**Question**: does complexity-lens completion fit the same factoring? **Substrate-grep finding (per `src/v3/lenses/complexity.dag` HEAD): NO.**

## Substrate-grep grounding (HEAD: `complexity.dag` at origin/main)

Status header at HEAD:
> Status: STRUCTURALLY TERMINAL; BEHAVIORALLY PROXY. ... Forward fold over `d.nodes` builds a per-port integer depth map. ... **This lens does not subsume v2's complexity analysis.** v2's `src/v2/complexity.dag` produces `ComplexitySummary { work, span, output_size, certainty }` with symbolic `CostExpr` / `SizeExpr` and asymptotic classification. This lens produces a single integer per port — the structural depth of the producing behavior in the DAG. ... **Genuine equivalence blocks on consuming the staged `DescentEvidence`, `CallPattern`, and `SubValueRelation` carriers at live call sites.**

Key structural facts:

1. **Output type at HEAD**: `Lookup<Int>` per port (single integer = structural depth). NOT a target-keyed algebra.
2. **PROXY → COMPLETE delta**: blocking-on consumption of `DescentEvidence` / `CallPattern` / `SubValueRelation` carriers — **all just landed via T-E-P P1 cascade (Slices 1-4 closed gates #76/#77/#78 + cementing per Slice 4 #2192 merged 2026-05-08 00:42Z)**.
3. **No target-realization-cost reading**: complexity-lens output is structural-depth-of-DAG, not per-target concrete-cost. No HashMap-build over `TypeRealization` / `CallableRealization` rows. No `LanguageSpec` parameter. No target-context dependence.
4. **No symbolic algebra**: `ComplexitySummary { work, span, output_size, certainty }` from v2 is **R3 scope per option (b) RATIFIED 2026-05-06** but not yet substrate-typed at HEAD; v3's `complexity.dag` produces only structural-depth `Int`.

## Factoring test against `feedback_abstraction_layering`

Re-applying the abstraction-layering test the cost-canvas used:

1. **Layer 1 (objective concepts pure)**: complexity-lens's "abstract shape" at HEAD = `Lookup<Int>` (structural depth). The `ComplexitySummary` algebra is the R3-scope target shape. Both are target-agnostic by construction (depth + work/span/output-size are properties of the DAG structure, not of any particular target language).
2. **Layer 2 (LanguageSpec)**: **N/A** — complexity composition does NOT consume per-primitive realization-cost rows. Structural depth is computed from the DAG topology alone; work/span/output-size from algebra-instance composition (Semiring / Semilattice operations on the carriers themselves). No LanguageSpec data flows in.
3. **Layer 3 (emit-side composition)**: **N/A** — there is no "concrete per-target value" downstream of the abstract shape. Complexity-lens output is the final answer, not an intermediate consumed by Rust-side per-target multiplication.

**Factoring conclusion**: ε's (target-agnostic × target-specific) factoring **does not apply** to complexity-lens because there is no target-specific axis. Complexity composition is **fully .dag-side substrate-native** — no cross-layer Rust consumer needed.

## What complexity-lens BEHAVIORAL COMPLETION actually requires

Per the lens's own status header + R3 scope per Q-Lens-Behavioral-Parity-R3-Closeability option (b) RATIFIED 2026-05-06 (gunb-ai/gunbc#828 #issuecomment-4385329180):

**Gate #79 `complexity_lens_behaviorally_complete`** ledger text (`docs/r3-program-plan.md:278`):
> symbolic CostExpr + work/span split + asymptotic classification + cementing test

The completion path is **direct .dag-side substrate authoring**:

1. **Symbolic CostExpr carrier authoring**: introduce `ComplexityCost` / `WorkSpan` / `AsymptoticClass` algebra in `src/v3/std/algebra.dag` (or sibling) — analogous to how `SymbolicCost` lives there for cost-lens. Carrier introduction is P1 substrate-fact-introduction; needs prior canvas if first-precedent, but the cost-lens precedent shows the algebra-authoring pattern is well-trod.
2. **Lens output type widening**: change `complexity.dag::cost_of` from returning `Lookup<Int>` to returning `Lookup<ComplexitySummary>` (or the chosen carrier shape).
3. **Algebra-composition rules**: Semiring-style sequential / parallel composition rules for work-span; asymptotic-classification fold over the carrier.
4. **Carrier consumption from T-E-P P1**: where the depth fold currently treats Transform behaviors uniformly (all weight 1), the completed lens consumes `DescentEvidence` / `CallPattern` / `SubValueRelation` to classify recursive-call behavior asymptotically (e.g., StrictSubValue = O(log n) recursive descent vs SubValueUnknown = unbounded).

Target-context independence holds throughout: complexity is a property of the DAG structure + algebra-instance composition, not of any target language.

## Mgr provisional reading

**ε-precedent does NOT apply**. Complexity-lens completion is structurally **direct .dag-side substrate authoring + carrier introduction + T-E-P P1 carrier consumption** — none of the cost-lens "Rust-side composition" pattern translates. The cross-layer factoring question is moot here because there's no target-specific axis to factor.

**Implication for dispatch**: complexity-lens BEHAVIORAL COMPLETION is **a fresh Substrate slice-tier work-item**, NOT a canvas-pair-deferred substrate-shape question. Concrete next steps:

1. **Carrier-introduction canvas (mini)**: brief substrate-shape question on `ComplexityCost` / `WorkSpan` / `AsymptoticClass` carrier shape. Likely a quick ratification — cost-lens's `SymbolicCost` carrier is precedent for the algebra-authoring discipline. Could be slice-tier directly if Director ratifies analogous-to-SymbolicCost shape without canvas.
2. **Slice authoring**: lens widening + algebra composition + T-E-P P1 carrier consumption. Mgr-tier dispatch to fresh worker pin (eager-bat-178 NOT a fit per Director read; this is different problem class than producer-broadening).
3. **Cementing test (#1950)**: downstream once substrate completion lands; consumes frozen v2-oracle snapshot per `r2-structure.md` §"Lane structure" framing. v2-oracle snapshot capture status pending Verification Mgr (#2075) cross-Mgr ping (in flight: #2075 c#4402679181).

## Honest scope-question delta vs cost-lens

The **load-bearing distinction** between cost-lens (ε ratified) and complexity-lens:

| | Cost-lens (ε path) | Complexity-lens (this canvas) |
|---|---|---|
| Cross-layer factoring needed? | YES (target-realization-cost reading) | NO (no target axis) |
| Rust-side composition? | YES (HashMap-build over realization rows) | NO (.dag-side substrate-native) |
| `LanguageSpec` parameter consumption? | At Rust-consumer boundary | Not used |
| Substrate-shape ambiguity? | Yes (ε vs β-extended) | No (one path: direct .dag substrate completion) |
| Canvas-vs-slice boundary | Canvas tier (substrate-shape question warranted) | Slice tier (carrier introduction is small ratification) |

## Director ratification ask

**Provisional reading for ratification**:

1. **Q1**: Is the substrate-grep finding correct that complexity-lens factoring does NOT match cost-lens ε? (i.e., no target-context axis; no Rust-side composition; pure .dag substrate completion)
2. **Q2**: If Q1 yes, do you ratify **slice-tier dispatch directly** (carrier-introduction follows `SymbolicCost` precedent + T-E-P P1 carrier consumption) — bypassing canvas-tier scope question for the carrier shape?
3. **Q3**: If Q2 yes, fresh worker pin or hold for queue?

**Mgr lean**: Q1 = yes; Q2 = yes (cost-lens carrier-introduction precedent makes this a follow-on, not first-precedent); Q3 = fresh worker pin once briefs author.

## Sibling canvases anchor

- `q-cost-composition-layering-canvas.md` — ε RATIFIED for cost
- `q-lens-target-context-canvas.md` — β-extended DEFERRED to N=2 trigger
- This canvas — complexity-lens factoring honestly tested and found NOT to fit ε; recommended slice-tier disposition

## Framework discipline anchors

- **`feedback_abstraction_layering`**: applied. Layer 1/2/3 test surfaces the no-target-axis finding directly.
- **`feedback_per_instance_ratification` (Pattern-A discipline analog)**: ε is per-lens, not blanket-extended. Complexity-lens explicitly tested fresh; finding may differ from cost.
- **`feedback_construction_over_ratchets`**: substrate completion proceeds by direct authoring (carrier + lens + composition), not by ratchet-over-PROXY.
- **`feedback_substrate_principle_audit`**: substrate-fact-introduction (carrier shape) gets canvas-tier ratification; if Director ratifies cost-lens precedent applies, slice-tier directly.

## Cross-Mgr coordination

- **Verification Mgr (#2075)**: v2-oracle snapshot capture status — cross-cutting prerequisite for cementing dispatch (#1950 + #1951 both gate on this); pinged at #2075 c#4402679181.
- **Grounding Mgr (#1944)**: not applicable — complexity-lens has no target-realization data dependency.
- **PM (#846)**: cross-lane T-LBP narrowed-scope partner brief (`docs/briefs/r3-v-t-lbp-narrowed-scope-partner-worker.md`) consumes this canvas's disposition for downstream cementing-discipline.

— Authored by warm-wolf-698 (Substrate Mgr) 2026-05-08 per Director authorization at gunb-ai/gunbc#828 #issuecomment-4402669133.
