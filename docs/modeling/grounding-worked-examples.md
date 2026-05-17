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
// CARRIER — a SPICE node voltage is an exact physical quantity: a real
// magnitude paired with the volt dimension. It is not a Float64 carrier.
type SpiceVoltage = Conj { magnitude: Real, unit: Volt }

// MEANING — identity: the real IS the voltage.
```

`Real` grounds — as a construction over `Nat`/`Rational` (Cauchy
sequences / Dedekind cuts): still primitives, a richer construction than
a finite bit-vector. `Volt` is a `Dimension`. IEEE-754 `f64` grounds
separately as an `ApproximateField` carrier with rounding and special-value
facts; it is related to `Real`, not identical to it.

**Step-by-step coercion — `SpiceVoltage ↔ Rust f64`:**
1. `SpiceVoltage` grounds to `Real` (continuous, uncountable);
   `f64` grounds to IEEE-754 binary64 (finite — 2⁶⁴ values —
   approximating `Real`; `std/float.dag` `ApproximateField`).
2. catamorphism: exact physical `Real` vs `ApproximateField(binary64)` —
   related but not equal.
3. finite `f64 → SPICE`: **exact** (each finite binary64 value denotes a
   specific real); `NaN` / `±∞` are IEEE-754 special values, not real
   voltages, so they **fail-closed** rather than being erased.
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
// constraint "length = n" grounds as a Witness (see src/v3/std/dimensions.dag),
// the substrate's proof-carrier.
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
decode. "negative forty-two" decodes to the math integer `−42`, which
coincides with IR `Int` value `−42` → coercion = **identity**. A number
can be ingested from English text into the IR.

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
each". A construction whose connective cannot be determined →
**fail-closed**.

### The fail-closed boundary — words under-determine structure

This is why English's groundable subset is an *island* and the
fail-closed boundary is large — **not** because the connective words are
missing, but because the words **under-determine** the structure.
"this and that" may be a `Conj` or a `Disj`; "or" may be exclusive or
inclusive; the word alone cannot say. Leaf phrases fail the same way —
"forty-two-ish", "about forty-two" have no deterministic decode;
"ship it when the build feels solid" grounds nowhere.

When the construction *is* unambiguous — a disciplined, controlled
English written to a fixed structural convention — English grounds
composite structure honestly, recursively, by the same `Node`
catamorphism. Free prose under-determines and fail-closes. Model what
determinately grounds; fail-close the rest. (This is the
`english_ingest_fail_closed.dag` boundary-honesty probe v4 already
plans.) English is not a degenerate case; it is a full language whose
words under-determine structure, hence a large, honest fail-closed
boundary.

---

## 7. Go — `chan T` (typed communication endpoint)

Go channels are typed FIFO communication endpoints. Their value model is not
"a list"; the channel value is a capability to send and/or receive values of
`T`, with ordering and blocking behavior supplied by the Go spec.

**Model:**

```
// CARRIER — a channel value is an endpoint over a FIFO stream of T.
// Direction is a closed target fact: send-only, receive-only, or bidirectional.
type GoChan<T> = Conj {
  element:   TypeGrounding<T>,
  direction: SendOnly | ReceiveOnly | Bidirectional,
  capacity:  Nat
}

// MEANING — ordered communication of T values, with blocking behavior derived
// from capacity and direction. Scheduling is an effect fact, not a payload fact.
```

**Step-by-step coercion — `chan int32` ↔ IR `Stream<Int32>` endpoint:**
1. `chan int32` grounds to endpoint facts plus `grounding(int32)`.
2. catamorphism: endpoint vs endpoint → match; recurse into element.
3. `int32` vs `Int32` coincide when both decode as 32-bit two's-complement.
4. payload coercion is **identity**; endpoint coercion requires the IR endpoint
   to carry the same direction and capacity facts. Missing direction/capacity
   facts fail-closed rather than defaulting to bidirectional or unbuffered.

The concurrency semantics are not annotations. They are grounded target facts
attached to the endpoint and consumed by effect/scheduling lenses.

---

## 8. C++ — `std::vector<T>` (template container with allocator facts)

`std::vector<T>` is a contiguous, finite, ordered sequence. Allocator and
capacity affect realization and cost; the observable value is the ordered
elements.

**Model:**

```
// CARRIER — finite ordered sequence of T. Contiguity and allocator are
// realization/cost facts, not extra value fields.
type CppVector<T> = Conj {
  elements: List<T>,
  allocator: AllocatorModel
}

