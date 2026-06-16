> Part of: [THESIS.md](THESIS.md) — the scan tool for checking a design against the durable directions of v2.

# v2 — Direction Conformance Checklist

> **Purpose.** A scan tool, not a task list. These are the *durable directions* of v2 — the goals and invariants a design must not **drop** (silently omit), **circumvent** (technically pass without doing the real thing), or **antagonize** (actively work against). Tasks live in the dispatch docs; this is the thing you hold a new design up against.
>
> **How to use.** For any new design, walk each item and mark **advances** / **neutral** / **contradicts-or-silently-drops**. A design need not advance every item — but it must contradict none, and must not silently drop one it touches. Any ✗ is a flag to resolve *before* GO. The "✗" lines are the battle-tested circumvention tells.
>
> **Authority.** Index, not a re-ledger. The `→` after each item names its **canonical home**; that home is authoritative for the fact, this file only points. `THESIS`, `INVARIANTS`, `ROADMAP`, `MODELING`, ctrl `goals-by-horizon`, and the doc map in [docs/thesis/doc-authority.md](docs/thesis/doc-authority.md) stay canonical. Pairs with `goals-by-horizon` (the *when/what*); this is the *must-not-violate*. **If this file ever disagrees with its `→` home, the home wins and this file is the bug.**

---

## Fast scan — the 7 questions

Hold any design to these first; if it's clean here, do the full pass. (Each digests a lettered item below — see that item for its authority.)

1. **Consumer & execution.** Is "done" a real consumer **green by execution** (`run`/`--claim-run`) — not compile-clean, grep, parse, or "spec written" — with a **discriminating control** (perturb → red; fixture/hand-fed input is *not* accepted as proof)? *(F1–F4)*
2. **One homomorphism.** Does it route **emit and ingest through the one shared relation** (`find_witness`), or quietly build an emit-only / parallel path? *(A4, B1–B6)*
3. **Faithfulness.** Is any IR↔surface claim proven by **normalized round-trip**, not a golden string? (Golden string = code generator; round-trip = homomorphism.) *(B3)*
4. **File-agnostic.** Does it keep files/positions as **surface metadata** and the IR file-agnostic — or leak files into the pipeline / node identity / the unit of compilation? *(D1)*
5. **Single roof.** Does it **converge** on the one queryable-graph authority (reflection, affected-set, lenses) — or fork a new parallel reader? *(C1–C2)*
6. **Construction over convention.** Does it make the wrong thing **structurally impossible** (and *show* the impossibility) — or add a rule / count-ceiling / ratchet that can be reached around? *(D2)*
7. **Ladders up + fold-DELETE.** Does it advance the **spine** (substrate → emit/ingest → platform → lenses) or a **keystone** (enforcement) — and does it **delete** the thing it replaces, not sit alongside it? *(G1, F7)*

---

## A. Identity — what v2 *is*
*(contradict these and it stops being v2)*

- **A1 — Closed/total typed graph language.** Assert once, never re-derive; cost-of-change → 1. → *INVARIANTS P2 · P4; CLAUDE.md "Cost of Change"*
  - ✗ re-derives a fact asserted elsewhere; adds a second source of a single truth.
- **A2 — Correctness is structural.** Dimensions are structural facts; user lenses use the **same mechanism** as built-in dimensions. → *THESIS · Correctness dimensions; docs/thesis/correctness-dimensions.md*
  - ✗ a correctness check bolted on as a special case instead of expressed as a dimension/lens.
- **A3 — The compiler is a homomorphism.** `Node → Outcome<Node>` via `fold_node`; **0 language branches**. → *THESIS · The derived homomorphism; docs/architecture.md*
  - ✗ per-language `if`/branch in the compiler; a transform that bypasses `fold_node`.
- **A4 — Coercion = emission = ingestion.** **One relation, run both ways.** (The load-bearing one — see §B.) → *THESIS · Coercion = emission; ROADMAP · Coercion in both directions*
  - ✗ treating emit as machinery distinct from ingest.
