> Part of: [THESIS.md](THESIS.md) — the scan tool for checking a design against the durable directions of v4.

# v4 — Direction Conformance Checklist

> **Purpose.** A scan tool, not a task list. These are the *durable directions* of v4 — the goals and invariants a design must not **drop** (silently omit), **circumvent** (technically pass without doing the real thing), or **antagonize** (actively work against). Tasks live in the dispatch docs; this is the thing you hold a new design up against.
>
> **How to use.** For any new design, walk each item and mark **advances** / **neutral** / **contradicts-or-silently-drops**. A design need not advance every item — but it must contradict none, and must not silently drop one it touches. Any ✗ is a flag to resolve *before* GO. The "✗" lines are the battle-tested circumvention tells.
>
> **Authority.** Index, not a re-ledger. `goals-by-horizon` (ctrl), `THESIS`, `INVARIANTS`, `ROADMAP`, `MODELING`, and the ctrl planning docs stay canonical for every fact; this points, it doesn't restate. Pairs with `goals-by-horizon` (the *when/what*); this is the *must-not-violate*. If this file ever disagrees with an authority, the authority wins and this file is the bug.

---

## Fast scan — the 7 questions

Hold any design to these first; if it's clean here, do the full pass.

1. **Consumer & execution.** Is "done" a real consumer **green by execution** (`run`/`--claim-run`) — not compile-clean, grep, parse, or "spec written" — with a **discriminating control** (perturb → red; fixture/hand-fed input is *not* accepted as proof)?
2. **One homomorphism.** Does it route **emit and ingest through the one shared relation** (`find_witness`), or quietly build an emit-only / parallel path?
3. **Faithfulness.** Is any IR↔surface claim proven by **normalized round-trip**, not a golden string? (Golden string = code generator; round-trip = homomorphism.)
4. **File-agnostic.** Does it keep files/positions as **surface metadata** and the IR file-agnostic — or leak files into the pipeline / node identity / the unit of compilation?
5. **Single roof.** Does it **converge** on the one queryable-graph authority (reflection, affected-set, lenses) — or fork a new parallel reader?
6. **Construction over convention.** Does it make the wrong thing **structurally impossible** (and *show* the impossibility) — or add a rule / count-ceiling / ratchet that can be reached around?
7. **Ladders up + fold-DELETE.** Does it advance the **spine** (substrate → emit/ingest → platform → lenses) or a **keystone** (enforcement) — and does it **delete** the thing it replaces, not sit alongside it?

---

## A. Identity — what v4 *is*
*(contradict these and it stops being v4)*

- **A1 — Closed/total typed graph language.** Assert once, never re-derive; cost-of-change → 1.
  - ✗ re-derives a fact asserted elsewhere; adds a second source of a single truth.
- **A2 — Correctness is structural.** Dimensions are structural facts; user lenses use the **same mechanism** as built-in dimensions.
  - ✗ a correctness check bolted on as a special case instead of expressed as a dimension/lens.
- **A3 — The compiler is a homomorphism.** `Node → Outcome<Node>` via `fold_node`; **0 language branches**.
  - ✗ per-language `if`/branch in the compiler; a transform that bypasses `fold_node`.
- **A4 — Coercion = emission = ingestion.** **One relation, run both ways.** (The load-bearing one — see §B.)
  - ✗ treating emit as machinery distinct from ingest.
- **A5 — Bounded substrate: six connectives + five behaviors.** Types are `Atom | Conj | Disj | Arrow | Cardinality | Instantiation`; computation is `Value | Transform | Branch | Loop | Bind`; surface forms (`service`/`fn`/`type`/`operation`) are **sugar that lowers** to this kernel. A 7th connective or 6th behavior is a **C1 STOP** — all four structural-decompression dissolutions must fail first.
  - ✗ a quiet substrate extension; surface that adds semantic power instead of lowering; a Declaration shape too narrow to host `dsl/std/algebra.dag`.
- **A6 — Epistemic stacking: operations fall out of inhabitance.** Concrete types attach to the algebra DAG by inhabitance (Int inhabits OrderedRing → `add` falls out); operations are **derived, never declared per-type**; the epistemic chain **is** the emission algorithm; math and domain types share one substrate.
  - ✗ an emitter special case (= an ungrounded concept upstream); a concept with no path back to the primitives; operations re-declared per type.
