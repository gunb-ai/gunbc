# v2 Grounding — Worked Examples

> Companion to the **D2 reversal + fact-bundle reseed plan** (the
> `src/v2/DECISIONS.md` record was retired in the visibility flip; the
> reversal receipt lives in INVARIANTS.md P1 §"Hollow alias").
>
> **Restored 2026-06-11** — deleted in the #4192 visibility flip while still
> cited by INVARIANTS.md as the definition of the *coincide* evidence bar.
> PLAN-ONLY: these demonstrate the modeling *shape*; no substrate is edited.

## The model

An `extdeps` target model is a **demonstration** that each of the target's
types **grounds** — decomposes, via the concept DAG, into a subset of the
universal substrate primitives (`Bool`, `Nat`, the connectives, `Node`). A
type is a **carrier** (the data) and a **meaning** (a `Node` decode of what
the data denotes).

A coercion between two types can fail, so it is a function into the
ratified `Outcome` carrier:

```
coerce : A -> Outcome<B>
// Outcome<T> = Produced { value: T } | Rejected { diagnostic: Diagnostic }
//   — std/diagnostic.dag:369, already used by int_div / int_mod.
```

`Produced` carries the coerced value; `Rejected` carries a `Diagnostic` —
a **named, located** failure (it has a `Locus`), never a silent drop.

**How a coercion is derived — read this.** Coercion is the
**verification facet** of the derived homomorphism — the fold that
checks a candidate structure-preserving map (see
[modeling-discipline.md](../modeling-discipline.md) "The three facets"
and [the-derived-homomorphism.md](../thesis/the-derived-homomorphism.md)).
A coercion is *derived* by
comparing two groundings, not hand-authored. The deriver is a **mechanical
zip-fold** (a catamorphism): it walks both groundings in parallel — `List`
against `List` recurses, `Conj` against `Conj` compares field-wise, leaves
compare directly. It is not an *engine* — per decision U1 (recorded in the retired
`DECISIONS.md` ledger; the decision stands, the ledger is gone) it makes
no decisions and runs no search.

**Coincidence is mechanical structural equality.** Two groundings
*coincide* when their canonical `Node` forms are equal — the equality
`node.dag`'s B1-CANON *contract* defines: `content_hash = merkle_fold ∘
canonical`. Comparing groundings *is* comparing content-hashes of
canonical `Node`s; decidable, not free-form semantic equivalence. (B1-CANON
is the operator-ratified *specification* of that fold; `node.dag` does not
yet carry a realized `canonical` / `content_hash` function body — see
**Design status** below.)

**The shared vocabulary keeps the fold mechanical.** Facts are *sourced*
independently — each type grounded from *its own* spec — but must be
*expressed* in one shared vocabulary: the `std/` primitives, authority
`std/algebra.dag`. Were `RustI32` and `Int32` grounded in different
vocabulary (`bit_count` vs `width`), their `Node` trees would differ
*structurally* though they mean the same — and the mechanical compare
would wrongly say "not equal." Shared vocabulary is load-bearing: it is
what makes the fold both mechanical *and* complete.

**Design status — specified, not yet realized.** The hard half is
*designed and ratified*, not *built*: `node.dag`'s B1-CANON ratifies the
canonical-form + content-hash **contract** (`content_hash = merkle_fold ∘
canonical`), and the T-9/C1 decisions (retired `DECISIONS.md` ledger)
specify the comparison as
*decidable by construction* (structural recursion over the closed declared
candidate set). But neither the canonical fold nor the coercion zip-fold
is a realized `.dag` function body yet — `node.dag` carries the contract,
not an implementation. What stays true is that this is *mechanical
design*, not a research problem: the parallel walk is determined by the
contract, not invented. The genuine hard work is the **modeling
discipline**: forcing every spec fact into the shared vocabulary so the
fold can see identity — Phase 1's job. One compile-time residue stays
deferred: refinement subsumption (`p ⟹ q`) is not a pure tree-walk and
fail-closes (substrate T-25).

This is *not* the reversed D2 alias. D2 *asserted* `type RustI32 = Int32`.
Here every type is grounded from its own spec and expressed in the shared
std vocabulary; any identity is a *claim discharged by the fold*, never an
assertion.

## What the "IR" is — it is `Node`

The examples below write "↔ IR `Int32`", "IR record", etc. There is no
separate "IR" artifact. **The IR is `Node`** — `std/node.dag`'s single
recursive type: the 6 connectives `Atom | Conj | Disj | Arrow |
Cardinality | Instantiation`. `core` produces `InferredTree` = `Node` +
the frozen flat `InferredFacts` coordinate (IR-1) — still not a separate
type. Per `compiler/00_compile.dag` the pipeline is the **OMNI pivot**:

```
ingest : (Source, LanguageModel)    -> Outcome<Node>
core   : Node                        -> Outcome<InferredTree>
emit   : (InferredTree, TargetModel) -> Outcome<Source>
```

Two corrections this forces on the examples' wording:

1. **Source is not "translated to" an IR.** There is one representation.
   `ingest` *builds* a `Node` that represents the source; `emit` *reads*
   a `Node` to produce target text. Source is parsed *into* the `Node` —
   which **is** the program. There is no python-IR / rust-IR pair.
2. **std types are vocabulary, not "IR types."** `Int32`, `List<T>` are
   `std/` vocabulary that *grounds into* `Node` connectives; the `Node`
   *represents* a construct using that vocabulary. "↔ IR `List<Int32>`"
   means: the `Node` represents the sequence with std `List` vocabulary;
   the *target model* — not the IR — decides how `List` realizes in Rust.

**Node-tree vs program-DAG.** The `Node` *type* is a finite tree (A1:
recursion lives solely in the children list). Program-level sharing and
cycles — def-use, loops — are by-reference via `Symbol` `Atom`s, never by
inlining. So the realized program IR is a **`Symbol`-linked DAG of
`Node`**, distinct from the finite `Node` tree.

**One mechanic, N+M ways.** The compiler is not a pile of special-case
stages. It has ONE mechanic: a **causal transform** = projection (read
facts off `Node`) + coercion (the `Outcome`-typed map). `infer`, `lower`,
the lenses, `emit` are all instances of it; tokenize/parse/emit are
"special" only in that text I/O is a common case packaged as a module —
the machinery is identical, and the tokenize/parse legs are the
`Transform` L1 behavior (one of `node.dag`'s 5: `Value | Transform |
Branch | Loop | Bind`). So the worked examples below are not a
coercion-verdict catalog — they are evidence of **one grounding mechanic
applied N+M ways**: you author N ingest models + M emit models (authoring
cost N+M), and every ingest composes with every emit through the one
`Node` pivot (capability N×M). N+M authored → N×M capabilities, because
the mechanic is pivot-shaped. The N×M *compiler-emit* capability claim is
scoped to **Shape A** targets — see the next section; Shape-B artifacts
are emitted by `.dag` user programs, not by the compiler.

### Shape A vs Shape B — what this catalog does and does not claim

This catalog demonstrates **grounding** — that each target's types
decompose into the shared `Node` / `std` vocabulary — and the **coercion
shape** between groundings. Grounding is **universal and shape-agnostic**:
a Rust type, a JSON Schema object, an OpenAPI operation, a SPICE voltage,
an English noun phrase all ground the same way, and the fact-bundle
discipline is identical for all of them.

It does **not** claim every target is a compiler render-target.
`THESIS.md` locks a distinction the worked examples must not blur
(THESIS.md §"Omni-emission" — the Shape A vs Shape B bullets; ROADMAP
Track 16):

- **Shape A — compiler language targets**: programming languages and HDLs
  — `rust`, `python`, `go`, `cpp`, `typescript`, `verilog` (and the
  low-level / IR targets `llvm_ir`, `ptx`, `machine_code`, likewise
  modeled in `extdeps/languages/`). The compiler emits these **directly**
  via `emit(TargetModel)` against a language spec. The N+M / N×M
  *compiler-emit* capability claim above is about these.
- **Shape B — user-program artifacts**: `openapi`, `json_schema`, `yaml`,
  `json`, `csv`, `toml`, `spice` netlists, English docs. These are **NOT**
  compiler render-targets — they are emitted by `.dag` **user programs**
  walking typed values (`concat` / `fold` / `match`). Adding a Shape-B
  target is writing one reusable `.dag` emitter program, not a compiler
  change.

So a Shape-B entry in this catalog is a **grounding / data-model
example** — it shows how that format's types ground and how a coercion
against them is shaped. It is **never** an instruction to implement the
format as a compiler `emit(TargetModel)` render path. The grounding
discipline is shared across both shapes; the *emission mechanism* is not.

## Spectrum

These examples are chosen to span the full range deliberately —
`machine_code` (pure bits) → `rust` (typed, generic) → `verilog`
(4-valued logic) → `spice` (continuous) → `lean` (dependent types) →
`english` (natural language). One model handling all of them is the
evidence that it is universal.

---

## Live-state tags (P0)

Every stage below is tagged for its honest live-state on `origin/main`:

- **`[DAG-REALIZED]`** — the `.dag` has a real function body for this
  path (not a stub) — but the v2 compiler is not yet bootstrapped into
  an executing pipeline; nothing *runs* it today.
- **`[MODELED]`** — the modeling shape only; no `.dag` body yet.

There is deliberately **no `[BUILT]`/runs-today tag**: v2's compiler is
itself `.dag`, Wave-1, not bootstrapped into an executing binary. The
strongest claim any stage can honestly make here is `[DAG-REALIZED]` —
and the doc never claims more.

## The end-to-end shape — one grounded chain, one modeled spine

### A. The most-grounded chain — the `.dag` self-hosting frontend `[DAG-REALIZED]`

The v2 frontend is **Wave-1** and `.dag`-realized, not executing:
`01_tokenize.dag` has a real function body for the **E0 void-lexical**
path and `02_parse.dag` for the **G0 void-grammar** path. So the
most-grounded end-to-end chain — real `.dag` function bodies, though
nothing executes them as a pipeline yet — is the empty/void one:

```
""  (empty .dag source)
  → tokenize("", file, LexRules=E0)            : (String,Symbol,LexRules) -> Outcome<TokenStream>
  → Produced(empty TokenStream)                                       [DAG-REALIZED]
  → parse(emptyTokenStream, Grammar=G0)         : (TokenStream,Grammar) -> Outcome<ParseTree>
  → Produced(degenerate ParseTree)              // ParseTree = Node    [DAG-REALIZED]
