# Interface summaries and the declared↔use arity family

> One artifact (`InterfaceSummary`) and one rule (declaration↔use correspondence over scoped root sets) reconcile three lanes that have been converging from different directions: the resolver's closure-denominated quadratic (resolver-graph-major), the witness quadratic behind the CI opt-in inversion (#6232), and the unused/inert detection family (inert-layer-lens). Nothing here opens a new lane; every deliverable folds into an existing one. DESIGN refs: §2 (one concept, every scale; net concepts must not grow by re-invention), §3 (single authority; below-boundary representation is opaque — the rename test), §5 (fail-closed; construction over validation; a failure arm must refuse, never widen), §6 (lens as residue; priced in displaced cost). Companion censuses: [inert-layer-lens](inert-layer-lens.md), [structural-quadratic-wall-coverage-audit](structural-quadratic-wall-coverage-audit.md), [affected-set-precompute-pruning](affected-set-precompute-pruning.md), [resolver-graph-major-design](resolver-graph-major-design.md), [ci-selection-vs-scheduling](ci-selection-vs-scheduling.md).

**Status: DESIGN — for operator review. Flags A–D at the end need a sign each before any carrier lands.**

## 1. The denomination error, seen twice

Two measured pathologies are the same disease at different scales: **dependence is declared at direct denomination but consumed at closure denomination.**

