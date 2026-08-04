# v2 complexity capability parity — restoring the engine, not porting the seed

> **Status: DRAFT for operator review (2026-08-04).** Design-note-first: **no code lands from this note.** It answers one question — *is v2 complexity analysis anemic compared to v1?* — with a verdict grounded in the tree's own status carriers, and declares the migration program that closes the gap.
> **Verdict: yes, and the tree already says so in its own enforcement model.** The anemia is real, it is localized (see §2 — it is *not* the whole cost stack), and one carrier currently declares it **permanent**, which is the finding worth acting on first (§4).

## 1. Scope seam with the sibling note (DESIGN §3 — do not fork)

This note and [`discrete-cost-derivation.md`](discrete-cost-derivation.md) are two halves of one program and must not restate each other. The split is by *question*:

| Question | Owner |
| --- | --- |
| What can the engine *express and derive*? (cost algebra, work/span, output size, peak space, interprocedural summaries, recursion) | **this note** |
| What does a derivation *mean and get used for*? (subject binding, valuation at a declared input, honesty states, cost justification, witness admission) | [`discrete-cost-derivation.md`](discrete-cost-derivation.md) |

**Consequence, stated so neither note quietly grows into the other:** the expression-algebra promotion and the work/span/space summary — which the sibling note listed under its C0 and C2 — are **this note's C1 and C2** and are removed from that note's scope. The valuation environment and the admission consumer remain **its** C1 and C5 and are absent here. The C0 authority ruling is shared and is stated **once, here**, because it is a ruling about the engine.

## 2. The verdict, and its exact boundary

**The cost half is live; the complexity half is not.** This distinction matters, because over-scoping the fix would be a second error. From `v2.lens.enforcement.contract`:

| Contract | Mode | Consumer witness | Boundary class |
| --- | --- | --- | --- |
| `lens_contract_cost` | **`Blocking`** | **`BoundConsumerWitness`** (`complexity_gate/budget_roster_completeness_test.dag`) | `WallNow` |
| `lens_contract_complexity` | `AuditOnly` | **`NoConsumerWitness`** | **`RatchetForever`** |
| `lens_contract_complexity_accumulator_copy` | `AuditOnly` | `NoConsumerWitness` | `WallAfterGrounding` |
| `lens_contract_complexity_linearity_audit` | `AuditOnly` | `NoConsumerWitness` | — |

So the generic structural engine `v2.lens.cost` is a real, enrolled, blocking wall — it is not a stub, and this program does not touch its kernel. Everything *above* it is audit-only with no consumer: the projection lens and all three specialized complexity lenses. That is the DESIGN §6 **coverage-by-illusion tier**, stated by the repo about itself.

A second, sharper tell sits inside the projection lens: `v2.lens.complexity` `complexity_variables_admits` is

```
fn complexity_variables_admits(xs: List<SizeVariable>) -> Bool {
  true
}
```

— a `Validation` whose admission predicate is the constant `true`. It refines nothing; the `Refined<List<SizeVariable>>` carrier it feeds is a type name standing where a constraint was supposed to be. DESIGN §4b names this class exactly: *'richer type names are not safety — a brand, wrapper, or `Validated<T>` is cosmetic until construction and acceptance enforce the distinction.'*

## 3. Capability comparison (verified, not recalled)

