> Part of: [THESIS.md](THESIS.md) — this is the **scan surface** for checking that a new
> design advances the thesis instead of dropping, circumventing, or antagonizing it.

# Direction Conformance Checklist

A single scannable list of everything the project is trying to be, phrased as
**conformance questions**. Run it against any new design, brief, or PR: for each
item, ask *"does this design satisfy this, or does it quietly trip the red flag?"*
A design that makes any answer **NO** — or that introduces the named anti-pattern —
is dropping/circumventing/antagonizing the direction and should be **escalated
before it lands**, not waved through.

## How to use it

- This is a **per-design scan**, not a project tracker. The `- [ ]` boxes are for
  ticking *while reviewing one design* ("does this design clear this bar?"), then
  discarding — they are not milestones to complete once.
- 🚩 marks the **anti-pattern** — the concrete shape that means the design is
  fighting the direction. Scanning for the red flag is usually faster than proving
  the positive.
- Each cluster cites its **authority** (`→ DOC §...`). **The authority always wins.**
  This file is a derived index, not a new source of truth: if it disagrees with
  THESIS / INVARIANTS / ROADMAP / MODELING, the authority is right and *this file is
  the bug* — fix it here, don't fork the rule. It deliberately does **not** restate
  the detailed facts those docs own (per the ledger standing principle in CLAUDE.md);
  it points at them.
- Not every item applies to every design. "N/A" is a fine answer. A **NO** is the
  signal. Two or more NOs, or any NO on §1–§4, is a hard stop / escalate.

---

## §0. The one bet (read before everything else)

> Everything below protects a single wager: **model local, derive global.** Every
> target — language, format, service, persistence layer — is modeled **once** in the
> shared substrate, and the compiler **derives** the N×M translations between them as
> homomorphisms it computes, never adapters anyone authors. The whole project exists
> to make that bet pay off. → THESIS §"The derived homomorphism", `docs/thesis/the-derived-homomorphism.md`

- [ ] **Serves the bet.** Does this design move us toward N+M models with derived
      translation, or toward N×M hand-written paths? 🚩 *Any per-pair adapter, per-target
      compiler branch, or "just special-case this one" path is the thing the whole
      design exists to delete.*
- [ ] **Cost of change → 1.** When the language grows by one type / expression /
      transport / target, how many files must change? The answer must approach **1**.
      🚩 *A change that requires editing many files in lockstep is duplicate authority
      wearing a feature costume.* → CLAUDE.md "Cost of Change"

---

## §1. Modeling faithfulness (P1) — the foundation; nothing recovers from a fiction here

> Every construct grounds in a declared external fact or a structural derivation from
> one. Grounding is **intersubjective** (math, CS consensus, machine standards), never
> an internal taxonomy restating itself. In a closed system, **heuristics are never
> structurally necessary** — a heuristic is evidence of a missing upstream fact.
> → INVARIANTS P1; MODELING "Core principle", M1–M10

- [ ] **Grounded, not invented.** Does every new node point at an external authority
      (cited) or derive structurally from one? 🚩 *A fact computed from "nearby signals"
      with no declared source — a heuristic. Every corner case becomes another rule.*
- [ ] **Fact-bundle, not hollow alias.** Does a type modeling a spec primitive carry
      the spec's facts (a `Conj` of named edges), or `type X = Y` and stop? 🚩 *A bare
      alias asserts an identity it never proves and drops every fact the spec states
      (width, signedness, representation…). It passes shape-checkers precisely because
      it's empty.* → INVARIANTS P1 "Hollow alias"; MODELING M1
- [ ] **Name is what it is.** Does the type's name reflect its actual structure?
      🚩 *A nickname (`FooBar` that isn't `Foo`∘`Bar`; `ModulePath` that's a
      `QualifiedName`) makes every consumer inherit the lie.* → INVARIANTS P1 "Nickname"
