> Part of: [THESIS.md](../../THESIS.md) > [ROADMAP.md](../../ROADMAP.md) > **Tier 1 + Tier 3** (algebraic laws)

# gunbc Compiler Laws

This document describes the compiler's structural laws, coercion model,
and execution lanes. See [ROADMAP.md](../../ROADMAP.md) for milestone plan
and [INVARIANTS.md](../../INVARIANTS.md) for enforcement.

## The pipeline law

A `String` is legitimate when it carries text. A `String` is wrong
when it chooses behavior. The rule, stage by stage:

| Stage | String rule |
|---|---|
| **Before resolve** | Strings are allowed as source payload (token text, identifiers, keywords) |
| **At resolve** | Strings are consumed to produce edges. Scope maps are resolver-local and die here. |
| **After resolve** | **No semantic decision may depend on free text.** Anything that changes behavior must be an edge/reference, a closed enum, or a typed boundary fact. |
| **In emit** | Only the renderer may produce strings. Shared emit walks the graph and invokes the renderer — it never returns `String`. |

This is the existing invariants restated as an API law: names are
opaque, boundaries must be sufficient, heuristics mean a fact was lost.

## What's legitimate as String

- Token.text, file paths, module paths (source payload)
- Identifiers during parse, before resolution (frontier artifact)
- Diagnostic messages (human-facing payload)
- TextFile.content and LanguageSpec templates (final rendered text)
- Resolver-local scope maps that die at the resolution boundary

These are text. They don't choose behavior.

## What must stop being String

| What | Current | Target | Sites |
|---|---|---|---|
| `Node.name` | String field, read everywhere | Deleted. Identity = the node. Text from `source_text_at(span)`. | 1,175 reads |
| `MethodSemantics.method_name` | String, re-dispatched in infer/complexity/emit | Edge to algebra method node in `std/algebra.dag` | 63 dispatch sites |
| `VarBindingKind.parent_enum` | String | Edge to parent type node | Scattered |
| `MatchPattern.Bind.name` | String | Child node with span | Scattered |
| `kernel_types` / `container_types` | `List<String>` | `List<Node>` — edges to definitions | 2 lists |
| `variant_to_enum`, `field_type_names` | `Map<String, String>` | Edges on type definition nodes | 4 maps |
| `builtin_function_registry` | `Map<String, Node>` | Loaded from algebra `.dag` declarations | 30 entries |
| Shared emit return type | `String` (680 concat calls) | Graph walker -> renderer (no string return) | 680 sites |
| Transport dispatch | `transport.name == "rest"` | Closed enum on transport node | 13 sites |
| Optional hardcoding | `variant_name == "Some"` | Structural Optional with known layout | 6 sites |

Not every bad String becomes a Node. Some become edges to nodes, some
become closed enums, some become renderer-local spec text. The question
is never "string or DAG?" — it's "does this string carry text, or does
it choose behavior?"

## Three bypasses to defend against

Even after cleanup, these can reopen the escape hatches:

**1. `source_text_at(span)` as a semantic API.** Deleting `Node.name`
is not enough if infer/complexity/ownership re-read text from spans.
After resolve, source text must be a privileged capability available
only to renderers and diagnostics — not to semantic stages.

**2. String return types on emit functions.** The emitter must not
return `String`. It returns the typed graph; the renderer produces
text. No intermediate string-producing IR is needed because the
compiler's output IS the graph.

**3. "Temporary" stringly side tables.** `Map<String, X>` looks
harmless, but once it crosses a stage boundary it becomes a second
authority. The invariants say: speculative or lossy boundary fact
tables should be deleted rather than carried forward. "Temporary"
without a ratchet means "permanent later."

## Backend model: graph coercion

The backend is a graph->graph coercion engine. The graph structure
tells the compiler WHAT kind of coercion is needed. It does NOT always
tell the compiler HOW to discharge the proof — some steps need
evidence or explicit transforms.

```
typed graph -> carrier + constraint analysis -> coercion plan -> target-basis graph -> renderer
```

Note: "subgraph" is too literal as the universal test. The real
relation is **carrier compatibility + constraint entailment** — same
carrier with fewer guarantees (widen), same carrier with more
guarantees (refine/validate), or different carrier entirely
(transform/lower).

### Five coercion kinds with witnesses

