# The Bootstrap as a Complete Fact Model

> Status: design — the Phase-1 design content for **TASKS.md T-32**
> ("minimum never-hand-edited bootstrap seed"). Operator-directed
> (briansrls, 2026-05-17). Extends the self-hosting story in
> `src/v3/SELF_HOSTING.md` and `docs/design-pure-bootstrap-zero.md`.

## 1. Principle

The bootstrap is **not** ambient code with a modeled compiler bolted on.
Every layer — the compiler, the seed, the seed's target language, the
runtime, the machine — is a **declared fact model**. Every executable
artifact is a **projection** (an emission) from those models.

The end state has **zero ambient hand-maintained code**. When something
changes, you update a *fact* in a model; the affected artifacts
re-project. You never hand-edit a generated artifact — there is no
generated artifact you *can* hand-edit, because each is the output of an
emission you re-run.

This is the self-hosting thesis followed all the way down. "The Rust
seed shrinks toward zero" does not only mean "less hand-Rust over time";
its end state is that **even the seed is a projection**, not hand-code.

## 2. The layer stack

Each layer is a fact model (declared `.dag` data), not code:

(All paths repo-root-relative.)

| Layer | Fact model | Status |
|-------|-----------|--------|
| v4 source language | the `LanguageModel` / grammar-as-data (`src/v4/extdeps/languages/dag.dag`) | modeled |
| The v4 compiler | the `src/v4/compiler/*.dag` pipeline (tokenize…emit) | modeled |
| The seed's comprehension boundary | the **frozen sub-model** — a subset of the `LanguageModel` (§4) | T-32 |
| The snapshot | the v4 compiler pinned at version N, expressed in the frozen subset | T-32 |
| The target language | `src/v4/extdeps/languages/rust.dag`; lower, `src/v4/extdeps/languages/machine_code.dag` | modeled |
| The runtime | the execution substrate — syscall surface, memory model, ABI; `src/v4/extdeps/process.dag` + `src/v4/extdeps/file_system.dag` are its start | partial |
| The bootstrap orchestration | `src/v4/workflow/bootstrap.dag` — the staged chain | T-20 scaffold on `main`; staged-chain expansion in flight (#3213) |

The layer models needed already exist individually (`rust.dag`,
`machine_code.dag`, `process.dag`, `file_system.dag`). What T-32 adds is
**composing** them so the seed is their joint projection.

## 3. The seed is a projection

The seed is not a hand-written compiler. It is:

```
seed  =  emit(snapshot, target_model, runtime_model)
```

— generated, then **frozen**. It is never hand-edited. When the seed
must change, you change a *fact* in one of those three models and
re-emit:

- the **runtime model** gains a versioned delta (a new OS syscall, an
  ABI change) — host drift becomes a modeled fact, not a hand-patch;
- the **frozen sub-model** gains a construct (§4) — a deliberate,
  ratified extension;
- the **target model** changes — a re-pin of the emission target.

"Host drift" — the one genuinely-ongoing risk named in the T-32
analysis — stops being a hand-edit to ambient seed code; it is "the
syscall table changed → update that fact → re-project the seed."

## 4. The comprehension boundary, modeled

"What the seed can comprehend" must not be implicit in the seed's code,
discovered only when the seed fails. It is modeled:

- **`seed_capability`** — the **frozen sub-model**: a declared subset of
  the v4 `LanguageModel` (a subset of grammar productions + `Node`
  connectives/behaviors + the lowering the seed performs). The seed is
  the **generic walker over that sub-model** (the B2-OMNI principle —
  not a hand-written parser). "What the seed comprehends" *is* the
  sub-model, by construction — readable data, not code-archaeology.
- **`footprint(snapshot)`** — a **fold over the snapshot's `Node`
  tree** collecting every construct it uses. A pure lens.
- **The gate** — `Witness< footprint(snapshot) ⊆ seed_capability >`:
  a decidable subset relation over two finite sets of declared
  constructs. It runs *without running the seed*, and on failure names
  the *specific construct* that escaped the subset.

This answers two otherwise-opaque questions structurally:

- **Where is the boundary?** — it is the `seed_capability` sub-model.
  You read it.
- **When must the seed extend?** — exactly when that `Witness` fails.
  Extension stops being "it broke" and becomes a diagnosed decision:
  rewrite the snapshot without the escaping construct, or ratify
  extending the frozen sub-model (a seed change).

**Minimum seed** is then measurable: the minimum frozen subset *is*
`footprint(snapshot)` — the seed must comprehend exactly what the
snapshot uses, nothing more. "Is the seed minimal?" = "is
`seed_capability == footprint(snapshot)`?" — checkable. The seed shrinks
by writing the v4 compiler frugally; the footprint fold measures it.

## 5. The bootstrap circularity, modeled

The circularity — *you need a compiler to produce the seed; the seed
produces the compiler* — is **not** an embarrassing chicken-and-egg. It
is a **fixed point**, and modeling it means declaring that fixed point
explicitly.

Let `S_C` be the v4 compiler's own source (a v4 program — the
`src/v4/compiler/*.dag` pipeline). Let `compile_X` be compilation performed by
compiler `X`. The self-hosting compiler is the fixed point:

```
C*   such that   compile_{C*}(S_C) = C*
```

— the compiler that compiles its own source to a bit-identical copy of
itself.

The circularity is resolved by a **convergence ladder**, entered via the
seed:

```
seed  →  C0 = compile_seed(S_C)
         C1 = compile_{C0}(S_C)
         C2 = compile_{C1}(S_C)
         …  →  Cn = compile_{Cn}(S_C) = Cn      (fixed point reached)
```

`src/v4/workflow/bootstrap.dag` is where this is carried. On `main` it
is a T-20 scaffold — its header names the staged `seed → stage0 →
stage1 → stage2` chain in prose; the T-20 expansion (in flight on
#3213) makes the staged-stage records, `FixptStage1Stage2`, and the
`bit_identical_check` actual workflow data. This Phase-1 model specifies
that shape; it is not yet all carried as `.dag` data on `main`. The
check `Cn(S_C) == Cn` is the **witness** that the fixed point is reached.

**Path-independence — the key fact.** The fixed point `C*` is a
mathematical object. *How you first stepped onto the ladder does not
matter.* Once `compile_{Cn}(S_C) == Cn` holds bit-identically, you *are*
`C*`, and `C*` validates itself — it is the compiler whose own source
compiles to it. The seed is merely the *entry point* onto the ladder; it
need not be `C*`, only close enough to converge to it.

The modeled circularity therefore has four declared parts:

1. **the fixed-point equation** `C* = compile_{C*}(S_C)` — the self-hosting fact;
2. **the convergence ladder** `seed → C0 → … → Cn` — the staged chain (`src/v4/workflow/bootstrap.dag`);
3. **the seed as entry point** — `emit(snapshot, …)`, §3;
4. **the bit-identical witness** — `Cn(S_C) == Cn`, the proof the fixed point is reached.

## 6. The honest floors

A modeled system always bottoms out somewhere. Naming the floors
precisely *is* the comfort: there are exactly two, and only one is
permanent.

**The physical axiom (permanent).** At the very bottom, a real CPU
executes real bits. You can model down through ISA → ABI → syscall
table — all finite, intersubjective specs; `machine_code.dag` *is* this
layer — but the model bottoms out at *"the physical machine executes
this ISA as the model says."* This is irreducible. It is also the
**right** floor: a CPU ISA is the most-scrutinized, most-stable
intersubjective artifact available, exactly the grounding INVARIANTS P1
asks for. The system does not trust nothing; it pushes trust down to one
well-specified physical fact and models everything above it.

**The origin "axiom" (dissolves).** The *first* seed was produced by
something outside the modeled system — historically the v2 compiler /
hand-bootstrap. This looks like a second axiom, but **path-independence
(§5) dissolves it**: once the fixed point is witnessed, how the ladder
was first entered is irrelevant — the fixed point is self-validating.
The origin is a *historical fact* (a specific commit produced the first
seed), not a *permanent axiom*. It is precisely the v2 / ambient code
that "shrinks toward zero": once v4 can emit its own seed and the
fixed-point witness holds, v2 is deleted and the system has no
un-modeled component except the physical axiom.

So the end state is: the fact models, the projections from them, and
**one** permanent axiom — the silicon matches the machine model.

## 7. What this means for T-32

T-32 is larger than "minimize the seed." Its Phase-1 deliverable is the
**layer model** itself — name every layer, every fact-model, every
projection edge, the comprehension gate, the modeled circularity, and
the named physical axiom. "Minimum never-hand-edited seed" then *falls
out*: minimum, because the projection is no larger than the snapshot's
footprint allows (§4); never-hand-edited, because it is a *projection at
all* — you edit facts, and the seed re-emits.

The `.dag` modeling extends from here: the frozen sub-model + the
`footprint` lens + the `⊆`-`Witness` (§4) are new substrate; the
circularity's fixed-point equation + witness (§5) extend
`src/v4/workflow/bootstrap.dag` beyond the staged chain (building on the
T-20 expansion); the runtime model (§2) is the accumulation of
`src/v4/extdeps/process.dag` / `src/v4/extdeps/file_system.dag` into a
complete execution substrate. None of it dispatches before the operator
ratifies this layer model as the Phase-1 definition.
