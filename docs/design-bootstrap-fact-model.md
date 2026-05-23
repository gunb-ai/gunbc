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

This is `modeling-discipline.md` **Practice 10 ("Don't hand-roll a
derived operation")** applied at bootstrap scale: the seed is produced by
*translation* and *emission* — derived operations (registry rows 3 and
6) — so a hand-written seed is the bootstrap-scale walker/template
dissolution finding. T-32 is the standing instance of that Practice.

## 2. The layer stack

Each layer is a fact model (declared `.dag` data), not code:

(All paths repo-root-relative.)

| Layer | Fact model | Status |
|-------|-----------|--------|
| v4 source language | the grammar-as-data for `.dag` (`src/v4/extdeps/languages/dag.dag`). On `main` `dag.dag` is an **admitted scaffold** — its `lex` / `grammar` `Node` trees are zero-production `Conj` roots (`Status: admitted; scaffold — fill per T-4 + T-6/T-7`): the carrier shape is admitted, the productions are not yet landed. The `LanguageModel` *carrier type* it instances is **not yet a declared authority** either — an open substrate item (Theme-A #9: `00_compile.dag` is parameterized over "declarative LanguageModels" but no `type LanguageModel` is declared; T-6/T-10 either declare the carrier or formally state "a model IS a `Node`"). | `dag.dag` admitted as a scaffold (zero-production lex/grammar, fill T-6/T-7); `LanguageModel` carrier = open (Theme-A #9) |
| The v4 compiler | the `src/v4/compiler/*.dag` pipeline (tokenize…emit) | front-end modeled (`01_tokenize.dag` / `02_parse.dag`, CP-1a #3214); `00_compile` / `03_normalize` / `03_resolve` / `04_infer` / `05_emit` / `05_eval` are T-8/T-9/T-10/T-22 scaffolds |
| The seed's comprehension boundary | the **frozen sub-model** — a subset of the `LanguageModel` (§4) | T-32 |
| The snapshot | the v4 compiler pinned at version N, expressed in the frozen subset | T-32 |
| The target language | `src/v4/extdeps/languages/rust.dag`; lower, `src/v4/extdeps/languages/machine_code.dag` | `rust.dag` modeled (T-4 rust-slice); `machine_code.dag` is a T-4.13 scaffold |
| The runtime | the execution substrate — syscall surface, memory model, ABI; `src/v4/extdeps/posix.dag` + `src/v4/extdeps/file_system.dag` are its start | both T-4.5 scaffolds (module declaration only) |
| The bootstrap orchestration | `src/v4/workflow/bootstrap.dag` — the staged chain | T-20 scaffold on `main`; staged-chain expansion in flight (#3213) |

Of the layer models the seed projects from, only `rust.dag` is modeled
today (the T-4 rust-slice); `machine_code.dag`, `posix.dag`, and
`file_system.dag` are scaffolds (T-4.13 / T-4.5). T-32's deliverable is
two-fold: the *layer model* — the composition itself, the projection
edges, the gate — and a precondition that those three scaffold files
reach modeled state. The seed is their joint projection; T-32 specifies
the joint, and depends on the per-file modeling landing.

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
  *Prerequisite:* this presumes the `LanguageModel` carrier itself is a
  declared authority — it is not yet (Theme-A #9, §2). Pinning that
  carrier (or formally fixing "a model IS a `Node`") is **new substrate
  T-32 Phase 1 must land before the frozen sub-model can be declared** —
  it is not an existing authority to point at.
- **`footprint`** — a **fold over the seed's projection inputs**
  `{snapshot, target_model, runtime_model}` (§3) collecting every
  construct the seed must consume: the source-language constructs the
  snapshot uses, *and* the lowering/runtime constructs its emission
  toward `target_model` / `runtime_model` touches. A pure lens. Folding
  over the snapshot alone undercounts — `seed_capability` includes "the
  lowering the seed performs" (bullet above), so `footprint` must span
  that same closure for the gate's `==` to be honest.
- **The gate** — `Witness< footprint ⊆ seed_capability >`:
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
`footprint` — the seed must comprehend exactly the constructs its three
projection inputs use, nothing more. "Is the seed minimal?" = "is
`seed_capability == footprint`?" — checkable. The seed shrinks
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

**What the bit-identical witness proves — and what it does not.**
`compile_{Cn}(S_C) == Cn` proves the fixed point is **reached**: the
compiler *reproduces itself*, stable under self-compilation. The seed is
the *entry point* onto the ladder — it need not be `C*`, only close
enough to converge. Two separate facts must not be conflated here:

- **Reproduction (what the witness proves).** Path-independence holds —
  *for benign entry paths*: two **honest** seeds, entering differently,
  converge to the same `C*` (`S_C` deterministically specifies the
  compiler, so any correct compiler compiling it yields the same
  output). Which honest path you took does not matter.
- **Honesty (what the witness does NOT prove).** Bit-identical
  reproduction is **not** evidence the seed was honest. A compromised
  seed — Thompson's "trusting trust" — reaches a *compromised* fixed
  point: a backdoor that re-inserts itself on every self-compile makes
  `compile_{Cn}(S_C) == Cn` hold bit-identically *with the backdoor
  intact*. The source `S_C` is clean; the backdoor lives in the compile
  step. The witness proves reproduction, never honesty.

The modeled circularity therefore has four declared parts:

1. **the fixed-point equation** `C* = compile_{C*}(S_C)` — the self-hosting fact;
2. **the convergence ladder** `seed → C0 → … → Cn` — the staged chain (`src/v4/workflow/bootstrap.dag`);
3. **the seed as entry point** — `emit(snapshot, …)`, §3;
4. **the bit-identical witness** — `Cn(S_C) == Cn`, the proof the fixed point is *reached* (reproduction). The *honesty* of the seed is a separate fact — §6.

## 6. The honest floors

A modeled system always bottoms out somewhere. Naming the floors
precisely *is* the comfort: there are **two** — the physical axiom
(permanent and irreducible) and the seed-honesty axiom (a real trust
assumption — *discharged* by an independent witness, not dissolved).

**The physical axiom (permanent).** At the very bottom, a real CPU
executes real bits. You can model down through ISA → ABI → syscall
table — all finite, intersubjective specs; `machine_code.dag` *is* this
layer — but the model bottoms out at *"the physical machine executes
this ISA as the model says."* This is irreducible. It is also the
**right** floor: a CPU ISA is the most-scrutinized, most-stable
intersubjective artifact available, exactly the grounding INVARIANTS P1
asks for. The system does not trust nothing; it pushes trust down to one
well-specified physical fact and models everything above it.

**The seed-honesty axiom (real — discharged, not dissolved).** The
*first* seed entered the ladder from outside the modeled system —
historically the v2 compiler / hand-bootstrap. This is a **genuine
second floor**, not a dissolving one. §5 is the reason: the bit-identical
fixed-point witness proves *reproduction*, not *honesty* — a compromised
seed reaches a compromised, self-consistent fixed point that passes the
witness. So "the seed that bootstrapped the chain was honest" is a real
trust assumption; path-independence does **not** erase it (path-
independence covers only benign entry paths converging — it says nothing
about a malicious one).

Two ways to handle it, and the model should prefer the second:

- *(a)* accept seed-honesty as a second permanent axiom alongside the
  physical one; or
- *(b)* **discharge it** with an independent witness — **diverse
  double-compilation** (Wheeler, 2005): compile `S_C` with two
  independently-derived compilers; if both reach a bit-identical `C*`, a
  backdoor would have to exist *identically* in both independent
  toolchains — vanishingly unlikely. (b) converts seed-honesty from
  *trust* to a *checked fact*.

v2 still "shrinks toward zero" as ambient *code* — but the seed-honesty
*question* does not vanish when v2 is deleted; it is discharged by the
diverse-double-compilation witness, not by v2's removal.

So the end state is: the fact models, the projections from them, and
**two floors** — the **physical axiom** (permanent, irreducible — the
silicon executes the machine model) and the **seed-honesty axiom** (a
real trust assumption, discharged by diverse double-compilation rather
than dissolved). Naming both honestly is the point — the system claims
exactly one irreducible axiom and one *checkable* one, not zero.

## 7. What this means for T-32

T-32 is larger than "minimize the seed." Its Phase-1 deliverable is the
**layer model** itself — name every layer, every fact-model, every
projection edge, the comprehension gate, the modeled circularity, and
**both honest floors of §6**: the physical axiom *and* the seed-honesty
axiom together with its diverse-double-compilation discharge witness.
Both floors are load-bearing deliverables — naming only the physical
axiom would let the trust-discharge witness fall out of T-32, which §6
forbids (seed honesty is discharged, never dissolved). "Minimum
never-hand-edited seed" then *falls out*: minimum, because the
projection is no larger than the snapshot's footprint allows (§4);
never-hand-edited, because it is a *projection at all* — you edit facts,
and the seed re-emits.

The `.dag` modeling extends from here: the frozen sub-model + the
`footprint` lens + the `⊆`-`Witness` (§4) are new substrate; the
circularity's fixed-point equation + witness (§5) extend
`src/v4/workflow/bootstrap.dag` beyond the staged chain (building on the
T-20 expansion); the runtime model (§2) is the accumulation of
`src/v4/extdeps/posix.dag` / `src/v4/extdeps/file_system.dag` into a
complete execution substrate. None of it dispatches before the operator
ratifies this layer model as the Phase-1 definition.