- **A5 — Bounded substrate: six connectives + five behaviors.** Types are `Atom | Conj | Disj | Arrow | Cardinality | Instantiation`; computation is `Value | Transform | Branch | Loop | Bind`; surface forms (`service`/`fn`/`type`/`operation`) are **sugar that lowers** to this kernel. A 7th connective or 6th behavior is a **C1 STOP** — all four structural-decompression dissolutions must fail first. → *THESIS · Substrate shape; docs/thesis/the-substrate-two-coordinated-shapes.md*
  - ✗ a quiet substrate extension; surface that adds semantic power instead of lowering; a Declaration shape too narrow to host `dsl/std/algebra.dag`.
- **A6 — Epistemic stacking: operations fall out of inhabitance.** Concrete types attach to the algebra DAG by inhabitance (Int inhabits OrderedRing → `add` falls out); operations are **derived, never declared per-type**; the epistemic chain **is** the emission algorithm; math and domain types share one substrate. → *THESIS · Epistemic stacking; docs/thesis/epistemic-stacking.md*
  - ✗ an emitter special case (= an ungrounded concept upstream); a concept with no path back to the primitives; operations re-declared per type.
- **A7 — Fail-closed end to end.** Every path succeeds fully or fails with a typed, located diagnostic; missing facts are errors, not fabricated plausible output. → *INVARIANTS P3 (C-1..C-10)*
  - ✗ a null / `<error:*>` / `"Unknown"` / `Dynamic` fallback (the C-1..C-10 family); a string-keyed open-set case list with a silent default branch; a fabrication sentinel (`__BUG_*`).
- **A8 — Decidable by bounded forward execution.** Every accepted form carries an explicit bound (recursion → `Loop` depth, iteration → bounded fold) or is rejected at the boundary; lowering is the receipt; cycles are relations over acyclic values, never cyclic values. → *INVARIANTS P4*
  - ✗ arbitrary recursion / uncapped iteration / a heuristic timeout; a verifier that re-derives its own parallel copy of the facts instead of reading the substrate.

## B. Homomorphism & bidirectionality
- **B1 — Two queries of one R, per layer.** Ingest forgets; emit chooses a canonical section (adjoint pair, not inverse). → *ROADMAP · Coercion in both directions; docs/thesis/the-derived-homomorphism.md*
  - ✗ an emit-only pipeline (projection sprawl, inline coercion arms) that ingest never touches.
- **B2 — Three surfaces kept distinct.** Surface (files/trivia/offsets) · source AST · semantic IR. Each round-trip names **which law** it proves. → *ROADMAP · Coercion in both directions (`.dag → IR → .dag`)*
  - ✗ merging `SourceAstEqual` and `SemanticIrEqual`; a round-trip that doesn't name its layer.
- **B3 — Faithfulness by round-trip.** Normalized equality, not golden-string, not bitwise. → *ROADMAP · Coercion in both directions*
  - ✗ generalizing emit while the round-trip stays deferred past the breadth tiers (proves a code generator).
- **B4 — Model a target once → derive both directions** (N×M). → *THESIS · The derived homomorphism; ROADMAP*
  - ✗ a target modeled emit-only with no labeled un-defer trigger for ingesting it (collapsed `R_target`).
- **B5 — One shared search engine.** `find_witness` is the single coercion search; coercion is not a separate phase, and ingest must not grow its own arms. → *src/v2/std/coercion.dag; ROADMAP · Coercion in both directions*
  - ✗ inline coercion arms accreting; a second engine on the ingest side.
- **B6 — Witness on success, located refusal on failure.** A realizable coercion carries a structure-preserving `HomomorphismWitness`; an unrealizable one fails closed with a located `CoercionMismatchKind` (`NoTargetCandidate` / `WouldLoseInformation` / opaque-atom-with-no-per-target-realization), and refinement is faithfulness-aware (`i32 → int` widening = witness; `int → i32` narrowing = `WouldLoseInformation`). → *ROADMAP · Coercion in both directions; src/v2/std/coercion.dag*
  - ✗ synthesizing silent glue for a missing inhabitant; assuming "translation always succeeds"; collapsing faithful widening and lossy narrowing.

## C. Programmatic access — the read/write roof
- **C1 — Single queryable-graph authority.** Readers converge; they don't fork. → *THESIS · Self-inspection; INVARIANTS P2*
  - ✗ a new "what changed" / "self-inspection" reader built parallel to the existing one.
