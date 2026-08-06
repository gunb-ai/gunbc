# v2 complexity capability parity — restoring the engine, not porting the seed

> **Status: DRAFT for operator review (2026-08-04).** Design-note-first: **no code lands from this note.** It answers one question — *is v2 complexity analysis anemic compared to v1?* — with a verdict grounded in the tree's own status carriers, and declares the migration program that closes the gap.
> **Verdict: yes, and the tree already says so in its own enforcement model.** The anemia is real, it is localized (see §2 — it is *not* the whole cost stack), and one carrier currently declares it **permanent**, which is the finding worth acting on first (§4).

## 1. Scope seam with the sibling note (DESIGN §3 — do not fork)

This note and [`discrete-cost-derivation.md`](discrete-cost-derivation.md) are two halves of one program and must not restate each other. The split is by *question*:

| Question | Owner |
| --- | --- |
| What can the engine *express and derive*? (cost algebra, work/span, output size, peak space, interprocedural summaries, recursion) | **this note** |
| What does a derivation *mean and get used for*? (subject binding, valuation at a declared input, honesty states, cost justification, witness admission) | [`discrete-cost-derivation.md`](discrete-cost-derivation.md) |

**Consequence, stated so neither note quietly grows into the other:** the expression-algebra promotion and the work/span/space summary — which the sibling note originally listed under its C0 and C2 — are **this note's C1 and C2** and have been removed from that note's scope. The valuation environment and the admission consumer remain **its** C1 and C5 and are absent here. **The authority ruling has exactly one home: §5 of this note**, signed as C0 deliverable (a) in §7. The sibling note's C0 is now purely a dependency edge on that signature — it states no ruling of its own, so there is no second place for the two to drift apart.

## 2. The verdict, and its exact boundary

**The cost half is live; the complexity half is not.** This distinction matters, because over-scoping the fix would be a second error. From `v2.lens.enforcement.contract`:

| Contract | Mode | Consumer witness | Boundary class |
| --- | --- | --- | --- |
| `lens_contract_cost` | **`Blocking`** | **`BoundConsumerWitness`** (`complexity_gate/budget_roster_completeness_test.dag`) | `WallNow` |
| `lens_contract_complexity` | `AuditOnly` | **`NoConsumerWitness`** | **`RatchetForever`** |
| `lens_contract_complexity_accumulator_copy` | `AuditOnly` | `NoConsumerWitness` | `WallAfterGrounding` |
| `lens_contract_complexity_linearity_audit` | `AuditOnly` | `NoConsumerWitness` | `WallAfterGrounding` → `SubstrateMandatoryTag` |
| `lens_contract_complexity_lowering` | `AuditOnly` | `NoConsumerWitness` | `WallAfterGrounding` → `SingleAuthority` |

**Table corrected 2026-08-05 (operator review).** Two rows were wrong in the merged revision and are fixed above, read off the live contract rather than recalled: `lens_contract_complexity_linearity_audit` showed no boundary when it carries a real `WallAfterGrounding` dissolving to `SubstrateMandatoryTag`, and `lens_contract_complexity_lowering` was omitted entirely though it carries `WallAfterGrounding` dissolving to `SingleAuthority` — a *distinct* dissolution mechanism, which matters because it means the specialized layer is not one undifferentiated block. Understating these boundaries understated how much of the complexity layer is already classified as decidable-but-unbuilt, which is the same direction of error as the `RatchetForever` mis-declaration below.

So the generic structural engine `v2.lens.cost` is a real, enrolled, blocking wall — it is not a stub, and this program does not touch its kernel. Everything *above* it is audit-only with no consumer: the projection lens and all three specialized complexity lenses. That is the DESIGN §6 **coverage-by-illusion tier**, stated by the repo about itself.

A second, sharper tell sits inside the projection lens: `v2.lens.complexity` `complexity_variables_admits` is

```
fn complexity_variables_admits(xs: List<SizeVariable>) -> Bool {
  true
}
```

