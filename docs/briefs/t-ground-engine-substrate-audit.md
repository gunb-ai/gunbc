# T-Ground-Engine Phase 0 — Substrate Audit

**Status:** AUDIT-ONLY. No implementation. Engine-Phase-1 implementation
deferred pending manager/Director routing of the substrate gaps surfaced
below.

**Parent:** T-Ground-Engine Phase 1 brief (R2 Grounding Manager dispatch).
Pilot inheritance: PR #765 (commit `2909f9e05`),
`src/v3/grounding_pilot/src/lib.rs`.

## Question

Can the production walker consume `dsl/extdeps/languages/rust/primitives.dag`
declarations directly via the v3 substrate — replacing the pilot's Rust
mirror — while preserving the pilot's parity, fail-closed contract, and
state-space discipline?

Three options, ordered per brief:

- (a) `.dag`-defined walker (e.g. `dsl/grounding/engine.dag`)
- (b) Sibling Rust crate (e.g. `src/v3/grounding_engine/`) consuming
  `primitives.dag` via the v3 substrate
- (c) `src/v3/compiler/` — REJECTED (SG-0 hand-Rust ratchet)

## Conclusion

**(a) is not viable today. (b) is not viable in pure form today either.**
Both block on the same substrate gaps. This finding is escalation-class
per the brief ("Phase 0 audit concludes (b) is required because the v3
substrate can't model the walker in `.dag`. This is a substrate-capability
question manager routes to Director").

The substrate gaps are independent of any walker design choice — they
are properties of the v3 bootstrap loader and the v3 surface grammar /
emission pipeline as-of `royal-cat-649` HEAD (parent `2909f9e05`).

## Substrate gap 1 — extdeps not loaded into the bootstrap Dag

`src/v3/compiler/src/bootstrap.rs:131-151`
(`load_runtime_bootstrap_authorities`) loads four authority sets via
committed generated snapshots:

- `std_fixtures` (`dsl/std/*.dag`)
- `STAGED_FILES` (`src/v3/std/*.dag`)
- `V3_SPECS` (`src/v3/spec/*.dag`)
- `COMPILER_FILES` (`src/v3/compiler/*.dag`)

`dsl/extdeps/languages/rust/primitives.dag` is **not** in any of these
sets. The header comment is explicit (`bootstrap.rs:16-19`):

> Production bootstrap does not inject target-language realizations.
> Realization facts for emitted languages live in `dsl/extdeps/languages/*`
> per the thesis; compiler code does not manufacture those.

**Consequence.** A `.dag`-side walker (option (a)) cannot reach
`rust_pilot_primitives` via name resolution — the declaration isn't in
the Dag. A sibling-crate walker (option (b)) that wants to consume the
`.dag` declaration symbolically (rather than re-mirror it) faces the same
gap: there is no API surface today that returns the parsed
`rust_pilot_primitives` value table to a downstream Rust caller, because
that table is not bootstrap-loaded and there is no public
`Dag::load_extdeps_language(…)` entry point.

This is the **same gap** the pilot confronted and explicitly deferred to
T-Ground-Engine. Engine-Phase-1 is the phase that must close it; the
brief's "Eliminating mirroring is the load-bearing scope of this lane"
language commits us to closing it here.

## Substrate gap 2 — surface/emission features required by a `.dag` walker are not yet shipped

Independent of the loader gap, option (a) requires writing a `.dag`
program that:

1. Loads `data rust_pilot_primitives: List<RustPrimitive>` as a value
   to walk.
2. Pattern-matches the `RustPrimitive` sum (`IntegerPrimitive |
   NonIntegerPrimitive`) to extract `algebra` and `carrier` fields.
3. Filters/folds across `List<RustPrimitive>` selecting on
   `(algebra, carrier)` agreement, where `algebra` is a heterogeneous
   variant-typed enum (`IntegerAlgebra` vs `NonIntegerAlgebra`).
4. Returns a sum-type result that carries either the matched primitive
   or a structured diagnostic (`Ambiguous { candidates }` /
   `NoInhabitant { key }`).

The relevant capability signals as-of HEAD:

- `src/v3/std/list.dag:6-15` (own admission): "the current compiler still
  lacks full structural recursion + list-body emission support." Even
  `fold` / `fold_right` / `concat` bodies live as block text awaiting
  end-to-end lowering/emission.
- `src/v3/std/list.dag:57-64`: "`data list_monoid<element>: Monoid<…> = …`
  is intentionally commented out for now because generic `data` items are
  not yet part of the surface grammar." Non-generic data lists exist (the
  pilot's `rust_pilot_primitives` is non-generic), but the broader gap
  signals the pipeline is still maturing the data-as-value path.
- The pilot's escalation list at `src/v3/grounding_pilot/src/lib.rs:46-53`
  flags the same boundary: pattern-matching `RustPrimitive`'s sum
  variants to dispatch on the heterogeneous `algebra` field is precisely
  the structural query the substrate has not yet been demonstrated to
  emit end-to-end.

Each of (1)–(4) above is plausibly individually within reach, but the
audit's standard is whether the walker can be authored in `.dag`
**today** with confidence the substrate emits a behavior-equivalent
program. The combination is not demonstrated by any existing
`src/v3/std/` or `src/v3/compiler/` `.dag` program — `algebra.dag`
declarations are static facts, not walks over heterogeneous-variant
data.

## Why (b) does not save us in pure form

The brief's (b) is "sibling Rust crate consuming `.dag` declarations
directly — no Rust-constant mirror." Concretely that requires either:

- The v3 compiler to load `dsl/extdeps/languages/rust/primitives.dag`
  into the bootstrap Dag and expose a public lookup so the sibling
  crate can read the parsed `rust_pilot_primitives` value as a
  `Declaration`. **Blocked by Substrate gap 1.**
- A standalone `.dag` parse/lower invocation against just
  `primitives.dag` from inside the sibling crate. This re-parses the
  file at runtime via `crate::parse::parse` + `crate::lower::…`, which
  *does* exist today, but: (i) the public surface for one-shot parsing
  of a non-bootstrap file and reading its `data` declarations as
  walkable values is not established (every existing call site goes
  through `Dag::new()`'s curated authority set); and (ii) this still
  requires a Rust-side traversal over `Declaration`/`Node` shapes to
  reconstruct the `(algebra, carrier)` routing key — which is closer to
  "interpret `.dag` AST in Rust" than to "read a structured target-fact
  table." That interpretation layer is exactly the thesis-critical
  surface this lane is supposed to *eliminate*, not invent.

A degenerate (b′) — sibling crate that re-mirrors `primitives.dag` as
Rust constants again — would be the pilot at scale. The brief
explicitly forbids it: "Engine must consume the `.dag` declarations
directly. Eliminating mirroring is the load-bearing scope of this lane."
For completeness in the manager-facing option space: (b′) was
considered (including a tracked-debt variant with a named dissolution
trigger of "loader-close lands ⇒ mirror deletes") and **rejected
per brief**. Not offered as a route below; named here so manager sees
the rejected alternative rather than its absence.

## What Engine-Phase-1 actually needs from the substrate

Restated as a substrate-capability ask (for manager → Director routing):

- **Loader.** `Dag::new()` (or a sibling public constructor) loads
  `dsl/extdeps/languages/rust/primitives.dag` into the bootstrap Dag,
  with `rust_pilot_primitives` reachable by name, **or** a documented
  one-shot public API (`Dag::load_extdeps_language("rust")` /
  equivalent) returns a Dag containing the file's declarations.
- **Value-as-data access.** Some public accessor returns the parsed
  `rust_pilot_primitives` as either (i) a walkable `.dag`-side
  `List<RustPrimitive>` value reachable from `.dag` code, or (ii) a
  structured `Declaration`/`Node` view stable enough for a Rust caller
  to extract `(target_name, algebra-variant, carrier-variant,
  is_copy[, overflow])` per element without re-implementing the
  parser.

Either branch closes the mirroring gap. The first matches the
project-thesis "compiler operations are `.dag` programs" framing; the
second is the bridge form a sibling crate could consume.

## Adjacent context (informational)

- `src/v3/std/list.dag` is staged with API surface but no end-to-end
  emission for fold/recursion bodies. Even if loader closure (gap 1)
  lands, walking `List<RustPrimitive>` from `.dag` code awaits the
  list-body emission work that staging note flags.
- `src/v3/SELF_HOSTING.md` §2 names L1.5 as "Clean bootstrap" with
  pipeline composition declarations and per-stage fixed-points; the
  "data items as runtime-walkable values" capability is not enumerated
  there as an active milestone, suggesting the closest-fit substrate
  work is upstream of (or sibling to) Engine, not within it.
- Pilot's existing tests (10 routing-parity, fail-closed, state-space)
  are reusable verbatim against any production walker; they are not
  blocked by substrate gaps. The blocker is the walker itself.

## Recommendation to manager

Engine-Phase-1 cannot land as scoped against current main. The
substrate gaps above are the load-bearing prerequisite. Two routes for
manager / Director judgment:

1. **Pre-empt with substrate work.** Land an extdeps loader entry
   (Substrate gap 1 close) before re-dispatching Engine-Phase-1. This
   is the smallest unblocking step and likely sufficient for a
   sibling-crate Engine that reads `Declaration`s structurally without
   re-mirroring (option (b), sharpened — call it (b.i)). Note the
   tension flagged in §"Why (b) does not save us in pure form": the
   `Declaration`/`Node` traversal under (b.i) is itself a mild form of
   "interpret `.dag` AST in Rust." The route is acceptable as a
   transitional bridge with a known dissolution into option (a) once
   Gap 2 closes; not endorsed as a terminal shape.
2. **Pursue (a) directly.** Bundle the loader close with the `.dag`
   walker authoring work, and additionally schedule list-body emission
   / heterogeneous-variant pattern-match capability as Engine-Phase-1
   prerequisites. Larger scope, but lands the project-thesis-aligned
   shape in one motion.

Either route changes the dispatch shape of Engine-Phase-1. Per the
brief's escalation discipline, this audit returns to manager without
inventing a workaround.

## Out of scope for this audit

- Authoring the production walker (deferred pending route choice).
- Editing `src/v3/grounding_pilot/` (the deprecation note is part of
  Engine-Phase-1's PR scope, not this audit).
- Deleting / amending `dsl/extdeps/languages/rust/primitives.dag` —
  the file's structural authority is sound; the gap is in the
  consumer, not the source.
- Touching emit-pipeline call sites (T-Ground-Dissolve scope per
  brief).