- **C2 — Read axis = real reflection.** A `.dag` fn reflects over the **live** program by execution, proven with a **no-host-enumeration control**. → *THESIS · Self-inspection; INVARIANTS P2 "Reflection evidence is not structural proof"; ctrl programmatic-access-single-roof*
  - ✗ a reader riding host-enumeration / a hand-fed `Node` that *claims* self-inspection. (Reflection is construction, not consolidation — measured.) Runtime name-keyed lookup over the typed stack is metaprogramming, not reflection — out of bounds; the type-introspection need is a compile-time arm-enum (the type-dual of the discriminant).
- **C3 — Write axis = structured edit through the IR, consumer-gated.** Beachhead = "show the correct code." → *THESIS · Error handling: show the correct code*
  - ✗ a rewrite/codemod capability with no consumer (elegance trap); CLI edit with no faithful write-back (the `edit → emit → RepoPath` closure missing).
- **C4 — "Show the correct code" = emit on a corrected IR.** → *THESIS · Error handling: show the correct code*
  - ✗ diagnostics that stay `Unavailable`.

## D. Substrate modeling
- **D1 — Files out of the pipeline.** Semantic IR is file-agnostic (`content_hash` identity); files/positions are surface-located metadata; the ingest unit is not "the file." → *docs/architecture.md*
  - ✗ file/position load-bearing in identity or comparison; single-file compile as the only ingest path.
- **D2 — Construction-tier over convention-tier.** Make the wrong use unreachable; a construction-tier claim must **show** the impossibility. → *INVARIANTS P1 · P3*
  - ✗ a rule that gets reached around; a count-ceiling/ratchet standing in for a representation that forbids the mistake.
- **D3 — Fix at the authority.** Derived once, not patched per site. → *INVARIANTS P2 "Root-Cause Depth"*
  - ✗ per-call-site band-aids; a script-per-row.
- **D4 — Modeling hygiene.** Consumer per type; typed enums; no sentinels; no duplicate records. → *THESIS · Modeling discipline; MODELING M1–M10*
- **D5 — Layering & import direction.** `std/ ⊄ extdeps/`; `gunbc/` composes. Module layers form a strict DAG (`std/` ← `extdeps/` ← `compiler/` ← `workflow/`; imports only toward `std/`). The **source** language's operators/precedence live in `std/`; targets' in `extdeps/languages/`; the compiler consumes them, never hardcodes them. → *INVARIANTS P2 "Cross-layer import"*
  - ✗ gunbc policy facts in `extdeps/`; the compiler knowing a language's operators; a wrong-direction import (`extdeps/ → compiler/`, `std/ → compiler/`).
- **D6 — Grounded, not hollow; heuristics are missing facts.** Every spec primitive carries its spec's facts as a **fact-bundle** (`Conj` of named edges); reuse a `std/` carrier only on **proven coincidence**, never a bare alias. In a closed system a heuristic is always evidence of a dropped upstream fact, never a necessity. → *INVARIANTS P1 "Hollow alias" · "Heuristics Indicate Lost Structure"; MODELING M1*
  - ✗ `type X = Y` that drops the facts the spec states (width, signedness, representation…); a heuristic that computes an un-authored fact from nearby signals.
- **D7 — Right name, right home, right parent.** Name reflects structure (no nicknames); a canonical CS concept uses its canonical name in `std/` (not a pipeline-stage nickname); DFS `std/` for the existing parent before declaring a new sibling; coproducts only for genuine alternatives (coordinates → record). → *INVARIANTS P1 "Nickname" · "Internal vocabulary"; MODELING M9*
  - ✗ a nickname (`ModulePath` for what is a `QualifiedName`); an internal name for a canonical concept (`GrammarSchema`, `ModeledLexRules`); a sibling type beside an unnamed shared parent (the `Interval<D>` / `Dimension<Unit,Carrier>` shape); a sum type a single inhabitant fills every arm of.