- [ ] **Canonical name for a canonical concept.** If the field already named this
      (lexical analysis, grammar, production), does the design use that name in `std/`?
      🚩 *Coining an internal name (`GrammarSchema`, `ModeledLexRules`) for a concept CS
      already named, especially in a pipeline-stage module.* → INVARIANTS P1 "Internal vocabulary"
- [ ] **DFS the ontology first.** Did the design search `dsl/std/` for the existing
      parent concept before declaring a new type, and attach by inhabitance? 🚩 *A new
      sibling type next to concepts that share an unnamed parent (the `Interval<D>` /
      `Dimension<Unit,Carrier>` pattern).* → MODELING M9; INVARIANTS P1 "substrate-fact introduction"
- [ ] **Coproduct vs coordinate.** For `Foo = A | B | C`: are these genuine
      alternatives, or coordinates of a record masquerading as a sum? 🚩 *A sum type
      where one inhabitant carries values for all variants at once (`Cost = Time|Space`
      → should be `{time, space}`).* → INVARIANTS P1 Step 2
- [ ] **No hand-rolled derived operation.** If a function's behavior is fixed by a
      modeled type's shape, is it consuming the derived primitive (catamorphism,
      traversal, grammar model) rather than re-deriving it by hand? 🚩 *A second
      authority for a fact the model already determines.* → MODELING M1 Practice 10
- [ ] **No silent default.** Missing data stays missing or becomes a diagnostic.
      🚩 *Silence as fabrication — defaulting an absent fact to a plausible value.*
      → MODELING M5; INVARIANTS P3
- [ ] **Landed ≠ correct.** Is the design re-examining the model it builds on rather
      than treating "it's already on main / multiple PRs approved it" as proof?
      🚩 *Replicating an existing pattern as grounding ("it already works this way").*
      → INVARIANTS P1 "Existing implementations…", "Pattern presence as grounding"

---

## §2. Substrate shape — two coordinated substrates; must not be flattened or casually extended

> Types are Node trees with **six** connectives (`Atom | Conj | Disj | Arrow |
> Cardinality | Instantiation`). Computation is **five** L1 behaviors (`Value |
> Transform | Branch | Loop | Bind`). `service`/`fn`/`type`/`operation` are surface
> sugar over this. → THESIS "Substrate shape"; `docs/architecture.md`

- [ ] **Inside the six + five.** Does the design express itself in the existing
      connectives/behaviors (or sugar over them), without needing a seventh connective
      or sixth behavior? 🚩 *A substrate extension. This is a C1-class STOP signal: all
      four dissolution patterns from "Structural decompression" must fail with
      structural arguments first.* → THESIS "Substrate shape"; `docs/thesis/structural-decompression.md`
- [ ] **Two substrates, not one bag.** Does it keep the type substrate and behavior
      substrate coordinated rather than flattening them into one undifferentiated node
      soup? → THESIS, `docs/thesis/the-substrate-two-coordinated-shapes.md`
- [ ] **Hosts the algebra.** Could any candidate Declaration shape host
      `dsl/std/algebra.dag` as-is? 🚩 *If not, the shape is too narrow.* → THESIS "Epistemic stacking"
- [ ] **Surface is sugar.** Is new surface syntax sugar that lowers to the kernel, not
      a new semantic primitive? → THESIS; INVARIANTS P4 "Recursive Syntax Is Sugar"

---

## §3. Epistemic stacking — every concept grounds in primitives; this is load-bearing for codegen

> Every concept is a node in an ontological DAG rooted at minimal primitives (Magma,
> Monoid, BooleanAlgebra, FreeMonoid<T>). Concrete types attach **by inhabitance**;
> operations fall out, never declared separately. **The epistemic chain IS the emission
> algorithm.** → THESIS "Epistemic stacking"; `docs/thesis/epistemic-stacking.md`

- [ ] **No opaque concept.** Does every concept trace to a primitive, or does it float
      as an opaque name? 🚩 *A concept introduced without a path back to the roots.*
