# v4 Grounding — Worked Examples

> Companion to the **D2 reversal + fact-bundle reseed plan** (`src/v4/DECISIONS.md`).
> PLAN-ONLY: these demonstrate the modeling *shape*; no substrate is edited.

## The model, in one paragraph

An `extdeps` target model is a **demonstration** that each of the target's
types **grounds** — decomposes, via the concept DAG, into a subset of the
universal substrate primitives (`Bool`, `Nat`, the connectives, `Node`). A
type is two things: a **carrier** (the data) and a **meaning** (a `Node`
decode of what the data denotes). Coercion between any two types is
**derived, not authored**: the `Node` catamorphism compares their
groundings — coincide → identity; related → the derived map (exact or
lossy); unrelated → fail-closed. A type that cannot be grounded is a
*named* fail-closed gap, never a silent one.

This is *not* the reversed D2 alias. D2 *asserted* `type RustI32 = Int32`.
Here every type is grounded *independently* from its own spec, and any
identity is **discovered** by comparing groundings — and discovered to be
true only because both genuinely decode the same way.

## Spectrum

These examples are chosen to span the full range deliberately —
`machine_code` (pure bits) → `rust` (typed, generic) → `verilog`
(4-valued logic) → `spice` (continuous) → `lean` (dependent types) →
`english` (natural language). One model handling all of them is the
evidence that it is universal.

---

## 1. Rust — `Vec<T>` (generics, compound coercion)

**Model** — grounded from the Rust Reference / `std` docs:

```
// CARRIER — observationally a Vec<T> value IS a finite ordered sequence
// of T. (pointer/capacity are allocation facts → cost lens, not the
// value model.) A finite ordered sequence grounds to List = FreeMonoid
// over the connectives; length is a Nat.
type RustVec<T> = List<T>          // parametric: grounds to List<grounding(T)>

// MEANING — identity on the element sequence.
```

`RustVec<RustI32>` grounds to `List<{ 32×Bool + twos_complement_decode }>`
— fully primitive, recursively.

**Step-by-step coercion — `RustVec<RustI32>` ↔ IR `List<Int32>`:**
1. groundings: `List<{32×Bool, decode}>` vs `List<{32×Bool, decode}>`.
2. catamorphism: outer `List` vs `List` → match; recurse into element.
3. element `grounding(RustI32)` vs `grounding(Int32)` → coincide.
4. both levels coincide → coercion = **identity**. No `Vec`-coercion was
   authored — `List` matched structurally and the element matched;
   `Vec<i32>` fell out by the catamorphism recursing. Compound coercion,
   free.

**Cross-language — `RustVec<i32>` ↔ `PythonList[int]`:** outer `List`
matches; element `{32-bit two's-complement}` vs `{arbitrary-precision}`
does **not** coincide. Derived element map is asymmetric — Rust→Python
exact (lossless widening), Python→Rust **fail-closed** on any value
outside `−2³¹..2³¹−1`. Read off the groundings, never authored.

---

## 2. machine_code — a 64-bit general-purpose register (the pure-bits endpoint)

**Model:**

```
// CARRIER — a register value is exactly 64 classical bits.
type Reg64 = List<Bool>            // |bits| = 64

// MEANING — none intrinsic. A raw register is an uninterpreted bit
// pattern; meaning is supplied by the instruction that consumes it
// (the same 64 bits are an integer to ADD, an address to LOAD, a
// float to ADDSD). Reg64 grounds carrier-only; meaning is deferred
// to the consumer.
```

This is the spectrum's trivial endpoint — `machine_code` *is* bits, so
the grounding is direct, no decode.

**Coercion:** `Reg64 ↔ List<Bool>` is identity; when an instruction
consumes the register, the register coerces to whichever IR carrier
that instruction's grounding selects. There is no "register type"
mismatch to catch — the bits are the bits.

---

## 3. Verilog — `reg [31:0]` (a target primitive *richer* than ours)

Verilog's logic value is **4-valued**: `{0, 1, x (unknown), z (high-Z)}`
— not our 2-valued `Bool`.

**Model:**

```
// CARRIER — 32 four-state bits.
type VBit   = Zero | One | Unknown | HighZ   // closed 4-sum (a coproduct)
type VReg32 = List<VBit>                     // |bits| = 32

// MEANING — the 32 four-state bits; when used as a number, two's-
// complement over the {0,1} bits, undefined if any bit is x/z.
```

`VBit` grounds — as a closed sum over the substrate's coproduct
connective. It does **not** ground as `Bool`: Verilog's primitive is
richer. `Bool` is exactly the `{Zero, One}` sub-part of `VBit`.