- **D8 — No hand-rolled derived operation.** If behavior is fixed entirely by a modeled type's shape, consume the derived primitive (catamorphism / traversal / grammar model / structural fact); don't re-implement it. → *MODELING M1; docs/modeling-discipline.md Practice 10; INVARIANTS P1*
  - ✗ a function that re-derives what the model already determines — a second authority for one fact. The deficiency is in the model, not the code.

## E. Enforcement & opacity (the AIM / keystone)
- **E1 — gunbc enforces type/nominal distinctness** at call-args (an asserted-but-open Tier-1 claim — measured *not* enforced today). → *THESIS · Tier 1; ctrl enforcement call-arg-check design*
  - ✗ a design that assumes distinctness is already enforced.
- **E2 — Opacity adopted where it matters** (e.g. source-position brand-twins), riding the enforcement relation; the mechanism (`nominal_opaque`) exists. → *THESIS · Compositional layering; docs/audit/v2-encapsulation-touch-once-contract-2026-06-05.md*
  - ✗ treating opacity as a capability gap rather than an adoption choice.

## F. Measurement discipline — how we know "done"
*(the meta-filter; catches most circumvention)*

- **F1 — Consumer invariant.** "Done" = a real consumer **green by execution**. Typecheck / grep / parse / compile-clean = **fake** consumers. → *INVARIANTS E-10; "the specification-without-execution trap"*
  - ✗ acceptance = "compiles clean" / "claims exist" / "spec written."
- **F2 — Green for the right reason.** Discriminating input + perturbation (real → true, perturbed → false) + control; measure via `run`, never `compile`. → *INVARIANTS E-10 "Reviewer's three questions"*
  - ✗ a witness that passes on a fixture / hand-fed grounding (fixture bypass = tautology).
- **F3 — Instrument, don't theorize.** Probe before betting; confident root-cause reads get falsified by running. → *INVARIANTS · "Read this first" (spec-without-execution trap)*
  - ✗ asserting a capability is "already there" with no by-execution probe. (Reflection, enforcement, and the homomorphism spine were each asserted-done and falsified.)
- **F4 — merged ≠ proven; landed ≠ proven.** Specification-without-execution is the disease. The foundation is the most dangerous place to mark "done" — base primitives are maximally isolated from consumers, so weight foundational claims by whether a consumer has *run* them. → *INVARIANTS · "Read this first"; E-10*
  - ✗ "the design is done" while its witnesses are CompilesClaim-only or unrun; a base primitive marked DONE-by-typecheck.
- **F5 — No orphans.** Don't adopt unwired artifacts or archived-owner work; momentum ≠ investment. → *INVARIANTS E-10 (route the code, block the claim)*
- **F6 — No ratchets as walls.** Detection = named CI tests + judgment in an owned lane; any fence carries its "why" (signpost, not wall). A one-way lock raises cost-of-change and cements a green-for-wrong-reason; promote greens-for-the-right-reason to ordinary tests instead of locking rows. → *INVARIANTS P5; operator "avoid ratchets"*
  - ✗ an asymmetric corpus-eval ratchet; a lock someone will disable under refactor pressure.
- **F7 — fold-DELETE, not fold-alongside.** Replace the old thing; don't add beside it. → *INVARIANTS P5*
- **F8 — No bridge / scaffold as steady state.** Land representation changes atomically — new exists, old deletes, every consumer migrates, one change. Every scaffold carries a **named, checkable dissolution trigger** in the same change. (Progress = dissolution: a change is progress only if it reduces ad-hoc state / duplicate authority / implicit behavior.) → *INVARIANTS P5 "No Bridges" · "Scaffold receipts"*
  - ✗ a bridge/shim "until consumers move" that becomes the normal case; a scaffold with no trigger that goes accidentally load-bearing; a change that only adds.
- **F9 — Tests are structural data.** Coverage is `.dag` `TestClaim` data (or a generated runner), 0-floor; a hand-authored `.rs` test is a language smell — it flags a predicate / effect-model / mock surface the language can't yet express. → *THESIS · Tests are structural data; TESTING.md*
  - ✗ acceptance via a new hand-`.rs` test instead of a claim; a `compile`-only witness for a type fact (false green — type facts need `run`/`--claim-run`).