```

The fail-closed direction is `.dag`-realized too — non-empty source
against E0:

```
"x"  → tokenize  → Rejected { Diagnostic { reason, at: Locus = WholeFile } }  [DAG-REALIZED]
```

That `Rejected` is a constructed `Diagnostic` with a `Locus` — "named,
never silent" is *shown* in the realized `.dag` body, not asserted in
prose. Richer `.dag` tokenization (real keywords, programs) is
declarative grammar-data **not yet realized** — `[MODELED]` from here
up. Both realized functions already return `Outcome<T>`.

**The round-trip — one `Node`, two directions.** The same void chain run
forward and back:

```
""  → tokenize → parse → ParseTree   [DAG-REALIZED]  — ingest builds the Node
    → emit → ""                      [MODELED]       — emit reads the Node back
```

`tokenize` / `parse` are causal transforms — the `Transform` L1 behavior
— each projecting facts off its input and coercing to its output; they
are not special-case stages. `emit` is the *same mechanic* run the other
direction (`05_emit.dag` is a `T-10` scaffold, so the emit leg is
`[MODELED]` — the mechanic-identity claim is structural, not a runtime
claim). This *shows* "emit is a section of ingest": one `Node`, ingest
constructs it, emit projects it back — not two hand-built stages.

### B. The modeled spine — `python int → IR → rust i32` `[MODELED]`

There is **no** Python or Rust parser (`extdeps/languages/python.dag`
models Python's type *surface*, not a parser). The whole chain is
`[MODELED]`. **`Node` is the explicit hub** — "IR" and the `IR_*` names
below are *not* a distinct IR type plane; per the "What the IR is" section
above, `IR_Int` etc. are the shared `std/` vocabulary carried *as* `Node`.
The chain has two coercions with *different totality*:

**Ingest — `python int → IR Int` (total).**

```
ingest_int : PythonInt -> Outcome<IR_Int>                                    [MODELED]
```

`IR_Int` is `GroupCompletion<Nat>` — unbounded. Every Python `int`
(arbitrary-precision) grounds to an `IR_Int`; ingest is **total** — always
`Produced`. No overflow exists at the IR: the IR integer has no width.

**Emit — `IR Int → rust i32` (partial — overflow lives here).**

```
emit_i32 : IR_Int -> Outcome<RustI32>                                        [MODELED]
// Spec: models rust core::convert::TryFrom for i32 — the fallible
//       conversion. NOT `as` (wraps/truncates), NOT `From` (widening-only).

emit_i32(ir) =
  if  −2³¹ ≤ value(ir) ≤ 2³¹−1  → Produced( rust i32 )
  else                          → Rejected { Diagnostic {
                                     reason: integer_out_of_range, at: Locus } }
