# Accelerator-kernel demo round-trip — show the daglang moat in XLA's terms

**Status:** design draft (operator-directed tangent, 2026-06-29). Expansion-band showcase, not a stability-band gate.
**Displaced cost (the §6 deliverable):** a communicable, *runnable* artifact that proves the daglang architecture's power to an accelerator/array-compiler audience (NVIDIA / the XLA team) **in their own vocabulary** — fusion, layout assignment, buffer assignment, numerical-semantics preservation — without making them learn the substrate first. The pain it removes: "the architecture is powerful but I can't show it to a hardware/compiler audience in 60 seconds." This is a *small, honest subset* chosen because it is the easiest slice to communicate, not because it is the deepest.

This draft is downstream of [DESIGN.md](../../DESIGN.md) §4 (one grammar, both directions), §5 (fail-closed honesty boundary), §7 (medium-agnostic), and the [model↔realization fork](model-realization-fork.md) / [realization-measurement loop](realization-measurement-loop.md). It is **not** a claim that the compiler runs on a GPU — see §0.

---

## 0. What this is *not* (kill the over-claim up front)

- **NOT "run the resolver/interpreter on a GPU."** A tree-walking resolver maximizes *both* SIMT anti-patterns at once: control divergence (every `Node` variant is a different `match` arm) and memory divergence (every child is a separate `Rc<Node>` at an arbitrary heap address). That workload belongs on a multicore CPU work-stealing scheduler; the CI-wall thesis already establishes the real lever is incrementality + caching, which is latency-bound, not FLOP-bound. GPU does nothing for it.
- **NOT opcode-bucketing scattered scalars.** SIMT width must come from *data-parallelism inside a node* (one op over a contiguous array), not *task-parallelism across nodes* (gathering scattered scalar ops by opcode — the gather/scatter is itself the pointer-chasing you were trying to escape, and the arithmetic intensity is hopeless).
- **NOT a general "reshape arbitrary programs" control plane.** That is post-self-host research. This demo handles exactly one recognized class: **pure elementwise array folds.**

## 1. The claim the demo proves (load-bearing)

> A pure `.dag` elementwise-array subgraph can be lowered to a contiguous-layout, fused, data-parallel kernel; the result is **bit-identical to the scalar interpreter under a declared numerical contract**; and any subgraph outside the lowerable class is **refused** with a typed, located honesty diagnostic — never silently miscompiled.

Three things an XLA/NVIDIA audience treats as *hard* and this gets *structurally*:

1. **Fusion/layout as a verified, content-addressed, inspectable value with a free differential oracle.** In XLA, layout-assignment + fusion are C++ passes whose correctness is *tested*; silent fusion miscompiles are a known pain. Here the scalar interpreter **is** the spec, so the fused kernel is bit-identical-or-refuse by construction, and the lowering carries a `DecodeFidelity` boundary stating exactly where it is exact vs declines.
2. **A new backend is a *row*, not a backend team.** Medium-agnostic emit (§4/§7) means "support this accelerator" = author a target row in `extdeps/languages/<accel>/`, not fork a codegen. The optional GPU run is the proof: *same plan, one target row changed.*
3. **The schedule/layout is data, so auto-tuning can't miscompile.** The plan is a first-class `RealizationPlan<S>` value and every rewrite is provably semantics-preserving (purity + explicit `EffectShape`), so searching over layouts is guaranteed meaning-preserving.

## 2. On-model framing — first live consumer of the inert placement carriers

The substrate **already models** the vocabulary; the roadmap (Ergonomics LANE) flags `Placement` / `Materialization` / `RealizationObjective` as *inert (no live consumer)* — the keystone inert-abstraction-lens's first RED witness. This demo makes them **load-bearing** by being their first real consumer:

- `std/realization.dag`: `Placement = LocalInProcess | LocalFilesystem | RemoteNetwork`, `Materialization = Recompute | Memoize | Share`, `RealizedStep<S> { shape, placement, materialization }`, `DecodeFidelity`, `RealizationPlan`, `RealizationObjective`.
- `std/realization_schedule.dag`: `RealizationPlan<S>`, `Runnable`, `Schedule = List<List<Runnable>>`, cost accounting.
- `product/placement_supply.dag`: `PlacementSupplyRow` (host capacity) — literally "shaping for **supply**."

The demo's substrate delta is therefore **minimal** (net concepts must not grow by re-invention, DESIGN §2):

- **+1 `Placement` variant** for an accelerator/device-bound kernel (e.g. `LocalAccelerator`). A SIMT target is "one concept, every breadth" (§2-horizontal) — a new *placement*, not a new scheduler.
- **+1 `extdeps/languages/<accel>/` target row** for kernel emit (sits beside `rust`, `go`, `typescript`, `python`), consumed by the existing `target_model_edge_translation_rules` fold — **N rows, not N×M emitters** (§4).
- **+1 numerical contract** on the plan (rounding-mode / FMA-contraction policy) — the float-exactness honesty boundary (§5/§4 `DecodeFidelity`).

Everything else is *consumption* of existing carriers.

