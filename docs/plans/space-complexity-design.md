# Space complexity — the dual of the time/termination analysis

Status: DRAFT for operator review (2026-07-16, session lively-heron-615). Operator direction: extend the complexity analysis to **space** — the current gap. Two readings wanted: **asymptotic** (`O(n)`, `O(n²)`, …) *and* **concrete derived space** ("this process is known/defined to these bounds" — a byte figure for closed inputs). **Skip measuring** — memory is *derived*, not observed (§4: bounded-and-forward ⇒ checked, not discovered). Staging: an underivable bound is a **counted frontier now, a hard error later** (fail-closed ratchet, operator 2026-07-16).

This folds together five threads onto one page: the memory-control audit (space unobservable/underived — F2/F3), P1 (the space algebra), P4 (`realize` packing), the 2026-07-12 ruling (per-runnable demand *derived, not authored literals*), and the effect-grants/allocation model (allocations known up front).

## 0. Why derived, not measured (the axiom)

Measuring peak RSS is the space equivalent of proving termination by running the program until it stops — the *"reflection evidence ≠ structural proof"* failure mode. The substrate is closed, bounded, and forward, so termination is **checked, not discovered** (`DescentEvidence`); space is the same kind of quantity and must be derived the same way. "How much can this instance of this program hold at a time" is decidable from the model (a working-set bound parameterized by input size), exactly as time complexity is `O(n)` while `n` is free. So P1's `ObservePeakResidentAtSubject` is demoted to at-most a **falsifier** (does actual usage stay under the derived bound? a fail-closed audit control), never a scheduler input.

## 1. Space is a *second reading* of the cost machinery that already exists — not a new analysis

`ComplexitySummary` (`src/v1/complexity.dag`) already carries three `CostExpr`s: `work` (total = time), `span` (critical path = parallel time), and `output_size` (the size of the result). `CostExpr` is **axis-agnostic** — the same type expresses time and size. So space is a fourth reading on the same vocabulary, not a fork (§2/§3).

`CostExpr = CostConst | CostAdd | CostMul | CostMax | CostSum{binder,upper,body} | CostLog | CostExtern | CostUnknown`. The load-bearing node is **`CostSum{binder, upper, body}`** — a fold: cost accumulated over `upper` (a `SizeExpr`) iterations.

## 2. The one transform: time SUMS, space takes the MAX (sequential residency releases)

Peak working set is the time cost-expr read under **P1's residency algebra** (`space_measure_seq = max`, `space_measure_par = add`) instead of the time algebra (`seq = sum`, `par = max`) — the exact dual:

| structure | time (`work`) | **peak space** | why |
|---|---|---|---|
| `CostConst` (an atom) | its unit cost | the atom's **byte size** | one value resident |
| `CostAdd A B` (A then B, sequential) | A + B | **`CostMax(space A, space B)`** | A's frame releases before B |
| `CostSum{i, upper, body}` (a fold) | Σ over upper of body | **`CostAdd(output_size, CostMax over i of space body)`** | accumulator persists + ONE element+body live at a time (iterations release) |
| `CostMul A B` (nested loops) | A × B | **`CostAdd(space A, space B)`** if both live; `CostMax` if inner releases per outer step | nesting co-resides only where frames overlap |
| parallel region (span-carrying) | `CostMax` (span) | **`CostAdd`** (concurrent frames co-reside — P1 `space_measure_par`) | the P1 dual, exactly |

So `peak_space : CostExpr = space_of(work_structure)` — a fold over the same node tree, swapping the sum/max roles. The accumulator term reuses the already-derived `output_size`. Recursion depth is bounded by the **same `DescentEvidence`** that proves termination: `peak_space(recursion) = depth_bound × frame_size`, and `depth_bound` is the descent measure already computed. Time-bound and space-bound are two readings of one descent structure (§2: one concept, both derived).

## 3. The two readings the operator asked for

