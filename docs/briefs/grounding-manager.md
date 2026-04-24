# Grounding Manager Brief — Post-R1 Grounding Completeness Program

## Orient before reading

- **Program scope authority:** [`ROADMAP.md`](../../ROADMAP.md)
  §"Post-R1 Program — Grounding Completeness" — lane list,
  acceptance gates, dependencies.
- **Thesis claim tracked:** `THESIS.md` §"Thesis claims —
  complete list" → Tier 1 → "Grounding completeness." This
  program is the ROADMAP counterpart of that thesis claim.
- **Architectural authority:** [`docs/single-emitter-design.md`](../single-emitter-design.md)
  (coercion = emission; algebra-homomorphism-not-lookup;
  `TypeCheckpoint` / `InhabitantDecl` dissolve). This
  manager operationalizes that architectural design.
- **Work breakdown + worked examples:** [`docs/thesis/target-grounding-proposal.md`](../thesis/target-grounding-proposal.md)
  (PROPOSAL mode as of this brief's authoring; promotion to
  committed mode happens when this program formally dispatches).
- **Coordination context:** [R1 Director Brief](r1-director-brief.md)
  for the manager-escalation model. This manager operates under
  the same governance — scope changes route to director; manager
  owns lane-level dispatch inside scope.

## Slice

This manager owns all seven lanes in the Post-R1 Grounding
Completeness program per `ROADMAP.md` §"Post-R1 Grounding
lanes":

- **`T-Ground-Pilot`** (S) — Rust integer family pilot. No
  substrate blockers. Demonstrates inhabitance-search routing
  parity against the current table-driven design.
- **`T-Ground-Rust`** (XL) — systematic modeling of Rust
  Reference §Types per
  <https://doc.rust-lang.org/reference/types.html>. Blocks on
  DB-11 (`ROADMAP.md` DB row catalog) + cardinality-substrate
  (`ROADMAP.md:305`).
- **`T-Ground-Python`** (L) — Python data model equivalent.
  Blocks on same substrate dependencies.
- **`T-Ground-Go`** (L) — Go specification equivalent. Blocks
  on same substrate dependencies.
- **`T-Ground-Engine`** (M) — inhabitance-search walker with
  minimum-satisfier selection and fail-closed tie-breaking.
- **`T-Ground-Tests`** (S) — routing-stability TestClaim class
  + L4 witness-based certification.
- **`T-Ground-Dissolve`** (S) — Track 13 closure; single PR
  deleting the scaffolding's routing-path usage.

## Framing question this manager answers

**Are target-side primitive types in Rust, Python, and Go
modeled structurally from their language references, with
coercion resolving by algebra-homomorphism search instead of
name-keyed table lookup — so the thesis's "structure, not
declaration" discipline holds at the realization boundary?**

Today (pre-dispatch state):
- Target-side coercion is table-driven:
  `dsl/std/coercion.dag` declares `TypeCheckpoint` /
  `InhabitantDecl` / `CallableRepr` / `CastSyntax`; per-target
  data in `dsl/extdeps/languages/*/types.dag` populates the
  lookup tables; `src/v3/compiler/src/emit.rs` and
  per-target emitters paste-render via `render_named_template`.
- The `single-emitter-design.md` architectural critique is
  committed design: "The mapping should fall out from the
  algebra, not from a hand-maintained table."
- `target-grounding-proposal.md` provides worked examples for
  five primitive types, the six-layer scope partition, fail-
  closed tie-breaking, and the three-way L4 split (routing /
  structural-shape / algebra-satisfaction).

The ask: close each lane. Produce structural target models,
wire the inhabitance-search engine, assert routing stability,
certify algebra satisfaction via L4, then dissolve the
table-driven scaffolding in a single Track 13 PR.

## Sequence + dispatch

- **Day 1 after program dispatches.** T-Ground-Pilot
  dispatches. No blockers. Scope: Rust `i8`–`i64`, `u8`–`u64`,
  `bool`, `Unit`. Produce structural declarations in a pilot
  `.dag` file (location TBD — a candidate is
  `dsl/extdeps/languages/rust/primitives.dag` sibling to the
  existing types.dag). Build a toy inhabitance-search engine
  against the pilot set. Assert routing parity with current
  table lookup on the ~10 pilot types.
- **Gated on DB-11 closure.** T-Ground-Rust, -Python, -Go
  dispatch for full-reference coverage. DB-11 unblocks
  refinement-carrying qualifiers (`signed`, `wrap_on_overflow`,
  `utf8_encoded`, etc.) as first-class where-clause refinements
  on target primitive declarations.
- **Gated on cardinality-substrate closure.** Container types
  (`Vec<T>`, `[T;N]`, `HashMap<K,V>`, `Option<T>`, `Result<T,E>`,
  slice, tuple, Python `list`/`dict`, Go `[]T`/`map[K]V`) can
  land with cardinality-carrier qualifiers.
- **As Pilot matures.** T-Ground-Engine dispatches the real
  inhabitance-search walker, replacing the toy. Upgrade to
  minimum-satisfier selection, fail-closed tie-breaking with
  structured diagnostic, cross-type coercion paths (UTF-8
  encoding for `String ↔ FreeMonoid<Char>`, etc.).