| Kind | What happens | Witness | Implicit? | Example |
|---|---|---|---|---|
| **Widen** | Erase guarantees | `Free` | Yes | `Url -> String` (predicate erasure) |
| **Refine** | Add guarantees already proven by structure | `Proven` | Yes | `NonEmpty<List<T>>` -> compiler sees non-empty evidence |
| **Validate** | Add guarantees by runtime check | `Checked` | No — explicit `as` or inserted check, visible in receipt | `String -> Url` (needs URL validator) |
| **Project** | Lose structural information | `Lossy` | No — explicit acknowledgment | `List -> Set` (loses order), `Float -> Int` (truncate) |
| **Transform/Lower** | Compute new representation | `Transformed` | No — `.dag` process or plugin lowering | `Celsius -> Fahrenheit` (domain), coproduct -> SPICE mux (backend) |

**Key distinctions:**

- **Upcasts (widen) are implicit.** Free, no evidence needed.
- **Refinements with proof already present are implicit.** The graph
  carries the evidence; the compiler reads it.
- **Downcasts (validate) are NOT generally implicit.** Only the subset
  already proven by structure is implicit. Everything else needs an
  explicit check — otherwise we reintroduce hidden fabrication.
- **Transform and lower are separate concerns that share a mechanism.**
  `Celsius -> Fahrenheit` is a user-space domain transform.
  `coproduct -> SPICE mux` is a backend lowering. Both need authored
  `.dag` processes, but they're not the same category — domain
  transforms live in user/library code, backend lowerings live in
  language plugins.

**Key insight: most domain transforms dissolve with proper modeling.**
If Celsius and Fahrenheit both model absolute temperature through
Kelvin (a shared structural base), the compiler finds the path:

```
Celsius -> Kelvin    (widen/refine: same carrier, offset transform)
Kelvin -> Fahrenheit (widen/refine: same carrier, scale + offset)
```

Both legs have proven witnesses. The compiler finds the path through
the type graph. If you find yourself writing an explicit transform,
your types may be under-modeled: there's a shared base you haven't
declared.

### Worked examples

```
WIDEN (implicit, Free):
  Url -> String -- predicate erasure. Same carrier, fewer guarantees.

REFINE (implicit if proven, Proven):
  xs |> filter(non_empty) |> first -> NonEmptyStr
  Compiler sees: filter guarantees non-empty. Evidence: Proven.

VALIDATE (explicit, Checked):
  String -> Url -- needs URL validator. Compiler shows: "String lacks
  scheme/host constraints. Insert url_parse(s) or use as_url(s)."
  The check is visible in the guarantee receipt.

PROJECT (explicit, Lossy):
  List -> Set -- compiler shows: "target lacks position, multiplicity.
  Use to_set(list). Acknowledge: order lost, duplicates removed."

TRANSFORM (explicit .dag process, Transformed):
  Celsius -> Fahrenheit -- through Kelvin shared base if modeled.
  Otherwise: user writes fn to_fahrenheit(c: Celsius) -> Fahrenheit.

LOWER (plugin .dag process, Transformed):
  coproduct -> SPICE mux -- language plugin declares the lowering.
  Lives in dsl/extdeps/languages/spice/coerce.dag, not in user code.
```

**The guarantee receipt records every non-Free step:** kind, witness
type, what evidence was used (or what check was inserted, or what
process was invoked). No silent coercions. The receipt is the audit
trail.

**For language targets, the same model applies:**

A language declares its **basis** — which structural patterns it can
represent natively. The compiler compares each source graph segment
against the basis:

| Source pattern | Target basis has it? | What happens |
|---|---|---|
| Product | Rust: struct | **Identity** — same structure, different syntax |
| Coproduct | Rust: enum | **Identity** |
| Coproduct | SPICE: no native tagged union | **Isomorphism** — mux from switches is structurally equivalent (one-of-N selection) |
| Cardinality | Verilog: tri-state | **Identity** |
| Function | English: paragraph | **Identity** |

When the source pattern IS in the target's basis -> identity (free).
When it's NOT -> the compiler looks for a structural isomorphism in
the target's basis (e.g., coproduct <-> mux: both are one-of-N
selection). The language plugin declares these isomorphisms in
`coerce.dag`. The compiler doesn't guess — it follows declared
structural equivalences.