```

`RustI32` grounds as `Compose<Int, MachineWidth<Word32>>` — the abstract
`Int` carrier refined by an independent machine-width axis (per
`std/integer.dag`; **not** a parallel bit-carrier — `integer.dag` forbids
that). The `−2³¹..2³¹−1` bound is the `MachineWidth<Word32>` refinement
predicate, not catamorphism output.

So Python `10**100` ingests fine (`Produced` an `IR_Int`) and **fails
closed at emit** (`Rejected`, a real `Diagnostic`). Overflow is an
**emit-boundary** fact — localized, typed, named.

### A worked operation

Operations live on the IR, where integers are unbounded — so
`int_add : (IR_Int, IR_Int) -> IR_Int` is **total** (no overflow in the
IR; cf. `std/integer.dag` `int_add`). Overflow appears only when the
*result* is emitted: `emit_i32(int_add(a, b))` is the partial step. "`a + b`
too big" is therefore **not** a property of `+` — it is a property of
*emitting `+`'s result to a fixed-width target*.

### Emit is a section of ingest, not a bijection

`ingest` and `emit` are the **two directions of one grounding map**, not
inverses. `ingest` is lossy on text (comments, whitespace, sugar collapse
to the same `Node`); `emit` picks one canonical text. `emit ∘ ingest` ≈
identity *on meaning*, never on text. At the carrier level above, ingest
is additionally *total* and emit *partial* — the asymmetry that localizes
overflow at the emit boundary.

---

## Per-target carrier groundings

## 0. `bool` / `!` — grounding at the primitive level (the base case of one universal discipline)

**There are no "quirked" types.** *Every* external type is grounded in
our primitives. The only question is **at what level the grounding
starts**: most types start as a **refinement some level above a
primitive**; a genuinely boutique type starts from the **raw
primitives** and is built up. `bool` is the base case only because it
sits *so close to a primitive* that there is almost nothing to get
wrong — not because it is a different category. We model each
language's type by **reading that language's spec for that type** (we
are modeling the spec), then expressing it in our vocabulary at the
appropriate level. Modeling accuracy is fundamentally an *integration*
problem — it is iterative and validated by testgen against the target
(separate followup), not asserted.

**The retired shape (D2a).** The pre-ruling form asserted identity by a
label + a spelling string:

```
type RustBool = Bool                                   // bare D2 assertion
data rust_bool_grounding: GroundingMap<RustBool> = GroundingMap {
  spelling: "bool"                                      // identity by a label string
}
```

`"bool"` is prose a checker cannot walk — the hollow form the
machine-readable-inhabitance bar forecloses. "Identity is a claim
discharged by the fold, never an assertion."

**The ratified shape (canonical-B) — declaration present, P2/E-6
staging.** The grounding *declaration* is a real, fold-traversable edge
that **references the single std authority** for the shared structure:

```
import v2.std.logic { Bool, bool_boolean_algebra }
import v2.std.algebra { BooleanAlgebra }
// Anchor: <the language's own bool spec>
data <lang>_bool_grounding: BooleanAlgebra<Bool> = bool_boolean_algebra
```

This is not an assertion: it grounds the language's `bool`, *read from
that language's spec*, into the shared `std/logic` + `std/algebra`
vocabulary by pointing at the one authoritative `BooleanAlgebra<Bool>`
instance (`std/logic.dag`). A checker walks `<lang>_bool_grounding` →
the real `bool_boolean_algebra` structure; coincidence with std `Bool`
is then a structural fact the fold discharges, not a label. **One
authority** (no per-file algebra duplication — M2); codewide
improvements to the bool concept centralise there, and any future
divergence is *fold-rechecked*, not silently inherited (the
centralisation/drift resolution). `!` grounds to the std `Never`
primitive directly (`type RustNever = Never`) — zero inhabitants, no
structure above the primitive, so the primitive identity *is* the
complete grounding (the degenerate base of the same discipline).

**Verified scope (honest) — declaration, not landed authority.** The
declaration compiles **0 diagnostics under the real bootstrap gate** —
`v1-compiler compile --source-root src/v2` (ci.yml `v2:` job; 72 modules
resolved). That is the *parse/resolve* half of bootstrap viability; the
v2-**run** gate is T-22-deferred. Crucially this is **not** landed
target-spec authority: no same-PR consumer reads `<lang>_bool_grounding`
(the canonical/coincidence fold — the consumer — is
*specified-not-realized*: `node.dag` B1-CANON contract, `[MODELED]`).
**INVARIANTS E-6 permits this as a *bounded exception* (clause (b))** —
a consumer-less target-spec decl that **sits behind a named scaffold
marker with a dissolution trigger**: each `data <lang>_bool_grounding`
carries the inline marker `🟡 feature:canonical-b-grounding-consumer`
that dissolves when the B1-CANON fold + coercion zip-fold
(decisions B1 · T-9/C1, retired `DECISIONS.md` ledger) consume it for
coincidence-checking. So the
*shape* is the ratified canonical-B decl-ref and parse-verified; the
*authority state* is **staging behind the E-6-named scaffold** —
consumer-gated, not landed, **not** speculative metadata — claimed no
further. (Aside: the frozen v3 interim parse-ratchets rejected decl-ref
`data` bodies — a v3-only artifact, v3 is *not* in the bootstrap chain;
they were dissolved, operator-authorized, in the PR that originally
landed this example (pre-restoration history), with the
v2 `v2:` job as the replacement parse gate.)

**The level spectrum — bool vs int (the lesson).** Same discipline,
different starting level:

- **`bool`** — starts *at* the primitive: `bool` *is* the shared
  2-element `BooleanAlgebra`. The grounding is the bare reference; no
  build-up. (Rust / C++ / Go / Lean / TS bool all read this way from
  their specs — strictly two values, standard ops, no object/null
  semantics — so all use the identical shared reference.)
- **`int`** — starts as a **refinement above** the primitive. Per
  `std/integer.dag`, `Int = GroupCompletion<Nat>` is unbounded ℤ and
  does **not** model overflow; `Int64 = Compose<Int,
  MachineWidth<Word64>>` is `Int` *refined* by a width dimension, and
  overflow is that refinement's boundary predicate. Python `int`
  (arbitrary precision) grounds to bare `Int`; Rust `i64` grounds to
  the `Compose` refinement + an overflow disposition — **same shared
  `Int`, different spec-sourced refinements.**
- **Python `bool`** — the worked deviation. The Python data model says
  `bool` *is a subtype of `int`* (`True == 1`). So Python's bool reads
  as: the shared truth grounding (`py_bool_grounding` → the same
  authority) **plus** numeric-tower membership — carried by the
  existing `PythonScalar.BoolScalar` classifier, the build-up
  above the primitive. (Object/singleton *identity* is a runtime-layer
  fact, outside the type model — the placement axis.) This is a *first
  iteration*; the exact build-up shape is expected to refine as testgen
  pressure-tests it against CPython.

The contrast is the whole point: there is one grounding engine —
*reference the shared authority for what the spec says is shared, build
up from the primitives for what the spec says deviates.* `bool` is
where the build-up is empty; `int` and Python-`bool` are where it
isn't. None of them are special-cased.

**Coercion shape.** `bool`: the fold compares canonical `Node` forms
leaf-to-leaf (`True`↔`True`, `False`↔`False`) — identity-quality
`Produced`; no width/sign/unit coordinate to drop or fabricate (the
clean endpoint). `!`: `Never` has no inhabitants, so `T -> Outcome<!>`
can only be `Rejected` (fail-closed by construction) and
`! -> Outcome<T>` is the unique empty map (ex falso, vacuously total).

## 1. Rust — `[T; N]` (const generics, compound coercion)

**Model** — grounded from the Rust Reference array type:

```
// CARRIER — an array value is exactly N elements of T in order. N is part
// of the type as a const Nat parameter, not a library allocation fact.
type RustArray<T, N: Nat> = Conj {
  elements: List<T>,
  length_proof: Witness< |elements| = N >
}

