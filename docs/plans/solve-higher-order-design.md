# `solve` is higher-order, not a primitive — the standing rationale

**Status:** design ruling, captured pre-un-shelve (the language-target self-host lane is §4-shelved until soon after the Rust fixed point). This doc is a **scaffold**: at un-shelve its authority migrates onto the std carrier that inhabits `solve` (the `Residual`/`Constraint` type's own doc-rows) plus a DESIGN §4 anchor, and this file dissolves. Until then it is the single place the rationale lives, so it is not re-litigated or misplaced.

**One-line ruling:** `solve` is **not** a substrate primitive. It is a higher-order operation over the existing closed vocabulary — a bounded `Loop` (fixed-point iteration) whose body evaluates a *residual* of an existing model, bounded by `DescentEvidence`, with the numerical inner-step as a realization handler in `extdeps/`. Minting a `solve` primitive would violate three axioms at once. The higher-order form is not merely *adequate* — the primitive form is a *modeling error*.

---

## Why — forced by the axioms, not chosen

### A closed system ⇒ a heuristic is never necessary (§4) ⇒ a black-box `solve` is forbidden

DESIGN §4: "in a closed system a heuristic is never necessary — the richer source always exists or can be written," and §2: "nothing is opaque that isn't *genuinely* atomic." A `solve` primitive is an opaque box that hides an iterative method behind a name. That is precisely the anemic-leaf pattern (`decompress → map → reduce` un-run) at the substrate level. The richer source *does* exist — the iteration, the residual, the convergence test are each nameable and grounded — so the box is never necessary.

### Bounded-forward execution ⇒ `solve` is the inverse/fixpoint *reading*, encoded acyclically (§4)

On its face `solve` fights "bounded and forward … never cyclic values." It doesn't, because the substrate already resolves that tension two ways, and `solve` reuses both:

- **"cyclic relations via acyclic encodings."** A constraint (a circuit's KCL node-equation `Σi = 0`, a sequential-logic fixed point) is *mutual/cyclic*. It is never represented as a cyclic value — it is encoded as an **acyclic residual function** (given a candidate assignment, forward-evaluate "how far off am I") plus a **`Loop`** that iterates the residual toward zero. Recursion is already sugar over `Loop`; a Newton / time-stepping / relaxation solver *is* that `Loop`.
- **"emission, ingestion, and coercion are one decision procedure run in different directions."** `solve` is one more direction of that same procedure — the *fixed-point reading*. The "find the thing that satisfies" machinery already exists and is load-bearing: **`find_witness`** (coercion's homomorphism search). `solve` points `find_witness` at equalities instead of type-inhabitance.

### Single authority ⇒ a `solve` primitive nicknames what already exists (§3)

`Loop` + `find_witness` + `DescentEvidence` already compose to "iterate a residual to a fixed point under a bound." A `solve` primitive is a **second name** for that composition — the §3 nicknaming violation, at the most expensive layer to fork (the substrate). We generate from concepts; a substrate nickname duplicates into every derived thing (emit, testgen, lenses).

---

## The decomposition — every ingredient already exists

```
solve  ≜  Loop(
            body:        step( residual(model, candidate) ),   // forward fold — HAVE
            bound:       DescentEvidence,                       // Converged | Refuse — HAVE
            on_exhaust:  Refuse{ iters, residual }              // §5 fail-closed — HAVE
          )
```

| piece | what it is | already in tree as |
| --- | --- | --- |
| **residual** | a *forward fold over an existing model*, read as "these must be consistent" rather than "evaluate forward" | `find_witness` / `coercion_fold` (§4), pointed at equality targets |
| **iteration** | the fixed-point loop | `Loop` behavior (recursion desugars to it) |
| **convergence bound** | `Converged` \| `Strict`-descent \| refuse-on-exhaust | `DescentEvidence` on `BoundedLattice`, reused verbatim (`dag/std/termination.dag`) |
| **numerical inner-step** | LU-solve, ODE integrator, relaxation kernel | a **realization handler** in `extdeps/` — the §2/§3 transport pattern (one agnostic shape, N handlers: LAPACK / ngspice / pure-dag Newton) |

The only part that is *not* pure substrate is the numerical kernel — and that is not a substrate concern at all. It is a transport handler bound to an agnostic shape, exactly like `cc` compiling or `ngspice` simulating. Dispatch over handlers lives in `extdeps/`, never in the interface (§3).

---

## The proof it is sufficient: **the compiler already solves, everywhere, and no solve is a primitive**

This is the decisive point. `solve` is not a proposal to add — it is already load-bearing in the tree, in higher-order form:

- **Type unification** is a constraint solve.
- **The resolver's fixed point** is a solve.
- **Affected-set closure** is a solve (iterate the edge relation to a stable fixpoint — the `|E|`-sweep).
- **Coercion** (`coercion_fold` via `find_witness`) is "find the assignment that makes one model inhabit another" — a solve.

Every one is a bounded fold / `Loop` with a descent bound. None is a primitive. So the higher-order form is proven by existence: the compiler that would *host* a `solve` primitive is itself already built out of solves that need no such primitive.

---

## The honest residue — what genuinely-new modeling *is* needed (and it is small)

Not a primitive, and not a new connective or behavior. Exactly two things, both resting on existing behaviors:

1. **One std type** — `Residual` / `Constraint`: wraps a model, names its free variables, and names its equality/consistency target. This is the "read a model as a constraint" carrier. It rests on `find_witness`.
2. **One higher-order fold** — `solve` over that type, producing `Loop` + `DescentEvidence` structure, delegating the inner numerical step to a bound realization handler.

Zero new connectives. Zero new behaviors. One std authority, one fold.

---

## The load-bearing fail-closed property (do not lose this)

A non-converging solve **refuses** — a typed, located diagnostic carrying iteration count and final residual. It **never** fabricates a plausible answer (a "return best-so-far", a silent clamp, a default). That is the §5 absorbing-fallback trap: substituting ⊤-as-answer for ⊤-as-ignorance, its frequency zeroed by construction so the deficit never ranks for fixing. A solver that degrades to keep going is not being cautious — it is a fail-open one config edit from silent. The failure arm must refuse, never widen. This is *why* the convergence bound is `DescentEvidence` (which already fails closed to `DescentUnknown` = refuse) and not a tolerance flag.

---

## Placement: `solve` is **off the critical path** for the language lane

Important for the dependency map: the exotic analog/HDL targets that motivated this question **do not need `solve` in the compiler**. SPICE and Verilog *emit* a netlist / RTL, and a **host simulator** (`ngspice`, `verilator`, `iverilog`) performs the solve. So there the solver is a `run_ngspice`-style **transport handler** — the same `host_tool_program_name` registry gap that C Phase 0 hit (only `cargo`/`cc` registered) — not a compiler-internal operation.

`solve`-as-first-class-substrate only bites if the *compiler itself* must solve internally. And there it is already done, higher-order (unification / resolve / affected-set). So `solve` is a foundation item **F6** that the language targets can proceed without; it is authored when a first-class in-substrate solver earns its keep (e.g. a pure-dag SPICE realization that does not shell out to ngspice), priced by that displaced cost — not before.

## Dissolution trigger (DESIGN §6)

This doc dissolves when the lane un-shelves and the `Residual`/`Constraint` std carrier is authored: the ruling above migrates onto that carrier's typed doc-rows (`.dag` has no comment trivia — encode as `data … : String`) and a DESIGN §4 anchor, and this file is deleted. Until then, this is the single authority for the ruling.