- [ ] **Operations fall out of inhabitance.** Are operations derived from algebra
      membership (Int inhabits OrderedRing → add falls out), not declared per-type?
      🚩 *An emitter special case — it's evidence of an ungrounded concept upstream.*
- [ ] **One substrate for math and domain.** Does a domain type (`CIWorkflow<Step>`)
      declare and project the same way `Int` does? → THESIS "Epistemic stacking"

---

## §4. The derived homomorphism + coercion in BOTH directions (emit ↔ ingest)

> Coercion is **one mechanism run both ways**: ingestion is coercion, emission is
> coercion — a structure-preserving search over declared inhabitants, performed by the
> compiler, not a hand-written adapter. The whole language is **`.dag → IR → .dag`**.
> It is a **total decision procedure**: every realization either yields a structure-
> preserving `HomomorphismWitness` or **fails closed** with a located
> `CoercionMismatchKind`. → ROADMAP "Coercion in both directions"; `src/v4/std/coercion.dag`; THESIS "Two groundings", "Target realization efficiency"

- [ ] **One mechanism, both directions.** Does the design treat emit and ingest as the
      *same* semantic-realization search, not two separate engines? 🚩 *A bespoke
      ingestion path that doesn't reuse the coercion/inhabitance machinery; a "coercion
      engine" separate from emission.* → THESIS "Coercion = emission"
- [ ] **Witness on success, fail-closed on failure.** Does a successful realization
      carry a `HomomorphismWitness`, and does an unrealizable one refuse with a located
      `CoercionMismatchKind` (`NoTargetCandidate` / `WouldLoseInformation` / opaque-atom
      with no per-target realization)? 🚩 *Synthesizing silent glue, partial/implicit
      realization, or "translation always succeeds" assumed.* → ROADMAP coercion §
- [ ] **Refinement-aware.** Does it distinguish *faithful widening* (`i32 → int`, witness)
      from *lossy narrowing* (`int → i32`, `WouldLoseInformation`)? → ROADMAP coercion §
- [ ] **Proven by run, not prose.** Are the coercion claims backed by `TestClaimRun`
      verdicts (positive, negative, and `emit → ingest` round-trip), not asserted in
      comments? → ROADMAP coercion §; `src/v4/test/claim/round_trip/`
- [ ] **Boundary projections stay boundaries.** Do tokenize/parse and print/render stay
      thin boundary projections, *not* a second adapter authority that competes with
      the coercion search? 🚩 *Per-pair adapters re-emerging at the I/O edge.* → ROADMAP coercion §
- [ ] **`.dag → IR → .dag` means canonical source.** Round-trip means canonical `.dag`
      *source* regeneration (normalized equality, not bit-identical unless claimed) —
      JSON IR stays a boundary/debug artifact. 🚩 *Promoting a JSON receipt to "the IR".*
      → ROADMAP coercion §

---

## §5. Correctness is structural, not behavioral — dimensions, tiers, lenses

> Every correctness dimension (type, arity, unit, effect, complexity, ownership,
> idempotency, any user invariant) is a **structural fact carried by the data model** —
> validation is *reading the structure*, not running the code. Correctness scales with
> structural surface, not human attention. → THESIS "Correctness dimensions", "What falls out"; `docs/thesis/correctness-dimensions.md`

- [ ] **Structural, not test-time.** Is the new correctness property a structural fact
      the compiler reads, not a behavioral check bolted on at test time? 🚩 *Catching an
      invariant via a runtime check/profiler/linter instead of by derivation.*
- [ ] **Right tier.** Tier 1 (impossible to write the bug) and Tier 2 (proven safe or
      total) close at **compile time** by reading structure; Tier 3 runs emitted code but
      its surface is `TestClaim` data. Is the obligation placed at the tier it belongs
      to? → THESIS "Tier 1/2/3"
- [ ] **User-extensible via lenses, not a second rule system.** Is a new invariant a
      lens (a fold over the same decomposition) the compiler validates the same way as
      built-ins? 🚩 *A parallel proof infrastructure for one new concern.* → THESIS "User-defined dimensions"
