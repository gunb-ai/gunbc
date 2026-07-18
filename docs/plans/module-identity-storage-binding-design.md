# Module identity vs storage — the path⇄module binding authority, and bidirectional surfaces

**Status:** DRAFT for review (operator-directed 2026-07-18). Model-before-implement; each phase names its receipt and its dissolution trigger. No code lands from this document alone.

**Operator intent (2026-07-18, verbatim in spirit):** we should not depend on literal files; we should depend on nodes/modules, where a module is a collection of nodes and may be represented by ≥0 files *at parse time*. The relationship must work both ways — if code references a file, the compiler knows what logical portion of the graph it refers to; more ideally, code references modules and never deals with the bidirectional mapping at all. The same statement covers the generated-markdown pair: writing to the `.md` should be able to update the `.dag` source, and writing to the `.dag` should update the `.md` — two projections of one graph. Prefer inference over per-site ceremony where the inference is sound.

---

## 1. The incident that priced this (§6 — displaced cost, not elegance)

`self_host_03_normalize_behavioral_receipt_holds` — the behavioral oracle for the emitter keystone (#6775) — was enrolled in the witness corpus and **never executed by CI**. Its transport declares an import closure of three library modules, while its *real* inputs are the seed emitter binary (built from `src/v1/05_emit_rust.dag`) and the module closure of `src/v2/compiler/03_normalize.dag`, the latter referenced only as a string literal:

```
data sn_source_rel: String = "src/v2/compiler/03_normalize.dag"
```

Affected-set selection — which is fail-closed *over declared inputs* — soundly judged an emitter-changing PR "unaffected" for this witness and skipped it. The false skip is indistinguishable from a correct skip. Two named failure shapes at once: **cache impurity** (key on declared-input content — the declaration was wrong, so every sound decision downstream was wrong) and **coverage by illusion** (§6's inert-machinery warning — a check nothing executes is a lie shaped like coverage). The #6775 regression (a 705-error full-regen break behind a green oracle) shipped through exactly this hole and was caught only by hand.

The general form: **a dependency living in a string is an edge living in prose.** §4 says a program is `Node` + `Edge`; every load-bearing file-path literal is an edge the graph cannot see — invisible to the affected set, to the resolver, to content-addressing, and to the effect story.

## 2. The model (§2/§3 — one concept, derived binding)

- **A module is a node.** A subtree root in the syntactic containment tree — exactly the namespace lane's authority (qualified name = nesting position). "A collection of nodes" is precisely "a parent and its children."
- **A file is a storage realization of module content** — one handler of N, the same §2 Realization shape as everywhere else (content-addressed pure spec → host-effect; shell is one transport of an operation, a file is one storage of a module). The binding is **many-to-many and time-varying**: one module in one file (common), one module across several files, several modules in one file, and **zero files** — produced declarations (the body producer) already create modules with no file, which is the standing proof that identity cannot be the path.
- **Therefore the path⇄module binding is a derived fact, never an authored one.** The compiler learns it at parse time — it parsed the files and knows which modules came out. It records that as queryable rows. Hand-maintained binding tables (the 27 `FrontierQualifiedModuleBinding` rows in `src/v2/compiler/self_host/frontier.dag`) are a parallel ledger of this fact and dissolve into projections of the derived table.
- **Home:** `v2.compiler.source_authority` — already self-described as "the cross-tree ingest and round-trip authority," already carrying `SourceIrArtifactProvenance`, `SourceAuthorityRoundTripLaw`, `NormalizedDagSourceParsePrintLaw`, `CanonicalDagSourceEmissionLaw`, and `DagSourceReadWitness`/`SourceRootIngest`. This design adds consumers to those laws; it does not mint a second authority.

Both requested directions become queries against one table:

- **file → graph region:** given a path, the modules (subtrees) parsed from it. This is what the affected set, the compile-clean scope, and the frontier each privately re-derive today — three consumers, one derivation after this lands.
- **module → file(s):** given a module reference, the current storage set. Only *host boundaries* ever need this projection (`gunbc --entry <path>`, `Filesystem.Read`); user code and tool `.dag` reference modules and never see paths.

## 3. Typed `SourceRef` vs inference (§5 — construction first, inference as census and backstop)

The operator's inference instinct — "if a string is fed into a file read, we can infer from the folds it's effectively a file" — is sound, and it is the *bridge*, not the end state:

- **End state (construction):** the host-effect boundary takes a typed reference — `Filesystem.Read`, exec-argv path positions, and transport specs consume a `SourceRef` (module reference, or an explicit typed path for genuinely-extra-graph files), not `String`. Modeled once at the op signature (§2), the bare-string file dependency becomes **unwritable** — authors are never "particular" because there is nothing to remember.
- **Bridge (inference):** a lens — a pure reader over the existing `Node` tree, zero substrate change — folds each witness/tool closure and marks every string literal that **flows into a file-read or exec-path sink**. This is a concrete instance of the parked open thread "can a lens mechanically diagnose the leaf-side of decomposition?" It does three jobs:
  1. **Classification now:** any witness whose closure reaches a host-effect sink is *derived* host-reading and routed to the never-predict-skip deferred lane. The 03_normalize incident becomes unrepresentable without waiting for the full migration — and without a per-test `ReadsLiveTree` declaration someone forgets (this incident being the proof that they do).
  2. **Census:** the migration inventory that scopes Phase 1 (the shell→intent census pattern: count the sites, collapse to operation families).
  3. **Backstop wall:** after migration, the lens keeps new bare-string file references out until the typed boundary makes it structurally impossible; then the lens row for this class is deleted (a named dissolution, not an inert lens — §6).

Inference is never the permanent mechanism (§4: a heuristic is never necessary in a closed system); it locates and guards while construction lands.

## 4. Bidirectional surfaces — markdown and `.dag` as two grammars over one graph (§4)

The generated-markdown pair is the same design, second medium. §4's "one grammar read in both directions" is the mechanism and yaml already proves it in-tree (`ingest_yaml_source`, grammar-owned, consumed by the ci.yml drift+parse gate):

- **Emit direction exists:** DESIGN.md / ROADMAP.md are emitted from their `.dag` sources (`gunbc.design_document` → `expected_design_md`, written by `dag/tools/generated_artifact_gate.dag` `main_wet`), byte-gated by the drift witness.
- **Ingest direction is the gap:** markdown grammar rows read *forward*. With them, "edit the `.md`" means: ingest md → `Node` tree → the **same graph delta** as editing the `.dag` → re-emit both surfaces. **Authority is neither file; authority is the graph.** The `.dag` text and the `.md` text are both projections, and `source_authority`'s round-trip laws are the receipt shape (normalized round-trip, never golden strings).
- **Honesty boundary (§7 `DecodeFidelity`):** bidirectional editing is sound exactly where ingest is `Lossless` — structured sections, prose as string leaves. An md edit outside the grammar **refuses with a located diagnostic, never guesses** (§5). The drift gate grows a *repair direction* (ingest-and-write-back) while keeping its fail-closed refusal for un-ingestable edits.
- **Per-surface authoring policy, declared not assumed:** bidirectionality is a per-artifact declaration — `AuthoringSurface = DagOnly | BothWays` on the `GeneratedArtifact` row. DESIGN.md is `BothWays` (the point of this design); `ci.yml` stays `DagOnly` deliberately (the spec is the authoring surface; accepting yaml edits back into `gunbc.ci_spec` invites two-way authority confusion for zero displaced cost). A `BothWays` artifact without a green round-trip witness is a contradiction the gate refuses.

## 5. Phases (each with receipt + dissolution trigger)

**Phase 0 — effect-reach census + derived witness classification.** The lens of §3: census TSV of string→file-sink flows across witness/tool closures; witnesses whose closures reach host-effect sinks are derived host-reading and land in the never-predict-skip lane; verify the deferred lane actually **drains** on the falsifier cadence (533 counted-deferred rows with no executing consumer would be the same illusion one level up).
*Receipt:* the 03_normalize behavioral receipt appears in the deferred-execution path of the next emitter-touching PR window, plus a RED control — a synthetic witness with a shelled dependency and a hermetic declaration must red the classifier.
*Dissolves:* hand `ReadsLiveTree`-style declarations for this class.

**Phase 1 — the binding authority + `ModuleRef` at boundaries.** `source_authority` records parse-time path⇄module rows; the affected set, compile-clean scope, and frontier bindings become projections; transports/tools reference modules, with paths derived only at host boundaries. **Flagship:** the cssl transport — `sn_source_rel` string → module reference; the emitter dependency becomes a real edge (the emitter module closure / its content hash).
*Receipt (by execution, both directions):* an emitter-touching PR **selects** the behavioral receipt; an unrelated PR **skips** it — both observed on live CI runs.
*Dissolves:* `FrontierQualifiedModuleBinding` hand rows; the census's bare-string sites (counted down a ratchet, migration before ratchet).

**Phase 2 — markdown backward read + write-back.** Markdown ingest rows (scoped to the generated-doc grammar subset), the normalized round-trip witness, the `AuthoringSurface` declaration, and the write-back workflow (drift gate's repair direction). **Flagship:** edit a DESIGN.md open-thread entry in the `.md`, observe `dag/gunbc/design_document.dag` updated and the drift gate green; a non-grammar md edit refuses with location.
*Receipt:* round-trip witness green + the refusal RED control.
*Dissolves:* the "regen must land in the same PR" toil for `BothWays` artifacts (either surface is now a valid authoring point; the gate reconciles).

**Phase 3 — containment-tree unification (gated, no code now).** The filesystem tree becomes one more containment tree beside code names; path admissibility = the same prefix relation the resolver walks — the effect-grants design's "fourth consumer" story. Explicitly gated on the namespace-only resolution lane (currently parked post-revert) and the effect-grants lane; this document takes no dependency on either for Phases 0–2.

## 6. Non-goals and interactions

- **Not the namespace terminal.** Phases 0–2 work with today's import-based resolution; nothing here waits on the reverted integration branch or the loyal-heron scaling receipt.
- **Not a file-watcher / sync daemon.** Write-back is a workflow invocation (the same `main_wet` shape that writes artifacts today), not a background process.
- **Not automatic bidirectionality everywhere.** `BothWays` is opt-in per artifact with a green round-trip witness as its precondition; everything else stays `DagOnly` with the existing one-way drift gate.
- **Sequencing with the Gate-A flip wave:** `source_authority` is itself a flip candidate; flip first, then extend (or the flip re-probe absorbs the extension). One sentence of coordination with the flip-wave lane, not a redesign.
- **Realization/materialization alignment:** once the receipt's inputs are content-hash edges (emitter-hash × closure-hash), the receipt result is memoizable under `std.realization` for free — the affected set selects it precisely *and* an unchanged input pair can return the cached verdict. That falls out; it is not scheduled work here.

## 7. Failure modes designed against

- **Dual authority / write-back loops:** the graph is the single authority; both files are projections; `BothWays` reconciliation is ingest→delta→re-emit-both, never file-to-file merging (§3).
- **Binding staleness:** the binding is derived at parse and keyed by content hash — there is no hand table to rot (§5 construction; the bad state is unwritable because the state is not authored).
- **Lossy round-trips:** refusal with location at the `DecodeFidelity` boundary; no absorbing "best-effort import" arm (§5 — a failure arm refuses, never widens).
- **Inert machinery:** every phase lands with its executing consumer named in the receipt; the census lens carries a dissolution trigger so it cannot linger as an unwired scaffold (§6).