**Step-by-step coercion — `VReg32 ↔ IR Int32`:**
1. IR `Int32` grounds to `32×Bool` (2-valued); `VReg32` to `32×VBit`
   (4-valued).
2. catamorphism, element-wise: `Bool` vs `VBit` — `Bool ⊊ VBit`.
3. IR → Verilog: **exact** (every `Bool` is a `VBit`).
4. Verilog → IR: **partial** — a `VReg32` whose 32 bits are all in
   `{Zero, One}` coerces; any bit `= Unknown/HighZ` → **fail-closed**
   (x/z is not an integer state).

This is "speaks a *subset*" made precise: Verilog and the IR share the
`{0,1}` subset; the `x/z` states are Verilog-only and honestly
fail-closed. The model handles a target whose primitive is richer than
ours without distortion.

---

## 4. SPICE — a node voltage (the continuous endpoint)

**Model:**

```
// CARRIER — a node voltage is a real number, in volts.
type SpiceVoltage = Conj { magnitude: Real, unit: Volt }

// MEANING — identity: the real IS the voltage.
```

`Real` grounds — as a construction over `Nat`/`Rational` (Cauchy
sequences / Dedekind cuts): still primitives, a richer construction than
a finite bit-vector. `Volt` is a `Dimension`.

**Step-by-step coercion — `SpiceVoltage ↔ Rust f64`:**
1. `SpiceVoltage` grounds to `Real` (continuous, uncountable);
   `f64` grounds to IEEE-754 binary64 (finite — 2⁶⁴ values —
   approximating `Real`; `std/float.dag` `ApproximateField`).
2. catamorphism: `Real` vs `binary64` — related but not equal.
3. `f64 → SPICE`: **exact** (every `f64` IS a specific real).
4. `SPICE → f64`: **lossy** — rounding; most reals are not
   representable. A *declared* loss, surfaced, not silent.

The continuous endpoint, and the middle coercion case: neither identity
nor fail-closed but a **declared-lossy** derived map.

---

## 5. Lean — `Vector α n` (dependent types)

A Lean type can depend on a *value*. `Vector α n` is a list of `α` whose
length `n` is part of the type itself.

**Model:**

```
// CARRIER + WITNESS — the dependent index n is a Nat; the type-level
// constraint "length = n" grounds as a Witness (std/witness.dag), the
// substrate's proof-carrier.
type LeanVector<A> = Conj {
  elements:     List<A>,
  length_proof: Witness< |elements| = n >     // n : Nat, lifted into the type
}
```

A dependent type = carrier + a `Witness` pinning the dependent value.

**Step-by-step coercion — `LeanVector<Int>(n=3) ↔ IR List<Int>`:**
1. grounds to `List<grounding(Int)>` + `Witness(length = 3)`.
2. an IR `List<Int>` of statically-known length 3 → the `Witness`
   holds → coercion = **identity**.
3. an IR `List<Int>` of unknown / other length → the `Witness` cannot
   be discharged → **fail-closed** (you cannot claim `Vector Int 3`
   without the length proof).

Dependent types ground via the `Witness` substrate — the type's proof
obligation becomes a `Witness` the coercion must discharge.

---

## 6. English — an integer noun phrase (the fail-closed endpoint)

English has no type system. What we model is the **groundable fragment**,
and we show the rest fail-closed.

**Model:**

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
decode.

**Coercion:** "negative forty-two" decodes to the math integer `−42`,
which coincides with IR `Int` value `−42` → coercion = **identity**. A
number can be ingested from English text into the IR.

**The fail-closed half (the honest part):**
- "forty-two-ish", "about forty-two", "forty-two, give or take" — the
  qualifiers have no deterministic decode → **fail-closed**. The model
  refuses; it does not invent a value.
- "ship it when the build feels solid" — "feels solid", intent, context
  — none ground → the whole sentence fail-closes.

English "speaks our primitives" only on closed, decodable islands
(numbers, booleans, a closed command vocabulary). Everything else is
fail-closed — and that refusal *is* the model being honest. (This is the
`english_ingest_fail_closed.dag` boundary-honesty probe v4 already
plans.)

---

## Remaining targets — fan-out

The same shape (one complex type · the model as carrier + meaning ·
step-by-step coercion) applies to the remaining targets, fanned out via
the `extdeps` lane against the template above:

- **Languages:** `go` (e.g. `chan T` — concurrency/effect as a grounded
  fact), `cpp` (`std::vector<T>` / template), `typescript` (union type
  `A | B` — structural + sum grounding), `llvm_ir` (an SSA value),
  `ptx` (a SIMT register), `dag` (the v4 language modeling itself).
- **Formats:** `json` (an object), `yaml`, `csv` (a row), `toml` (a
  table), `json_schema`, `openapi`.

Each lands here, plan-only, before any substrate is reseeded.