`src/v1/complexity.dag` is 4,857 lines and 171 declarations. What it carries that v2 does not:

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
| Call graph / SCC | yes | **no** |
| Recursion / descent bounds | descent + recursive-path accounting | loop-bound edge only |
| Parser-progress reasoning | yes (mutual recursion) | **no** |
| Concrete-input evaluation | partial machinery | **no** (sibling note's C1) |
| Live repo-wide consumer | seed-internal | cost yes / complexity **no** |

**The gap is real and large. It is migration debt, not a deliberate retirement** — nothing in the tree declares these capabilities unwanted.

## 4. The finding to act on first: `RatchetForever` is very likely mis-declared

`lens_contract_complexity` carries `ConstructionJustification { class: RatchetForever }`. That is a **permanence claim**: this class is undecidable, so it can never become a wall and will always be a validating ratchet. DESIGN §5 is explicit that this is the trap to check rather than the safe default:

> *'The word "never" is the trap: it lets a ratchet masquerade as a wall, so check decidability before claiming one.'*

Applied here the claim does not survive contact with its own contents. **Optimality is undecidable by Rice — but most of what is missing is not optimality.** Work and span composition over a closed kernel is decidable. Summation over a *bounded* collection is decidable — that is what the modeled loop bound exists to provide. Output size and peak space over the same bounded fragment are decidable. Interprocedural summaries over an acyclic declaration graph are decidable, and the cyclic case is exactly what SCC plus `DescentEvidence` already decides fail-closed. So `RatchetForever` on the whole contract is a single permanence verdict standing over a **mixed** population, and its effect is to make the anemia unrankable: a class declared permanently undecidable never appears on anyone's list of things to climb.

**C0's first deliverable is therefore to re-classify this contract**, splitting the genuinely-undecidable residue (optimality; arbitrary predicate refinement) from the decidable capability that should carry `WallAfterGrounding` with a named trigger. Until that split lands, every phase below is arguing against a declared 'never'.

## 5. Authority ruling (C0 — to sign)

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

- function and collection identity → `DeclarationRef` / `ValueIdentity` (the G1 cited-symbol resolution work already makes these resolvable)
- `CostExtern { name }` → `CostEffect { operation: DeclarationRef }`, joining the sibling note's `ExecuteEffect` atom rather than a second extern channel
- `CostUnknown { reason: String }` → a typed, located, **countable** cause — a free-text reason cannot be tallied, so its frequency never ranks the deficit for fixing (DESIGN §5)
- `Certainty = Proven | Conservative` → reconciled with the existing `DescentEvidence` and `Measured` bases rather than minted as a third confidence vocabulary

**Registry note:** `llvm_instruction_cost` is not merely present in `v2.lens.cost` — it is *registered* as owned by it (`v2.lens.registry` `lens_owned_fn_llvm_instruction_cost`, `owner_module_path: "v2.lens.cost"`). Re-homing that uncited machine table to a cited per-target model is therefore a registry change too, not a file move.

## 7. Phases

1. **C0 — capability census + boundary re-classification.** One typed inventory row per v1 capability with disposition `AlreadyInV2 | AvailableInSharedSubstrate | NeedsMigration | SupersededByBetterModel | DeliberatelyRetired`; plus the §4 `RatchetForever` split. **Accept:** no capability is unclassified, and the permanence claim covers only genuinely undecidable rows.
2. **C1 — one richer expression authority.** Promote `SizeExpr` / `CostExpr` into `v2.lens.cost` with grounded identities per §6; `SymbolicCost` becomes their normalized projection. **Accept:** every existing cost witness stays green (the `Blocking` wall does not regress), and a bounded summation is representable where it currently is not.
3. **C2 — `ComplexitySummary` restored.** `work`, `span`, `output_sizes`, `peak_space`, `certainty`, `effect_demand`, with sequential and parallel composition laws matching `std.realization_measurement`'s measured side (sequential: work adds, span adds, space maxes; parallel: work adds, span maxes, space adds). **Accept:** two graphs of equal work and different shape derive different span.
4. **C3 — bounded-input evaluation.** *Owned by the sibling note's C1;* listed here only so the dependency is visible. C4 must not begin before it.
5. **C4 — interprocedural and recursive analysis.** Declaration-keyed call graph, SCC summaries, summary memoization, descent bounds, worklist/tree/cardinality shapes; parser progress becomes **one specialized consumer** of generic descent facts, not a special case inside the core. Repoint onto `std.termination` / `std.computation` / `v2.std.cardinality` rather than copying v1's walkers. **The largest and highest-risk slice.**
6. **C5 — realization bridge.** Feed the derived summary into `CostAccount`, and split the basis honestly into `Derived | PredictedFromCalibration | Measured` — a closed symbolic derivation and a calibrated machine estimate are different kinds of evidence and today's two-valued basis conflates them. **Accept:** the sibling note's calibration RED (planted omission → falsifier red).
7. **C6 — live consumers, then and only then flip the mode.** Witness-cost admission (the sibling note's C5), floor scheduling demand, and a complexity regression wall catching accidental O(n)→O(n²) over the affected graph. Plus **self-application**: the complexity implementation costs itself or carries a narrow justified exemption. **`AuditOnly` → `Blocking` is the last step, never the first** — an enrolled blocking claim with no live consumer is the inert-lens lie DESIGN §6 forbids.
8. **C7 — parity corpus and v1 retirement.** Flat iteration, nested iteration, branch max, output-size growth, recursive tree descent, worklist drain, mutual parser recursion, external effect, unbounded refusal, and the parallel work/span distinction. v2 must preserve every meaningful v1 conclusion while using grounded identities and typed refusals. Then `v1.compiler.complexity` dispositions or deletes.

**Sequencing note:** this runs parallel to the self-host frontier and blocks none of it. It *does* block further expansion of directory-driven `long/` policy, because each new resident deepens the proxy this program exists to retire. Roughly 10–16 merge surfaces. Calendar estimates are deliberately omitted: the C4 slice's size depends on how much of v1's SCC and parser logic is already discharged by landed shared substrate, and that is a C0 output, not a guess to make now.

## 8. Discriminating controls

- A bounded summation derives an exact count where the current engine can only say `ClassLinear`.
- Sequential and parallel graphs of identical work derive **different span** — a span equal to work in both proves span was never derived.
- An interprocedural summary is reused across two call sites (memo hit) and the derived work does not double-count.
- A mutually recursive pair with descent evidence derives a bound; the same pair with descent removed **refuses**, and does not fall back to a class.
- `complexity_variables_admits` rejects at least one input — a validation that admits everything is the current state and must stop being it.
- Every v1 parity-corpus conclusion is reproduced by v2, or is explicitly dispositioned as superseded with a reason.
- The complexity lens applied to itself produces a bound (self-application), or a narrow, named exemption.
- Flipping `lens_contract_complexity` to `Blocking` while `consumer_witness` is still `NoConsumerWitness` **fails** — the mode and the consumer cannot disagree.

## 9. Open questions for operator review

1. **Is the §4 `RatchetForever` re-classification agreed?** It is the highest-leverage single edit here — while it stands, the anemia is declared permanent and cannot rank for climbing. It is also a *narrowing* of a permanence claim, which is normally a safety-increasing move, but it touches an enrolled contract's boundary class and so deserves an explicit sign-off.
2. **Does `parser-progress` survive as a modeled capability, or retire with the seed?** It is v1's most implementation-specific analysis. If the v2 parser reaches its bound through the generic descent facts, this capability is `SupersededByBetterModel` rather than `NeedsMigration` — a C0 answer that materially changes C4's size.
3. **Three-valued `CostBasis` (`Derived | PredictedFromCalibration | Measured`) — confirm.** It is a widening of an existing enrolled carrier that the scheduling design signed as two-valued, so it needs an explicit amendment rather than an incidental change.
4. **Does the specialized-lens layer collapse?** `complexity_accumulator_copy`, `complexity_lowering`, and `complexity_linearity_audit` are focused rules built around the thin center. Some may dissolve into the richer engine once C2 lands; that would be a §2 win, but each currently has its own contract row and one carries a `WallAfterGrounding` trigger, so the collapse is a decision rather than a cleanup.

## Dissolution trigger (DESIGN §6)

Delete this doc when the capability census is closed with every v1 row dispositioned, lens_contract_complexity's boundary class distinguishes the genuinely undecidable residue from the decidable capability, v2.lens.cost carries grounded SizeExpr/CostExpr with work/span/output-size/peak-space/certainty on a restored ComplexitySummary, interprocedural summaries and descent bounds derive over declaration-keyed identities, the parity corpus reproduces or dispositions every meaningful v1 conclusion, lens_contract_complexity has a live consumer witness and has flipped off AuditOnly, and v1.compiler.complexity is deleted or dispositioned — at which point the carriers state the capability and this note retires.