**Why this avoids duplicate representations:** the graph IS the type
relationship. Widening, narrowing, and projection are structural
observations — the compiler reads them from the graph. Isomorphisms
through shared bases are structural paths — the compiler finds them.
Only language-specific structural equivalences (coproduct <-> mux)
need to be declared, and those are properties of the target domain,
not rules duplicated from the source types.

**Rendering happens AFTER coercion.** The renderer walks the
target-basis graph (already in native patterns) and produces text.
Trivial. All intelligence is in the structural comparison (automatic)
and the sidecast processes (authored in `.dag`).

**The guarantee receipt records the coercion plan:** for each graph
segment, what direction (upcast/downcast/sidecast), what cost, what
the sidecast process was (if any). Deterministic and auditable.

## End-to-end example: aspiration target

A non-trivial `.dag` program compiled to three targets, showing how
every piece of the roadmap connects. This is what the system looks
like when it works.

```dag
// Source: a temperature converter service
module weather.convert

import std.types { Float, String }

// Kelvin is the shared structural base for temperature
type Kelvin = Float where label("K") where range(0.0, max_float)

// Celsius and Fahrenheit are isomorphic views of Kelvin
// The compiler finds the path: Celsius -> Kelvin -> Fahrenheit
type Celsius = Kelvin where offset(-273.15) where label("C")
type Fahrenheit = Kelvin where scale(9.0/5.0) where offset(-459.67) where label("F")

// No explicit conversion functions needed -- the compiler derives
// Celsius -> Fahrenheit through Kelvin (both legs are pure isomorphisms)

type TemperatureReading {
  value: Celsius
  location: String
  timestamp: Int
}

type TemperatureReport
  = SingleReading { reading: TemperatureReading }
  | DailyAverage { readings: List<TemperatureReading>, avg: Celsius }
  | Error { message: String }
```

**What the compiler proves (guarantee receipt):**

```json
{
  "discovered": { "types": 4, "functions": 2 },
  "structural": {
    "decidability": "proven (all functions terminate)",
    "complexity": {
      "to_fahrenheit": "O(1) Proven",
      "to_celsius": "O(1) Proven"
    },
    "ownership": {
      "to_fahrenheit": "SoleOwner (all bindings consumed once)",
      "to_celsius": "SoleOwner"
    }
  },
  "coercion_plan": {
    "Celsius -> Float": "widening, free (Celsius refines Float)",
    "Float -> Celsius": "narrowing, needs validation (range + offset)",
    "Celsius -> Fahrenheit": "isomorphism via Kelvin (compiler-derived, pure)"
  }
}
```

**Compiled to Rust (identity coercion — all patterns native):**

```rust
// Every structural pattern maps directly:
//   Product -> struct, Coproduct -> enum, Function -> fn
//   Cardinality -> Option, Sequence -> let bindings
struct TemperatureReading { value: f64, location: String, timestamp: i64 }
enum TemperatureReport {
    SingleReading { reading: TemperatureReading },
    DailyAverage { readings: Rc<Vec<TemperatureReading>>, avg: f64 },
    Error { message: String },
}
// to_fahrenheit derived by compiler: celsius + 273.15 then * 9/5 - 459.67
fn celsius_to_fahrenheit(c: f64) -> f64 { c * 9.0 / 5.0 + 32.0 }
```

**Compiled to SPICE (sidecast for coproduct — mux from switches):**

```spice
* TemperatureReading: subcircuit with 3 ports (product)
.subckt TemperatureReading value location timestamp
.ends

* TemperatureReport: 3-way mux (coproduct -> synthesized from switch)
* Selector signal chooses which variant is active
.subckt TemperatureReport sel_0 sel_1 reading_port avg_port err_port
V_mux_ctrl sel_0 sel_1 DC 0
.ends

* to_fahrenheit: subcircuit (function -> subcircuit)
.subckt to_fahrenheit c_in f_out
E_scale f_out 0 VALUE={V(c_in)*9.0/5.0+32.0}
.ends
```

**Compiled to English (identity — all patterns have natural mappings):**

```markdown
## Temperature Reading
A temperature reading has:
- a **value** in Celsius
- a **location** (text)
- a **timestamp** (integer)

## Temperature Report
A temperature report is one of:
- a **single reading** containing one temperature reading
- a **daily average** containing a list of readings and an average
- an **error** with a message

## Conversions
- To convert Celsius to Fahrenheit: multiply by 9/5 and add 32.
- To convert Fahrenheit to Celsius: subtract 32 and multiply by 5/9.
```