- **As Engine + full-reference lanes reach parity.**
  T-Ground-Tests dispatches the routing-stability TestClaim
  class. Per-claim: "user's `.dag T` resolves to target's
  `primitive P` on target `L`." L4 witness-based certification
  per `verifiability-invariant.md` for the algebra-
  satisfaction half (Rust's actual `i64` runtime behavior
  obeys `OrderedRing` axioms).
- **Final.** T-Ground-Dissolve fires Track 13 closure. Single
  PR deleting:
  - `TypeCheckpoint.target_type: String` (routing-path use)
  - `InhabitantDecl.template: String` (routing-path use)
  - `TypeRealization.carrier: String` field
  - Per-target `dsl/extdeps/languages/*/types.dag`
    `TypeCheckpoint`/`InhabitantDecl` data (routing-path
    entries; surviving entries graduate to testgen-assertion
    role per `verifiability-invariant.md`'s L0 framing)
  - Call sites in emit pipeline that read the old surface

## Hand-off points

- **Sideways to Surface Manager.** Surface Manager (R1 T-Emit
  lane) owns emit-pipeline template validation, operator
  carrier templates, etc. Grounding's engine work touches the
  same emit pipeline boundary. Coordinate on whether engine
  changes require emit-pipeline changes (likely yes, since
  the engine replaces the declared-carrier read path).
- **Sideways to Testgen Manager.** Testgen Manager (R1
  T-TestGen lane) owns the `TestClaim` runner. Grounding's
  T-Ground-Tests lane lands new `TestClaim` variants
  (routing-stability, L4 witness-based certification). These
  need Testgen runner capabilities that may require further
  schema extensions beyond what R1 ships.
- **Up to director.** Substrate-capability claims this
  program relies on (DB-11 closure, cardinality-substrate,
  optionally DB-18) — if any of those lanes shift schedule,
  flag to director. Grounding's timeline is substantially
  determined by theirs.
- **Up to director.** If pilot surfaces a class of problems
  that suggests the proposal's architecture needs amendment
  (e.g., the minimum-satisfier discipline produces ambiguous
  choices faster than expected), escalate to director for
  design amendment rather than patch-in-lane.
- **Up to director.** Any proposal to amend the
  grounding-completeness claim in THESIS.md (e.g., scoping
  fewer targets, adjusting the Track 13 dissolution trigger)
  is a scope change.

## Pilot-gate acceptance

`T-Ground-Pilot` acts as the dispatch gate for the whole
program. Its deliverables and pass criteria:

- Structural `.dag` declarations for Rust integer family +
  bool + Unit, with algebra-inhabitance declared per target
  primitive.
- Toy inhabitance-search engine that consumes the pilot
  declarations and returns target primitives for sample
  `.dag` types.
- Routing-stability test assertions showing the engine picks
  the same primitive the current table lookup does, across
  the pilot set.
- No regression in existing emit output for the pilot types.
- Brief receipt document summarizing what the pilot learned
  that informs the full-reference lanes' scope.

If Pilot passes, full-reference lanes dispatch as substrate
capabilities unblock. If Pilot surfaces unexpected blockers
(the inhabitance-search approach doesn't compose cleanly for
some class of types, or the minimum-satisfier discipline
produces ambiguous results at unacceptable rates), escalate
to director before dispatching full-reference work.

## Working state

Lane-owner dispatch status (update as sub-deliverables close):

**T-Ground-Pilot:**
- [ ] Rust primitive `.dag` declarations (i8–i64, u8–u64, bool, Unit)
- [ ] Toy inhabitance-search engine
- [ ] Routing-stability tests against pilot set (parity with current table)
- [ ] Pilot receipt document

**T-Ground-Rust:**
- [ ] Numeric types (integer, float, character)
- [ ] Textual types (`char`, `str`, `String`)
- [ ] Sequence types (array, slice, `Vec`, tuple)
- [ ] Struct / enum / union primitives
- [ ] Pointer types (references, raw pointers, `Box`, `Rc`, `Arc`)
- [ ] Function types + closure types (base shapes)
- [ ] Trait-object types
- [ ] Never type

**T-Ground-Python:**
- [ ] Numeric types
- [ ] Sequences (`list`, `tuple`, `range`)
- [ ] Text (`str`)
- [ ] Mappings (`dict`)
- [ ] Sets
- [ ] Callables
- [ ] `None`, modules, classes

**T-Ground-Go:**
- [ ] Numeric + bool + string primitives
- [ ] Array, slice, map, struct, pointer
- [ ] Function, interface, channel

**T-Ground-Engine:**
- [ ] Inhabitance-search walker
- [ ] Minimum-satisfier selection
- [ ] Fail-closed tie-breaking with structured diagnostic
- [ ] Cross-type coercion paths (e.g., UTF-8 `Char ↔ u8`)

**T-Ground-Tests:**
- [ ] Routing-stability TestClaim variant declared
- [ ] Algebra-satisfaction TestClaim variant (L4 witness-based)
- [ ] Per-target coverage of declared primitives

**T-Ground-Dissolve:**
- [ ] Parity verification across layers 1–5
- [ ] Track 13 closure PR (single-step deletion of routing-path scaffolding)

Decisions log (append as they happen):

- _(none yet)_

Open questions for director:

- _(none yet — program is pre-dispatch)_

Cross-manager notifications queued:

- _(none yet — awaits R1 close + substrate-capability closure signals from Surface and Testgen Managers)_