— a `Validation` whose admission predicate is the constant `true`. It refines nothing; the `Refined<List<SizeVariable>>` carrier it feeds is a type name standing where a constraint was supposed to be. DESIGN §4b names this class exactly: *'richer type names are not safety — a brand, wrapper, or `Validated<T>` is cosmetic until construction and acceptance enforce the distinction.'*

## 3. Capability comparison (verified, not recalled)

`src/v1/complexity.dag` is 4,857 lines and **171 top-level `fn` declarations** — the method matters and was previously unstated: counting `fn` + `type` + `data` + `let` gives **201** (26 `type`, 4 `data`), which is why a sibling artifact quoted a different figure. Neither number was wrong; the missing method made them read as a contradiction. C0(b2) replaces both with a population derived from `DeclFact`. What v1 carries that v2 does not:

| Capability | v1 | v2 |
| --- | --- | --- |
| Asymptotic class + dominance lattice | yes | **yes** (and better factored) |
| Symbolic cost expression | `CostConst/Add/Mul/Max/Sum/Log/Extern/Unknown` | partial — no `Max`, no binder-carrying `Sum`, no extern |
| Size expression | `SizeConst/SizeVar/SizeLen/SizeAdd/SizeMax` | **none** — `SizeVariable { source: Node }` only |
| Exact summation over a bounded collection | `CostSum { binder, upper, body }` | **no** |
| Work vs span | both, on `ComplexitySummary` | **neither** |
| Output-size derivation | `output_size: Map<String, CostExpr>` | **no** |
| Peak space | `peak_space: CostExpr?` | **no** |
| Certainty | `Proven | Conservative` | **no** |
| Interprocedural function summaries | `CostInternTable` memo | **no** |
| Call graph / SCC | by import from `std.graph` | **not consumed** — corrected by census, see C0(b1) finding 1 |
| Recursion / descent bounds | vocabulary shared (`std.termination` / `std.computation` / `std.induction`); **walkers** v1-AST-local | loop-bound edge only |
| Parser-progress reasoning | yes (mutual recursion) | **no** |
| Concrete-input evaluation | partial machinery | **no** (sibling note's C1) |
| Live repo-wide consumer | seed-internal | cost yes / complexity **no** |

**The gap is real and large. It is migration debt, not a deliberate retirement** — nothing in the tree declares these capabilities unwanted.

## 4. The finding to act on first: `RatchetForever` is very likely mis-declared

`lens_contract_complexity` carries `ConstructionJustification { class: RatchetForever }`. That is a **permanence claim**: this class is undecidable, so it can never become a wall and will always be a validating ratchet. DESIGN §5 is explicit that this is the trap to check rather than the safe default:

> *'The word "never" is the trap: it lets a ratchet masquerade as a wall, so check decidability before claiming one.'*

Applied here the claim does not survive contact with its own contents. **Optimality is undecidable by Rice — but most of what is missing is not optimality.** Work and span composition over a closed kernel is decidable. Summation over a *bounded* collection is decidable — that is what the modeled loop bound exists to provide. Output size and peak space over the same bounded fragment are decidable. Interprocedural summaries over an acyclic declaration graph are decidable, and the cyclic case is exactly what SCC plus `DescentEvidence` already decides fail-closed. So `RatchetForever` on the whole contract is a single permanence verdict standing over a **mixed** population, and its effect is to make the anemia unrankable: a class declared permanently undecidable never appears on anyone's list of things to climb.

**Re-classifying this contract is therefore C0 deliverable (c) (§7)**, splitting the genuinely-undecidable residue (optimality; arbitrary predicate refinement) from the decidable capability that should carry `WallAfterGrounding` with a named trigger. Until that split lands, every phase below is arguing against a declared 'never'.

## 5. The authority ruling (content; signed as C0's first deliverable — §7)

Do **not** port `src/v1/complexity.dag` wholesale. It is powerful but coupled to the v1 AST, to parser implementation details, and to string-keyed identity throughout — porting it would cement the seed into the new architecture, which is the one move the project spirit forbids. The split:

```
v2.lens.cost              cost + size expressions, work/span composition,
                          output-size and space expressions, effect demand

v2.lens.complexity        asymptotic projection of cost; dominance and
                          complexity-class comparison. Nothing else.

std.realization_schedule  predicted / derived / measured resource accounts
                          and scheduling consumption

v1.compiler.complexity    legacy realization, capability inventory,
                          parity oracle, deletion target
```

**`SymbolicCost` stays** as the normalized asymptotic projection of the richer `CostExpr` — one authority with two levels of detail, never two competing cost types. The existing `Blocking` cost wall and its bound consumer witness keep working throughout; no phase below may regress them.

## 6. What must NOT be carried across

v1's identities are string-keyed in five places, verified: `SizeVar { name: String }`, `SizeLen { collection: String }`, `CostExtern { name: String }`, `CostUnknown { reason: String }`, and `CostInternTable { summaries: Map<String, ComplexitySummary> }`. Every one is the anemic-modeling class DESIGN §2 names — a `String` leaf standing where a grounded identity exists. The migration re-grounds each:

- function and collection identity → `DeclarationRef`, which exists today and resolves through `v2.std.decl_ref_resolution`. **`ValueIdentity` does NOT exist in the tree** — corrected 2026-08-05, after this line misled the C1 lane. It appears only inside the sibling note's *proposed* `CostSubject` and `CostJustification` code blocks; a corpus grep resolves it to no declaration of any kind. The original wording said the G1 cited-symbol work 'already makes these resolvable', which is true of `DeclarationRef` and false of `ValueIdentity` — the DESIGN §3 cited-symbol class, a name-level claim a grep decides, sitting false in an authority doc while the mechanism it describes stayed correct. C1 must therefore either ground size identity on the existing `SizeVariable` carrier or land a value-identity authority first; it may not cite a symbol that does not exist.
- `CostExtern { name }` → `CostEffect { operation: DeclarationRef }`, joining the sibling note's `ExecuteEffect` atom rather than a second extern channel
- `CostUnknown { reason: String }` → a typed, located, **countable** cause — a free-text reason cannot be tallied, so its frequency never ranks the deficit for fixing (DESIGN §5)
- `Certainty = Proven | Conservative` → reconciled with the existing `DescentEvidence` and `Measured` bases rather than minted as a third confidence vocabulary

**Registry note:** `llvm_instruction_cost` is not merely present in `v2.lens.cost` — it is *registered* as owned by it (`v2.lens.registry` `lens_owned_fn_llvm_instruction_cost`, `owner_module_path: "v2.lens.cost"`). Re-homing that uncited machine table to a cited per-target model is therefore a registry change too, not a file move.

## 7. Phases

1. **C0 — sign the ruling, census the capabilities, re-classify the boundary.** Three deliverables. **(a) SIGNED 2026-08-05** — the §5 authority ruling is approved and is now carried as a typed vocabulary, THREE typed axes in `gunbc.v1_complexity_capability_census` — `CostProgramRole` (what a module IS, including the legacy oracle), `CostMigrationDestination` (where owed work may LAND, the legacy oracle absent by construction), and `CapabilityFactProducer` (who produces the facts). An earlier revision fused all three into one enum, which made *migrate into the deletion target* writable and forced parser-progress to name the cost lens as its own producer. The signature carries one nuance: `v2.lens.cost` is the authority's CURRENT IDENTITY, not necessarily its permanent namespace — any later move to a shared or non-lens home must be an ATOMIC relocation with every consumer repointed in the same motion, never a second live authority. **(b1) DELIVERED ON MERGE of the carrier PR** — a reviewed v1 complexity *inventory*, below. Stated as on-merge rather than already-delivered because the artifact is the merged carrier, not the note describing it. **(b2) DELIVERED ON MERGE of #7840** — declaration-to-capability totality: `gunbc.v1_complexity_decl_classification` derives 201 declarations from `DeclFact`, classifies each exactly once via `v1_complexity_decl_classification_roster`, and witnesses prove population↔roster bijection plus every `SemanticCapabilityItem` has an `AuthorityOf` declaration. **(c) OPEN** — the `RatchetForever` split, ruled below and now the next implementation slice. **Accept:** the ruling is signed, the declaration population is closed, and the permanence claim covers only genuinely undecidable rows.
2. **C0(b1) — the reviewed roster, and exactly what it does not prove.** Carrier: `gunbc.v1_complexity_capability_census` `v1_complexity_inventory_roster`. It is an INVENTORY, not a capability roster: legacy residues (the eviction hazard, the v1 walker realization) are `V1ComplexityInventoryItem` rows carrying the capability they belong to, not peer variants beside real capabilities, so counting rows and counting capabilities are different questions with different answers. PROVEN: every row carries a typed `DeclarationRef` that resolves to exactly one declaration through `v2.std.decl_ref_resolution`, with planted refusals for absent declaration, absent module, and — the discriminating one — a name that exists only inside a note string, which a substring check would have accepted. Capability identity is a closed coproduct, so an identity OUTSIDE the vocabulary is unrepresentable — but the roster is a `List`, so distinctness is **validated** by an executing fold with a planted-duplicate control, not constructed. b2's derived population supplies the wall this list cannot. Every disposition arm lands in exactly one work-state, with a controlled fixture for EVERY arm including the two no row inhabits. **NOT PROVEN: declaration-grain completeness.** A 27th capability omitted from the roster would not be detected, and no assertion in this lane should be read as claiming otherwise — which is why this is b1 and not b. **Execution split, stated rather than buried:** the exact resolution census and the executing eviction control live in `test.claim.long.v1_complexity_capability_census_resolution_test`, which per-PR CI does NOT run (it needs a decl_facts scan over a pool including `src/v1`); its local recipe is named in `gunbc.ci_layer_roots`. The fast structural half does run per-PR.
3. **C0(b2) — declaration-to-capability totality (the actual C0(b) closing slice). DELIVERED ON MERGE of #7840.** Carrier: `gunbc.v1_complexity_decl_classification` + `v1_complexity_decl_classification_roster` (201 rows). Derive the top-level declaration population of `v1.compiler.complexity` from `DeclFact` and classify each declaration as authority-of / helper-of / legacy-hazard-of a capability, or deliberately out of scope. PROVEN: every discovered declaration classified exactly once (long-lane `test.claim.long.v1_complexity_decl_classification_totality_test`), every classification points at a real capability row, every `SemanticCapabilityItem` has at least one `AuthorityOf` declaration (fast-lane `test.claim.v1_complexity_decl_classification_witness_test`), zero stale rows. **NOT PROVEN:** discovery of a 26th semantic capability omitted from b1 — classification is authored against the closed `ComplexityCapabilityId` vocabulary (same b1 limitation). **Method identity:** 201 = `fn` + `type` + `data` + `let` over `src/v1/complexity.dag` at this revision.
4. **C0(b1) findings — five things the tree said that this note did not.** Each is executed. **(1) C4 is materially smaller than §3's table implied.** `std.graph` carries the SCC machinery, `std.termination` `DescentEvidence`/`TerminationProof`, `std.computation` `LoweringTarget`/`lower_call_pattern`, `std.induction` `CostBound`/`derive_bound` — and `v1.compiler.complexity` reaches all of them BY IMPORT, while v2's lens modules import none. **Corrected on operator review:** the work is INTEGRATION, not nothing. The first revision projected shared-substrate rows as *discharged*, which understated C4 — v2 must still consume the substrate and produce the facts it consumes. The census now carries `IntegrationOwed` as its own work-state, distinct from both migration and completion. **(2) `std.induction`'s cost-bound vocabulary** was missing from the do-not-re-mint list; added. **(3) A migration hazard, now proven by execution rather than by matching source text:** cache a summary with work 42, evict it, look it up — the seed returns a Present summary with work 0, span 0, certainty `Proven`, instead of `Absent`. A consumer reads *costs nothing, proven* for a function whose cost was never derived. The paired control shows a never-cached function DOES return `Absent`, so the seed can express *not computed* and eviction declines to use it. **(4) A receipt for the `CostBasis` ruling:** `cost_account_space_from_summary`'s note cites `CostAccount.space` with a `Derived` basis; it returns a bare `ByteSize?`, constructs no `CostAccount`, the seed does not import `std.realization_schedule`, and `CostBasis` has no `Derived` arm. **(5) The vacuity of `complexity_variables_admits`** is established by the module's own typed `VacuousValidation` carrier plus execution, not by asserting the function exists — which a real implementation would also satisfy.
5. **C0(c) — the `RatchetForever` split (RULED 2026-08-05; the next implementation slice).** Do NOT simply flip the whole mixed `lens_contract_complexity` row to `WallAfterGrounding`. Partition the population. **`complexity.derived-kernel`** — bounded work/span, bounded summation, output size, peak space, bounded-input evaluation, acyclic summaries, descent-grounded SCCs — is decidable-but-unbuilt: `WallAfterGrounding`, `AuditOnly` until a live consumer exists, `Blocking` only after execution coverage. **`complexity.optimality`** — global optimality, complete rewrite discovery, unrestricted semantic complexity equivalence — is the genuinely undecidable residue: `RatchetForever`, permanent `AuditOnly`. The existing construction taxonomy already defines these two classes exactly this way, so the split uses the vocabulary rather than extending it.
6. **C1 — one richer expression authority.** Promote `SizeExpr` / `CostExpr` into `v2.lens.cost` with grounded identities per §6; `SymbolicCost` becomes their normalized projection. **Accept:** every existing cost witness stays green (the `Blocking` wall does not regress), and a bounded summation is representable where it currently is not.
7. **C2 — `ComplexitySummary` restored.** `work`, `span`, `output_sizes`, `peak_space`, `certainty`, `effect_demand`, with sequential and parallel composition laws matching `std.realization_measurement`'s measured side (sequential: work adds, span adds, space maxes; parallel: work adds, span maxes, space adds). **Accept:** two graphs of equal work and different shape derive different span.
8. **C3 — bounded-input evaluation.** *Owned by the sibling note's C1;* listed here only so the dependency is visible. C4 must not begin before it.
9. **C4 — interprocedural and recursive analysis.** Declaration-keyed call graph, SCC summaries, summary memoization, descent bounds, worklist/tree/cardinality shapes; parser progress becomes **one specialized consumer** of generic descent facts, not a special case inside the core. Repoint onto `std.termination` / `std.computation` / `v2.std.cardinality` rather than copying v1's walkers. **The largest and highest-risk slice.**
10. **C5 — realization bridge.** Feed the derived summary into `CostAccount`, and split the basis honestly into `Derived | PredictedFromCalibration | Measured` — a closed symbolic derivation and a calibrated machine estimate are different kinds of evidence and today's two-valued basis conflates them. **Accept:** the sibling note's calibration RED (planted omission → falsifier red).
11. **C6 — live consumers, then and only then flip the mode.** Witness-cost admission (the sibling note's C5), floor scheduling demand, and a complexity regression wall catching accidental O(n)→O(n²) over the affected graph. Plus **self-application**: the complexity implementation costs itself or carries a narrow justified exemption. **`AuditOnly` → `Blocking` is the last step, never the first** — an enrolled blocking claim with no live consumer is the inert-lens lie DESIGN §6 forbids.
12. **C7 — parity corpus and v1 retirement.** Flat iteration, nested iteration, branch max, output-size growth, recursive tree descent, worklist drain, mutual parser recursion, external effect, unbounded refusal, and the parallel work/span distinction. v2 must preserve every meaningful v1 conclusion while using grounded identities and typed refusals. Then `v1.compiler.complexity` dispositions or deletes.

**Sequencing note:** this runs parallel to the self-host frontier and blocks none of it. It *does* block further expansion of directory-driven `long/` policy, because each new resident deepens the proxy this program exists to retire. Roughly 10–16 merge surfaces. Calendar estimates are deliberately omitted: the C4 slice's size depends on how much of v1's SCC and parser logic is already discharged by landed shared substrate, and that is a C0 output, not a guess to make now. **That output landed 2026-08-05** and it cuts both ways — the SCC and descent *vocabulary* is already shared substrate that v2 need only consume (C0(b1) finding 1), while the *walkers* that produce it remain v1-AST-coupled and are the real C4 surface, together with the interprocedural memo. Parser progress is now sized: the operator ruled its disposition on 2026-08-05 — the semantic capability survives as a migration row against the v2 parser plus `std.termination`, and the v1 walker realization is retirement-owed with a trigger naming the capability whose execution discharges it.

## 8. Discriminating controls

- A bounded summation derives an exact count where the current engine can only say `ClassLinear`.
- Sequential and parallel graphs of identical work derive **different span** — a span equal to work in both proves span was never derived.
- An interprocedural summary is reused across two call sites (memo hit) and the derived work does not double-count.
- A mutually recursive pair with descent evidence derives a bound; the same pair with descent removed **refuses**, and does not fall back to a class.
- `complexity_variables_admits` rejects at least one input — a validation that admits everything is the current state and must stop being it.
- Every v1 parity-corpus conclusion is reproduced by v2, or is explicitly dispositioned as superseded with a reason.
- The complexity lens applied to itself produces a bound (self-application), or a narrow, named exemption.
- Flipping `lens_contract_complexity` to `Blocking` while `consumer_witness` is still `NoConsumerWitness` **fails** — the mode and the consumer cannot disagree.

## 9. Operator rulings (all four questions answered 2026-08-05)

The four questions this note raised are ruled. They are recorded here as decisions, and the ones with a typed carrier are named so the carrier stays the authority rather than this prose.

1. **`RatchetForever` re-classification — APPROVED, as a split rather than a flip.** See C0(c) in §7: `complexity.derived-kernel` takes `WallAfterGrounding` (AuditOnly until a live consumer, Blocking only after execution coverage); `complexity.optimality` keeps `RatchetForever` as permanent AuditOnly. This is the next implementation slice.
2. **Parser-progress — the capability SURVIVES; the v1 realization retires.** The semantic facts (parser recursion advances token position, cycles carry strict descent, depth derives from input) are produced by the v2 parser plus `std.termination` integration. The v1 AST and source-index walker does not survive, but it is **not retired yet** — retiring before the replacement executes would delete a live capability. The census models this as two rows: `ParserProgressFactProduction` as migration-owed, `ParserProgressV1Walker` as retirement-owed with a trigger naming the capability whose execution discharges it.
3. **Three-valued `CostBasis` — APPROVED, applied PER RESOURCE AXIS.** `Derived | PredictedFromCalibration | Measured`, but not once per account: one account may legitimately carry time `PredictedFromCalibration`, space `Derived` and power `Measured`, and a record-level basis cannot represent that honestly. C5 lands the per-axis shape, not a widened single field.
4. **Specialized-lens layer — NO blanket collapse.** C0 and C2 test each against the richer engine and disposition it individually: subsumed → delete and repoint; still-useful specialized rule → retain as a consumer or row; still ungrounded → preserve its existing frontier. The corrected table in §2 is why this is a decision rather than a cleanup — `complexity_lowering` dissolves to `SingleAuthority` while the other two dissolve to `SubstrateMandatoryTag`, so they are not one undifferentiated block.
5. **Span stays in C2, not deferred.** Deferring it would create an incomplete summary immediately and would prevent derived and measured parallel composition from agreeing, since `std.realization_measurement` already models that asymmetry on the measured side.
6. **v1 `CostExpr` ownership — this program's C1, not the generic seed-shrink lane.** Seed deletion CONSUMES the migrated authority; it does not decide its semantics.

## Dissolution trigger (DESIGN §6)

Delete this doc when the capability census is closed with every v1 row dispositioned, lens_contract_complexity's boundary class distinguishes the genuinely undecidable residue from the decidable capability, v2.lens.cost carries grounded SizeExpr/CostExpr with work/span/output-size/peak-space/certainty on a restored ComplexitySummary, interprocedural summaries and descent bounds derive over declaration-keyed identities, the parity corpus reproduces or dispositions every meaningful v1 conclusion, lens_contract_complexity has a live consumer witness and has flipped off AuditOnly, and v1.compiler.complexity is deleted or dispositioned — at which point the carriers state the capability and this note retires.