**What the tests verify (per M3 test tracks):**

| Track | What it checks for this example |
|---|---|
| Discovery | All 3 targets discovered and compiled with 0 diagnostics |
| Behavioral (type roundtrip) | `TemperatureReading` construct -> serialize -> deserialize -> equal |
| Behavioral (function) | `to_fahrenheit(100.0) == 212.0`, `to_celsius(32.0) == 0.0` |
| Edge contracts | `Celsius -> to_fahrenheit -> Fahrenheit`: sidecast process exists, types match |
| Coercion correctness | Widening `Celsius -> Float` is free. Narrowing `Float -> Celsius` requires validation. Isomorphism `Celsius -> Fahrenheit` via Kelvin is compiler-derived. |
| Differential/parity | All 3 targets produce structurally equivalent output |
| Guarantee receipt | Receipt matches expectations, no `report_only` gaps |

## Execution lanes

### Lane A: Definition-edge dispatch (M2/M4) — LANDED

**Status:** Merged to main (PR #242). Transport dispatch uses structural
property detection (no enum). Method dispatch carries the algebra
definition node through `AlgebraMethodSemantics.method_def: Node`.
Cost shape table reformatted as data declaration. Builtin registry is
an acknowledged bridge (deletion point: method-syntax conversion PR).
`extern fn` syntax deleted.

| Change | What was done |
|---|---|
| Transport dispatch | Structural predicates (`is_rest_transport` etc.) based on config properties. Transport extdep .dag files added (RFC 9110, POSIX). |
| Method dispatch | `AlgebraMethodSemantics` carries `method_def: Node` (the algebra field definition). `MethodFieldResult` returns both field node + result type from single lookup. |
| Cost shapes | `method_cost_shape_table` reformatted as data declaration. |
| Builtin registry | Acknowledged bridge. Extern fn syntax deleted. Deletion point: convert ~260 standalone calls to method syntax. |
| **Remaining** | Emit still reads `method_def.name` for per-language rendering (Lane C). Builtin registry still exists (cleanup PR). |

### Lane B: Node.name deletion (M4/D6) — IN PROGRESS

**Status:** PR #244. Infrastructure complete, rendering reads migrated.
Node.name field deletion blocked by synthetic node identity (M4).

| Phase | Status | What was done |
|---|---|---|
| B0 | DONE | `source_text_at(source, span)` + test proving span→text recovery |
| B1 | DONE | Tuple field constants centralized, module/import markers moved to property values |
| B2 | DONE | `source_text` threaded through InferScope + TypeEnv + ResolvedModule + TypedModule |
| B3 | REVERTED | Emit rendering reads migrated then reverted: parser item spans point to keyword tokens, not identifiers. Needs identifier span stored separately. |
| B4 | REVERTED | Resolve type lookups migrated then reverted: same span issue as B3. |
| B5 | BLOCKED | Delete Node.name field. Blocked by ~70 synthetic nodes with zero spans (kernel types, algebra methods). Needs M4 type constructor dissolution. |
| **Remaining** | | ~463 `.name` refs: 256 constructions (B5), 131 accessor calls (B5), ~76 synthetic/coupled reads (M4) |

### Lane C: Coercion engine + language plugin extraction (M5)

The compiler stops producing strings. `05_emit.dag` becomes the
coercion engine — graph->graph transformation using target-declared
rules. Language-specific code moves out of `src/v1/` into
`dsl/extdeps/languages/` as coercion rule sets + renderers.

| File | Change | Sites closed |
|---|---|---|
| `src/v1/05_emit.dag` | Coercion engine: match patterns, apply rules, produce target-basis graph | ~77 concat -> coercion search |
| `src/v1/05_emit_rust.dag` | **DELETE** — Rust coercion rules + renderer move to plugin | 4,121 lines, 309 language mentions |
| `src/v1/05_emit_python.dag` | **DELETE** — Python coercion rules + renderer move to plugin | 1,349 lines, 96 language mentions |
| `src/v1/05_emit_go.dag` | **DELETE** — Go coercion rules + renderer move to plugin | 1,387 lines, 84 language mentions |
| `src/v1/runtime_rust.dag` | **DELETE** — Rust runtime moves to extdep | 5 language mentions |
| `dsl/extdeps/languages/rust/coerce.dag` | **NEW** — Rust coercion rules (graph patterns -> Rust-basis patterns) | Single file |
| `dsl/extdeps/languages/rust/render.dag` | **NEW** — Rust renderer (target-basis graph -> text, trivial) | Single file |
| Same for python/, go/, verilog/, spice/, english/ | Coercion rules + renderer per target | |
| **Total** | | **6,857 lines + 632 language mentions removed from compiler core** |

### Lane D: Edge-only fact references (M5)

Replace `List<String>` and `Map<String, X>` metadata with node edges.

| File | Change | Sites closed |
|---|---|---|
| `00_core.dag` | `kernel_types: List<Node>`, `container_types: List<Node>` | 2 string lists |
| `04_emit_info.dag` | `variant_to_enum`, `field_type_names` become node-keyed | 4 string-keyed maps |
| `04_method.dag` | `builtin_function_registry` keyed by node, not string | 1 map |
| `complexity.dag` | Function summary cache keyed by node | 1 map |
| Other files | Remaining `Map<String, X>` -> `Map<Node, X>` | 6 maps |
| **Total** | | **14 string-keyed maps** |

## Lane dependencies

```
Lane A (method/transport dispatch)    independent
Lane B (Node.name deletion)          depends on A
Lane C (graph rendering + plugin extraction) depends on B
Lane D (edge-only facts)             independent, parallel with A/B
```

## End state

**`src/v1/` contains only language-agnostic compiler code:**

```
src/v1/
  00_core.dag          Node, Edge types (no String on Node, no emit IR)
  01_tokenize.dag      Source text -> tokens
  02_parse.dag         Tokens -> Node tree
  03_normalize.dag     Structural normalization
  03_resolve.dag       Name resolution (scope dies here)
  04_*.dag             Inference (reads structure, not names)
  05_emit.dag          Coercion engine (graph -> target-basis graph, no strings)
  compile.dag          Pipeline orchestration
  complexity.dag       Cost proofs (reads method cost from algebra nodes)
  ownership.dag        Ownership proofs
  trace.dag            Debug tracing
  artifact.dag         Output artifact planning
  languages.dag        LanguageSpec type definitions
```

**No `05_emit_rust.dag`, no `05_emit_python.dag`, no `05_emit_go.dag`.**
Zero mentions of Rust/Python/Go in any compiler file. Zero `concat()`
calls producing target syntax. Zero `if type_name == "..."` branches.

**Language plugins live in `dsl/extdeps/languages/`:**

```
dsl/extdeps/languages/
  rust/
    coerce.dag         Coercion rules (graph patterns -> Rust basis)
    render.dag         Renderer (Rust-basis graph -> text, trivial)
    emit.dag           Container templates, type maps, sharing policy
    lint.dag           Import rules, naming conventions
    runtime.dag        Runtime function signatures
    naming.dag         Case conventions
  python/
    coerce.dag, render.dag, emit.dag, lint.dag, runtime.dag, naming.dag
  go/
    coerce.dag, render.dag, emit.dag, lint.dag, runtime.dag, naming.dag
  dag/
    syntax.dag         SyntaxSpec for .dag frontend

  # Challenge targets (design validation):
  verilog/
    coerce.dag         Products -> module ports, coproducts -> mux (Lowered)
    render.dag         Verilog-basis graph -> Verilog text
  spice/
    coerce.dag         Products -> subcircuit params, coproducts ->
                       comparators + switches (Synthesized, expensive)
    render.dag         SPICE-basis graph -> SPICE netlist
  english/
    coerce.dag         Products -> bullet lists, coproducts -> "either/or"
    render.dag         English-basis graph -> Markdown
```

**Adding a new language** = add `coerce.dag` (sidecast processes for
non-native patterns) + `render.dag` (trivial text from target-basis
graph) under `dsl/extdeps/languages/`. Zero compiler changes.

**Challenge targets** validate the architecture: if the coercion
engine works for Verilog, SPICE, and English, it works for anything.
These are the hardest targets — they force the compiler to find
minimal representations for patterns that don't map natively (e.g.,
coproducts in pure analog SPICE require synthesizing from
comparators, which the cost algebra reports as Synthesized/expensive).

**Ratchets at zero:**
- Language mentions in `src/v1/*.dag`: 0 (currently 632)
- `node.name` reads: 0 (currently 1,175)
- String-keyed metadata maps: 0 (currently 14)
- Method name dispatch sites: 0 (currently 63)
- Escape hatches total: 0 (currently ~290)