// MEANING — identity on elements; allocator grounds allocation behavior.
```

**Step-by-step coercion — `std::vector<int32_t>` ↔ IR `List<Int32>`:**
1. outer carrier grounds to `List<grounding(int32_t)>` plus allocator facts.
2. catamorphism matches `List` against `List`; allocator has no IR value role.
3. `int32_t` vs `Int32` coincide only when the target model proves width and
   representation from the C++ implementation/spec binding.
4. value coercion is **identity** when the element grounding coincides; allocator
   facts remain target realization facts, not hidden list semantics.

If the target model cannot prove `int32_t` width for a platform binding, the
coercion fails closed. It does not silently assume 32 bits.

---

## 9. TypeScript — `A | B` (structural union)

TypeScript unions are closed alternatives at a use site, checked against the
structural shapes of their members.

**Model:**

```
// CARRIER — a value inhabits exactly one member grounding that accepts it.
type TsUnion<A, B> = Disj {
  left:  A,
  right: B
}

// MEANING — the member's own meaning, tagged by the successful alternative.
```

**Step-by-step coercion — `number | string` ↔ IR `Float64 | String`:**
1. TypeScript `number` grounds to IEEE-754 binary64; `string` grounds to a
   Unicode scalar sequence.
2. IR alternatives ground independently: `Float64`, `String`.
3. catamorphism matches outer `Disj`; then compares each alternative.
4. `number` ↔ `Float64` is **identity**; `string` ↔ `String` is identity only
   if the string models share Unicode normalization facts. Otherwise that arm is
   a named lossy or fail-closed gap.

The union coercion is derived from its arms. No target-specific "union bridge"
is authored.

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

**Step-by-step coercion — `i32 %x` ↔ IR `Int32`:**
1. LLVM `i32` grounds to 32 bits interpreted by the consuming operation.
2. an integer operation such as `add i32` supplies two's-complement integer
   meaning for the same 32 bits.
3. catamorphism compares `32×Bool + integer decode` to IR `Int32`.
4. when the consuming operation fixes integer meaning, coercion is **identity**;
   without that use-site meaning, raw `i32` is only a bit-vector and integer
   coercion is not silently invented.

SSA identity grounds as the DAG definition edge. Mutability is not modeled
because LLVM SSA values are not mutable cells.

---

## 11. PTX — SIMT predicate register (lane-indexed truth)

PTX predicate registers are boolean-like values evaluated per lane in a SIMT
execution context. The lane coordinate is part of the fact.

**Model:**

```
// CARRIER — one classical truth value per active lane.
type PtxPredicate = Conj {
  lanes:       List<Bool>,
  active_mask: List<Bool>
}

// MEANING — a lane-indexed condition; inactive lanes do not assert false,
// they are outside the current execution mask.
```

**Step-by-step coercion — `PtxPredicate` ↔ IR `List<Bool>`:**
1. predicate grounds to `{ lanes: List<Bool>, active_mask: List<Bool> }`.
2. IR `List<Bool>` grounds only to lane truth values.
3. catamorphism finds related but non-identical groundings: the mask coordinate
   exists in PTX but not in the plain list.
4. PTX → plain list is **lossy** unless the IR target also carries the mask;
   plain list → PTX requires a supplied active mask, otherwise fail-closed.

The model prevents treating inactive lanes as false values. That distinction is
a structural coordinate, not a convention.

---

## 12. `.dag` — `Arrow` declaration (the language modeling itself)

In v4, a function-like declaration is an `Arrow`: parameter facts, result fact,
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

**Step-by-step coercion — `.dag Arrow<Int32 -> Int32>` ↔ IR function type:**
1. params and result ground as type nodes in the shared substrate.
2. catamorphism matches `Arrow` against function type structure.
3. parameter/result types recurse; `Int32` coincides with itself.
4. the signature coercion is **identity**; body coercion is not a type alias. It
   is valid only when the body or external realization is the declared authority.

This is self-grounding without circularity: the carrier is substrate data, and
the meaning is checked through the same structural rules as user code.

---

## 13. JSON — object (unordered named fields)

A JSON object is an unordered mapping from strings to JSON values. Member order
is serialization syntax, not object meaning.

**Model:**

```
// CARRIER — finite map from string keys to recursively grounded JSON values.
type JsonValue  = Null | Bool | Number | String | Array<JsonValue> | Object
type JsonObject = Map<String, JsonValue>

