# Direction

This document records architectural direction: use cases we may not
have today but must not lock ourselves out of. Where SUSTAINABILITY.md
tracks cost-of-change for the compiler we have, this tracks
compatibility with the compilers we might need.

See `SUSTAINABILITY.md` for the causal tree of current technical debt.

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

Digital hardware emission is the natural next target. The type system
already speaks hardware (Classical → Bit → Byte → Word, branded
clock/reset signals, width refinements). The work is:

1. `lower_verilog.rs` — map LoweredOp to Verilog constructs
2. `render_verilog.rs` — emit Verilog syntax
3. Clock domain inference — graph walk from Brand("Clock") ports to
   partition nodes into combinational vs sequential

This stays entirely within the causal DAG model. Combinational circuits
map directly: DAG dataflow = `assign` statements. Sequential circuits
need the clock-domain pass to determine which nodes belong in
`always @(posedge clk)` blocks.

---

## Long-term direction: analog / continuous

The path to SPICE or analog emission:

1. **Domain type hierarchy** in `dsl/std/` — `Signal`, `Voltage`,
   `Current`, `Impedance` types with `Domain()` refinements and
   `Brand()` nominal safety. Component types (`Resistor`, `Capacitor`,
   `MOSFET`) as records with physical-unit-typed fields.

2. **Domain-driven unfolding** — compiler passes that refine
   high-level intent ("amplifier with gain G") into component-level
   topology, guided by domain models in `.dag` files. Same mechanism
   as the software compiler unfolding `func` into typed dataflow.

3. **Loop-based simulation semantics** — feedback and convergence
   modeled via LoopUnpack/LoopPack. SPICE transient analysis =
   outer time-stepping loop × inner Newton-Raphson convergence loop.

4. **`render_spice.rs`** — emits `.subckt` definitions, component
   instantiations, net connectivity (projected from directed edges),
   and analysis directives (`.tran`, `.ac`, `.dc`).

5. **Cross-domain analysis passes** — frequency response, thermal,
   layout constraint derivation. These are new derive passes, not
   changes to the core DAG model.

No changes to the core graph infrastructure. No undirected edges. No
new execution model. The DAG remains a causal process; domain knowledge
lives in `.dag` type hierarchies and compiler analysis passes.

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