- [ ] **No parallel taxonomy for effects.** Are read/write effects read off the
      type-signature shape (returned-modified-resource = write), not a separate
      enumerated `OperationEffect` annotation layer? 🚩 *Tracking effects as a separate
      concept IS the bug pattern.* → THESIS impossible-bug classes (unenumerated effects)

---

## §6. Omni-emission — Shape A vs Shape B; O(1) per target, never O(N×M)

> One declaration projects onto every layer (DB schema, backend, API client, frontend,
> docs) from one source; coherence is structural, drift impossible. **Shape A** =
> compiler language targets (one language spec in `extdeps/languages/`, zero compiler
> changes). **Shape B** = user-program artifacts (YAML/HCL/SQL/docs) emitted by `.dag`
> programs walking typed values. → THESIS "Omni-emission"; `docs/thesis/what-else-falls-out.md` §"Two shapes"

- [ ] **One spec per target, no compiler edits.** Does adding a Shape A target cost one
      language spec and zero emitter/compiler changes? 🚩 *A new compiler path per target.*
- [ ] **Shape A vs Shape B kept distinct.** Is a programming-language target emitted by
      the compiler, and everything else (configs, manifests, docs) emitted by user `.dag`
      programs? 🚩 *Blurring the two — building a compiler render path for SQL/YAML, or
      pushing a language into user-space.* → THESIS "the distinction must not be blurred"
- [ ] **Cost scales with content, not layers.** Shape A is O(1)/language, Shape B is
      O(1)/artifact class — neither O(N×M). → THESIS omni-emission cost scaling
- [ ] **Emission independent of intent.** Is *what the system does* declared separately
      from *what artifacts it becomes*? → THESIS

---

## §7. Self-inspection / reflection — read axis; by execution, not runtime metaprogramming