// MEANING — identity on the element sequence.
```

`RustArray<RustI32, 3>` grounds to `List<grounding(RustI32)>` plus a
length witness, and `RustI32` grounds as `Compose<Int,
MachineWidth<Word32>>` (§B — the substrate's fixed-width discipline,
`std/integer.dag`; not a `{32×Bool}` bit-carrier).

**Step-by-step coercion shape — `RustArray<RustI32, 3> -> Outcome<IR Conj{ elements: List<Int32>, length_proof: Witness<|elements| = 3> }>`:**
1. groundings: `List<Compose<Int,MachineWidth<Word32>>>` plus
   `Witness(length = 3)` on the Rust side. The IR target is the
   **length-witnessed** form — `List<Int32>` carried *with* its
   `Witness(length = 3)` (in `Node` terms, the `Cardinality` connective) —
   **not** a plain unbounded `List`. The length is a source fact, so it
   must appear in the target carrier.
2. the coercion fold compares outer `List` vs `List`,
   then recurses into the element grounding.
3. element `grounding(RustI32)` vs `grounding(Int32)` → coincide.
4. coincident element groundings return `Produced { value }` where the
   value is the IR length-witnessed list — `elements: List<Int32>` **and**
   `length_proof: Witness<|elements| = 3>`. The length-3 fact **flows
   forward into the output carrier** (P2 facts-flow-forward); it is not
   dropped to a plain `List`, and no accepted-loss is needed because
   nothing is lost. No array coercion is authored; the compound result,
   length witness included, follows from the structural comparison.

**Cross-language shape — `PythonList[int] -> Outcome<RustArray<i32, 3>>`:**
outer `List` matches, but each element must pass `IR_Int -> Outcome<RustI32>`
as in §B, and the source length must discharge `Witness(length = 3)`.
In-range values of the right length produce the Rust array; any element
outside the `MachineWidth<Word32>` predicate or any non-3 length returns
`Rejected { diagnostic: Diagnostic }`.

---

## 2. machine_code — a 64-bit general-purpose register (the pure-bits endpoint)

**Model:**

```
// CARRIER — a register value is exactly 64 classical bits. The width is a
// structural fact (a length witness), not a comment — same shape as the
// Rust array above; a bare `List<Bool>` would admit any length.
type Reg64 = Conj {
  bits: List<Bool>,
  length_proof: Witness< |bits| = 64 >
}

// MEANING — none intrinsic. A raw register is an uninterpreted bit
// pattern; meaning is supplied by the instruction that consumes it
// (the same 64 bits are an integer to ADD, an address to LOAD, a
// float to ADDSD). Reg64 grounds carrier-only; meaning is deferred
// to the consumer.
```

This is the spectrum's trivial endpoint — `machine_code` *is* bits, so
the grounding is direct, no decode.

**Coercion shape — `Reg64 -> Outcome<Conj{ bits: List<Bool>, length_proof: Witness<|bits| = 64> }>`:**
the coercion is **identity-quality** — `Reg64` *is already* the
length-witnessed bit-list, so the target carrier is the same shape, not a
bare `List<Bool>`. Projecting to a bare `List<Bool>` would silently drop
the width-64 witness — the facts-flow-forward violation §1 (the Rust
array) warns against; the width-64 fact must flow forward into the target
carrier (or be an *explicit* accepted-loss, which a "pure-bits endpoint"
identity coercion has no reason to be). Returns `Produced { value }`
carrying **both** `bits` and `length_proof`. Instruction-specific reads
are separate coercions: the consumer supplies the meaning (`Int64`,
address, binary64, ...), and an unsupported or missing meaning returns
`Rejected { diagnostic: Diagnostic }` rather than inventing one.

---

## 3. Verilog — `reg [31:0]` (a target primitive *richer* than ours)

Verilog's logic value is **4-valued**: `{0, 1, x (unknown), z (high-Z)}`
— not our 2-valued `Bool`.

**Model:**

```
// CARRIER — exactly 32 four-state bits. Width is a structural length
// witness, not a comment — same shape as the Rust array.
type VBit   = Zero | One | Unknown | HighZ   // closed 4-sum (a coproduct)
type VReg32 = Conj {
  bits: List<VBit>,
  length_proof: Witness< |bits| = 32 >
}

// MEANING — the 32 four-state bits. A plain `reg [31:0]` is UNSIGNED in
// Verilog; signedness is NOT a fact of this carrier — it requires the
// explicit `signed` modifier (`reg signed [31:0]`). Read as a number the
// {0,1} bits are a 32-bit two's-complement BIT PATTERN; the
// signed-vs-unsigned interpretation is supplied by the declaration, not
// by the vector. Undefined if any bit is x/z.
```

`VBit` grounds — as a closed sum over the substrate's coproduct
connective. It does **not** ground as `Bool`: Verilog's primitive is
richer. `Bool` is exactly the `{Zero, One}` sub-part of `VBit`.

**Step-by-step coercion shape — `VReg32 -> Outcome<IR Int32>`:**
1. IR `Int32` grounds to `Compose<Int, MachineWidth<Word32>>` (the
   fixed-width integer discipline from §B); `VReg32` grounds to
   `Conj { bits: List<VBit>, length_proof: Witness<|bits| = 32> }`.
2. the coercion fold compares element-wise:
   `Bool` vs `VBit` — `Bool ⊊ VBit`.
3. `IR Int32 -> Outcome<VReg32>` is total over this relation: every
   `Bool` embeds as `Zero` or `One`, so the result is `Produced`.
4. `VReg32 -> Outcome<IR Int32>` is partial on **two** axes. (i) Any
   `Unknown`/`HighZ` bit returns `Rejected { diagnostic: Diagnostic }` —
   x/z is not an integer state. (ii) For all-`{Zero, One}` bits the 32-bit
   pattern is well-defined, **but a plain `reg [31:0]` is unsigned** — it
   carries no signedness fact, and IR `Int32` is *signed*
   (`Compose<Int, …>`). Producing signed `Int32` from a plain `reg [31:0]`
   would fabricate the signedness fact — the same hazard as the LLVM
   `add i32` case (§10). The honest coercion: a plain `reg [31:0]` lands
   an **unsigned** 32-bit carrier; signed `IR Int32` is `Produced` only
   when the source is `reg signed [31:0]` — the signedness-bearing
   declaration — otherwise `Rejected`.

This is "speaks a *subset*" made precise: Verilog and the IR share the
`{0,1}` subset; the `x/z` states are Verilog-only and honestly
fail-closed; and signedness, absent from a plain vector, is required as
an explicit declaration fact rather than fabricated. The model handles a
target whose primitive is richer than ours without distortion — and
without inventing facts the carrier never modeled.

---

## 4. SPICE — a node voltage (approximate carrier vs exact-voltage gap)

**Model:**

```
// CARRIER — current std authority: dsl/std/float.dag pins Real to
// ApproximateField<FieldOfFractions<Int>>, not an exact continuum.
type SpiceApproxVoltage = Conj {
  magnitude: ApproximateField<FieldOfFractions<Int>>,
  unit: Volt
}