- **A7 — Fail-closed end to end.** Every path succeeds fully or fails with a typed, located diagnostic; missing facts are errors, not fabricated plausible output.
  - ✗ a null / `<error:*>` / `"Unknown"` / `Dynamic` fallback (the C-1..C-10 family); a string-keyed open-set case list with a silent default branch; a fabrication sentinel (`__BUG_*`).
- **A8 — Decidable by bounded forward execution.** Every accepted form carries an explicit bound (recursion → `Loop` depth, iteration → bounded fold) or is rejected at the boundary; lowering is the receipt; cycles are relations over acyclic values, never cyclic values.
  - ✗ arbitrary recursion / uncapped iteration / a heuristic timeout; a verifier that re-derives its own parallel copy of the facts instead of reading the substrate.

## B. Homomorphism & bidirectionality
- **B1 — Two queries of one R, per layer.** Ingest forgets; emit chooses a canonical section (adjoint pair, not inverse).
  - ✗ an emit-only pipeline (projection sprawl, inline coercion arms) that ingest never touches.
- **B2 — Three surfaces kept distinct.** Surface (files/trivia/offsets) · source AST · semantic IR. Each round-trip names **which law** it proves.
  - ✗ merging `SourceAstEqual` and `SemanticIrEqual`; a round-trip that doesn't name its layer.
- **B3 — Faithfulness by round-trip.** Normalized equality, not golden-string, not bitwise.
  - ✗ generalizing emit while the round-trip stays deferred past the breadth tiers (proves a code generator).
- **B4 — Model a target once → derive both directions** (N×M).
  - ✗ a target modeled emit-only with no labeled un-defer trigger for ingesting it (collapsed `R_target`).
- **B5 — One shared search engine.** `find_witness` is the single coercion search; coercion is not a separate phase, and ingest must not grow its own arms.
  - ✗ inline coercion arms accreting; a second engine on the ingest side.
- **B6 — Witness on success, located refusal on failure.** A realizable coercion carries a structure-preserving `HomomorphismWitness`; an unrealizable one fails closed with a located `CoercionMismatchKind` (`NoTargetCandidate` / `WouldLoseInformation` / opaque-atom-with-no-per-target-realization), and refinement is faithfulness-aware (`i32 → int` widening = witness; `int → i32` narrowing = `WouldLoseInformation`).
  - ✗ synthesizing silent glue for a missing inhabitant; assuming "translation always succeeds"; collapsing faithful widening and lossy narrowing.

## C. Programmatic access — the read/write roof
- **C1 — Single queryable-graph authority.** Readers converge; they don't fork.
  - ✗ a new "what changed" / "self-inspection" reader built parallel to the existing one.
- **C2 — Read axis = real reflection.** A `.dag` fn reflects over the **live** program by execution, proven with a **no-host-enumeration control**.
  - ✗ a reader riding host-enumeration / a hand-fed `Node` that *claims* self-inspection. (Reflection is construction, not consolidation — measured.) Runtime name-keyed lookup over the typed stack is metaprogramming, not reflection — out of bounds; the type-introspection need is a compile-time arm-enum (the type-dual of the discriminant).
- **C3 — Write axis = structured edit through the IR, consumer-gated.** Beachhead = "show the correct code."
  - ✗ a rewrite/codemod capability with no consumer (elegance trap); CLI edit with no faithful write-back (the `edit → emit → RepoPath` closure missing).
- **C4 — "Show the correct code" = emit on a corrected IR.**
  - ✗ diagnostics that stay `Unavailable`.

## D. Substrate modeling
- **D1 — Files out of the pipeline.** Semantic IR is file-agnostic (`content_hash` identity); files/positions are surface-located metadata; the ingest unit is not "the file."
  - ✗ file/position load-bearing in identity or comparison; single-file compile as the only ingest path.
- **D2 — Construction-tier over convention-tier.** Make the wrong use unreachable; a construction-tier claim must **show** the impossibility.
  - ✗ a rule that gets reached around; a count-ceiling/ratchet standing in for a representation that forbids the mistake.
- **D3 — Fix at the authority.** Derived once, not patched per site.
  - ✗ per-call-site band-aids; a script-per-row.
