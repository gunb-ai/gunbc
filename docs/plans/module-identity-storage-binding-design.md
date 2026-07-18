# Module identity vs storage — the path⇄module binding authority, and bidirectional surfaces

**Status:** DRAFT for review (operator-directed 2026-07-18). Model-before-implement; each phase names its receipt and its dissolution trigger. No code lands from this document alone.

**Operator intent (2026-07-18, verbatim in spirit):** we should not depend on literal files; we should depend on nodes/modules, where a module is a collection of nodes and may be represented by ≥0 files *at parse time*. The relationship must work both ways — if code references a file, the compiler knows what logical portion of the graph it refers to; more ideally, code references modules and never deals with the bidirectional mapping at all. The same statement covers the generated-markdown pair: writing to the `.md` should be able to update the `.dag` source, and writing to the `.dag` should update the `.md` — two projections of one graph. Prefer inference over per-site ceremony where the inference is sound.

---

## 1. The incident that priced this (§6 — displaced cost, not elegance)

`self_host_03_normalize_behavioral_receipt_holds` — the behavioral oracle for the emitter keystone (#6775) — was enrolled in the witness corpus and **never executed by CI**. That is two distinct defects, not one (review correction, PR #6856):

**Defect A — the operative one: orphaned enrollment.** The witness never reaches selection at all. Three facts compose: `dag/gunbc/ci_layer_roots.dag::witness_exclusion_substrings` carries a hand roster row excluding `self_host_03_normalize_behavioral_witness_test.dag`; the frontier falsifier (`falsifier_self_host_wet_entries`) enrolls only `SelfEmitted` rows with a wet-receipt binding, and `compiler_frontier_row_03_normalize` is `SeedRetained`; and no other executing consumer exists. Zero executions — **coverage by illusion** in pure form (§6's inert-machinery warning), and the exclusion roster is itself the parallel-ledger shape: a hand list standing where a derived classification should be, with no un-quarantine trigger wired to the fix that landed.

**Defect B — the latent one: the string-hidden dependency.** The transport declares an import closure of three library modules, while its *real* inputs are the seed emitter binary (built from `src/v1/05_emit_rust.dag`) and the module closure of `src/v2/compiler/03_normalize.dag`, the latter referenced only as a string literal:

```
data sn_source_rel: String = "src/v2/compiler/03_normalize.dag"
```

If the witness were merely *admitted* today, its undeclared disposition fail-closed-defaults to `ReadsLiveTree`, for which `floor_row_would_skip` is constant-false — it always runs and pins the whole-tree precompute (safe, but the never-skip lane is exactly what does not scale). Admitted to **precise** selection — the end-state this design wants — selection keys on declared inputs, the declaration omits the real ones, and a sound fail-closed selector would false-skip an emitter-touching PR with the false skip indistinguishable from a correct skip: **cache impurity** in latent form (the key is wrong before the cache is even built).

The #6775 regression (a 705-error full-regen break behind a green oracle) shipped through defect A and was caught only by hand. Defect A is repaired by an admission invariant (Phase 0); defect B is what the rest of this design removes, so that admission can be precise rather than never-skip.

The general form: **a dependency living in a string is an edge living in prose.** §4 says a program is `Node` + `Edge`; every load-bearing file-path literal is an edge the graph cannot see — invisible to the affected set, to the resolver, to content-addressing, and to the effect story.

## 2. The model (§2/§3 — one concept, derived binding)

- **A module is a node.** A subtree root in the syntactic containment tree — exactly the namespace lane's authority (qualified name = nesting position). "A collection of nodes" is precisely "a parent and its children."
- **A file is a storage realization of module content** — one handler of N, the same §2 Realization shape as everywhere else (content-addressed pure spec → host-effect; shell is one transport of an operation, a file is one storage of a module). In the model the binding is **many-to-many and time-varying**: one module in one file (common), one module across several files, several modules in one file, and **zero files** — produced declarations (the body producer) already create modules with no file, which is the standing proof that identity cannot be the path.
- **Live snapshot vs end-state — staged honestly (review correction, PR #6856).** The live tree enforces **1:1**: `insert_module_path` panics on a duplicate module path within a source root; `source_authority` parses one `SourceFile` into one module artifact; and `source_ir_artifact_provenance_from_model` rejects every non-`SourceFile` artifact, so today's provenance is parse-only and zero-file modules cannot enter it at all. Phase 1 therefore derives the **live 1:1 binding** as rows — sufficient for every consumer named below — and the many-to-many semantics (fragment identity and order, merge/collision rules, several-modules-one-file) stay a **named deferral** until a consumer prices them (§6), never a silent assumption. What does land now is the provenance **coproduct**: `ParsedFromSource { artifact, span_index } | ProducedByBehavior { producer }` — the produced arm represents zero-file modules honestly instead of pretending they came from parse.
- **Therefore the path⇄module binding is a derived fact, never an authored one.** The compiler learns it at parse time — it parsed the files and knows which modules came out. It records that as queryable rows. Hand-maintained binding tables (the 27 `FrontierQualifiedModuleBinding` rows in `src/v2/compiler/self_host/frontier.dag`) are a parallel ledger of this fact and dissolve into projections of the derived table. So is the **host-side producer**: compile-clean scope reads `build_module_path_index` (`src/v1/stage0/src/cli_run.rs`), an independent Rust derivation of the same fact — Phase 1's migration must repoint that producer too, else two authorities remain and the projection buys nothing (§3).
- **Home:** `v2.compiler.source_authority` — already self-described as "the cross-tree ingest and round-trip authority," already carrying `SourceIrArtifactProvenance`, `SourceAuthorityRoundTripLaw`, `NormalizedDagSourceParsePrintLaw`, `CanonicalDagSourceEmissionLaw`, and `DagSourceReadWitness`/`SourceRootIngest`. This design adds consumers to those laws; it does not mint a second authority.

Both requested directions become queries against one table:

- **file → graph region:** given a path, the modules (subtrees) parsed from it. This is what the affected set, the compile-clean scope, and the frontier each privately re-derive today — three consumers, one derivation after this lands.
- **module → file(s):** given a module reference, the current storage set. Only *host boundaries* ever need this projection (`gunbc --entry <path>`, `Filesystem.Read`); user code and tool `.dag` reference modules and never see paths.

## 3. Typed `SourceRef` vs inference (§5 — construction first, inference as census and backstop)

The operator's inference instinct — "if a string is fed into a file read, we can infer from the folds it's effectively a file" — is sound, and it is the *bridge*, not the end state:

- **End state (construction):** the host-effect boundary takes a typed reference — `Filesystem.Read`, exec-argv path positions, and transport specs consume a `SourceRef` (module reference, or an explicit typed path for genuinely-extra-graph files), not `String`. Modeled once at the op signature (§2), the bare-string file dependency becomes **unwritable** — authors are never "particular" because there is nothing to remember.
- **Bridge (inference):** a lens — a pure reader over the existing `Node` tree, zero substrate change — folds each witness/tool closure and marks every string literal that **flows into a file-read or exec-path sink**. This is a concrete instance of the parked open thread "can a lens mechanically diagnose the leaf-side of decomposition?" It does three jobs:
  1. **Classification now:** any witness whose closure reaches a host-effect sink is *derived* host-reading and routed to the never-predict-skip deferred lane. The fail-closed default already covers the *undeclared* case (`ReadsLiveTree`, never skipped); what the derived classification adds is the **falsely declared** case — a witness declared hermetic whose closure actually shells out (the RED control of Phase 0). The orphaned-enrollment half of the incident is Phase 0(b)'s invariant, not this lens.
  2. **Census:** the migration inventory that scopes Phase 1 (the shell→intent census pattern: count the sites, collapse to operation families).
  3. **Backstop wall:** after migration, the lens keeps new bare-string file references out until the typed boundary makes it structurally impossible; then the lens row for this class is deleted (a named dissolution, not an inert lens — §6).

Inference is never the permanent mechanism (§4: a heuristic is never necessary in a closed system); it locates and guards while construction lands.

## 4. Bidirectional surfaces — per-surface authority, declared and derived (§3/§4)

**The composed relation (review correction, PR #6856):** a storage surface is not just a path set — it is `(QualifiedModule × TargetModel × StorageRoot) → (paths, authority, AuthoringSurface)`. *Which side is the authority* is already a typed in-tree fact for the Rust surface: the **frontier disposition**. `SeedRetained` → the Rust file is the authority (hand-edits to the seed are the legitimate authoring surface; **not** `BothWays` — there is no `.dag` side to write back to). `SelfEmitted` → the `.dag` graph is the authority and the `.rs` is a projection — `BothWays` becomes *possible* there, gated on a **declared `Lossless` Rust-ingest fragment**, never general Rust ingest. The authority laws must be **medium-parameterized**: `source_authority` is already `_from_model(lm: LanguageModel)`-shaped but defaults to `dag_language_model()`, and `frontier_probe_emit_from_ingest` reads forward only (`.dag` ingest → Rust emit) — markdown and the Rust fragment become additional language models bound to the *same* laws, which is where the backward reads live.

**Three oracles, kept separate — a green in one is not evidence in another:** (i) the surface round-trip law (`ingest(emit(authority)) == authority`), (ii) behavioral self-host equivalence (the wet receipt), (iii) `regen_stage0` byte drift. Conflating them is how a round-trip green would quietly stand in for behavioral proof.

The generated-markdown pair is the same design's second medium. §4's "one grammar read in both directions" is the mechanism; yaml ingest (`ingest_yaml_source`, grammar-owned, consumed by the ci.yml drift+parse gate) is in-tree evidence for **grammar-owned backward reads** — *not* for write-back (`ci.yml` is deliberately `DagOnly` and has no inverse into `gunbc.ci_spec`):

- **Emit direction exists:** DESIGN.md / ROADMAP.md are emitted from their `.dag` sources (`gunbc.design_document` → `expected_design_md`, written by `dag/tools/generated_artifact_gate.dag` `main_wet`), byte-gated by the drift witness.
- **Ingest direction is the gap:** markdown grammar rows read *forward*. With them, "edit the `.md`" means: ingest md → the typed document value → a **keyed delta** against the emitted value → rewrite of the corresponding `.dag` declarations → re-emit both surfaces. **Authority is neither file; authority is the graph.** The `.dag` text and the `.md` text are both projections, and `source_authority`'s round-trip laws are the receipt shape (normalized round-trip, never golden strings).
- **Honesty boundary (§7 `DecodeFidelity`):** bidirectional editing is sound exactly where ingest is `Lossless` — structured sections, prose as string leaves. An md edit outside the grammar **refuses with a located diagnostic, never guesses** (§5). The drift gate grows a *repair direction* (ingest-and-write-back) while keeping its fail-closed refusal for un-ingestable edits.
- **Per-surface authoring policy, declared not assumed:** bidirectionality is a per-artifact declaration — `AuthoringSurface = DagOnly | BothWays` on the `GeneratedArtifact` row. DESIGN.md is `BothWays` (the point of this design); `ci.yml` stays `DagOnly` deliberately (the spec is the authoring surface; accepting yaml edits back into `gunbc.ci_spec` invites two-way authority confusion for zero displaced cost). A `BothWays` artifact without a green round-trip witness is a contradiction the gate refuses — which replaces today's vacuous `artifact_extra_valid(DesignArtifact) => true` with a law that can actually fail.

## 5. Phases (each with receipt + dissolution trigger)

**Phase 0 — admission invariant + effect-reach census.** Two halves, admission first (review correction, PR #6856 — counting a deferred row without executing it reproduces the incident one tier later):

- **(a) Admit the incident witness.** The Gate-A flip (#6858, in flight) makes 03_normalize `SelfEmitted` *and* adds its wet-receipt binding, so `falsifier_self_host_wet_entries()` enrolls the behavioral receipt on the falsifier cadence — admission for *this* witness rides the existing mechanism. Observed execution on that cadence is the receipt.
- **(b) The orphaned-enrollment invariant (the general fix for defect A).** Executable rule: **every witness row names an executing consumer** — discovered rows execute per selection; `SelfEmitted` receipts execute on the falsifier wet cadence; and a `SeedRetained`-era receipt (known-red, legitimately quarantined by an exclusion row) executes on a probe cadence *expecting red*, so the moment it greens is a counted, reported event — the un-quarantine trigger — never a hand discovery. "Enrolled, zero executing consumers" itself reds the gate. This is what makes the incident's shape unrepresentable for the *next* module's receipt, which will be authored red before its flip exactly as this one was.
- **(c) Effect-reach census.** The lens of §3: census TSV of string→file-sink flows across witness/tool closures; witnesses whose closures reach host-effect sinks are derived host-reading and land in the never-predict-skip lane.

*Receipt:* the 03_normalize receipt observed executing on the wet cadence; the invariant redding on a synthetic orphan row (enrolled, excluded, no consumer); the census RED control — a synthetic witness with a shelled dependency and a hermetic declaration must red the classifier.
*Dissolves:* hand `ReadsLiveTree`-style declarations for this class; the silent form of `witness_exclusion_substrings` rows (each surviving row carries an executing probe or an explicit consumer).

**Phase 1 — the binding authority + `ModuleRef` at boundaries.** `source_authority` records the parse-time path⇄module rows — scoped to the **live 1:1 binding** (§2 staging) with the provenance coproduct; the affected set, compile-clean scope, and frontier bindings become projections, **including the host-side producer** (`build_module_path_index` repointed, not left as a second authority); transports/tools reference modules, with paths derived only at host boundaries. **Flagship:** the cssl transport — `sn_source_rel` string → module reference; the emitter dependency becomes a real edge (the emitter module closure / its content hash).
*Receipt (by execution, both directions, and only meaningful after Phase 0's admission):* an emitter-touching PR **selects** the behavioral receipt; an unrelated PR **skips** it — both observed on live CI runs.
*Dissolves:* `FrontierQualifiedModuleBinding` hand rows; the census's bare-string sites (counted down a ratchet, migration before ratchet).

**Phase 2 — markdown backward read + write-back.** The executable inverse, specified (review correction, PR #6856):

- **Authority carrier:** the typed document value — `design_document() -> MarkdownDocument`. The `.dag` source is a program that *builds* this value; the inverse therefore never rewrites arbitrary code — it maps a delta on the **value** onto the string-literal declaration rows the builder reads.
- **Keyed edit identity:** an ingested md edit is located by (section-heading path × list-item position) against the emitted value, and rewrites exactly the corresponding literal row(s) in `dag/gunbc/design_document.dag`. An edit that lands outside the literal-row construct set — structural changes, computed spans — **refuses with location**.
- **Fidelity prerequisite, priced:** the current ingester collapses inline structure to `TextInline` and the round-trip keystone *deliberately asserts* emphasis does not round-trip (`witness_frontier_not_roundtripped`). The fidelity frontier must first move to cover `design_document()`'s **actual construct set** (inline emphasis, links, nested list blocks); the `BothWays` gate is `ingest(emit(authority)) == authority` over that set — not generic markdown parseability.
- **Dual-edit semantics:** md-only or dag-only edits reconcile via the keyed delta; **concurrent divergent edits produce a typed conflict refusal, never last-writer-wins** (§5 — a merge heuristic here would be the absorbing fallback).

**Flagship:** edit a DESIGN.md open-thread entry in the `.md`, observe `dag/gunbc/design_document.dag` updated and the drift gate green; a non-grammar md edit refuses with location.
*Receipt:* round-trip witness green over the construct set + the refusal RED control + the conflict RED control.
*Dissolves:* the "regen must land in the same PR" toil for `BothWays` artifacts (either surface is now a valid authoring point; the gate reconciles); the vacuous `artifact_extra_valid(DesignArtifact) => true` arm.

**Phase 3 — the Rust surface (the model↔realization edit loop, gated on Phase 2 + the flip wave).** The §4 composed relation instantiated for generated Rust: `SeedRetained` modules are explicitly **not** `BothWays` (the seed `.rs` is the authority there until the frontier flips); a `SelfEmitted` module may declare `BothWays` over a **`Lossless` Rust-ingest fragment** — the declaration surface the emitter itself produces (types, signatures, data rows), never general Rust. **Flagship:** edit a supported declaration in one self-emitted `.rs` → ingest via the declared fragment → the same semantic graph delta → rewrite the `.dag` → re-emit Rust → behavioral self-host receipt green. RED control: a Rust edit outside the fragment refuses with location. The three §4 oracles stay separate: the round-trip law proves the surface, the wet receipt proves behavior, `regen_stage0` proves byte drift — this phase cites all three, conflates none.
*Receipt:* the flagship loop by execution + the refusal RED.
*Dissolves:* the hand-sync toil for self-emitted modules (today an `.rs` fix must be re-derived in `.dag` by hand or it reverts on the next regen).

**Phase 4 — containment-tree unification (gated, no code now).** The filesystem tree becomes one more containment tree beside code names; path admissibility = the same prefix relation the resolver walks — the effect-grants design's "fourth consumer" story. Explicitly gated on the namespace-only resolution lane (currently parked post-revert) and the effect-grants lane; this document takes no dependency on either for Phases 0–3.

## 6. Non-goals and interactions

- **Not the namespace terminal.** Phases 0–2 work with today's import-based resolution; nothing here waits on the reverted integration branch or the loyal-heron scaling receipt.
- **Not a file-watcher / sync daemon.** Write-back is a workflow invocation (the same `main_wet` shape that writes artifacts today), not a background process.
- **Not automatic bidirectionality everywhere.** `BothWays` is opt-in per artifact with a green round-trip witness as its precondition; everything else stays `DagOnly` with the existing one-way drift gate.
- **Not general Rust ingest.** Phase 3's backward read is a declared `Lossless` fragment (the emitter's own declaration surface); parsing arbitrary Rust is out of scope permanently — the fragment grows only when a consumer prices the growth (§6).
- **Sequencing with the Gate-A flip wave:** `source_authority` is itself a flip candidate; flip first, then extend (or the flip re-probe absorbs the extension). One sentence of coordination with the flip-wave lane, not a redesign.
- **Realization/materialization alignment:** once the receipt's inputs are content-hash edges (emitter-hash × closure-hash), the receipt result is memoizable under `std.realization` for free — the affected set selects it precisely *and* an unchanged input pair can return the cached verdict. That falls out; it is not scheduled work here.

## 7. Failure modes designed against

- **Dual authority / write-back loops:** the graph is the single authority; both files are projections; `BothWays` reconciliation is ingest→keyed-delta→re-emit-both, never file-to-file merging (§3); concurrent divergent edits refuse with a typed conflict, never last-writer-wins.
- **Counted-but-never-executed rows:** Phase 0's invariant makes "enrolled, zero executing consumers" itself a red — the incident's shape cannot recur silently for the next receipt authored red before its flip.
- **Binding staleness:** the binding is derived at parse and keyed by content hash — there is no hand table to rot (§5 construction; the bad state is unwritable because the state is not authored).
- **Lossy round-trips:** refusal with location at the `DecodeFidelity` boundary; no absorbing "best-effort import" arm (§5 — a failure arm refuses, never widens).
- **Inert machinery:** every phase lands with its executing consumer named in the receipt; the census lens carries a dissolution trigger so it cannot linger as an unwired scaffold (§6).
