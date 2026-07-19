# Ingest-manifest carrier redesign — rows reference source, they do not inline it

**Status:** DESIGN NOTE for review. No code lands from this document.
**Lane:** module-identity vs storage (wise-bee-768). Ruling (d), parent 2026-07-19.
**Parent:** [module identity vs storage design](module-identity-storage-binding-design.md) §2.

---

## 1. The priced defect (measured, not inferred)

`discover_source_root_ingest` emits `host_source_root_ingest_manifest.dag`, whose rows inline the **full `Lossless` source text** of every file read. Because a row's size is the size of a source file, the emitter carries a corpus-protection cap:

```
MANIFEST_INLINE_LIST_MAX = 64          # cli_run.rs
if records.len() > MAX  ->  emit Empty  # the ENTIRE row list, not a prefix
```

Measured on the compiler closure:

```
discover_source_root_ingest --entry src/v2/compiler/00_compile.dag
  -> 91 source reads
  -> data host_source_root_ingest: SourceRootIngest = Empty
```

91 > 64, so the carrier is empty. Every witness reading `host_source_root_ingest` in that closure has been evaluating against **zero rows**.

This was invisible for two compounding reasons, both now repaired (#6867):

1. The elision was **silent** — `Empty` is a legal `SourceRootIngest`, indistinguishable from "this source root has no files". ⊤-as-ignorance presented as ⊤-as-answer (§5 absorbing fallback).
2. The receipt **fabricated agreement with itself** — it emitted `produced_row_count = read_count` and `coverage_complete = true` over that empty carrier, so the gate witness asserting complete coverage read `91 == 91 && true` and passed. A receipt that cannot be false reports nothing.

The repair made the elision a typed, counted refusal (`SourceRootManifestElided { read_count: 91, cap: 64 }`) and quarantined the completeness claim as known-red. **That makes the deficit loud, located, and countable. It does not fix it.** This note is the fix.

## 2. Why the obvious remedies are wrong

**Raise or remove the cap.** Rejected (parent, explicitly). The cap is real protection: 91 compiler modules of inlined `Lossless` text is a multi-megabyte generated `.dag` that the frontend must then parse on every consumer run. The cap is a symptom of the carrier shape, and raising it trades a visible refusal for an invisible cost — the same trade §5 warns about, one layer down.

**Shrink the closure until it fits.** Rejected. Shrinking what the gate proves in order to stay green is vacuity with extra steps — it produces a green gate over a smaller lie.

**Emit a prefix instead of eliding all.** Rejected, and worth naming because it looks like a kindness. A truncated row list is *worse* than an empty one: it is a partial answer wearing the shape of a complete one, and every consumer that folds over it silently computes over a subset. Elide-all at least fails loudly once the receipt is honest.

## 3. The shape: a row references its source

The row's payload is the problem. A `DagSourceReadWitness` today carries `source: Medium<String>` — the bytes. Instead it should carry **a reference to the bytes plus their content hash**:

- **`SourceRef`** — the typed reference the parent design already names as the Phase-3 end state (§3: "the host-effect boundary takes a typed reference … not `String`"). This note does not mint it; it is the first real consumer of it. For a module read from the tree, the ref is the module's storage realization — exactly the path⇄module binding this lane already homed on `v2.compiler.source_authority`.
- **`ContentHash`** — the existing branded carrier (`dag/std/types.dag`), grounded on the same fnv1a64 authority as `std.content_hash`. Not a new hash surface (§3 convergence thread).

Row size becomes O(1) in the source file's size. The cap stops binding — for the same reason the module-binding manifest this lane already shipped is uncapped: **it carries no source text.** That is not a coincidence to note in passing; it is the precedent, already in tree and already exercised by a wet gate.

Consumption inverts: the parse oracle **reads through the ref** at consumption time and verifies the bytes hash to the recorded `ContentHash` before parsing.

## 4. The consequence this note must not hide: hermeticity flips

An inlined manifest is *self-contained* — a consumer folding over it touches no filesystem. A referencing manifest is **not**. Reading through a ref is a live read, and every witness that consumes the ingest becomes `ReadsLiveTree` rather than `SubstrateInputsOnly` (`src/v2/std/live_tree.dag`).

This is a real cost and it must be priced, not waved through:

- `SubstrateInputsOnly` is what makes a witness **affected-set selection-eligible**. Flipping the ingest consumers to `ReadsLiveTree` makes them never-predict-skip, which pins them to every run — the exact scaling problem the parent design flags for the fail-closed default.
- So the redesign trades *"the carrier is empty and the gate is vacuous"* for *"the carrier is complete and its consumers are unselectable"*. Both are real costs. The second is strictly better — a correct answer that runs too often beats a fabricated one that runs cheaply — but it is a trade, not a free win, and the follow-on (making ref-reads selection-eligible via the content hash as the selection key) should be named now rather than discovered later.

**The content hash is what makes that follow-on possible:** a row whose bytes are pinned by hash has a *declared input identity*, which is precisely what affected-set selection needs. So the hash is not only a fidelity check — it is the thing that can eventually restore selection eligibility. That is the argument for putting it on the row from the start rather than adding it when drift first bites.

## 5. What hash verification buys, and what it does not

**Buys:** the manifest is emitted at time T and consumed at time T+n. Between them the tree can change. Today an inlined manifest is immune (it carries the bytes) and a naive referencing manifest would silently parse *different source than was recorded* — cache impurity, the key wrong before the cache exists. Verifying the hash at read time converts that into a typed, located refusal.

**Does not buy:** the hash does not make the read hermetic (§4 above), and it does not detect a file that changed *and* was re-emitted consistently — that is not drift, that is a new input, correctly.

**Failure arm:** hash mismatch must **refuse with the ref and both hashes**, never re-read-and-proceed and never fall back to whatever is on disk. The refusal is the point.

## 6. Staging, each stage with its trigger

1. **Model the row.** `DagSourceReadWitness` gains the ref+hash shape alongside `Medium<String>`; nothing switches. Trigger to proceed: the shape reviewed and the `SourceRef` carrier agreed with the Phase-3 boundary work, so this is not a second spelling of it.
2. **Emit both, consume inlined.** Host emits ref+hash on every row *and* keeps inlining under the cap. Over-cap manifests now carry rows (ref-only) where they previously carried `Empty`. Trigger: over-cap manifests non-empty and the quarantined completeness witness re-run to see what it says.
3. **Consume through the ref, with verification.** The parse oracle reads through the ref; witnesses flip to `ReadsLiveTree` with that cost declared. Trigger: the compiler-closure completeness witness **greens by itself** — that is the counted un-quarantine event this lane already registered, and it is the acceptance test for the whole redesign.
4. **Drop the inlined text.** Cap deleted, `MANIFEST_INLINE_LIST_MAX` and its elision arm deleted with it. Trigger: stage 3 green across every ingest consumer, not just the closure.
5. **Restore selection eligibility.** Ref-reads become selection-eligible keyed on the content hash. Trigger: named here so stage 3's cost has a declared end; scheduling is the affected-set lane's call, not this note's.

Stages 1–2 are additive and cannot regress a consumer. Stage 3 is the behavioral flip and carries the real risk.

## 7. Open questions for review

1. **Is `SourceRef` here the same `SourceRef` as the Phase-3 effect-boundary type?** It should be — two would be a §3 fork. If the boundary type is not ready, this lane should consume a draft of it rather than mint a parallel one, and stage 1's trigger is exactly that agreement.
2. **Does the ref name a path or a module?** This lane's own answer is that code references modules and only host boundaries project paths. A ref that is a `QualifiedName` + source root is the consistent choice; a ref that is a path re-introduces the string dependency the lane exists to remove. But the ingest reads *files*, including files that may not be modules — so the ref may need to be a coproduct, and that decision belongs with the binding authority, not here.
3. **Is the hermeticity flip acceptable at stage 3, or must stage 5 land with it?** I believe it is acceptable and separable (a correct-but-unselectable witness is a known, bounded cost). If the affected-set lane disagrees, stages 3 and 5 merge and the runway is longer.
4. **Fidelity:** `Medium<String>` carries `DecodeFidelity`. A ref+hash row still needs to declare the fidelity of what the ref resolves to — dropping it would lose the `Lossless` boundary the round-trip laws depend on.