// MEANING — the map. Duplicate source keys are a parse-boundary diagnostic,
// not two object fields.
```

**Step-by-step coercion — JSON object ↔ IR record `{ name: String, age: Nat }`:**
1. JSON object grounds to `Map<String, JsonValue>`.
2. IR record grounds to a closed `Conj` with required fields.
3. catamorphism compares map entries to field coordinates by key.
4. coercion succeeds only when every required key exists and each value
   recursively coerces; missing keys, duplicate keys, or non-natural `age`
   fail-closed.

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

**Step-by-step coercion — YAML mapping ↔ IR record:**
1. resolve aliases through the declared anchor map; unresolved aliases fail.
2. mapping grounds to a finite map of key/value node groundings.
3. IR record grounds to named coordinates.
4. coercion succeeds when keys are scalar strings matching field names and
   values recursively coerce; non-scalar keys or unresolved aliases fail-closed.

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

**Step-by-step coercion — CSV row ↔ IR record `{ id: Nat, name: String }`:**
1. row grounds to `List<String>`; schema grounds to field names and decoders.
2. catamorphism aligns positions to record coordinates through the schema.
3. `id` string decodes through `Nat` grammar; `name` remains string.
4. coercion succeeds when arity matches and every decoder succeeds; extra,
   missing, or undecodable fields fail-closed.

Without the schema, a CSV row cannot coerce to a named record. It remains an
ordered string list.

---

## 16. TOML — table (typed configuration map)

A TOML table is a named map whose values come from TOML's closed value set:
strings, integers, floats, booleans, dates/times, arrays, and tables.

**Model:**

```
// CARRIER — finite map from dotted keys to closed TOML values.
type TomlValue = String | Integer | Float | Bool | DateTime | Array<TomlValue> | Table
type TomlTable = Map<KeyPath, TomlValue>

// MEANING — a hierarchical record assembled from key paths.
```

**Step-by-step coercion — TOML table ↔ IR config record:**
1. table grounds to a map of key paths to closed value variants.
2. IR config grounds to record coordinates with expected types.
3. catamorphism aligns dotted key paths to nested record fields.
4. coercion succeeds when each required path exists and each TOML variant
   recursively matches; duplicate paths, conflicting table/value paths, and
   missing required values fail-closed.

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
  required:     List<String>
}

// MEANING — a predicate: JsonValue -> Bool, plus witnesses for accepted values.
```

**Step-by-step coercion — JSON Schema object ↔ IR record type:**
1. schema grounds to constraints: allowed JSON kind, property schemas, required
   property names.
2. IR record type grounds to required coordinates and their value groundings.
3. catamorphism compares required properties to record fields and recurses into
   property schemas.
4. coercion to an IR record type succeeds only for the closed fragment whose
   constraints exactly determine the record shape; open `additionalProperties`
   or unconstrained fields remain predicate facts, not record coordinates.

The schema is not the data. Its meaning is validation, so coercion produces type
facts only when the predicate is structurally precise enough.

---

## 18. OpenAPI — operation object (HTTP contract)

An OpenAPI operation describes a request/response contract over HTTP. The
operation is not an endpoint implementation; it is a typed boundary declaration.

**Model:**

```
// CARRIER — method/path plus parameter, request-body, response, and status facts.
type OpenApiOperation = Conj {
  method:      HttpMethod,
  path:        PathTemplate,
  parameters:  List<Parameter>,
  request:     Optional<MediaTypedSchema>,
  responses:   Map<HttpStatus, MediaTypedSchema>
}

// MEANING — a partial function from grounded HTTP requests to grounded HTTP
// responses, indexed by status code and media type.
```

**Step-by-step coercion — OpenAPI operation ↔ IR service arrow:**
1. method/path ground to HTTP target facts; parameters ground through location
   facts (`path`, `query`, `header`, `cookie`).
2. request and response schemas ground through their JSON Schema meanings.
3. catamorphism compares the request side to the IR arrow input record and the
   response map to the IR result sum.
4. coercion succeeds when every parameter/body/response schema grounds to the
   corresponding IR type. Missing status cases or ungrounded schemas fail-closed.

This keeps OpenAPI as a boundary contract. The implementation body remains a
separate authority, connected only after the contract groundings line up.