- **D4 — Modeling hygiene.** Consumer per type; typed enums; no sentinels; no duplicate records. (INVARIANTS / MODELING M1–M10.)
- **D5 — Layering & import direction.** `std/ ⊄ extdeps/`; `gunbc/` composes. Module layers form a strict DAG (`std/` ← `extdeps/` ← `compiler/` ← `workflow/`; imports only toward `std/`). The **source** language's operators/precedence live in `std/`; targets' in `extdeps/languages/`; the compiler consumes them, never hardcodes them.
  - ✗ gunbc policy facts in `extdeps/`; the compiler knowing a language's operators; a wrong-direction import (`extdeps/ → compiler/`, `std/ → compiler/`).
- **D6 — Grounded, not hollow; heuristics are missing facts.** Every spec primitive carries its spec's facts as a **fact-bundle** (`Conj` of named edges); reuse a `std/` carrier only on **proven coincidence**, never a bare alias. In a closed system a heuristic is always evidence of a dropped upstream fact, never a necessity.
  - ✗ `type X = Y` that drops the facts the spec states (width, signedness, representation…); a heuristic that computes an un-authored fact from nearby signals.
- **D7 — Right name, right home, right parent.** Name reflects structure (no nicknames); a canonical CS concept uses its canonical name in `std/` (not a pipeline-stage nickname); DFS `std/` for the existing parent before declaring a new sibling; coproducts only for genuine alternatives (coordinates → record).
  - ✗ a nickname (`ModulePath` for what is a `QualifiedName`); an internal name for a canonical concept (`GrammarSchema`, `ModeledLexRules`); a sibling type beside an unnamed shared parent (the `Interval<D>` / `Dimension<Unit,Carrier>` shape); a sum type a single inhabitant fills every arm of.
- **D8 — No hand-rolled derived operation.** If behavior is fixed entirely by a modeled type's shape, consume the derived primitive (catamorphism / traversal / grammar model / structural fact); don't re-implement it.
  - ✗ a function that re-derives what the model already determines — a second authority for one fact. The deficiency is in the model, not the code.

## E. Enforcement & opacity (the AIM / keystone)
- **E1 — gunbc enforces type/nominal distinctness** at call-args (an asserted-but-open Tier-1 claim — measured *not* enforced today).
  - ✗ a design that assumes distinctness is already enforced.
- **E2 — Opacity adopted where it matters** (e.g. source-position brand-twins), riding the enforcement relation; the mechanism (`nominal_opaque`) exists.
  - ✗ treating opacity as a capability gap rather than an adoption choice.

## F. Measurement discipline — how we know "done"
*(the meta-filter; catches most circumvention)*

- **F1 — Consumer invariant.** "Done" = a real consumer **green by execution**. Typecheck / grep / parse / compile-clean = **fake** consumers.
  - ✗ acceptance = "compiles clean" / "claims exist" / "spec written."
- **F2 — Green for the right reason.** Discriminating input + perturbation (real → true, perturbed → false) + control; measure via `run`, never `compile`.
  - ✗ a witness that passes on a fixture / hand-fed grounding (fixture bypass = tautology).
- **F3 — Instrument, don't theorize.** Probe before betting; confident root-cause reads get falsified by running.
  - ✗ asserting a capability is "already there" with no by-execution probe. (Reflection, enforcement, and the homomorphism spine were each asserted-done and falsified.)
- **F4 — merged ≠ proven; landed ≠ proven.** Specification-without-execution is the disease. The foundation is the most dangerous place to mark "done" — base primitives are maximally isolated from consumers, so weight foundational claims by whether a consumer has *run* them.
  - ✗ "the design is done" while its witnesses are CompilesClaim-only or unrun; a base primitive marked DONE-by-typecheck.
- **F5 — No orphans.** Don't adopt unwired artifacts or archived-owner work; momentum ≠ investment.
- **F6 — No ratchets as walls.** Detection = named CI tests + judgment in an owned lane; any fence carries its "why" (signpost, not wall). A one-way lock raises cost-of-change and cements a green-for-wrong-reason; promote greens-for-the-right-reason to ordinary tests instead of locking rows.
  - ✗ an asymmetric corpus-eval ratchet; a lock someone will disable under refactor pressure.