> The same substrate that models user programs models the compiler's own structures —
> *types-as-data*. This is the **read axis** of programmatic access. The runtime
> *reflection-by-execution* half is a **build, not a settled fact** (measured unbuilt,
> ctrl#1480/#1481). → THESIS "Self-inspection"; `docs/thesis/self-inspection.md`; `gunbc-planning/programmatic-access-single-roof-2026-06-07.md`

- [ ] **Self-inspection, not metaprogramming.** Does the design inspect declared
      structure (coproduct arms, record fields, declaration roster) as *data folded by
      execution*? 🚩 *Runtime name-keyed lookup over a typed stack = metaprogramming —
      out of bounds. The type-introspection need is a compile-time arm-enum (the
      type-dual of the discriminant), not reflection.*
- [ ] **Proven by execution.** Does a reflection claim show a `.dag` fn returning the
      fact **by running**, plus a control proving the host did *not* pre-enumerate it?
      🚩 *"Self-inspection works" asserted from a hand-literal mirror or a typecheck.*
      → INVARIANTS P2 "Reflection evidence is not structural proof"; THESIS self-inspection
- [ ] **One read authority.** Does reflection consolidate onto the substrate surface
      rather than adding a parallel reflected accessor next to a hand walker? 🚩 *Two
      paths to the same fact (`discriminant(v)` vs a hand `_discriminant` bridge).*
      → INVARIANTS P2 "Parallel authority"; THESIS "Compiler–std consolidation"

---

## §8. Fail-closed (P3) — typed diagnostics, never fabricated plausible output

> Every path either succeeds fully or fails with a typed diagnostic. No fabricated
> fallback ever stands in for a missing fact. → INVARIANTS P3; C-1..C-10

- [ ] **No fabricated fallback.** On missing/malformed input, does the design emit a
      typed failure carrier naming what was missing, and refuse to proceed? 🚩 *A null
      sentinel, `<error:unknown>` type, `"Unknown"` string, silent clone, or `Dynamic`
      as universal fallback — any of the C-1..C-10 patterns.* → INVARIANTS P3
- [ ] **No open-set case enumeration.** Is open-ended behavior driven by a typed data
      table with provable exhaustiveness, not a string-keyed case list with a silent
      default branch? → INVARIANTS P3 "Case enumeration for open sets"
- [ ] **Earliest detection.** Does a structural error fail at the earliest stage that
      can prove it, not cascade from a downstream symptom? → INVARIANTS P3 "Early Detection"
- [ ] **No fabrication sentinels.** No `__BUG_*` / `__EMIT_BUG_*` / empty-node fabrication.
      → THESIS "Modeling discipline"; C-9

---

## §9. Decidability (P4) — bounded forward execution; recursion is sugar

> Every accepted program stays in a closed system whose correctness is structurally
> decidable. The premise is **bounded forward execution**: time flows forward, execution
> walks a bounded structure, cycles are relations over acyclic values — never direct
> cyclic values. → INVARIANTS P4

- [ ] **Bounded.** Does every accepted form have an explicit upper bound (recursion →
      `Loop` with max depth, iteration → bounded fold), or is it rejected at the
      boundary? 🚩 *Arbitrary recursion / uncapped iteration / a heuristic timeout.*
- [ ] **Recursion is sugar over the bound.** Does recursive surface lower to the bounded
      primitive rather than add new semantic power? → INVARIANTS P4 "Recursive Syntax Is Sugar"
- [ ] **Lowering is the receipt.** Is there an explicit lowering from the surface form to
      the decidable substrate (the proof it's in-bounds)? → INVARIANTS P4 "Lowering Table"
- [ ] **Verification is a consumer, not an authority.** Do verification predicates read
      the same declared facts every other consumer reads, not a parallel re-derivation?
      🚩 *A verifier with its own copy of the facts that can drift.* → INVARIANTS P4 "Verification as separate pass"

---

## §10. Progress is dissolution (P5) — no bridge/scaffold as steady state

> A change is progress only if it **reduces** ad-hoc state, duplicate authority, or
> implicit behavior. Bridges, deprecations, and migrations-as-steady-state are not
> legitimate permanent shapes. → INVARIANTS P5

- [ ] **Atomic representation change.** Does the new representation land *and* the old
      one delete *and* every consumer migrate in the same change? 🚩 *A bridge/adapter/
      shim "until consumers move" — half-migrations that become the normal case. If it's
      too big for one change, the new representation usually isn't ready; don't ship the
      bridge.* → INVARIANTS P5 "Bridge as steady state"; "No Bridges"
- [ ] **Scaffold has a dissolution trigger.** Does every scaffold land with a specific,
      checkable condition that closes it, named in the same change? 🚩 *A scaffold with
      no trigger becomes accidentally load-bearing.* → INVARIANTS P5 "Scaffold without dissolution trigger"
- [ ] **No new duplicate authority.** Does the design avoid creating a second place a
      fact lives (incl. a ledger doc duplicating model marks / inline comments)? 🚩 *Two
      authorities for one fact.* → INVARIANTS P2; CLAUDE.md "Ledger standing principle"
- [ ] **Net reduction.** Would a more holistic model reduce total lines/concepts? A
      change that only adds is suspect. → INVARIANTS P1 calibration; P5

---

## §11. The consumer law (E-10) — code without a consumer; "done" = green by execution

> A **consumer is anything that breaks when the behavior is wrong.** Typecheck and
> `.contains()` greps are NOT consumers. "Done" means a consumer running green **by
> execution**, with a case that goes **red when the behavior is wrong**. This is the
> single most-relearned lesson in the repo — read INVARIANTS "the specification-without-
> execution trap" first. → INVARIANTS E-10 + "Reviewer's three questions"; `docs/v4-compiler-migration.md`

- [ ] **Q1 — Consumer?** Is there something that breaks if this behavior is wrong (not a
      typecheck, not a grep)? 🚩 *No → the code is experimental: it goes to
      `src/v4/experimental/`, it does not enter the active tree, it is not "done."*
- [ ] **Q2 — Green by execution, shown?** Does the design/PR contain the consumer *run*
      (command + output), not "typechecks" / "merged" / "looks right"? 🚩 *No → the done
      claim is denied. Merged ≠ runs.*
- [ ] **Q3 — Green for the right reason?** Does it show the discriminating input that goes
      red when the behavior is wrong? 🚩 *A green that only ever hit the first arm proves
      nothing (the match-on-symbol silent fail-open).*
- [ ] **Foundation isn't exempt.** Are foundational/base-primitive claims weighted by
      whether a *consumer ran them*, not by how settled they look? 🚩 *"DONE-by-typecheck"
      at the base of the stack — the most dangerous place to mark done.*
- [ ] **A type has a consumer.** Does every declared type have ≥1 structural consumer?
      → THESIS "Modeling discipline"; INVARIANTS E-10

---

## §12. Boundary discipline (P2) — single authority, strict layer DAG

> Boundaries carry enough declared information for mechanical consumers; every fact
> lives in exactly **one** authoritative place. Module layers form a strict DAG:
> `std/` ← `extdeps/` ← `compiler/` ← `workflow/` (imports only toward `std/`).
> → INVARIANTS P2

- [ ] **No cross-layer (wrong-direction) import.** Does the design avoid `extdeps/ →
      compiler/` or `std/ → compiler/` edges? Any type shared between `extdeps/` and
      `compiler/` belongs in `std/`. 🚩 *The compiler owning the schema of a language it
      should be reading (e.g. `python.dag` importing `v4.compiler.parse`).* → INVARIANTS P2 "Cross-layer import"
- [ ] **Consumer reads a typed query, not storage shape.** Does a downstream stage read
      a lower layer through declared accessors, not by walking its internal storage? 🚩 *A
      consumer that must evolve in lockstep with storage layout.* → INVARIANTS P2 "Consumer reverse-engineers storage"
- [ ] **Landed = declaration + realization + generated consumer.** Is the boundary backed
      by a *generated* consumer proof, not just a declaration and a hand-written caller?
      → INVARIANTS P2 "When a boundary counts as landed"
- [ ] **Host-process boundary is typed.** If it spawns host processes (ExecuteCommand),
      are outcomes typed carriers (not string probes), with setup-failure ≠ logical-exit,
      isolated child I/O, and no implicit re-execution? → INVARIANTS P2 "Host-process boundary"
- [ ] **CI authority respected.** Is `.github/workflows/ci.yml` edited only if the brief
      names it? 🚩 *Bundling an incidental CI fix into an unrelated PR.* → INVARIANTS P2 "Task-scope drift"

---

## §13. Tests are structural data — TestClaim, 0-floor, no Rust-authored tests

> All tests are `TestClaim` declarations in `.dag` under the 0-floor cascade promotion.
> A hand-authored `.rs` test is a **language smell** — it flags a predicate / effect-
> model / mock surface the language doesn't yet express. → THESIS "Tests are structural data"; TESTING.md; `docs/design-pure-bootstrap-zero.md`

- [ ] **TestClaim, not hand-Rust.** Is new coverage expressed as `.dag` `TestClaim`
      data (or generated runner), not a hand-authored `.rs` test? 🚩 *Every hand `.rs`
      test is debt with a named dissolution; the release gate is **zero** outside the
      (now-empty) pure-bootstrap residual.*
- [ ] **Behavioral or external-oracle, not declaration-mirror.** Does the test fold the
      compiled Node and discriminate (mutate → red), or is it a white-box test that just
      mirrors what the declaration already states? 🚩 *A decl-shape pin / source-grep of
      the code's own structure = "2FA for code" — a second copy of one authority, zero
      real coverage → delete, don't migrate.* → TESTING.md; feedback on white-box tests
- [ ] **Relocation must not launder.** When a test moves layers, does it stay a real
      discriminating consumer, or does it become a green tautology? 🚩 *"0 dropped" must
      mean zero coverage lost, not zero-because-converted-to-a-grep.* → relocation-launder rule

---

## §14. Self-hosting — four facets; hand-authored Rust → 0

> Self-hosting is four targets: (1) compiler written in `.dag`; (2) self-emits a
> fixed-point (the `.dag` graph is the source of truth, emitted Rust is one
> realization); (3) tests are `.dag` data; (4) gunbc applies its own lenses to its own
> build/CI (`bootstrap.dag`, `ci.dag`). → THESIS "Self-hosting — four facets"; src/v3/SELF_HOSTING.md; BOOTSTRAP.md

- [ ] **`.dag` is the source of truth.** Does the design treat emitted Rust as one
      realization of the `.dag` graph, never a parallel authority needing manual sync?
      🚩 *Hand-editing emitted Rust; "cementing" Rust into templates to satisfy a census
      ratchet (the ratchet is downstream of substrate migration, not a path to it).*
      → session project-spirit; THESIS facet 2
- [ ] **Shrinks toward zero.** Does it reduce (or at least not grow) the hand-authored
      Rust seed? The `hand_maintained_src` list monotonically shrinks to empty. 🚩 *New
      durable load-bearing Rust, especially in a frozen crate.* → THESIS facets; ROADMAP "Pure bootstrap"
- [ ] **Model-before-implement.** Did the domain types land in `std/`/`extdeps/` before
      a pipeline stage migrated to consume them? → session project-spirit
- [ ] **Facet-4 scope respected.** Lens self-application targets `{bootstrap, ci}` (+
      bounded `release.dag`) only — *not* the work-direction process. 🚩 *Modeling
      briefs/cycles as `.dag` data.* → THESIS facet 4 scope

---

## §15. Audience duality + adoption by economics (don't antagonize either)

> The core language stays approachable (types, fns, match, effects, workflows); the
> advanced surface (lenses, proofs, user reflection) is **opt-in depth**. Adoption is
> gated by **economics, not enforcement**: low cost of entry × high free value.
> → THESIS "Audience duality", "Adoption model"

- [ ] **Base stays approachable.** Does the design add depth without making the base
      language harder for an engineer who just wants multi-target emission? 🚩 *Forcing
      the lens/proof surface onto every user; an annotation surface as the cost of entry.*
- [ ] **Guarantees by construction, not opt-in.** Are complexity/effects/termination/
      ownership free *by using the language at all*, not behind a flag? 🚩 *A license
      check / static-analyzer-flagging-non-compliance recruiting mechanism.*

---

## §16. Enumerable impossible-bug classes (thesis commitments — don't silently drop)

> The thesis names specific bug classes that become impossible by construction. Adding
> one is a thesis commitment; removing one requires a named dissolution (the proof
> became trivial). → THESIS "Enumerable impossible-bug classes"

- [ ] **Doesn't regress a committed class.** Does the design keep the release-scoped
      classes provable — suboptimal-complexity contract, idempotency-contract, transport/
      type drift? 🚩 *A change that reintroduces any of these as catchable-only-at-runtime.*
- [ ] **Removal is a named dissolution.** If a design drops a class, is it because the
      proof became trivial (cited), not because it got inconvenient? → THESIS

---

## Quick triage (when you only have 30 seconds)

Scan these five — a NO on any is almost always a real problem:

1. **§0** Does it move toward *derive global* (N+M), or toward N×M hand-paths?
2. **§1** Is every new fact grounded externally, or is there a heuristic / hollow alias?
3. **§4** If it touches translation: one bidirectional coercion mechanism, witness-or-fail-closed?
4. **§10/§11** Does it *reduce* duplicate authority, and does a consumer run it green by execution (red-when-wrong shown)?
5. **§2** Does it stay inside the six connectives + five behaviors (no quiet substrate extension)?

> If a design trips any of these and you're not certain it's still consistent with the
> authority docs, **STOP and escalate** rather than improvising past the doubt. The bar
> is higher for files INVARIANTS.md / SELF_HOSTING.md name as load-bearing.
