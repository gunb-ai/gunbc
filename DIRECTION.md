# Direction

This document records architectural direction: use cases we may not
have today but must not lock ourselves out of. Where SUSTAINABILITY.md
tracks cost-of-change for the compiler we have, this tracks
compatibility with the compilers we might need.

See `SUSTAINABILITY.md` for the causal tree of current technical debt.

---

## v2 self-hosted compiler

The v2 compiler is written in DSL (7 .dag files, 7,139 lines). All 5
pipeline stages are implemented and tested (48/48 tests passing).
Self-hosting requires emitting a native binary (Phase 1), progressive
self-compilation (Phase 2), and fixed-point verification (Phase 3).
See `SUSTAINABILITY.md` Branch 7 for the bootstrap gap analysis and
stack depth tradeoff.

- `DESIGN-v2-compiler.md` — design rationale: why self-hosting, what
  went wrong with v0/v1, core principles.
- `DESIGN-v2-target-models.md` — target type system specification.
  The v2 code does not yet match these models; this is what it converges toward.

---

## Core model: DAG as causal process

The DAG models causality: directed edges mean "A must happen before B."
This is not limited to discrete computation — it is a model of process
forward in time, which all physical systems obey. Current flows
because voltage was applied. A component is soldered because it was
placed. A signal is amplified because it entered the gain stage.

Causality is always directed. Even analog circuits with feedback loops
are causal in time — the output at time t feeds back to affect the
input at t+dt. The existing loop construct (LoopUnpack → body →
LoopPack) models exactly this: iterative convergence over time steps.

SPICE netlists appear undirected, but this is a modeling shortcut for
simultaneous constraint solving, not a reflection of the underlying
physics. The DAG captures the causal reality; backends that need
undirected representations (SPICE netlists) project away the direction
at emit time.

---

## Everything is an interface

The `extdeps/` pattern treats every external system as a composable
interface: `github.dag` is an interface to the GitHub API, `shell.dag`
is an interface to local shell execution, `make.dag` is an interface
to the Make build system. The user writes against interfaces; the
compiler resolves them through the DAG.

This is the universal pattern. Every domain — circuit analysis,
simulation, frequency response, physical layout — is an interface.
Interfaces compose. The user writes at whatever level of intent they
choose, and the compiler unfolds through the interface chain.

### Software (what we have today)

```
extdeps/github.dag     → interface to GitHub REST API
extdeps/shell.dag      → interface to local shell
extdeps/make.dag       → interface to Make build system
```

User writes against `github.dag`; the compiler resolves through
shell transport, HTTP transport, authentication — the user doesn't
care about those layers.

### Circuit domains (same pattern)

```
extdeps/ohm.dag        → interface to V=IR relationships
extdeps/kirchhoff.dag  → interface to KVL/KCL analysis
extdeps/spice.dag      → interface to SPICE simulation engine
extdeps/verilog.dag    → interface to Verilog synthesis
```

`spice.dag` imports from `kirchhoff.dag` because SPICE *implements*
Kirchhoff's laws — just like `github.dag` composes with `shell.dag`
transport. The user who writes at the SPICE level is implicitly using
circuit analysis. The user who writes at the Kirchhoff level is
implicitly using Ohm's law. Each level is an interface; interfaces
compose through the DAG.

### The unfolding chain

Each domain is a composable layer of interfaces. The user picks their
level of intent:

```
"I want to simulate this circuit"       → writes against spice.dag
"I want to analyze this node voltage"   → writes against kirchhoff.dag
"I want to define V=IR for a resistor"  → writes against ohm.dag
```

The compiler unfolds through the interface chain, just as it unfolds
software intent through function composition → typed dataflow →
target-specific lowering.

The domain models live in `.dag` files — type hierarchies and interface
definitions — the same way `dsl/std/` encodes the type chain
`Classical → Bit → Byte → Word → Int → String`. The circuit
equivalent:

```
Signal → AnalogSignal → Voltage → BandlimitedVoltage
Signal → DigitalSignal → LogicLevel → Bus<N>
```

### Multiple simultaneous views