- **F7 — fold-DELETE, not fold-alongside.** Replace the old thing; don't add beside it.
- **F8 — No bridge / scaffold as steady state.** Land representation changes atomically — new exists, old deletes, every consumer migrates, one change. Every scaffold carries a **named, checkable dissolution trigger** in the same change. (Progress = dissolution: a change is progress only if it reduces ad-hoc state / duplicate authority / implicit behavior.)
  - ✗ a bridge/shim "until consumers move" that becomes the normal case; a scaffold with no trigger that goes accidentally load-bearing; a change that only adds.
- **F9 — Tests are structural data.** Coverage is `.dag` `TestClaim` data (or a generated runner), 0-floor; a hand-authored `.rs` test is a language smell — it flags a predicate / effect-model / mock surface the language can't yet express.
  - ✗ acceptance via a new hand-`.rs` test instead of a claim; a `compile`-only witness for a type fact (false green — type facts need `run`/`--claim-run`).
- **F10 — White-box tests = DELETE; relocation must not launder.** A test that mirrors what a declaration already states (decl-shape pin, source-grep of its own structure) is "2FA for code" — a second copy of one authority, zero real coverage → delete, don't migrate. When a test moves layers it stays a discriminating consumer, not a green tautology ("0 dropped" = 0 coverage lost, not 0-because-converted-to-a-grep).
  - ✗ a decl-mirror / source-grep test kept or "migrated"; a relocated test that no longer goes red when the behavior is wrong.

## G. Spine & sequencing
- **G1 — Everything ladders up the spine.** substrate runs → emit/ingest (one R) → platform → lenses. Compiler work earns its place by advancing the spine or a keystone.
  - ✗ local craft with no spine/keystone consumer (codegen-for-its-own-sake).
- **G2 — Enforcement is the keystone.** Off-spine but feeds coercion + opacity + infer-conformance + transport-drift.
- **G3 — Self-conformance is an overlay.** Internals produced *through* coercion/layering/projection; free except the front end; each target **consumer-gated**.
  - ✗ "conform the stages" with no named consumer (strictly-better-if-it-finishes trap).
- **G4 — Self-host fixed point.** `compiler.dag` emits bit-identical stage0; `hand_maintained → 0`; tests are data. The `.dag` graph is the source of truth; emitted Rust is one realization, never a parallel authority.
  - ✗ cementing stage0 to satisfy a ratchet; hand-editing emitted Rust.

## H. Product & payoff
- **H1 — Lenses = user-extensible dimensions** (same mechanism as built-ins); pick one via cheap experiments, don't build all three.
- **H2 — Release impossible-bug demos:** complexity, idempotency, transport-drift. Each named class is a thesis commitment — adding one commits the thesis; removing one requires a *named dissolution* (the proof became trivial), not convenience.
  - ✗ silently dropping a committed class, or reintroducing one as catchable-only-at-runtime.
- **H3 — Omni-emission.** Shape A (language targets, one spec in `extdeps/languages/`, O(1)/target, zero compiler changes) + Shape B (YAML/TF/K8s/SQL via `.dag` programs walking typed values, O(1)/artifact class); `ci.dag` emitting its own config = the Shape-B beachhead. The two shapes must not be blurred.
  - ✗ a new compiler path per target; a compiler render path for a Shape-B artifact, or a language pushed into user-space.
- **H4 — Distribution.** daglang as the disciplined target an LLM agent emits into (MCP-callable); humans needn't learn it.
- **H5 — Positioning.** Rust is a target, not a competitor; decidable, not heuristic; derive behavior, not just shape.
- **H6 — Audience duality + adoption by economics.** The base language stays approachable (types/fns/match/effects/workflows); the advanced surface (lenses, proofs, user reflection) is **opt-in depth**; guarantees are free **by construction** (by using the language at all), not behind a flag. Adoption is gated by economics (low entry × high free value), not enforcement.
  - ✗ forcing the lens/proof surface or an annotation tax onto every user; a "compliance check / analyzer flag" as the recruiting mechanism.

---

> **Standing note.** v1 — yours to own and edit. The directions are durable; when one genuinely changes, change it *here* (with its "why"), so designs can't silently drift it. When you scan a design and find a ✗, the question is never "is the design clever" — it's "does it drop / circumvent / antagonize a direction," and if so, fix the design or change the direction on purpose.