## 3. The round trip (one fixture, runs in <60s)

Fixture: a pure elementwise chain — `y = relu(a*b + c)` (mul → add → max). Three ops so the demo shows **fusion**, not a single kernel.

| Station | What it shows | Substrate |
|---|---|---|
| 1. source `.dag` → core graph | pure `Node`+`Edge`, `EffectShape = pure` — the precondition that *licenses* everything | `std/effects.dag`, ingest fold |
| 2. recognize + plan ("shaping for supply") | the elementwise-array-fold subgraph → a `RealizationPlan<S>` value: contiguous SoA buffers, 3 ops fused into one pass, `placement: LocalAccelerator`. **Inspectable, printed.** | `std/realization.dag`, `std/realization_schedule.dag` (first live consumer) |
| 3. lower → fused kernel (forward fold) | the *same* translate fold run backward → one fused contiguous-loop / SIMD kernel | `target_model` translate + new `extdeps/languages/<accel>` row |
| 4. differential vs oracle | scalar interpreter on same input = ground truth; assert match on a discriminating input | existing v2 interpreter |
| 5. fail-closed arm | feed a non-elementwise / effectful subgraph (data-dependent gather, or carries an effect) → **typed located `DecodeFidelity` refusal**, no silent fallback | `std/realization.dag` `DecodeFidelity` |

## 4. Acceptance bars (this is what makes it credible to *that* audience)

- **Integer fixture → bit-exact.** Unarguable; no rounding ambiguity. (Demo v1, primary witness.)
- **Float fixture → exact under a *declared numerical contract*.** An XLA person will immediately object "fusing `a*b+c` into an FMA changes the rounding — your bit-identity is a lie." We get ahead of it: the plan carries an explicit **numerical contract** (rounding mode / FMA-contraction allowed?), the differential holds *under that declared contract*, and a fusion that would change rounding is **refused unless the contract permits it.** This is the `--xla_allow_excess_precision` / contraction-flag headache made into a *typed property on the plan* instead of a global flag. **Single most credible artifact to this audience — include in v1.**
- **Refusal arm is non-vacuous.** The fail-closed station must fire on a real discriminating input (a subgraph that *looks* lowerable but isn't), not a strawman.

## 5. Do we need a real GPU?

**No — not for confidence, and say so in the room as a strength.** The novel claim is the *seam* (verified layout/fusion + free oracle + typed honesty boundary incl. numerics). The CPU-SIMD/contiguous-loop differential + the refusal arm + the inspectable plan **is** the substance. A GPU run adds zero evidence to that claim — it only demonstrates medium-agnosticism (claim #2 of §1).

- **Build the seam on CPU** (contiguous loop / SIMD). This is the architecturally interesting, fully-buildable-today part.
- **Keep "same plan, one target row → GPU" as an optional 60-second closer** on the operator's local device (SPIR-V via `wgpu`, or CUDA). Rhetorically powerful, epistemically a footnote — and *that framing itself* ("we don't need the GPU to trust the lowering — that's the point") is the pitch.

## 6. Sequencing — seam first, silicon last

1. **Modeling lane** — `+LocalAccelerator` placement variant; the recognizer's output as a `RealizationPlan<S>`; the `NumericalContract` type + its `DecodeFidelity` coupling; the refusal diagnostic. All in `std/` + `extdeps/languages/<accel>/`. Lands consumed-or-marked (Ergonomics-lane "wire the seams" rule).
2. **Execution lane** — the recognizer pass (elementwise-array-fold detection over the core graph) + the SoA fused-kernel handler (CPU contiguous loop) in the v2 interpreter; the differential harness; the **integer bit-exact witness** + the **refusal witness**, green-by-execution with a red-on-revert discriminator.
3. **Numerical-contract bar** — float fixture under declared contract; refuse-on-contraction-violation witness.
4. **GPU epilogue (optional, staged)** — swap the target row to a GPU backend on the operator's device; same bit-identity bar.

## 7. Honesty line (prototype vs aspirational)

- **Real today** (current v2 Rust interpreter, pre-self-host): the recognizer, the SoA fused-kernel handler, the differential + numerical-contract harness, the refusal arm, integer-bit-exact + float-under-contract bars. A runnable prototype, honestly labeled.
- **Aspirational / post-self-host**: the plan being *fully* substrate-native end-to-end, and reshaping *arbitrary* programs (vs the recognized elementwise class). For the demo the plan is a real modeled value even where some lowering is Rust-side; each prototype-Rust seam is marked so the pitch never overclaims.

## Open threads

- Exact spelling of the accelerator `Placement` variant (device-parameterized `LocalAccelerator<Device>` vs flat) — DFS the concept DAG before minting; a device is plausibly a `Vendor<Hardware>`-adjacent entity (DESIGN §3), not a fresh enum.
- Whether the numerical contract is a new std type or a refinement of an existing `float.dag` / `approximate_field.dag` carrier (reuse-first).
- Does the recognizer belong as a lens (read-only over the `Node` tree) or a translate-stage pass? (Lens-first per §6, unless it must produce the plan as part of lowering.)