- **Asymptotic** — the polynomial degree of `peak_space`, reusing `std.induction`'s `CostBound = ConstantBound | AtomicBound | ForeverBound | ErrorBound` and `PolyCost{PolynomialExponent}`. `O(1)` = `peak_space` has no `SizeVar`; `O(n)` = degree 1; `O(n²)` = degree 2. **Free** — the same `derive_bound` the time side runs, on the space expr.
- **Concrete derived space** — evaluate `peak_space` at concrete inputs: substitute `SizeVar`/`SizeLen` with the closure's known collection lengths and each element type's byte width (the `MachineWidth`/`BitWidth` grounding already in `std`), fold the `CostExpr` arithmetic → a `ByteSize`. This fills `CostAccount.space` with a third, honest **`basis: Derived`** (neither `Measured` — the cop-out — nor authored `Predicted` — the pins the 2026-07-12 ruling killed). Witnesses are the first target: their closure is closed (data literals are known bytes, folds are bounded, recursion bounded by descent), so `peak_space` evaluates to an exact bound with no free variables.

## 4. Fail-closed staging (operator 2026-07-16 — counted frontier now, error later)

`peak_space` bottoms out at **`SpaceBoundUnknown{cause}`** — the space dual of `DescentUnknown`, a `BoundedLattice` bottom, fail-closed. Today it is **tolerated but counted** (typed, located, per-site — the `DecodeFidelity`/wrapper-retained frontier pattern): a function whose space we cannot yet derive carries a counted `SpaceBoundUnknown`, the corpus stays green, and the count is the prioritization signal. **Later** (declared trigger, not now) it ratchets to a hard error — an underivable bound refuses, the same wall `DescentUnknown` will become. The cause is typed (`MissingConcept | UnmodeledPrimitive | BoundDependsOnUndeclared`), so the frontier *locates* the modeling debt, never a graveyard (§6).

## 5. Integration points (where this lands)

- **`src/v1/complexity.dag`** — `ComplexitySummary` gains `peak_space: CostExpr`; `space_of(work)` derives it (the §2 transform); the concrete evaluator lowers it to `ByteSize`. Rebuild-gated (core analysis).
- **`CostAccount.space`** (`realization_schedule`) — filled from the concrete evaluation, `basis: Derived`. Closes audit F3 (the empty space carrier).
- **P4 `realize_pack`** — re-pointed: its input becomes the **derived bound**, not `MeasuredPeak`. `width = budget / derived_bound`, known up front. The `MeasuredPeak`/`PeakUnknown` types become `DerivedBound`/`SpaceBoundUnknown`; the refuse-not-fabricate discipline stays. Retires the reactive governor (the 2026-07-12 ruling's stated end-state).
- **Allocations known up front** — the derived per-witness bound *is* the up-front allocation; the slot model (`runner_slot_allocation`, declared 16 GiB × 5) is the coarse host-grain version, and the derived bound is the fine per-runnable version the ruling said "returns when derivable from the graph." They agree by construction: Σ derived-bounds ≤ budget is the same GUARANTEED-mode check at witness grain.

## 6. Sequence

1. **Witness-grain concrete space** (first shot — closed closures): `space_of` for the fold/atom/recursion shapes above, concrete evaluator, `basis: Derived` into `CostAccount.space`, witnesses proving exact bounds on closed fixtures + the `SpaceBoundUnknown` counted frontier with a RED control.
2. **Asymptotic space** — the `derive_bound` reading on `peak_space` (`O(n)` etc.), reusing the time-side machinery; witnesses on parameterized fixtures.
3. **Re-point P4** onto the derived bound; retire the measured-peak path (P1 → falsifier-only or dropped).
4. **Ratchet** `SpaceBoundUnknown` toward hard error once the derivation covers the corpus (declared trigger, operator).

Non-goals: measuring (skipped by design); authored per-runnable literals (the retired pins); a separate space vocabulary (it is `CostExpr` read under the residency algebra — one authority).