// MEANING — approximate voltage fact with explicit approximation policy.
// GAP — SpiceExactVoltage requires a grounded exact-real/quantity carrier.
```

`Real` is not treated here as an exact continuous carrier: the live
authority (`dsl/std/float.dag`) defines `Real =
ApproximateField<FieldOfFractions<Int>>`. `Volt` is a `Dimension`.
IEEE-754 `f64` grounds separately as a fixed-width approximate carrier with
rounding and special-value facts. An ideal SPICE voltage that requires an
exact mathematical continuum is therefore a named fail-closed gap:
`SpiceExactVoltageMissingExactRealCarrier`, not a silent reuse of `Real`.

**Step-by-step — coercing a `Rust f64` toward `SpiceApproxVoltage` (the unit must be supplied, not fabricated):**
1. `SpiceApproxVoltage` grounds to `ApproximateField<FieldOfFractions<Int>>`
   **plus a `Volt` unit fact**; `f64` grounds to IEEE-754 binary64 — a
   **unitless** approximate magnitude (`dsl/std/float.dag` /
   `src/v2/std/float.dag`).
2. the coercion fold compares the groundings: the approximate-field
   *magnitude* carriers are related (different width/policy facts) — but
   `SpiceApproxVoltage` carries a `Volt` coordinate the `f64` grounding
   **does not have**.
3. Because the `Volt` fact is absent from the source, a bare
   `f64 -> SpiceApproxVoltage` **cannot be `Produced`** — conjuring the
   unit coordinate would fabricate a fact, the exact violation this doc
   models against (the emit-side dual of a hollow alias). The unit must be
   **supplied**: the honest coercion is
   `(f64, unit: Volt) -> Outcome<SpiceApproxVoltage>`, the `Volt` an
   explicit boundary input (the SPICE context asserts "this number is
   volts"). The f64-only coercion lands only the magnitude —
   `f64 -> Outcome<ApproximateField<…>>` — returning `Produced` (quality
   `Lossy`) when the approximation policy is explicitly accepted and
   recorded; the consumer that has the `Volt` fact constructs the full
   `SpiceApproxVoltage`.
4. `NaN` / `±∞` return `Rejected { diagnostic: Diagnostic }` because they
   are IEEE-754 special values, not voltage magnitudes. A requested
   `Rust f64 -> Outcome<SpiceExactVoltage>` returns
   `Rejected { diagnostic: Diagnostic { reason:
   SpiceExactVoltageMissingExactRealCarrier, ... } }` until an exact carrier
   is modeled.

The endpoint here is no longer mislabeled as exact, and the unit is
**supplied, never fabricated**. The approximate path is a
**declared-lossy** derived map over the *magnitude*; the `Volt`
coordinate enters only as an explicit boundary fact; the exact-voltage
path is a named fail-closed gap.

---

## 5. Lean — `Fin n` (dependent types)

A Lean type can depend on a *value*. `Fin n` is the type of natural numbers
strictly less than `n`, so the bound `n` is part of the type itself.

**Model:**

```
// CARRIER + WITNESS — the dependent index n is an explicit Nat; the type-level
// constraint "value < n" grounds as a Witness (see src/v3/std/dimensions.dag),
// the substrate's proof-carrier.
type LeanFin<n: Nat> = Conj {
  value: Nat,
  bound_proof: Witness< value < n >     // n : Nat, lifted into the type
}
```

A dependent type = carrier + a `Witness` pinning the dependent value.

**Step-by-step coercion shape — `IR Nat -> Outcome<LeanFin<3>>`:**
1. grounds to `Nat` + `Witness(value < 3)`.
2. an IR `Nat` value `0`, `1`, or `2` discharges the witness and returns
   `Produced { value: LeanFin<3> }`.
3. an IR `Nat` value `>= 3` cannot discharge the witness and returns
   `Rejected { diagnostic: Diagnostic }`; you cannot claim `Fin 3`
   without the bound proof.

Dependent types ground via the `Witness` substrate — the type's proof
obligation becomes a `Witness` the coercion must discharge.

---

## 6. English — noun phrases (the fail-closed endpoint, and composite grounding)

English has no type system. What we model is the **groundable fragment** —
and that fragment is not only leaf values: English can *express* the
**connectives** (though not via a simple word-mapping — see below), so it
can ground *composite structure*.

### Leaves — number words → `Nat`

```
// CARRIER — a sequence of words from English's CLOSED number grammar:
// units, teens, tens, scale words (hundred/thousand/…), sign
// ("negative"). Closed + enumerable → it grounds.
type NumberWord     = One | Two | … | Hundred | Thousand | Negative | …
type EnglishInteger = List<NumberWord>