- **F10 — White-box tests = DELETE; relocation must not launder.** A test that mirrors what a declaration already states (decl-shape pin, source-grep of its own structure) is "2FA for code" — a second copy of one authority, zero real coverage → delete, don't migrate. When a test moves layers it stays a discriminating consumer, not a green tautology ("0 dropped" = 0 coverage lost, not 0-because-converted-to-a-grep). → *TESTING.md; operator "white-box tests = 2FA → delete"*
  - ✗ a decl-mirror / source-grep test kept or "migrated"; a relocated test that no longer goes red when the behavior is wrong.

## G. Spine & sequencing
- **G1 — Everything ladders up the spine.** substrate runs → emit/ingest (one R) → platform → lenses. Compiler work earns its place by advancing the spine or a keystone. → *ROADMAP · Milestone shape; ctrl goals-by-horizon*
  - ✗ local craft with no spine/keystone consumer (codegen-for-its-own-sake).
- **G2 — Enforcement is the keystone.** Off-spine but feeds coercion + opacity + infer-conformance + transport-drift. → *THESIS · Tier 1; ctrl goals-by-horizon*
- **G3 — Self-conformance is an overlay.** Internals produced *through* coercion/layering/projection; free except the front end; each target **consumer-gated**. → *THESIS · Self-hosting facet 4*
  - ✗ "conform the stages" with no named consumer (strictly-better-if-it-finishes trap).
- **G4 — Self-host fixed point.** `compiler.dag` emits bit-identical stage0; `hand_maintained → 0`; tests are data. The `.dag` graph is the source of truth; emitted Rust is one realization, never a parallel authority. → *THESIS · Self-hosting four facets; docs/design-pure-bootstrap-zero.md*
  - ✗ cementing stage0 to satisfy a ratchet; hand-editing emitted Rust.

## H. Product & payoff
- **H1 — Lenses = user-extensible dimensions** (same mechanism as built-ins); pick one via cheap experiments, don't build all three. → *THESIS · User-defined dimensions*
- **H2 — Release impossible-bug demos:** complexity, idempotency, transport-drift. Each named class is a thesis commitment — adding one commits the thesis; removing one requires a *named dissolution* (the proof became trivial), not convenience. → *THESIS · Enumerable impossible-bug classes*
  - ✗ silently dropping a committed class, or reintroducing one as catchable-only-at-runtime.
- **H3 — Omni-emission.** Shape A (language targets, one spec in `extdeps/languages/`, O(1)/target, zero compiler changes) + Shape B (YAML/TF/K8s/SQL via `.dag` programs walking typed values, O(1)/artifact class); the Shape-B beachhead is CI config generated from `.dag` (the prior descriptive-only `ci.dag` mirror was deleted; `.github/workflows/ci.yml` stays the hand-authored authority until a generated CI model with a real runtime consumer lands — see THESIS "Meta-process modeling"). The two shapes must not be blurred. → *THESIS · Omni-emission; docs/thesis/what-else-falls-out.md*
  - ✗ a new compiler path per target; a compiler render path for a Shape-B artifact, or a language pushed into user-space.
- **H4 — Distribution.** daglang as the disciplined target an LLM agent emits into (MCP-callable); humans needn't learn it. → *ctrl goals-by-horizon*
- **H5 — Positioning.** Rust is a target, not a competitor; decidable, not heuristic; derive behavior, not just shape. → *THESIS · What .dag catches that normal compilers don't; ctrl goals-by-horizon*
- **H6 — Audience duality + adoption by economics.** The base language stays approachable (types/fns/match/effects/workflows); the advanced surface (lenses, proofs, user reflection) is **opt-in depth**; guarantees are free **by construction** (by using the language at all), not behind a flag. Adoption is gated by economics (low entry × high free value), not enforcement. → *THESIS · Audience duality · Adoption model*
  - ✗ forcing the lens/proof surface or an annotation tax onto every user; a "compliance check / analyzer flag" as the recruiting mechanism.

---

> **Standing note.** v1 — yours to own and edit. The directions are durable; when one genuinely changes, change it *here* (with its "why"), so designs can't silently drift it. When you scan a design and find a ✗, the question is never "is the design clever" — it's "does it drop / circumvent / antagonize a direction," and if so, fix the design or change the direction on purpose.
