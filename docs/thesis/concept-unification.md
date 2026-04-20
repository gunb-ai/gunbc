### Concept unification

In a closed system, apparently distinct concepts often collapse into
each other. This is not an optimization — it is a structural fact.
When two "different" mechanisms turn out to be the same mechanism
viewed from different angles, maintaining them separately is a dual
representation (INVARIANTS: "No duplicate representations," "No
parallel implementations").

Known unifications:
- **Coercion cost = complexity.** A type coercion is a .dag function.
  Its cost is whatever CX proves, not a separate lattice.
- **Coercion = emission.** Coercion is not a step before emission —
  it IS emission. The compiler reads a target spec and generates
  code. Whether that code is "a Rust struct" or "a SPICE subcircuit"
  or "an HTTP client" is determined by the spec, not by a separate
  coercion engine.
- **Target language spec = transport spec = interpreter runtime.**
  A Rust language spec, a REST transport spec, and the interpreter's
  execution model serve the same role: they declare **what the
  target is** — its primitives, its syntax, its capabilities — and
  the compiler translates mechanically. The emitter doesn't "know
  Rust" or "know REST." It reads the spec and translates.

  This unification has a concrete sustainability consequence: the
  interpreter does not have per-transport handlers. It reads the
  same transport specs as the emitter (`extdeps/transports/`). The
  transport spec says "shell means: construct argv, invoke subprocess,
  map stdout/stderr/exit to output fields." The emitter renders this
  as Rust source code. The interpreter renders this as a direct
  call to one of three platform primitives (process, HTTP, file).
  Adding a new transport (gRPC, WebSocket, etc.) means adding a
  spec in `extdeps/transports/` — zero compiler changes, zero
  emitter changes, zero interpreter changes.

  The same applies to language specs. Adding a new emission target
  (Swift, Kotlin, etc.) means adding a spec in `extdeps/languages/`
  — zero compiler changes. The spec IS the implementation.

  **The sustainability test:** when the system grows by one transport
  or one language, how many files need editing? The answer should
  be 1: the spec file. If it's more, there's a parallel list
  somewhere that will drift and break.

- **Idempotency + cancellation + redundancy = algebraic
  simplification.** These appear to be three distinct concepts:
  - Idempotency: `f ∘ f = f` (doing it twice = doing it once)
  - Cancellation: `f ∘ f⁻¹ = id` (doing and undoing = nothing)
  - Redundancy: `f₁ ∘ ... ∘ fₙ = g` where `cost(g) < cost(f₁∘...∘fₙ)`
  
  They are all instances of **one mechanism**: the compiler knows the
  algebraic laws on operations (group, monoid, lattice, involution)
  and simplifies compositions symbolically. Three right turns = one
  left turn is not a special case — it's the rotation group Z₄.
  `serialize ∘ deserialize = id` is not a special case — it's an
  inverse pair. The compiler has the algebra; simplification falls
  out. See `std/effects.dag` and `std/algebra.dag`.

**The test:** if adding a new concept requires a new mechanism rather
than being an instance of an existing mechanism, investigate whether
the new concept is really distinct. In a closed system, new concepts
should compose from existing ones. A parallel mechanism is evidence
of a missed unification.