// MEANING — the decode to an integer: a Node of Nat arithmetic.
//   "forty-two"          → tens(4)*10 + units(2)  = 42
//   "negative forty-two" → −(that)                = −42
```

An English integer literal **grounds** — closed carrier + deterministic
decode. `EnglishInteger -> Outcome<IR_Int>` returns `Produced` when the
grammar deterministically decodes the phrase: "negative forty-two" maps
to the IR `Int` value `−42`. A qualifier such as "about" or "-ish" has
no deterministic decode and returns `Rejected { diagnostic: Diagnostic }`.

### Connectives — English can ground composite structure, but the word is *not* the connective

English can express the substrate connectives — but **the surface word
does not determine which connective it is.** The decode reads the
construction's *meaning*; the word is a weak, often-misleading hint. The
modeling here is genuinely non-trivial.

**`Conj` vs `Disj` — "and" does not pick.**
- "an order has a customer **and** a total" — both-present coordinates
  → `Conj{ customer, total }`.
- "we accept cash **and** card" — a *menu of alternatives*, one is
  chosen → `Disj{ Cash, Card }`. The same word "and", a different
  connective.

**`Disj` is exclusive — and English does not mark that with a bare word.**
A substrate `Disj` (a sum) is a value that is *exactly one* variant.
- "**either** soup **or** salad" — the *either…or* construction marks
  the exclusivity → exclusive `Disj`.
- a bare "milk **or** sugar?" is often *inclusive* (both allowed) — and
  inclusive-or is **not a `Disj` at all** (it is closer to a `Conj` of
  optionals / a non-empty subset).

So a faithful decode must establish (a) conjunction-of-coordinates vs
choice-among-alternatives, and (b) if a choice, exclusive vs inclusive —
and the surface word reliably tells you *none* of it. The same
under-determination holds for "of" (`Instantiation` vs the possessive
"a friend **of** mine"), plurals (`Cardinality` vs idiom), and "for
each". A construction whose connective cannot be determined returns
`Rejected { diagnostic: Diagnostic }`.

### The `Outcome` boundary — words under-determine structure

This is why English's groundable subset is an *island* and the
fail-closed boundary is large — **not** because the connective words are
missing, but because the words **under-determine** the structure.
"this and that" may be a `Conj` or a `Disj`; "or" may be exclusive or
inclusive; the word alone cannot say. Leaf phrases reject the same way:
`EnglishPhrase -> Outcome<Node>` returns `Rejected { diagnostic:
Diagnostic }` for "forty-two-ish", "about forty-two", or "ship it when
the build feels solid".

When the construction *is* unambiguous — a disciplined, controlled
English written to a fixed structural convention — English grounds
composite structure honestly, recursively, as a `Node` decode. Free prose
under-determines and rejects. Model what determinately grounds; reject the
rest. (This is the
`english_ingest_fail_closed.dag` boundary-honesty probe v2 already
plans.) English is not a degenerate case; it is a full language whose
words under-determine structure, hence a large, honest `Rejected`
boundary.

---

## 7. Go — `chan T` (typed communication endpoint)

Go channels are typed FIFO communication endpoints. Their value model is not
"a list"; the channel value is a capability to send and/or receive values of
`T`, with ordering and blocking behavior supplied by the Go spec.

**Model:**

```
// CARRIER — the channel TYPE `chan T`. Per the Go spec, a channel type
// is EXACTLY element type + direction — two channel types are identical
// iff same element type and same direction. Nothing else is a type fact.
type GoChan<T> = Conj {
  element:   TypeGrounding<T>,
  direction: SendOnly | ReceiveOnly | Bidirectional
}
// SCOPE — this carrier models the channel TYPE, not a channel VALUE.
// `capacity` is NOT a type fact: it is a value-construction argument to
// `make(chan T, n)` — two `chan int` types are identical regardless of
// the capacity their values were made with. Capacity, and the runtime
// value-state — open/closed, live buffer occupancy, queued contents,
// nil-ness — all belong to the channel VALUE / `make` / eval layer,
// explicitly outside this type carrier.

// MEANING — the channel type's communication contract: ordered FIFO
// transfer of T values, with `direction` the static type fact. Blocking
// behavior is a RUNTIME fact — it depends on the make-time capacity and
// the live buffer occupancy / open/closed-ness, none of which the static
// type determines — so it is not claimed by this carrier.
```

**Step-by-step coercion shape — `GoChan<int32> -> Outcome<IR Stream<Int32>>`:**
1. `chan int32` grounds to endpoint facts plus `grounding(int32)`.
2. the coercion fold compares endpoint vs endpoint,
   then recurses into the element grounding.
3. `int32` vs `Int32` coincide when both decode as 32-bit two's-complement.
4. matching element grounding plus the `direction` type fact return
   `Produced`. A missing `direction` returns `Rejected { diagnostic:
   Diagnostic }` rather than defaulting to bidirectional. `capacity` is
   *not* a type fact, so its absence is **not** a rejection criterion for
   the channel type — rejecting a `chan T` type for "missing capacity"
   would itself be an unfaithful model (a plain `chan T` type genuinely
   has no capacity).

The channel type fact is `direction` (with the element type) — a grounded
target fact, not an annotation. Capacity (a `make`-time value-construction
argument) and the runtime scheduling state — live buffer occupancy,
open/closed-ness — are a separate channel-value / eval-layer concern, not
part of this type grounding.

---

## 8. C++ — `int` (implementation-defined primitive width)

C++ `int` is a language primitive. Per **C++20** (ISO/IEC 14882:2020,
§[basic.fundamental]) — the edition that **fixed** signed integer
representation — its **signed value representation is two's complement**;
only its **width** is implementation-defined (16+ bits; LP64 / ILP32
commonly pick 32). That fixity carries forward unchanged to the current
edition, C++23 (ISO/IEC 14882:2024). The model reads width from the
target/ABI binding (T-29) and treats representation as a fixed *spec*
fact, not an implementation-defined axis. (Pre-C++20, signed
representation was itself implementation-defined — modeling that would
require pinning an older edition; this example is anchored to C++20, the
edition where the fact became fixed.)

**Model:**

```
// Anchor: ISO/IEC 14882:2020 (C++20) §[basic.fundamental]
// CARRIER — a C++ `int` fact-bundle: the signed-integer base plus the
// width and representation facts. `Compose` is BINARY in std/
// (`Compose<carrier, dimension>`, integer.dag) — a multi-fact bundle is
// a `Conj` of facts, not a variadic `Compose`. PLAN-ONLY note: the
// `Representation` axis is Phase-2 std/ substrate (T-3-extended —
// signedness/representation do not exist in std/ yet); it is shown here
// as the intended fact, marked future-substrate, not a live carrier.
type CppInt = Conj {
  base:           Int,                       // signed-integer carrier (live std/)
  width:          Nat,                       // implementation-defined
  width_proof:    Witness< width >= 16 >,    // C++ guarantees int is >= 16 bits
  representation: Representation              // = TwosComplement, C++20-fixed;
                                             // Representation axis is Phase-2
                                             // substrate (T-3-extended), not live
}

// MEANING — integer value within the range fixed by width under
// two's-complement.
```

**Step-by-step coercion shape — `CppInt -> Outcome<IR Int>`:**
1. `CppInt` grounds to `Int` plus a **fixed** two's-complement
   representation fact plus **one** implementation-defined fact — `width`
   (the genuine per-ABI variable; range is derived from width).
2. the coercion fold compares the abstract integer value to
   IR `Int`; the width fact stays attached as a target fact.
3. `CppInt -> Outcome<IR Int>` returns `Produced` because every inhabited
   C++ `int` denotes an integer value.
4. `IR Int -> Outcome<CppInt>` is partial: it returns `Produced` only when
   the value is inside the implementation-defined range; otherwise
   `Rejected { diagnostic: Diagnostic }`.

This contrasts Rust `i32`: Rust fixes the width in the type name; C++ `int`
requires the target/ABI model to carry the implementation-defined **width**
fact. If the binding cannot provide it, the coercion fails closed instead
of assuming LP64/ILP32.

---

## 9. TypeScript — `number | string` (structural union with disjoint primitive arms)

TypeScript unions are structural acceptance sets. Generic `A | B` is not
automatically an exclusive substrate `Disj` because structural members can
overlap. This example uses disjoint primitive arms (`number | string`) so the
accepted value has a unique arm after the target's own type classification.

**Model:**

```
// CARRIER — membership in one of two disjoint primitive acceptance sets.
type TsNumberOrString = StructuralUnion {
  members: Set { TsNumber, TsString },
  disjointness: Witness<Disjoint<TsNumber, TsString>>
}

