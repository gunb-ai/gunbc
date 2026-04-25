# R2 Grounding Manager Brief

**Status:** `PROPOSAL` (formal) → `ACTIVE` (Director-discretionary).
Formal R2 promotion still pending R1 all-gates-green closure +
[`docs/r2-structure.md`](../r2-structure.md) promotion to ROADMAP.
Director-discretionary dispatch has been used since 2026-04-25 for
T-Ground-Pilot (PR #765, merged) and T-Ground-Engine-Phase-1 audit
(PR #768, merged). Engine implementation parked pending substrate
routing — see [`t-ground-engine-substrate-escalation.md`](t-ground-engine-substrate-escalation.md).
Working state at the bottom of this brief reflects current dispatch state.

## Orient before reading

- **R2 structure authority:** [`docs/r2-structure.md`](../r2-structure.md)
  (currently committed to main via PR #754 + amendments via
  PR #758, still in PROPOSAL mode until R1 closes and it
  promotes to ROADMAP). Names this manager as **R2's single
  standing manager alongside Director**. Director dispatches
  T-Modeling / T-Substrate / T-ImpossibleBugs /
  T-PerMethodMetadata ad-hoc; no second standing manager.
  T-LensMigration / T-ShimFloor / T-EFamilyClose are **R1 gates,
  not R2 lanes** per the r2-structure.md "Lanes deliberately
  absent" section. On promotion, the authoritative section
  becomes `ROADMAP.md §"Release R2 Program"`.
- **Program scope authority:** [`ROADMAP.md`](../../ROADMAP.md)
  §"Post-R1 Program — Grounding Completeness" (`:149`+) — lane
  list, acceptance gates, dependencies. Promotes into R2 lane
  `T-Ground` on R2 proposal merge; the pre-promotion section
  remains the scope reference until then.
- **Thesis claim tracked:** [`THESIS.md`](../../THESIS.md)
  §"Thesis claims — complete list" → Tier 1 → "Grounding
  completeness." This program is the ROADMAP counterpart of that
  thesis claim.
- **Architectural authority:** [`docs/single-emitter-design.md`](../single-emitter-design.md)
  (coercion = emission; algebra-homomorphism-not-lookup;
  `TypeCheckpoint` / `InhabitantDecl` dissolve). This manager
  operationalizes that architectural design.
- **Work breakdown + worked examples:** the target-grounding
  proposal — [PR #695](https://github.com/gunb-ai/gunbc/pull/695),
  landing at `docs/thesis/target-grounding-proposal.md` on merge.
  The proposal doc is in PROPOSAL mode; promotion to committed
  happens when #695 and [PR #721](https://github.com/gunb-ai/gunbc/pull/721)
  both merge and this program formally dispatches.
- **Coordination context:** on R2 promotion, this brief's
  coordination context becomes the R2 Director Brief (refactored
  from [R1 Director Brief](r1-director-brief.md)'s Staffing
  section). This manager operates under the same governance —
  scope changes route to director; manager owns lane-level
  dispatch inside scope.

## Slice

This manager owns all seven lanes in the Post-R1 Grounding
Completeness program per `ROADMAP.md` §"Post-R1 Grounding
lanes":

- **`T-Ground-Pilot`** (S) — Rust integer family pilot. No
  substrate blockers. Demonstrates inhabitance-search routing
  parity against the current table-driven design.
- **`T-Ground-Rust`** (XL) — two-authority modeling: **(a)** Rust
  Reference §Types (<https://doc.rust-lang.org/reference/types.html>)
  — language-level structural types (boolean, numeric, textual,
  never, tuple, array, slice, struct, enum, union, function item,
  function pointer, closure, reference, raw pointer, trait object,
  `impl Trait`); **(b)** std-library carriers (std documentation is
  the authority) — `String`, `Vec<T>`, `Box<T>`, `Rc<T>`, `Arc<T>`,
  `HashMap<K,V>`, `BTreeMap<K,V>`, `HashSet<T>`, `BTreeSet<T>`,
  `Option<T>`, `Result<T, E>`. Authority must not mix — each
  category cites its own source-of-record. Blocks on DB-11
  (`ROADMAP.md` DB row catalog) + cardinality-substrate (row
  "Fixed-width types aren't structurally fixed" under the
  tracked-debt ledger; current line `ROADMAP.md:338`, but line
  numbers drift — search the row title if drift-resistant).
- **`T-Ground-Python`** (L) — two-authority modeling: (a) Python
  language reference built-ins; (b) CPython stdlib (`typing`,
  `collections`) when scope grows beyond built-ins.
  Blocks on same substrate dependencies.
- **`T-Ground-Go`** (L) — two-authority modeling: (a) Go language
  specification (<https://go.dev/ref/spec>) primitives; (b) Go
  standard library carriers beyond the spec. Blocks on same
  substrate dependencies.
- **`T-Ground-Engine`** (M) — inhabitance-search walker with
  minimum-satisfier selection and fail-closed tie-breaking.
- **`T-Ground-Tests`** (S) — routing-stability TestClaim class
  + L4 witness-based certification.
- **`T-Ground-Dissolve`** (S) — Track 13 closure; single PR
  deleting the coercion scaffolding entirely:
  `dsl/std/coercion.dag` (schema), the per-target
  `dsl/extdeps/languages/*/types.dag` instantiation files, the
  `TypeRealization.carrier: String` field, and emit-pipeline
  call sites reading the old surface.

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
- The target-grounding proposal (PR #695) provides worked
  examples for five primitive types, the six-layer scope
  partition, fail-closed tie-breaking, and the three-way L4
  split (routing / structural-shape / algebra-satisfaction).
  Will land at `docs/thesis/target-grounding-proposal.md` on
  merge.

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
  PR **deleting the coercion scaffolding entirely**:
  - `dsl/std/coercion.dag` — the schema file declaring
    `TypeCheckpoint`, `InhabitantDecl`, `CallableRepr`,
    `CastSyntax`. Gone.
  - `dsl/extdeps/languages/rust/types.dag` — per-target
    instantiation tables. Gone.
  - `dsl/extdeps/languages/python/types.dag` — same. Gone.
  - `dsl/extdeps/languages/go/types.dag` — same. Gone.
  - `dsl/extdeps/languages/dag/types.dag` — same. Gone.
  - `TypeRealization.carrier: String` field in
    `src/v3/std/emit_model.dag` — removed (the field's
    structural replacement lands in the new target-primitive
    declarations; the emit-pipeline call sites read from
    those directly via the inhabitance-search engine).
  - All emit-pipeline call sites reading the old surface.
  - Routing-stability assertions that tests depend on are
    authored as **direct `TestClaim` declarations** under
    T-Ground-Tests, not as graduated `TypeCheckpoint` /
    `InhabitantDecl` data. Tests own their own assertion
    surface; the scaffolding has no surviving role.

## Hand-off points

R2 manager structure is **1 standing manager + Director** per
[`docs/r2-structure.md`](../r2-structure.md). R1's Surface /
Testgen / Substrate / Self-hosting / Release managers archive on
R1 close; T-LensMigration / T-ShimFloor / T-EFamilyClose carrier
work closes in R1, not R2. Director dispatches R2's remaining
work (T-Modeling, T-Substrate, T-ImpossibleBugs, §6a
per-method-metadata residual) ad-hoc rather than through a second
standing manager. Grounding's cross-manager handoffs therefore
route exclusively to Director.

- **Sideways / up to Director — emit-pipeline boundary.**
  Grounding's engine work (T-Ground-Engine) replaces the
  declared-carrier read path through the emit pipeline. The
  emit-pipeline surface itself is closed by R1 T-Emit; any
  engine-driven change that requires emit-pipeline amendment
  after R1 close routes via Director for ad-hoc dispatch. Flag
  early rather than absorbing emit-pipeline work into
  T-Ground-Engine.
- **Sideways / up to Director — substrate-capability overlap.**
  Grounding's T-Ground-Rust / -Python / -Go lanes depend on
  cardinality-substrate + DB-11 closure for full-reference
  coverage. R2's T-Substrate sub-lanes (dispatched by Director
  ad-hoc) are **scoped to T-Modeling unblocks only**, not full
  substrate-capability completion. If Grounding needs broader
  substrate work than T-Substrate's scoped acceptance covers,
  surface that as a substrate scope-creep flag to Director
  rather than silently expanding T-Substrate scope.
- **Sideways / up to Director — testgen predicate extensions.**
  The `TestClaim` runner Grounding's T-Ground-Tests lane depends
  on is no longer a standing-manager authority in R2 (R1 Testgen
  Manager archives on R1 close). New `TestClaim` variants
  (routing-stability, L4 witness-based certification) that
  require predicate schema extensions beyond R1's shipped set
  route to Director for ad-hoc dispatch.
- **Up to director.** Substrate-capability schedule-shift flags
  (DB-11, cardinality-substrate, parametric-algebra-attachment
  subset). Grounding's timeline is partly determined by theirs.
- **Up to director.** If pilot surfaces a class of problems
  suggesting the proposal's architecture needs amendment (e.g.,
  minimum-satisfier discipline produces ambiguous choices faster
  than expected), escalate for design amendment rather than
  patch-in-lane.
- **Up to director.** Any proposal to amend the grounding-
  completeness claim in THESIS.md (e.g., scoping fewer targets,
  adjusting the Track 13 dissolution trigger) is a scope change.

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

**T-Ground-Pilot:** ✅ COMPLETE (PR #765 merged 2026-04-25 commit `2909f9e05`; receipt in [`grounding-pilot-receipt.md`](grounding-pilot-receipt.md))
- [x] Rust primitive `.dag` declarations (i8–i64, u8–u64, bool, Unit)
- [x] Toy inhabitance-search engine
- [x] Routing-stability tests against pilot set (parity with current table)
- [x] Pilot receipt document

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

**T-Ground-Engine:** ⏸️ PARKED — Phase 0 substrate audit complete (PR #768 merged 2026-04-25 commit `4afc0d794`); Director routed Route 1 (small loader-close, ad-hoc Director dispatch). Engine re-dispatches in sharpened-(b) form once the loader-close PR merges. See [`t-ground-engine-substrate-escalation.md`](t-ground-engine-substrate-escalation.md).
- [x] Phase 0 substrate audit (concluded escalation-class)
- [ ] Inhabitance-search walker (Phase 1, blocked on loader-close PR)
- [ ] Minimum-satisfier selection (Phase 1)
- [ ] Fail-closed tie-breaking with structured diagnostic (Phase 1; pilot established the contract baseline)
- [ ] Cross-type coercion paths (e.g., UTF-8 `Char ↔ u8`) (Phase 2; blocks on full-reference)

**T-Ground-Tests:**
- [ ] Routing-stability TestClaim variant declared
- [ ] Algebra-satisfaction TestClaim variant (L4 witness-based)
- [ ] Per-target coverage of declared primitives

**T-Ground-Dissolve:**
- [ ] Parity verification across layers 1–5
- [ ] Delete `dsl/std/coercion.dag`
- [ ] Delete `dsl/extdeps/languages/rust/types.dag`
- [ ] Delete `dsl/extdeps/languages/python/types.dag`
- [ ] Delete `dsl/extdeps/languages/go/types.dag`
- [ ] Delete `dsl/extdeps/languages/dag/types.dag`
- [ ] Remove `TypeRealization.carrier: String` field from `src/v3/std/emit_model.dag`
- [ ] Remove emit-pipeline call sites reading the old surface
- [ ] All test claims assert against the new structural authority (no residual reads of deleted scaffolding)
- [ ] Delete `src/v3/grounding_pilot/` workspace member (scope-expansion confirmed in PR #765)
- [ ] Delete Engine sibling crate if Route 1/3 dispatched as (b.i) (scope-expansion contingent on Director routing)
- [ ] Track 13 closure PR lands — single step

Decisions log (append as they happen):

- **2026-04-25** — Pilot dispatched and merged under Director-discretionary dispatch (R2 promotion gates not yet formally fired). PR #765.
- **2026-04-25** — Pilot lessons synthesized into [`grounding-pilot-receipt.md`](grounding-pilot-receipt.md). Five carry-forwards: Stratum-B finding, mirroring-as-substrate-ask, fail-closed contract baseline, SG-0 location reasoning, state-space discipline as full-reference precedent.
- **2026-04-25** — Pilot's `RustPrimitive` partition (`IntegerPrimitive | NonIntegerPrimitive`) locked as full-reference precedent shape via codex P2 adjudication. Future widening to flat record is out of scope.
- **2026-04-25** — Engine-Phase-1 brief landed (PR #767, `t-ground-engine-phase-1.md`). Audit-first discipline; no Rust-constant mirroring; variant-aware walker; SG-0 untouched.
- **2026-04-25** — Engine-Phase-1 Phase 0 audit landed (PR #768, `t-ground-engine-substrate-audit.md`). Conclusion: both options (a) and (b) block on the same substrate gaps; escalation-class. Engine implementation parked.
- **2026-04-25** — Engine substrate ask escalated to Director via [`t-ground-engine-substrate-escalation.md`](t-ground-engine-substrate-escalation.md). Manager recommended Route 3: route Gap 1 close through Pure Bootstrap to Zero program (PB-1 / PB-Bootstrap-Process scope overlap). *(Past tense — overruled in next entry.)*
- **2026-04-25** — Director chose **Route 1** (small loader-close, ad-hoc Director dispatch). Manager recommendation overruled with substantive reasoning: PB-1 migrates *existing* fixture sets; deciding the bootstrap shape includes a new fifth set is upstream of PB-1's pattern. Faster unblock: sharpened (b) Engine within days vs quarters for full Pure Bootstrap to Zero scope. Manager internalized the upstream-vs-pattern distinction for future cross-manager flagging.

Open questions for director:

- ~~**Engine substrate routing**~~ — RESOLVED 2026-04-25. Route 1 chosen by Director.
- **R2 promotion timing** — formally still PROPOSAL pending R1 all-gates-green closure. R1 Census Close landed (PR #763) but Director-discretionary dispatch has been used for Pilot + Engine-Phase-1-audit. Consider formalizing the promotion or making the discretionary mode explicit in `r2-structure.md`. **See Tracked debt below.**

Tracked debt (manager surfaces, Director resolves):

- **Dual-status convention (`PROPOSAL` formal / `ACTIVE` discretionary).** Currently load-bearing for two dispatched lanes (Pilot, Engine-Phase-1 audit) and likely a third (loader-close → Engine re-dispatch). **Owner**: Director. **Forcing function**: if this convention survives one more dispatch (i.e., loader-close lands and Engine re-dispatches under it), manager surfaces as escalation rather than queued open-question — the cost of dual-authority dispatch state grows non-linearly past three live dispatches per `feedback_state_space_vs_behavioral_invariants` (status as an enum-with-implicit-modes admits illegal combinations like "PROPOSAL but ACTIVE"). **Resolution paths** (Director picks one): (a) formalize R2 promotion explicitly via `r2-structure.md` amendment; (b) add a "Director-discretionary dispatch" mode to `r2-structure.md` as a first-class status alongside PROPOSAL/ACTIVE; (c) defer formalization with explicit "do not let this accumulate" sunset commitment.

Cross-manager notifications queued:

- ~~**Pure Bootstrap to Zero Manager** (PR #766) — substrate ask~~ — Director chose Route 1 (ad-hoc dispatch, not routed through Zero-Floor). Director handles Zero-Floor heads-up directly per cross-program coordination. No manager-side notification queued.
- **Surface / Testgen Managers** (R1, archived on R1 close per `r2-structure.md`) — pilot's fail-closed-by-construction shape (`GroundingError::{Ambiguous, NoInhabitant}`) is general-purpose, not Grounding-specific. If their wind-down work hasn't closed yet, this shape is reusable as a contract baseline.