A single circuit DAG (the user's intent) can be analyzed from multiple
domain perspectives:

- **Assembly/fabrication** — component placement + wiring order
- **Signal flow** — how signals propagate through the circuit
- **Physical layout** — spatial placement, routing, thermal coupling
- **Frequency response** — behavior across frequencies
- **Thermal** — heat dissipation, coupling between components

These are not separate user-authored DAGs. They are **compiler-derived
analyses** over the same source DAG — just as typecheck, lowering, and
derive passes all analyze the same software source DAG today. The user
writes one causal description (intent + connectivity); the compiler
derives the rest.

Cross-cutting interactions (frequency response depends on layout,
thermal depends on signal flow) are a compiler pass ordering problem,
not a model problem. The core DAG still represents causal intent;
domain-specific passes jointly reason about derived properties.

---

## Concrete architectural constraints

These are the rules that preserve optionality for future domains:

### C1: Edges are always causal — backends project as needed

The DAG's directed edges represent causal process forward in time.
This is correct for all physical systems, not just software.

Backends that need different connectivity models (SPICE's undirected
nets, layout's geometric adjacency) derive them by projecting the
causal DAG at emit time. The core graph infrastructure does not need
undirected edges — it needs backends that can interpret directed
topology in domain-appropriate ways.

*Test:* A `render_spice.rs` should be able to emit `.subckt`
definitions by reading the same `Dag<LoweredOp>` that `render_rust.rs`
uses — extracting component instantiations and net connectivity from
the directed edges without requiring new edge types.

### C2: Keep the unfolding machinery generic

The compiler unfolds user intent through domain layers. The unfolding
machinery (type DAG composition, lowering passes, analysis passes)
must not be specialized to software concepts.

Today: `Classical → Bit → Byte → Word → Int` is a type-level
unfolding chain. This same mechanism should support
`Signal → Voltage → BandlimitedVoltage` or
`Material → Conductor → Copper` without compiler changes.

*Test:* Can a new domain layer be added entirely in `.dag` files (type
definitions + domain models) without modifying the compiler's Rust
code?

### C3: Do not specialize Domain() refinements

`Domain(String)` is intentionally open. It could mean
`"ieee754_binary32"`, `"si_volt"`, `"laplace_s"`, or
`"spice_bsim4_model"`. Do not replace it with a closed enum. The
openness lets new application domains define their own physics without
compiler changes.

If compile-time dimensional analysis is ever needed, implement it as a
typecheck pass that *interprets* domain strings (e.g., parsing SI unit
vectors from `"si_volt"`, `"si_ampere"`), not as a change to the
`Predicate` enum.

### C4: Do not couple type structure to execution model

The type DAG (`Dag<TypeOp>`) describes what values a port accepts.
The execution DAG (`Dag<LoweredOp>`) describes how values flow. These
must remain independent. A `Voltage` type should be usable in both a
SPICE netlist (emitted for simulation) and a Rust program (computed
numerically) without the type definition knowing which backend applies.

*Today this holds.* Keep it that way.

### C5: Feedback loops use the loop construct, not special edges

Analog feedback (op-amp feedback networks, oscillators, PLL loops) and
digital feedback (state machines, pipeline hazard resolution) both
model the same thing: the output at time t influences the input at
time t+dt.

The existing loop construct (LoopUnpack → body → LoopPack) handles
this. Convergence loops (Newton-Raphson in SPICE transient analysis)
are iterative loops with a convergence predicate. Do not introduce
cycle-permitting edges or special feedback edge types — model feedback
as what it physically is: iteration forward in time.

*Test:* Can an op-amp feedback circuit be modeled as a DAG with a loop
construct, where the loop body computes one timestep of the feedback
network?

### C6: Domain analyses are compiler passes, not user-authored graphs

The user writes one DAG (intent + connectivity). Domain-specific views
(frequency response, thermal analysis, layout constraints) are derived
by compiler analysis passes, not authored separately.

This means the pass infrastructure must support domain-pluggable
analyses — a frequency response pass that walks the DAG and computes
transfer functions, a thermal pass that estimates power dissipation per
node. These are the analog-domain equivalents of today's typecheck and
derive passes.

*Test:* Could a `FrequencyResponsePass` be added as a new derive pass
alongside `CallableProperties` without changing the core pipeline
structure?

---

## Near-term direction: Verilog emission

Digital hardware follows the same interface pattern as software and
analog. The domain interfaces:

```
extdeps/logic_gate.dag   → interface to combinational primitives (AND, OR, MUX)
extdeps/register.dag     → interface to sequential storage (flip-flop, latch)
extdeps/clock.dag        → interface to clock domain management
extdeps/verilog.dag      → interface to Verilog synthesis backend
```

These compose: `verilog.dag` imports `clock.dag` which imports
`register.dag` which imports `logic_gate.dag`. The user who writes
"synchronous counter" against `register.dag` is implicitly using
clock domains and logic gates — just as the user who writes a GitHub
workflow is implicitly using shell transport.

The type system already speaks this domain:

```
Classical → Bit → Byte → Word          (data widths)
Bit where brand("Clock")               (clock signals)
Bit where brand("Reset")               (reset signals)
List<Bit> where length(N)              (buses)
```

The domain type hierarchy for digital hardware:

```
dsl/std/logic.dag       → Classical truth values (foundation)
dsl/std/bit.dag         → Bit, Nibble, Byte, Word (width-typed signals)
dsl/std/integer.dag     → UInt8..64, Int8..64 (arithmetic-typed words)
```

These already exist. The user writes at their chosen level of intent:

```
"I want a 4-bit adder"              → writes against logic_gate.dag
"I want a register with async reset" → writes against register.dag
"I want an SPI controller"           → writes against a protocol interface
```

The compiler unfolds through the interface chain, then the emit
backend lowers to Verilog:

- `Callable{kind: Fn}` → `module` declaration
- `Primitive{BinaryOp}` → `assign` (combinational) or
  `always @(posedge clk)` (sequential, inferred from clock-branded
  ports)
- Port cardinality → `input [N-1:0]` / `output [N-1:0]` wire widths
- `Brand("Clock")` ports → clock domain partitioning
- DAG dataflow edges → Verilog wire connections
- Loop construct → generate loops (`genvar`, `for`)
- SubDag → submodule instantiation

The backend work:

1. `lower_verilog.rs` — map LoweredOp to Verilog constructs
2. `render_verilog.rs` — emit Verilog syntax
3. Clock domain inference — graph walk from Brand("Clock") ports to
   partition nodes into combinational vs sequential

This stays entirely within the causal DAG model. No new execution
semantics. The only new work beyond the emit backend is writing the
domain `.dag` interface files — which is domain knowledge authoring,
not compiler engineering.

---

## Long-term direction: analog / continuous

The path to SPICE or analog emission is the same as everything else:
define domain interfaces in `.dag` files and let the compiler unfold.

1. **Domain interfaces** — `extdeps/ohm.dag`, `kirchhoff.dag`,
   `spice.dag`. These are interfaces, the same as `github.dag` or
   `shell.dag`. They define the operations (voltage divider, node
   analysis, transient simulation) and the types they operate on.

2. **Domain type hierarchy** in `dsl/std/` — `Signal`, `Voltage`,
   `Current`, `Impedance` types with `Domain()` refinements and
   `Brand()` nominal safety. Component types (`Resistor`, `Capacitor`,
   `MOSFET`) as records with physical-unit-typed fields. Same
   compositional pattern as the existing type chain.

3. **Interface composition** — `spice.dag` imports `kirchhoff.dag`
   which imports `ohm.dag`. The user writes "simulate this circuit"
   against `spice.dag`; the compiler unfolds through circuit analysis
   interfaces down to component-level netlists.

4. **Loop-based simulation semantics** — feedback and convergence
   modeled via LoopUnpack/LoopPack. SPICE transient analysis =
   outer time-stepping loop × inner Newton-Raphson convergence loop.
   Same loop construct, different domain.

5. **`render_spice.rs`** — emits `.subckt` definitions, component
   instantiations, net connectivity (projected from directed edges),
   and analysis directives (`.tran`, `.ac`, `.dc`).

6. **Cross-domain analysis passes** — frequency response, thermal,
   layout constraint derivation. New derive passes over the same DAG.

No changes to the core graph infrastructure. No new execution model.
The DAG remains a causal process; domain knowledge lives in `.dag`
interface definitions, type hierarchies, and compiler analysis passes.
The only new work is writing the domain `.dag` files — the machinery
already exists.

---

## Relationship to sustainability

SUSTAINABILITY.md's deep root is "incomplete compile-time resolution."
The constraints here are orthogonal — they're about not over-specifying
the *semantics* of resolved information, not about when resolution
happens.

The v2 self-hosted compiler (which eliminates most of Branch 1-4 in
SUSTAINABILITY.md) should adopt these constraints from the start:
types are structural, not strings (sustainability concern), AND the
unfolding machinery is domain-generic (direction concern).

The key alignment: both documents converge on the same principle —
**the compiler resolves everything at compile time, and what it
resolves is determined by domain models, not hardcoded Rust logic.**
Sustainability says "resolve structurally, not by string." Direction
says "resolve through domain layers, not through software-specific
assumptions."

---

## Appendix: Hypothetical domain models

These are hypothetical `.dag` files showing how circuit, physics, and
simulation domains compose using the existing type system, patterns,
and interface machinery. Each follows the same conventions as
`std/logic.dag`, `std/fermi.dag`, `std/patterns.dag`, etc.

### Layer 0: Physical dimensions

The SI system has 7 base dimensions. Every physical quantity is a
product of base dimensions with integer exponents — the same
compositional pattern as `Classical → Bit → Byte → Word`.

```dag
// std/dimension.dag -- SI base dimensions as compositional types.
//
// Every physical quantity is a vector of 7 integer exponents over
// base dimensions. Voltage is kg·m²·s⁻³·A⁻¹ = [1,2,-3,-1,0,0,0].
// This is the physical equivalent of bit.dag's width constraints.

module std.dimension

// The 7 SI base dimensions (atomic, like Classical in logic.dag)
type BaseDimension = Mass | Length | Time | Current
                   | Temperature | Amount | Luminosity

// A dimension vector: product of base dimensions with exponents.
// This is the dimensional equivalent of Word32 = List<Byte>[4].
type DimensionVector {
  mass: Int           // kg exponent
  length: Int         // m exponent
  time: Int           // s exponent
  current: Int        // A exponent
  temperature: Int    // K exponent
  amount: Int         // mol exponent
  luminosity: Int     // cd exponent
}

// --- Tautological dimension vectors (like fermi_timeouts) --------

data dimensionless: DimensionVector =
  DimensionVector { mass: 0, length: 0, time: 0, current: 0,
                    temperature: 0, amount: 0, luminosity: 0 }

data voltage_dim: DimensionVector =
  DimensionVector { mass: 1, length: 2, time: -3, current: -1,
                    temperature: 0, amount: 0, luminosity: 0 }

data current_dim: DimensionVector =
  DimensionVector { mass: 0, length: 0, time: 0, current: 1,
                    temperature: 0, amount: 0, luminosity: 0 }

data resistance_dim: DimensionVector =
  DimensionVector { mass: 1, length: 2, time: -3, current: -2,
                    temperature: 0, amount: 0, luminosity: 0 }

data capacitance_dim: DimensionVector =
  DimensionVector { mass: -1, length: -2, time: 4, current: 2,
                    temperature: 0, amount: 0, luminosity: 0 }

data inductance_dim: DimensionVector =
  DimensionVector { mass: 1, length: 2, time: -2, current: -2,
                    temperature: 0, amount: 0, luminosity: 0 }

data power_dim: DimensionVector =
  DimensionVector { mass: 1, length: 2, time: -3, current: 0,
                    temperature: 0, amount: 0, luminosity: 0 }

data frequency_dim: DimensionVector =
  DimensionVector { mass: 0, length: 0, time: -1, current: 0,
                    temperature: 0, amount: 0, luminosity: 0 }

// --- Dimension arithmetic (like fermi_max, fermi_gt) -------------

fn dim_multiply(a: DimensionVector, b: DimensionVector) -> DimensionVector {
  DimensionVector {
    mass: a.mass + b.mass,
    length: a.length + b.length,
    time: a.time + b.time,
    current: a.current + b.current,
    temperature: a.temperature + b.temperature,
    amount: a.amount + b.amount,
    luminosity: a.luminosity + b.luminosity
  }
}

fn dim_divide(a: DimensionVector, b: DimensionVector) -> DimensionVector {
  DimensionVector {
    mass: a.mass - b.mass,
    length: a.length - b.length,
    time: a.time - b.time,
    current: a.current - b.current,
    temperature: a.temperature - b.temperature,
    amount: a.amount - b.amount,
    luminosity: a.luminosity - b.luminosity
  }
}

fn dim_eq(a: DimensionVector, b: DimensionVector) -> Bool {
  a.mass == b.mass
    && a.length == b.length
    && a.time == b.time
    && a.current == b.current
    && a.temperature == b.temperature
    && a.amount == b.amount
    && a.luminosity == b.luminosity
}
```

### Layer 1: Physical quantities

Quantities compose dimensions with numeric values — the same pattern
as `integer.dag` composing `Byte` with `signed`/`unsigned` refinements.

```dag
// std/quantity.dag -- Physical quantities: value + dimension.
//
// A Quantity is to physics what Int64 is to computing:
// a refined numeric type with domain semantics.
// Int64 = Word64 where signed, arithmetic
// Voltage = Quantity where dimension(voltage_dim), brand("Voltage")

module std.quantity

import std.float { Float64 }
import std.dimension { DimensionVector, dimensionless,
    voltage_dim, current_dim, resistance_dim, capacitance_dim,
    inductance_dim, power_dim, frequency_dim }

// Base quantity: a value with a dimension vector.
// Analogous to Byte (raw bits before signed/unsigned refinement).
type Quantity {
  value: Float64
  dimension: DimensionVector
}

// --- Branded physical types (like IntentId = NonEmptyStr where brand) --
//
// Each is a Quantity with a fixed dimension and nominal distinction.
// Voltage and (Current * Resistance) have the same dimension vector,
// but brand() prevents accidental substitution — just like IntentId
// and IssueId are both NonEmptyStr but nominally disjoint.

type Voltage     = Quantity where domain("si_volt"),    brand("Voltage")
type Current     = Quantity where domain("si_ampere"),  brand("Current")
type Resistance  = Quantity where domain("si_ohm"),     brand("Resistance")
type Capacitance = Quantity where domain("si_farad"),   brand("Capacitance")
type Inductance  = Quantity where domain("si_henry"),   brand("Inductance")
type Power       = Quantity where domain("si_watt"),    brand("Power")
type Frequency   = Quantity where domain("si_hertz"),   brand("Frequency")
type Charge      = Quantity where domain("si_coulomb"), brand("Charge")

// Dimensionless quantities (like Bool — structurally simple but
// semantically important)
type Ratio = Quantity where domain("dimensionless"), brand("Ratio")
type Gain  = Quantity where domain("dimensionless"), brand("Gain")
type Phase = Quantity where domain("si_radian"),     brand("Phase")

// --- Prefixed constructors (tautological data, like language defs) --

type SiPrefix {
  name: String
  symbol: String
  exponent: Int
}

data si_prefixes: List<SiPrefix> = [
  SiPrefix { name: "femto", symbol: "f",  exponent: -15 },
  SiPrefix { name: "pico",  symbol: "p",  exponent: -12 },
  SiPrefix { name: "nano",  symbol: "n",  exponent: -9 },
  SiPrefix { name: "micro", symbol: "u",  exponent: -6 },
  SiPrefix { name: "milli", symbol: "m",  exponent: -3 },
  SiPrefix { name: "kilo",  symbol: "k",  exponent: 3 },
  SiPrefix { name: "mega",  symbol: "M",  exponent: 6 },
  SiPrefix { name: "giga",  symbol: "G",  exponent: 9 },
]
```

### Layer 2: Circuit components

Components compose quantities into records with port semantics — the
same pattern as `resources.dag` composing types into resource
capabilities.

```dag
// std/circuit.dag -- Circuit component vocabulary.
//
// Components are records whose fields are physical quantity types.
// This is the circuit equivalent of std/resources.dag: components
// are acquirable things with typed ports and behavioral properties.
//
// Follows behavioral.dag pattern for classifying component behavior.

module std.circuit

import std.quantity { Voltage, Current, Resistance, Capacitance,
    Inductance, Frequency, Power, Gain, Phase }
import std.dimension { DimensionVector }

// --- Component classification (like TransportClass in fidelity.dag) --

type ComponentClass = Passive | Active | Source | Reactive

type Linearity = Linear | NonLinear

type ComponentProperties {
  class: ComponentClass
  linearity: Linearity
  frequency_dependent: Bool
  has_memory: Bool           // stores energy (C, L) or state (flip-flop)
}

// --- Passive components --------------------------------------------------

type Resistor {
  resistance: Resistance
  tolerance_pct: Float64 where range(min: 0, max: 100)
  power_rating: Power
  // temperature coefficient (ppm/C) -- future: Temperature type
}

type Capacitor {
  capacitance: Capacitance
  voltage_rating: Voltage
  esr: Resistance?           // equivalent series resistance
}

type Inductor {
  inductance: Inductance
  dc_resistance: Resistance  // DCR
  saturation_current: Current
}

// --- Active components ---------------------------------------------------

// MOSFET: the fundamental active device.
// Port structure mirrors SPICE's M-element (4 terminals).
type MosfetKind = NMOS | PMOS

type Mosfet {
  kind: MosfetKind
  w: Float64 where domain("si_meter")    // gate width
  l: Float64 where domain("si_meter")    // gate length
  model: String                          // SPICE model name
}

type OpAmpSpec {
  gain_bandwidth: Frequency
  dc_gain: Gain
  slew_rate: Float64 where domain("si_volt_per_second")
  input_offset: Voltage
  cmrr: Gain                            // common-mode rejection
}

// --- Sources (like test fixtures: known stimulus) -------------------------

type SourceKind = DC | AC | Pulse | PWL

type VoltageSource {
  kind: SourceKind
  dc_value: Voltage?
  ac_amplitude: Voltage?
  ac_phase: Phase?
  frequency: Frequency?
}

type CurrentSource {
  kind: SourceKind
  dc_value: Current?
  ac_amplitude: Current?
}

// --- Component classification (like classify_transports) -----------------

fn component_class(kind: String) -> ComponentProperties {
  match kind {
    "resistor"  => ComponentProperties {
      class: Passive, linearity: Linear,
      frequency_dependent: false, has_memory: false }
    "capacitor" => ComponentProperties {
      class: Reactive, linearity: Linear,
      frequency_dependent: true, has_memory: true }
    "inductor"  => ComponentProperties {
      class: Reactive, linearity: Linear,
      frequency_dependent: true, has_memory: true }
    "mosfet"    => ComponentProperties {
      class: Active, linearity: NonLinear,
      frequency_dependent: true, has_memory: false }
    _           => ComponentProperties {
      class: Active, linearity: NonLinear,
      frequency_dependent: true, has_memory: true }
  }
}
```

### Layer 3: Circuit analysis interfaces

These are interfaces — the circuit equivalent of `extdeps/github.dag`.
They compose from component types just as GitHub interfaces compose
from HTTP transport types.

```dag
// extdeps/ohm.dag -- Ohm's law interface.
//
// The most fundamental circuit analysis interface. All other circuit
// interfaces compose from this, just as all software interfaces
// compose from shell/HTTP transport.

module extdeps.ohm

import std.quantity { Voltage, Current, Resistance, Power }

// --- Constitutive relations (pure functions, like std/logic.dag) ------
//
// These are tautological: they define what the quantities ARE in
// relation to each other. V=IR is not a computation — it is a fact,
// like Classical.True being a truth value.

fn voltage_from(i: Current, r: Resistance) -> Voltage {
  Voltage { value: i.value * r.value, dimension: voltage_dim }
}

fn current_from(v: Voltage, r: Resistance) -> Current {
  Current { value: v.value / r.value, dimension: current_dim }
}

fn resistance_from(v: Voltage, i: Current) -> Resistance {
  Resistance { value: v.value / i.value, dimension: resistance_dim }
}

fn power_dissipated(v: Voltage, i: Current) -> Power {
  Power { value: v.value * i.value, dimension: power_dim }
}

// --- Series/parallel composition (like fermi_max: composing magnitudes) --

fn series_resistance(resistors: List<Resistance>) -> Resistance {
  fold(resistors, init: Resistance { value: 0.0, dimension: resistance_dim },
    f: (acc, r) => Resistance { value: acc.value + r.value,
                                dimension: resistance_dim })
}

fn parallel_resistance(r1: Resistance, r2: Resistance) -> Resistance {
  Resistance {
    value: (r1.value * r2.value) / (r1.value + r2.value),
    dimension: resistance_dim
  }
}
```

```dag
// extdeps/kirchhoff.dag -- Kirchhoff's laws interface.
//
// Composes from ohm.dag. KVL and KCL are conservation constraints:
// voltages around a loop sum to zero, currents at a node sum to zero.
//
// This is the circuit equivalent of patterns.dag: it defines the
// fundamental wiring rules that all higher-level patterns obey.

module extdeps.kirchhoff

import extdeps.ohm { Voltage, Current, voltage_from, current_from }
import std.quantity { Voltage, Current }

// --- Circuit topology (like DagTopology in std/types.dag) --------

type NetId = String where non_empty, brand("NetId")

type Pin {
  component: String
  terminal: String
  net: NetId
}

type Net {
  id: NetId
  pins: List<Pin>
}

type CircuitTopology {
  nets: List<Net>
  components: List<String>
}

// --- KCL: current conservation at a node --------------------------
//
// Sum of currents entering a node = sum of currents leaving.
// This is a constraint (like a type predicate), not a computation.

type NodeAnalysis {
  net: NetId
  currents_in: List<Current>
  currents_out: List<Current>
  balanced: Bool              // KCL satisfied?
}

fn sum_currents(currents: List<Current>) -> Current {
  fold(currents, init: Current { value: 0.0, dimension: current_dim },
    f: (acc, i) => Current { value: acc.value + i.value,
                             dimension: current_dim })
}

fn check_kcl(node: NodeAnalysis) -> Bool {
  let total_in = sum_currents(currents: node.currents_in)
  let total_out = sum_currents(currents: node.currents_out)
  // Within floating-point tolerance
  let diff = total_in.value - total_out.value
  diff > -1e-12 && diff < 1e-12
}

// --- KVL: voltage conservation around a loop ----------------------

type LoopAnalysis {
  net_ids: List<NetId>
  voltages: List<Voltage>
  balanced: Bool              // KVL satisfied?
}

fn sum_voltages(voltages: List<Voltage>) -> Voltage {
  fold(voltages, init: Voltage { value: 0.0, dimension: voltage_dim },
    f: (acc, v) => Voltage { value: acc.value + v.value,
                             dimension: voltage_dim })
}

fn check_kvl(loop: LoopAnalysis) -> Bool {
  let total = sum_voltages(voltages: loop.voltages)
  total.value > -1e-12 && total.value < 1e-12
}
```

### Layer 4: Signal domain and analysis

Signals compose from quantities + time — the frequency/laplace domain
is a classification over signals, analogous to `fidelity.dag`
classifying transports.

```dag
// std/signal.dag -- Signal domain vocabulary.
//
// Signals are quantities that vary over time. The signal domain
// (time, frequency, laplace) is a classification — the same pattern
// as TransportClass in fidelity.dag. Different domains reveal
// different properties of the same signal, just as different compiler
// passes reveal different properties of the same DAG.

module std.signal

import std.quantity { Voltage, Current, Frequency, Gain, Phase }

// --- Signal classification (like TransportClass) -------------------

type SignalDomain = TimeDomain | FrequencyDomain | LaplaceDomain

// A signal is a quantity that varies with time or frequency.
// In TimeDomain: value at each timestep.
// In FrequencyDomain: magnitude + phase at each frequency.
// In LaplaceDomain: transfer function coefficients.
type SignalKind = Analog | Digital | MixedSignal

type SignalProperties {
  domain: SignalDomain
  kind: SignalKind
  bandwidth: Frequency?
  sample_rate: Frequency?      // for digital/mixed
}

// --- Frequency response (derived classification, like DerivedClassification)

type BodePoint {
  frequency: Frequency
  magnitude: Gain              // dB
  phase: Phase                 // radians
}

type FrequencyResponse {
  points: List<BodePoint>
  unity_gain_freq: Frequency?  // gain crossover
  phase_margin: Phase?         // stability metric
  gain_margin: Gain?           // stability metric
}

// --- Stability classification (like classify_transports) ----------
//
// Follows the fermi.dag ordinal pattern: stability is a magnitude.

type StabilityClass = Stable | MarginallyStable | Unstable

fn classify_stability(response: FrequencyResponse) -> StabilityClass {
  match response.phase_margin {
    None => Unstable
    Some(pm) => if pm.value > 0.785 { Stable }         // > 45 degrees
               else { if pm.value > 0.0 { MarginallyStable }
               else { Unstable } }
  }
}
```

### Layer 5: Simulation as a pattern

Simulation composes from analysis interfaces + the loop construct —
the same way `content_upsert` composes from `file_content_matches` +
`fs.write`. The loop construct provides time-stepping; convergence
is a predicate over state.

```dag
// std/simulate.dag -- Simulation patterns.
//
// Simulation is a pattern (like ensure, upsert, content_upsert)
// that composes analysis interfaces with the loop construct.
// Time-stepping = loop. Convergence = predicate. State = accumulator.
//
// This is the physics equivalent of patterns.dag.

module std.simulate

import std.quantity { Voltage, Current }

// --- Simulation state (like the accumulator in fold) ---------------

type SimState {
  time: Float64 where domain("si_second")
  dt: Float64 where domain("si_second")
  node_voltages: List<Voltage>
  branch_currents: List<Current>
  iteration: Int
  converged: Bool
}

type SimConfig {
  start_time: Float64 where domain("si_second")
  end_time: Float64 where domain("si_second")
  max_dt: Float64 where domain("si_second")
  min_dt: Float64 where domain("si_second")
  tolerance: Float64
  max_iterations: Int where range(min: 1, max: 1000)
}

// --- Convergence check (like fermi_within_budget) -----------------

fn is_converged(prev: SimState, curr: SimState, tol: Float64) -> Bool {
  let max_delta = fold(
    zip(prev.node_voltages, curr.node_voltages),
    init: 0.0,
    f: (acc, pair) => {
      let diff = pair.first.value - pair.second.value
      let abs_diff = if diff < 0.0 { 0.0 - diff } else { diff }
      if abs_diff > acc { abs_diff } else { acc }
    }
  )
  max_delta < tol
}

// --- Transient simulation pattern --------------------------------
//
// This is the simulation equivalent of content_upsert:
// a composition of fundamental steps with the loop construct.
//
// Outer loop: time steps (like iterating over a list of files)
// Inner loop: Newton-Raphson convergence (like retry with backoff)
//
// The pattern:
//   for t in time_steps {                    // LoopUnpack
//     state = fold(1..max_iter, init: state, // inner convergence
//       f: (s, _) => {
//         next = solve_one_step(s)           // analysis interface
//         if is_converged(s, next, tol) { next } else { next }
//       })
//   }                                        // LoopPack
//
// This is exactly how SPICE transient analysis works. The DAG
// models the causal process: each time step depends on the
// previous one. The loop construct provides the iteration.
// No special execution model needed.
```

### Layer 6: Circuit resource (like Filesystem)

A circuit net is a resource — the same way Filesystem is a resource.
Components acquire access to nets, read/write voltages and currents.

```dag
// std/circuit_resources.dag -- Circuit resources.
//
// Follows std/resources.dag exactly. A circuit net is an acquirable
// capability, just like a file path. Components don't forge net
// handles — the compiler's acquire nodes mint them.

module std.circuit_resources

import std.quantity { Voltage, Current }
import std.circuit { NetId }

// A net handle is opaque proof of connectivity — like ResourceHandle.
type NetHandle {
  net_id: NetId
  cap: Secret
}

resource CircuitNet {
  kind: Capability
  mode: ReadWrite
  acquire {}
  release {}

  // Read the voltage at a net (like Filesystem.read)
  capability sense_voltage {
    input { net: NetHandle }
    output { voltage: Voltage }
  }

  // Inject current into a net (like Filesystem.write)
  capability drive_current {
    input { net: NetHandle, current: Current }
    output { applied: Bool }
  }

  // Probe without driving (like Filesystem.probe)
  capability probe {
    input { net: NetHandle }
    output { voltage: Voltage, current: Current }
  }
}

// A component instance acquires nets through its pins.
// This is the circuit equivalent of a func acquiring Filesystem.
//
// pattern resistor_instance(r: Resistance) -> { v: Voltage, i: Current }
//   uses net_a: CircuitNet(mode: Read),
//        net_b: CircuitNet(mode: Read)
// {
//   va = net_a.sense_voltage(net: net_a)
//   vb = net_b.sense_voltage(net: net_b)
//   v_drop = Voltage { value: va.voltage.value - vb.voltage.value }
//   i = current_from(v: v_drop, r: r)
//   return { v: v_drop, i: i }
// }
```

### How the layers compose

The full composition chain, showing the parallel with software:

```
LAYER   SOFTWARE                      CIRCUIT
------  ----------------------------  ----------------------------
0       Classical (truth)             BaseDimension (mass/length/time/...)
1       Bit = Classical[width(1)]     Quantity = Float64[dimension]
2       Byte = List<Bit>[8]           Voltage = Quantity[si_volt, brand]
3       Int64 = Word64[signed]        Resistor = {resistance, tolerance, ...}
4       Filesystem (resource)         CircuitNet (resource)
5       file_content_matches (step)   ohm::voltage_from (constitutive law)
6       content_upsert (pattern)      simulate::transient (simulation pattern)
7       github.dag (interface)        spice.dag (interface)
user    "ensure clippy.toml"          "simulate voltage divider"
```

Each row composes from the row above, using the same mechanisms:
refinement predicates, branded nominal types, record composition,
resource capabilities, pattern wiring, and interface imports. The
compiler doesn't know whether it's compiling software or circuits —
it unfolds through domain layers regardless.