// MEANING — the accepted member's own meaning. Generic unions require
// overlap/narrowing facts before they can be treated as a Disj.
```

**Step-by-step coercion shape — `TsNumberOrString -> Outcome<IR (Float64 | String)>`:**
1. TypeScript `number` grounds to IEEE-754 binary64; ECMAScript `string`
   grounds to a sequence of UTF-16 code units, including possible ill-formed
   surrogate subsequences.
2. IR alternatives ground independently: `Float64`, `String`.
3. the coercion fold first uses the disjoint primitive
   membership witness to choose the arm, then compares that arm's grounding.
4. `number` to `Float64` returns `Produced`. `string` to IR `String`
   returns `Produced` only if the IR string model is UTF-16-code-unit-shaped
   or a validated decode from UTF-16 code units to scalar values succeeds;
   otherwise it returns `Rejected { diagnostic: Diagnostic }` or a
   `Produced` value with an explicit accepted-loss fact.

The generic union case stays a structural membership/acceptance fact until
overlap or discriminant evidence proves an exclusive `Disj`.

---

## 10. LLVM IR — SSA value (typed single-assignment register)

An LLVM SSA value is a typed result of one definition. Its identity is a graph
edge, not a mutable storage slot.

**Model:**

```
// CARRIER — one definition edge with a declared LLVM type.
type LlvmSsaValue<T> = Conj {
  definition: NodeRef,
  llvm_type:  LlvmType<T>,
  value:      T
}

// MEANING — the value produced by the defining instruction, decoded through
// llvm_type. Single-assignment is a graph invariant over definition.
```

**Step-by-step coercion shape — `LlvmSsaValue<i32> -> Outcome<IR Int32>`:**
1. LLVM `i32` grounds to **32 bits** — a width fact only. LLVM integer
   types are **sign-agnostic**; `i32` itself carries no signedness.
2. the consuming operation supplies arithmetic meaning — but not always
   signedness. `add i32` is **signedness-neutral**: two's-complement
   `add` is the *identical* operation for signed and unsigned operands,
   so `add` grounds a 32-bit two's-complement integer, **not** a signed
   one. LLVM carries signedness only in the operations where it differs —
   `sdiv`/`udiv`, `srem`/`urem`, `sext`/`zext`, etc.
3. IR `Int32` is **signed** (`Compose<Int, MachineWidth<Word32>>` — `Int`
   is the signed carrier). So the coercion fold needs a signedness fact
   the source must actually carry.
4. a **signedness-bearing** source supplies that fact → return `Produced`.
   The genuine signedness-bearing sources are: a **signed instruction**
   (`sdiv`/`srem`/`sext`/`ashr`, …) or **signedness metadata** (a
   `signext`/`zeroext` parameter attribute, or `!range` metadata) — NOT a
   bare typed parameter, which is itself sign-agnostic. `add i32` alone,
   or raw `i32` with no signedness-bearing source, returns
   `Rejected { diagnostic: Diagnostic }` — reading *signed* `Int32` out of
   a sign-neutral operation is fact fabrication. (A sign-agnostic 32-bit
   two's-complement integer is itself a faithful target; only the
   **signed** `IR Int32` needs the extra fact.)

SSA identity grounds as the DAG definition edge. Mutability is not modeled
because LLVM SSA values are not mutable cells.

---

## 11. PTX — SIMT predicate register (lane-indexed truth)

PTX predicate registers are boolean-like values evaluated per lane in a SIMT
execution context. The lane coordinate is part of the fact.

**Model:**

```
// CARRIER — one per-lane record per lane: the lane's truth value and
// whether the lane is within the current execution mask. A SINGLE list
// of per-lane records — value and active flag co-index by construction,
// so a mismatched-length mask is structurally unrepresentable (P2). Two
// independent List<Bool> (lanes / active_mask) would admit a length
// mismatch — an illegal state — and is rejected for that reason.
type PtxLane      = Conj { value: Bool, active: Bool }
type PtxPredicate = List<PtxLane>

// MEANING — a lane-indexed condition; inactive lanes do not assert false,
// they are outside the current execution mask.
```

**Step-by-step coercion shape — `PtxPredicate -> Outcome<IR List<Bool>>`:**
1. predicate grounds to `List<PtxLane>` — each element a `{ value, active }`
   record; value and mask co-index by construction.
2. IR `List<Bool>` grounds only to lane truth values.
3. the coercion fold finds related but non-identical groundings: the
   per-lane `active` flag exists in PTX but not in the plain list.
4. PTX to plain list returns `Produced` only if an explicit accepted-loss
   policy drops the `active` flags, or if the IR target also carries them.
   Plain list to PTX requires the `active` flags supplied per lane;
   otherwise it returns `Rejected { diagnostic: Diagnostic }`.

The model prevents treating inactive lanes as false values. That distinction is
a structural coordinate, not a convention.

---

## 12. `.dag` — `Arrow` declaration (the language modeling itself)

In v2, a function-like declaration is an `Arrow`: parameter facts, result fact,
and a body/realization authority.

**Model:**

```
// CARRIER — a typed mapping plus its authoritative realization.
type DagArrow = Conj {
  params: List<Param>,
  result: TypeNode,
  body:   Body | ExternalRealization
}

// MEANING — a total transform from parameter groundings to result grounding,
// checked by the body or named external realization.
```

**Step-by-step coercion shape — `DagArrow<Int32 -> Int32> -> Outcome<IR FunctionType>`:**
1. params and result ground as type nodes in the shared substrate.
2. the coercion fold matches `Arrow` against function type
   structure.
3. parameter/result types recurse; `Int32` coincides with itself.
4. the signature comparison returns `Produced`; body coercion is not a type
   alias. It returns `Produced` only when the body or external realization is
   the declared authority, otherwise `Rejected { diagnostic: Diagnostic }`.

This is self-grounding without circularity: the carrier is substrate data, and
the meaning is checked through the same structural rules as user code.

---

## 13. JSON — object (unordered named fields)

A JSON object is an unordered mapping from strings to JSON values. Member order
is serialization syntax, not object meaning.

**Model:**

```
// CARRIER — finite map from string keys to recursively grounded JSON values.
// Each recursive-container arm carries its payload, so `JsonValue` is the
// single authority for a JSON value's content (no label-only variant whose
// payload lives in a separate type).
type JsonValue  = Null | Bool | Number | String
                | Array<List<JsonValue>>
                | Object<JsonObject>
type JsonObject = Map<String, JsonValue>