- **Resolver.** `.dag` imports are explicit and item-grained (`import v2.std.logic { Bool }`) — declared dependence is depth-1 and minimal. But resolution materializes each entry's whole ancestry (the "private universe"): ~543 witness entries sharing ~160 modules, each re-resolving them; Σ|closure(i)| is O(n²) on a chain; receipts: 35+ min floor prelude, `build_type_env` whole-ancestry blowup (B1/B2, #6155/#6216). Union-resolve S1 (#6234) dedupes the *payment* (one shared index per process); it does not change the *denomination* — the unit is still the closure.
- **Witnesses.** A witness is definable between any two pipeline segments — the span space over n segments is O(n²) — and tree-glob enrollment converted that definability into obligation: ~1,551 discovered witnesses, 90-minute timeout, "maximum cost, zero delivered signal" (DESIGN Building & checks). The opt-in inversion (#6232) stopped the bleeding by making enrollment declared; it did not yet give the declared population a *basis* that keeps it linear.

The shared fix is a boundary artifact the consumers actually stop at.

## 2. The artifact: `InterfaceSummary`

A content-addressed summary of a module's exported surface: exported names, their signatures, and a contract slot (v0: signature-only; semantic contracts populate as they ground — FLAG D). It is a **projection of declarations already in the tree** — derived, never hand-authored, never a parallel ledger (§3). It is the carrier of DESIGN §3's existing doctrine "below-boundary representation is opaque": the rename test, given a hash.

The key structure it induces:

```
key(d) = hash(source(d), interface_hash(each direct import of d))
```

— transitive ancestors appear in `key(d)` only *through* the interface hashes of direct imports. For a chain `a→b→c→d`, `a` and `b` do not appear in `key(d)` at all.

Five consumers — the §2/§3 signal that this is one authority, not a convenience:

1. **Typecheck boundary (resolver S2a/S2b).** Typecheck `d` against summaries of its direct imports, not resolved bodies. Denomination flips from Σ|closures| (quadratic) to Σ|direct edges| (linear in the import graph). This is also the **reintroduction wall** the structural-quadratic audit names as its only on-dial increment (§5 recommendation 3): once typechecking is only *given* summaries, the whole-ancestry `build_type_env` shape is unwritable, not fenced.
2. **Merkle key component (S2b).** Early cutoff becomes hash-mechanical: a change recomputes at the frontier and **stops at the first node whose interface hash is unchanged** — downstream keys literally did not move. The static reverse closure (v2.lens.affected_set today) remains the honest possibility cone and first approximation; the wavefront refines it per-change. Corollary: the toolchain edge stops being a special case — a compiler change stops propagating at the first module whose emitted artifact is bit-identical. Requires determinism (#5941); nondeterministic output makes the comparison noise.
3. **Proof cut (span-witness factorization).** A span witness `w(a→d)` decomposes into segment witnesses `w₁(a→b), w₂(b→c), w₃(c→d)` exactly when its claim is inductive through statable intermediate contracts `P_b, P_c` — the Hoare cut. The summary's contract slot is where `P` lives. Non-factorability is diagnostic, not fatal: either the seam contract is anemic (fix the model; the span witness carries a dissolution trigger naming the seam) or the property is genuinely global — the sanctioned O(1) end-to-end residue, kept executable on purpose (§5: trusting the factorization entirely would be specification-without-execution at the meta level).
4. **Witness assumption (the quadratic's basis).** Segment witnesses key on (segment, boundary summaries) and are **shared by every span that crosses the seam**: n² definable spans collapse onto an O(n) segment basis. A span verdict becomes a *derived fact* — a monoidal fold of cached segment verdicts along the path, valid while each seam hash is unchanged. No runtime cursor/aggregator is needed: **the DAG plus the cache is the aggregator**; only pieces whose keys moved re-run. Quantitative span claims (cost/latency budgets) factor the same way with a numeric contract slot and a sum-fold.
5. **Liveness root set at module scale (§3 below).** `live(decl) = reachable(own-module uses ∪ exports)`. Without an interface concept, "exported but internally unused" and "dead" are indistinguishable at module scale — the arity family cannot even be *stated* there without this artifact.

## 3. The declared↔use arity family

The operator's framing: unused import, missing import, unused variable are one problem — an **arity disagreement between declaration and use** — and per-case "unused" checkers would be authored forever. The tree already contains the general rule, designed and partitioned correctly in [inert-layer-lens](inert-layer-lens.md) §1.1; this section homes the family there and adds nothing but rows.

**The rule (stated once, there):** a declared concept must be reachable from its scope's roots, be on a named shrinking exception roster with a dissolve-on, or be deleted. 0-reachability is a **wall** (decidable: graph fixpoint; reference-count explicitly rejected — a dead cluster keeps itself alive). Under-consumption ("should have N consumers") is **Rice-undecidable and out of scope** — detected as redundancy (someone hand-rolled the equivalent) by the nicknaming/anemia lens, never by counting expected consumers. The missing-side (use without declaration) is already walled by resolve.

**Rows = (scope, root set).** One reachability engine, N rows — the intent-linearity shape:

| declaration scope | root set | state |
| --- | --- | --- |
| variable binding | the fn's result + effects | new row, same engine |
| import item | the module's own body | new row — **by construction, below** |
| module / carrier / fn | corpus run-roots ∪ exports (needs `InterfaceSummary`) | designed (inert-layer census) |
| lens module | discovered witnesses | live wall (#5433) |
| StandingIntent | LensContracts | enforcement-intent lane |

**Construction-first for the import row (§5: prefer unwritable over detected).** The import block is *derivable from the body's names*: the binding-edge resolution semantics needed for unambiguous name→owner mapping landed as the constructor-owner ruling (#6235; ambiguity is an error). Derive imports as a projection of use and **both directions of the arity disagreement — unused and missing — become unwritable at once**. The lens row for imports then never exists; the lens covers only the scales whose consumers are genuinely external (module/corpus), which is the honest §5 residue. Unused-variable is the same projection question at fn scope, not a new case. No per-case "unused" vocabulary is ever authored again; a new scale is a new (scope, root-set) row.

## 4. What this does to the affected-set end-state

- The static reverse closure (`affected_set_closure`) stays: it is the possibility cone, the correct superset, and the fail-closed floor. Early cutoff (consumer 2) refines it per-change into the actual wavefront — this *falls out of the key structure*; no new selection mechanism is built.
- The witness enrollment carrier gains the **span/seam fact**: which seam(s) a witness asserts. Three consumers: (i) a factorability check at enrollment — a span derivable from the seam basis is refused as redundant (§2's net-concepts rule applied to witnesses); (ii) fold-reassembly of span verdicts from cached segment verdicts; (iii) the kind-projection that makes pick-and-choose enrollment declarative rather than row-by-row.
- The opt-in roster's in-tree dissolve-on (`ci_witness_optin_inversion`, dag/gunbc/commit_workflow.dag:181 — "affected-set selection + floor memoization make per-PR selection-by-affectedness affordable; enrollment returns to discovery shrunk by the affected set") becomes *achievable at change-denominated cost*: discovery over a population whose basis is linear, selected by keys that stop at unchanged interfaces.

## 5. Sequencing — every deliverable folds into an existing lane

1. **`InterfaceSummary` modeled in std** (model-before-implement; the type + projection from declarations, signature-only v0). Blocks nothing; unblocks consumers 1/2/5. Lands with its first consumer per JIT-modeling.
2. **Resolver consumes summaries** at S2a/S2b — that lane's owner, co-designed, not raced (the audit doc's own sequencing rule for the reintroduction wall).
3. **Liveness rows on the inert-layer engine** — module-scale root set gains exports; import row *skipped* in favor of:
4. **Import-block-as-projection** (post-#6235) — transport per FLAG A.
5. **Span/seam fact on the witness carrier** — rides the witness kind-modeling PR (this lane).

**Non-goals (explicit):** no new complexity oracle or cost algebra (audit doc prohibition); no second liveness/reachability engine (§3 single authority — inert-layer is the home); no runtime span-aggregator (consumer 4: the DAG+cache is the aggregator); no confidence-thresholded selection arm anywhere (§5: a heuristic is never necessary; a failure arm refuses, never widens).

**Dissolution triggers:** the import projection dissolves the import lens row before it is born; S2b keys + determinism dissolve static-cone-only selection into wavefront selection; this note dissolves into DESIGN §-anchors and the carriers themselves once landed (no parallel ledger).

## 6. Flags for operator sign

- **FLAG A — import-projection transport.** Hard wall (hand-written import block that disagrees with the derived one is a compile error) vs canonicalizer (formatter/pre-push rewrites the block; drift-gate asserts fixed point). Recommend: canonicalizer first (ergonomics, same fail-closed net through the drift gate), wall when the emitter round-trip owns the medium.
- **FLAG B — summary grain.** Module-level vs per-declaration. Recommend per-declaration with a module rollup hash: tighter firewalls (a change to one export doesn't dirty consumers of another), and it is the grain consumers 3/4 need anyway.
- **FLAG C — span/seam fact timing.** Land with the witness kind-modeling PR now (recommended: it is one field and the factorability check can start advisory), or defer to enrollment-return-to-discovery.
- **FLAG D — contract slot v0.** Signature-only now (delivers consumers 1/2/5 immediately); semantic contracts (`P` cut formulas) populate per-seam as they ground, unblocking consumers 3/4 seam-by-seam. An unpopulated contract slot is typed absent, not fabricated — a span crossing an ungrounded seam simply cannot factor yet and stays an executable witness.
