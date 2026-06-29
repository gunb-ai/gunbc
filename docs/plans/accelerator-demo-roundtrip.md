# Accelerator-kernel demo round-trip — show the daglang moat in XLA's terms

**Status:** design draft (operator-directed tangent, 2026-06-29). Expansion-band showcase, not a stability-band gate.
**Displaced cost (the §6 deliverable):** a communicable, *runnable* artifact that proves the daglang architecture's power to an accelerator/array-compiler audience (NVIDIA / the XLA team) **in their own vocabulary** — fusion, layout assignment, buffer assignment, numerical-semantics preservation — without making them learn the substrate first. The pain it removes: "the architecture is powerful but I can't show it to a hardware/compiler audience in 60 seconds." This is a *small, honest subset* chosen because it is the easiest slice to communicate, not because it is the deepest.

This draft is downstream of [DESIGN.md](../../DESIGN.md) §4 (one grammar, both directions), §5 (fail-closed honesty boundary), §7 (medium-agnostic), and the [model↔realization fork](model-realization-fork.md) / [realization-measurement loop](realization-measurement-loop.md). It is **not** a claim that the compiler runs on a GPU — see §0.

---

## 0. What this is *not* (kill the over-claim up front)

- **NOT "run the resolver/interpreter on a GPU."** A tree-walking resolver maximizes *both* SIMT anti-patterns at once: control divergence (every `Node` variant is a different `match` arm) and memory divergence (every child is a separate `Rc<Node>` at an arbitrary heap address). That workload belongs on a multicore CPU work-stealing scheduler; the CI-wall thesis already establishes the real lever is incrementality + caching, which is latency-bound, not FLOP-bound. GPU does nothing for it.
- **NOT opcode-bucketing scattered scalars.** SIMT width must come from *data-parallelism inside a node* (one op over a contiguous array), not *task-parallelism across nodes* (gathering scattered scalar ops by opcode — the gather/scatter is itself the pointer-chasing you were trying to escape, and the arithmetic intensity is hopeless).
- **NOT a general "reshape arbitrary programs" control plane.** That is post-self-host research. This demo handles exactly one recognized class: **pure elementwise array folds.**

## 0.5. Intuition — how your programs relate to GPU execution (the shared language)

The phrase to hold onto is **"arbitrary program DAG → numerical graph."** Your program is already a dependency graph; the accelerator wants a *particular kind* of dependency graph. The demo is the bridge that finds the second kind inside the first.

**"DAG" alone does not pick the hardware — the *nodes* do.** Two programs can both be pure dependency graphs and belong on opposite chips:

- A **task DAG** — coarse, *heterogeneous* nodes ("resolve this", "fold that"), wired by dependencies, scheduled dynamically. This is what a build system (Bazel) or a compiler pipeline is. → belongs on a **multicore CPU**: a handful of big, different jobs running in parallel across cores.
- A **dataflow / numerical DAG** — fine, *homogeneous* nodes (one arithmetic op applied across a whole array), static structure, no branching. This is what an ML graph (XLA's HLO, a tensor program) is. → belongs on a **GPU**.

So the mental model for the two chips:

| | CPU (MIMD) | GPU (SIMT) |
|---|---|---|
| What it wants | any mix of different instructions on demand | **one** instruction over a **big contiguous array** of data |
| Strength | heterogeneous, branchy, irregular work | thousands of identical, branch-free arithmetic ops |
| The bet | big caches + speculation absorb unpredictability | march thousands of lanes in lockstep, no caches needed |

A GPU is essentially a **batch executor for arithmetic**: if at one moment you can hand it *one* operation over a million contiguous data elements, it's ideal; if you hand it a grab-bag of different operations on scattered data, it falls apart. Two things break the lockstep and collapse it back toward serial: **branches** (lanes want different instructions) and **scattered memory** (lanes want different addresses). That's why a tree-walking interpreter is the *worst* case — every node is a different `match` arm (branches) and every child is a random heap pointer (scattered memory). It maxes out both.

**Where the GPU-friendly graph hides in your program.** The crucial move is that SIMT width comes from **data-parallelism *inside* a node** (one `map (+)` over a 10⁶-element array), **not** task-parallelism *across* nodes (gathering a million scattered scalar adds — the gathering is itself the scattered-memory problem you were trying to escape). So you don't "send the whole program to the GPU." You **recognize the subgraph that is already a numerical graph** — a chain of pure, elementwise, array-shaped ops like `relu(a*b + c)` — and lower *that* to a kernel. The rest of the program stays on the CPU. The accelerator gets the array math; the orchestration stays where orchestration belongs.

**"Shaping for supply" — why daglang can do this and a normal language can't.** Whether your array data sits scattered across the heap or packed in one contiguous buffer is **not physics — it's a layout decision the compiler makes.** Today the naive choice (scattered) is what makes pointer-chasing look mandatory. Because the `.dag` *owns* lowering, it can instead *choose* a contiguous layout that turns latent-but-hidden data-parallelism into the coalesced, lockstep-friendly form the GPU wants. And because `.dag` programs are **pure with explicit effects**, that relayout is *provably* meaning-preserving — a normal language can't prove it safe because hidden side-effects might depend on the old order. That provable freedom to reshape layout is the moat; the GPU kernel is just the payoff.

**The honest bound.** This reveals data-parallelism that layout was *hiding*; it cannot *manufacture* parallelism your data dependencies forbid. A serial chain (each step needs the last) has a hard floor — its critical path — that no layout beats. So the demo deliberately targets the one place the win is real (pure elementwise array folds) and is fail-closed everywhere else.

So the one-sentence relationship: **your arbitrary program is a task DAG; inside it live numerical subgraphs; the demo recognizes one, chooses a layout that makes it SIMT-shaped, and lowers it — proving the recognition is semantics-exact or honestly refused.**

## 1. The claim the demo proves (load-bearing)

> A pure `.dag` elementwise-array subgraph can be lowered to a contiguous-layout, fused, data-parallel kernel; the result is **bit-identical to the scalar interpreter under a declared numerical contract**; and any subgraph outside the lowerable class is **refused** with a typed, located honesty diagnostic — never silently miscompiled.

Three things an XLA/NVIDIA audience treats as *hard* and this gets *structurally*:

1. **Fusion/layout as a verified, content-addressed, inspectable value with a free differential oracle.** In XLA, layout-assignment + fusion are C++ passes whose correctness is *tested*; silent fusion miscompiles are a known pain. Here the scalar interpreter **is** the spec, so the fused kernel is bit-identical-or-refuse by construction, and the lowering carries a `DecodeFidelity` boundary stating exactly where it is exact vs declines.
2. **A new backend is a *row*, not a backend team.** Medium-agnostic emit (§4/§7) means "support this accelerator" = author a target row in `extdeps/languages/<accel>/`, not fork a codegen. The optional GPU run is the proof: *same plan, one target row changed.*
3. **The schedule/layout is data, so auto-tuning can't miscompile.** The plan is a first-class `RealizationPlan<S>` value and every rewrite is provably semantics-preserving (purity + explicit `EffectShape`), so searching over layouts is guaranteed meaning-preserving.

## 1.5. The relational projection — the demo artifact that is *for the author* (and doubles as the pitch)

A first-class deliverable, not a debug print: the demo must **render the relationship** "arbitrary program DAG → numerical graph → kernel" as an inspectable artifact, because making that relationship legible is half the point (to the author, who thinks in array compilers; and to the audience, who thinks in HLO). The **recognize + plan** station's `RealizationPlan` value is *promoted from a side-effect to the product.*

Shape — a **triptych** over one input program:

| LEFT — your program | MIDDLE — the numerical graph inside it | RIGHT — the lowering |
|---|---|---|
| the program as written (the full **task DAG**) | the pure elementwise-array subgraph **lifted out**, shown as a **dataflow graph** — literally "arbitrary dag → numerical graph", the HLO-shaped view | the layout/fusion plan (SoA buffers, fused ops, `placement: LocalAccelerator`) + the **fidelity verdict** (Lossless / Lossy / Refused) |

Why this is on-model, not scope creep:

- It's a **lens** — read-only over the `Node` tree + the `RealizationPlan` value, storing nothing (DESIGN §6). A new analysis costs zero substrate edits.
- It elevates the project's own pitch point ("the schedule/layout is *data*, so it's inspectable and can't silently miscompile") from a claim into the **rendered artifact** that demonstrates it.
- Rendering is **"one more medium"** (§4/§7): a text / DOT structural projection first (provable, in-substrate), an HTML/React visual on top — *the same projected data, two media.* This **converges with §7** (the website/React rendering lane): the triptych is a natural candidate for the "demo beside the TS emit" milestone.

Acceptance: the projection is *derived* from the same `RealizationPlan` the kernel is lowered from (single authority — the picture cannot drift from what actually executes), and the MIDDLE→RIGHT carving shows the exact subgraph that was lowered, with the refusal arm rendering a located refusal rather than a blank.

## 2. On-model framing — first live consumer of the inert placement carriers

The substrate **already models** the vocabulary; the roadmap (Ergonomics LANE) flags **`Placement` / `Materialization`** as *still inert (no live consumer)* — part of the keystone inert-abstraction-lens's RED set. (`RealizationObjective` / `realization_width` are **already wired** via `ci_floor_plan` — `inert_layer_lens.dag` records the schedule/width arm as live; do **not** treat that arm as greenfield.) This demo makes the *placement/materialization* arm **load-bearing** by being its first real consumer:

- `std/realization.dag`: `Placement = LocalInProcess | LocalFilesystem | RemoteNetwork`, `Materialization = Recompute | Memoize | Share`, `RealizedStep<S> { shape, placement, materialization }`, `RealizationPlan`, `RealizationObjective`.
- `extdeps/communication/medium.dag`: `DecodeFidelity = Lossless | Lossy` (the honesty-boundary type — declared here, an extdeps-layer type, *not* in std; consume it, never re-fork it).
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
| 5. fail-closed arm | feed a non-elementwise / effectful subgraph (data-dependent gather, or carries an effect) → **typed located `DecodeFidelity` refusal**, no silent fallback | `extdeps/communication/medium.dag` `DecodeFidelity` |

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

**Hand-Rust scaffold receipt (§7 self-host — Rust shrinks to zero, so new Rust must name its exit).** The execution lane's three Rust additions on the v2-interpreter seed — the elementwise recognizer pass, the SoA fused-kernel handler, and the differential harness — are **explicitly scaffold, not load-bearing Rust.** Each lands with a named **dissolution trigger** and is tracked by the `accel-exec` roadmap row (owner `tidy-deer-560`, folded into #5968), which does not flip `[x]` until the dissolution is either done or re-deferred with a reason:

  - *recognizer* → dissolves into a `.dag` **lens** (read-only over the `Node` tree) once the lens can express the elementwise-fold predicate; dissolution trigger `feature:dag-elementwise-recognizer-lens`.
  - *SoA fused-kernel handler* → dissolves into a `.dag` **Realization handler** bound to the `extdeps/languages/<accel>` target row (emit is the same fold, both directions); trigger `feature:dag-kernel-realization-handler`.
  - *differential harness* → dissolves into a `.dag` **witness** (the scalar interpreter is already the in-substrate oracle); trigger `feature:dag-differential-witness`.

  Until then they are `🟡`-marked prototype Rust on the doomed seed — they add **no** census ratchet pressure and must not be cemented into emit templates (DESIGN §7; the anti-cement rule). Net Rust delta is bounded and has a written path to zero; it is not new permanent compiler surface.

## Open threads

- Exact spelling of the accelerator `Placement` variant (device-parameterized `LocalAccelerator<Device>` vs flat) — DFS the concept DAG before minting; a device is plausibly a `Vendor<Hardware>`-adjacent entity (DESIGN §3), not a fresh enum.
- Whether the numerical contract is a new std type or a refinement of an existing `float.dag` / `approximate_field.dag` carrier (reuse-first).
- Does the recognizer belong as a lens (read-only over the `Node` tree) or a translate-stage pass? (Lens-first per §6, unless it must produce the plan as part of lowering.)
