# Worker brief — Substrate T-LBP complexity-lens substrate completion

**Sub-issue**: TBD (parented under R3 Substrate Mgr lane #1939; new sub-issue created at dispatch time per same-window-dispatch discipline).
**Authority**: Director ratification of Q1/Q2/Q3 at gunb-ai/gunbc#828 #issuecomment-4402714255 — Q-Complexity-Composition-Layering canvas (`docs/proposals/q-complexity-composition-layering-canvas.md`) finding ratified: ε precedent does NOT apply (no target-context axis); slice-tier dispatch authorized; `SymbolicCost` precedent applies for carrier introduction (canvas-tier bypassed for carrier shape).
**Closure predicate**: §1.8 row #79 `complexity_lens_behaviorally_complete` — symbolic CostExpr + work/span split + asymptotic classification + cementing test. **This brief covers substrate completion only**; cementing test (#1950) is a separate downstream brief at `docs/briefs/r3-substrate-t-lbp-complexity-lens-cementing-test-worker.md`.

## Important framing — this is NOT P1 substrate-fact-introduction

`SymbolicCost` is the established precedent for algebra carrier introduction (substrate-tier; canvas-tier-bypass-able when precedent applies per Director Q2 ratification). `ComplexityCost` / `WorkSpan` / `AsymptoticClass` parallel that shape; carrier introduction is **follow-on substrate authoring**, NOT first-precedent P1 work.

`src/v3/lenses/complexity.dag` ALREADY EXISTS at HEAD as `STRUCTURALLY TERMINAL; BEHAVIORALLY PROXY` (forward fold over `d.nodes` building `Lookup<Int>` per port = single-integer structural depth). This brief advances it to `STRUCTURALLY TERMINAL; BEHAVIORALLY COMPLETE` via carrier introduction + lens widening + T-E-P P1 carrier consumption.

## Scope (binding per Director Q1/Q2/Q3 ratification)

Three coordinated authoring deltas:

### Deliverable 1 — Carrier introduction (algebra)

Author `ComplexityCost` (or chosen alternate carrier name) algebra parallel to `SymbolicCost` 7-variant shape at `src/v3/std/algebra.dag` (or sibling location appropriate per algebra organization at HEAD). Required carriers per ledger row #79 framing:

- **Symbolic cost expression** carrier (analog of `SymbolicCost`)
- **Work/span split** — typically two carriers or a record `WorkSpan { work: ComplexityCost, span: ComplexityCost }`
- **Asymptotic classification** — finite enum of asymptotic classes (`O(1)` / `O(log n)` / `O(n)` / `O(n log n)` / `O(n^2)` / `O(2^n)` / `Unknown` — exact set is worker call per substrate-tooling at HEAD; v2's classification at `src/v2/complexity.dag` is the precedent reference)

Sequential / branch / iterate composition rules via `Semiring<ComplexityCost>` (or analogous algebra instance) — same shape as `Semiring<SymbolicCost>` per cost-lens precedent. `MaxPath` semantics for span composition (parallel max; sequential sum).

### Deliverable 2 — Lens widening at `src/v3/lenses/complexity.dag`

Change `cost_of(d, port_id)` from returning `Lookup<Int>` to returning `Lookup<ComplexitySummary>` (or chosen carrier shape). The forward-fold structure stays — substrate invariant that producers precede consumers in `d.nodes` continues to make this a pure catamorphism. Widening is mechanical: `entry_for(acc, behavior)` returns the structurally-typed carrier rather than a single integer.

Status header refines: `STRUCTURALLY TERMINAL; BEHAVIORALLY PROXY` → `STRUCTURALLY TERMINAL; BEHAVIORALLY COMPLETE`.

### Deliverable 3 — T-E-P P1 carrier consumption

For Transform behaviors that are recursive self-calls, consume `DescentEvidence` / `CallPattern` / `SubValueRelation` carriers (landed via T-E-P P1 Slices 1-4 at #2167/#2178/#2182/#2192) to classify recursive-call asymptotic behavior:

- `StrictSubValue` per-arg → bounded structural descent → finite recursion → composes into asymptotic class per call-pattern (e.g., halving → `O(log n)`; element-wise → `O(n)`)
- `SubValueUnknown` → asymptotic class falls through to `Unknown`
- `PreservedValue` → non-descending arg; doesn't drive recursion bound

**Indirect-call dependency** (worker grep at slice authoring time):

T-E-P P1 Slice 5+ (Indirect/`TransformDispatch::Indirect` / `ArrowPortRef` variant) is a separate active work stream (eager-bat-178, dispatched at gunb-ai/gunbc#2166 c#4402718554; PR #2198 in flight on a different scope, sequencing TBD). If the complexity-lens slice's representative-source corpus exercises higher-order-fn patterns (`fold(list, init, f)` / `map(list, f)` etc.) that use indirect dispatch, this brief's worker may need to **wait on Slice 5+ Indirect-variant land** for those classifications. Worker should grep at slice-authoring time:

- If complexity-lens completion can demonstrate on direct-call-only representative sources, proceed without Indirect-variant dependency.
- If indirect-call coverage is structurally required for the asymptotic-classification carrier consumption, STOP and surface to Mgr for sequencing coordination with eager-bat-178.

## Out-of-scope (deferred)

- **Cementing test (#1950)**: covered by separate downstream brief `docs/briefs/r3-substrate-t-lbp-complexity-lens-cementing-test-worker.md`; gates additionally on frozen v2-oracle snapshot capture (cross-Mgr ownership Q4 pending at #828 c#4402702798; PB Mgr lean per Verification authoritative read at #2075 c#4402691581).
- **`Lens<C>` generic carrier-shape refactor**: β-extended DEFERRED to N=2 trigger per `q-lens-target-context-canvas.md`; STOP-and-PING if surfaces during this slice.
- **Cost-lens substrate completion**: separate active stream (fierce-ram-21 ε path at PR #2194); not absorbed.

## Acceptance gates (same-slice, all must pass)

1. `complexity.dag` advanced from `STRUCTURALLY TERMINAL; BEHAVIORALLY PROXY` → `STRUCTURALLY TERMINAL; BEHAVIORALLY COMPLETE` per `docs/v3-lens-capability-register.md` audit; status header updated in the file itself.
2. Carrier introduction: `ComplexityCost` (or chosen name) + `WorkSpan` + `AsymptoticClass` algebra at appropriate `std` location with Semiring composition; carriers consumable structurally.
3. Lens output type widened from `Lookup<Int>` to `Lookup<ComplexitySummary>` (or chosen carrier shape); forward-fold structure preserved.
4. T-E-P P1 carrier consumption: recursive-call asymptotic classification reads `DescentEvidence` / `CallPattern` / `SubValueRelation` per call-pattern.
5. **#79 satisfied by construction**: lens output documented to carry symbolic CostExpr + work/span split + asymptotic classification. Code-level cite in PR description.
6. Bootstrap regen: `cargo test -p v3-compiler bootstrap_regen_fresh -- --ignored` clean.
7. Full suite: `cargo test --workspace --exclude v2-compiler-tests` green; `cargo clippy --all-targets -- -D warnings` clean.
8. **§10.3 row text refresh**: `docs/r3-program-plan.md` §10.3 T-LBP complexity-lens substrate-completion row updated to cite Q-Complexity-Composition-Layering canvas RATIFIED + this PR's # as the receipt.

## STOP / PING criteria

- **STOP** if Indirect-call coverage is structurally required for the chosen representative-source asymptotic classifications — surface to Mgr for sequencing coordination with eager-bat-178 Slice 5+ work.
- **STOP** if `SymbolicCost` precedent shape doesn't actually fit `ComplexityCost` (e.g., asymptotic-classification carrier needs structurally different shape than 7-variant). Surface to Mgr; may re-open canvas-tier scope question for carrier shape (Director Q2 was ratified contingent on precedent applying).
- **STOP** if lens widening reveals `Lens<C>` generic at `lens.dag:70-77` needs carrier-shape refactor — that's β-extended DEFERRED to N=2 trigger per sibling canvas; do NOT absorb into this slice.
- **PING** Verification Mgr (#2075 / wise-bear-525) at PR-open time per Pattern-A executable-gate ratchet discipline (gate #79 transition from PROXY → COMPLETE).

## Worker pin

Fresh pool pin TBD (NOT eager-bat-178 per Director Q3 framing — different problem class than T-E-P P1 producer-broadening; NOT fierce-ram-21 — busy with cost-lens ε Slice 1a→1b chain). Mgr's call on which fresh-pool worker at dispatch time.

## Cross-Mgr coordination

- **Verification Mgr (#2075)**: PING at PR-open per gate #79 PROXY → COMPLETE transition; ratchet authoring is Verification's standing concern.
- **Eager-bat-178**: cross-stream coordination if complexity-lens slice's worker finds Indirect-variant dependency surfaces during T-E-P P1 carrier consumption authoring.
- **#1950 downstream**: complexity-lens cementing test dispatch (separate brief) gates on this brief's substrate-completion landing + frozen v2-oracle snapshot capture (Q4 cross-Mgr ownership).

## Auto-spawn caveat

Per Director's standing note + cache-staleness cluster ctrl#217: HOLD dispatch on this brief until auto-spawn fix lands per L-sized substrate-behavioral-completion threshold, OR Mgr-direct authoring if scope-cohering small-enough (this is L-sized; Mgr-direct is unlikely fit). Same-window-dispatch discipline applies post-canvas-merge.

— Authored by warm-wolf-698 (Substrate Mgr) 2026-05-08 per Director Q1/Q2/Q3 RATIFIED at gunb-ai/gunbc#828 #issuecomment-4402714255; sibling canvas at `docs/proposals/q-complexity-composition-layering-canvas.md` (RATIFIED via PR #2197).