// MEANING — the map. Duplicate source keys are a parse-boundary diagnostic,
// not two object fields.
```

**Step-by-step coercion shape — `JsonObject -> Outcome<IR { name: String, age: Nat }>`:**
1. JSON object grounds to `Map<String, JsonValue>`.
2. IR record grounds to a closed `Conj` with required fields.
3. the coercion fold compares map entries to field
   coordinates by key.
4. every required key plus recursively produced values return `Produced`.
   Missing keys, duplicate keys, or non-natural `age` return
   `Rejected { diagnostic: Diagnostic }`.

Object member order never participates in the coercion. If a consumer needs
source order, that is a separate syntax/provenance fact.

---

## 14. YAML — mapping with anchors (graph-shaped syntax, tree-shaped data)

YAML can express aliases and anchors, so its syntax may be graph-shaped even
when the decoded data model is a mapping/sequence/scalar tree.

**Model:**

```
// CARRIER — resolved YAML node plus optional source anchor identity.
type YamlNode =
  Scalar<YamlScalar>
| Sequence<List<YamlNode>>
| Mapping<Map<YamlNode, YamlNode>>
| Alias<AnchorName>

// MEANING — after alias resolution, a YAML value tree; anchor identity remains
// provenance unless the target asks for graph identity.
```

**Step-by-step coercion shape — `YamlMapping -> Outcome<IR Record>`:**
1. resolve aliases through the declared anchor map; unresolved aliases return
   `Rejected { diagnostic: Diagnostic }`.
2. mapping grounds to a finite map of key/value node groundings.
3. IR record grounds to named coordinates.
4. scalar-string keys matching field names plus recursively produced values
   return `Produced`; non-scalar keys or unresolved aliases return
   `Rejected { diagnostic: Diagnostic }`.

The model does not pretend YAML is JSON. Anchors are grounded syntax facts with
a declared resolution step.

---

## 15. CSV — row (positional fields under a schema)

A CSV row alone is only a list of fields. Names and types come from an external
schema or header row, not from the row bytes themselves.

**Model:**

```
// CARRIER — ordered fields plus the schema that assigns names and decoders.
type CsvRow<S> = Conj {
  fields: List<String>,
  schema: S
}

// MEANING — a record only after schema/header facts decode positions.
```

**Step-by-step coercion shape — `CsvRow<S> -> Outcome<IR { id: Nat, name: String }>`:**
1. row grounds to `List<String>`; schema grounds to field names and decoders.
2. the coercion fold aligns positions to record coordinates
   through the schema.
3. `id` string decodes through `Nat` grammar; `name` remains string.
4. matching arity plus successful decoders return `Produced`; extra, missing,
   or undecodable fields return `Rejected { diagnostic: Diagnostic }`.

Without the schema, a CSV row cannot coerce to a named record. It remains an
ordered string list; an attempted record coercion returns `Rejected`.

---

## 16. TOML — table (typed configuration map)

A TOML table is a named map whose values come from TOML's closed value set:
strings, integers, floats, booleans, dates/times, arrays, and tables.

**Model:**

```
// CARRIER — finite map from dotted keys to closed TOML values. Each
// recursive-container arm carries its payload, so `TomlValue` is the
// single authority for a TOML value's content.
type TomlValue = String | Integer | Float | Bool | DateTime
               | Array<List<TomlValue>>
               | Table<TomlTable>
type TomlTable = Map<KeyPath, TomlValue>

// MEANING — a hierarchical record assembled from key paths.
```

**Step-by-step coercion shape — `TomlTable -> Outcome<IR ConfigRecord>`:**
1. table grounds to a map of key paths to closed value variants.
2. IR config grounds to record coordinates with expected types.
3. the coercion fold aligns dotted key paths to nested
   record fields.
4. existing required paths plus recursively produced TOML variants return
   `Produced`; duplicate paths, conflicting table/value paths, and missing
   required values return `Rejected { diagnostic: Diagnostic }`.

TOML's value variants are a closed sum. Modeling them as strings would erase the
target's typed facts.

---

## 17. JSON Schema — object schema (constraints over JSON values)

JSON Schema does not ground as a JSON object value. It grounds as a predicate
over JSON values, encoded in JSON syntax.

**Model:**

```
// CARRIER — JSON syntax for a closed set of validation keywords.
type JsonSchemaObject = Conj {
  type_keyword: Optional<JsonTypeSet>,
  properties:   Map<String, JsonSchema>,
  required:     Conj {
    names:        List<String>,
    unique_proof: Witness< all_distinct(names) >   // JSON Schema: the `required`
                                                   // array elements MUST be unique
  }
}

// MEANING — a predicate: JsonValue -> Bool, plus witnesses for accepted values.
```

**Step-by-step coercion shape — `JsonSchemaObject -> Outcome<IR RecordType>`:**
1. schema grounds to constraints: allowed JSON kind, property schemas, required
   property names.
2. IR record type grounds to required coordinates and their value groundings.
3. the coercion fold compares required properties to record
   fields and recurses into property schemas.
4. the closed fragment whose constraints exactly determine the record shape
   returns `Produced`; open `additionalProperties` or unconstrained fields
   return `Rejected { diagnostic: Diagnostic }` for record-type coercion and
   remain predicate facts.

The schema is not the data. Its meaning is validation, so coercion produces type
facts only when the predicate is structurally precise enough.

---

## 18. OpenAPI — operation object (HTTP contract)

An OpenAPI operation describes a request/response contract over HTTP. The
operation is not an endpoint implementation; it is a typed boundary declaration.

**Model:**

```
// CARRIER — method/path plus parameter, request-body, response, and status facts.
// `content` (OpenAPI Request Body / Response Object) is a MEDIA-TYPE MAP —
// one schema per media type, with multiple alternatives (e.g. application/json
// AND application/xml) — never a single schema.
type ContentMap = Map<MediaType, Schema>
type OpenApiOperation = Conj {
  method:      HttpMethod,
  path:        PathTemplate,
  parameters:  Conj {
    items:        List<Parameter>,
    unique_proof: Witness< all_distinct_by(items, name_and_location) >
                  // OpenAPI: parameters MUST be unique by (name, `in`) location
  },
  request:     Optional<ContentMap>,
  responses:   Map<HttpStatus, ContentMap>   // status → content map; the keys
                                             // at both levels unique by Map
}

// MEANING — a partial function from grounded HTTP requests to grounded HTTP
// responses, indexed by status code and then by media type.
```

**Step-by-step coercion shape — `OpenApiOperation -> Outcome<IR ServiceArrow>`:**
1. method/path ground to HTTP target facts; parameters ground through location
   facts (`path`, `query`, `header`, `cookie`).
2. request and response schemas ground through their JSON Schema meanings.
3. the coercion fold compares the request side to the IR
   arrow input record and the response map to the IR result sum.
4. every parameter/body/response schema grounding to the corresponding IR type
   returns `Produced`; missing status cases or ungrounded schemas return
   `Rejected { diagnostic: Diagnostic }`.

This keeps OpenAPI as a boundary contract. The implementation body remains a
separate authority, connected only after the contract groundings line up.
