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

0. **Declare `SourceRef` itself.** Pulled forward from Phase 3 by parent ruling (2026-07-19): the *type declaration only*, not the effect-boundary enforcement, homed with the storage-binding authority on the `v2.compiler.source_authority` side. This lane owns the Phase-3 boundary type, so waiting on it would have been waiting on itself. Trigger to proceed: the declaration lands as the **single** authority that both the manifest rows and the later effect boundary consume — one spelling, established before it has two consumers rather than reconciled after.

1. **Model the row.** `DagSourceReadWitness` gains the ref+hash shape alongside `Medium<String>`; nothing switches. Trigger to proceed: stage 0's declaration landed and the row shape reviewed against it.
2. **Emit both, consume inlined.** Host emits ref+hash on every row *and* keeps inlining under the cap. Over-cap manifests now carry rows (ref-only) where they previously carried `Empty`. Trigger: over-cap manifests non-empty and the quarantined completeness witness re-run to see what it says.
3. **Consume through the ref, with verification.** The parse oracle reads through the ref; witnesses flip to `ReadsLiveTree` with that cost declared. **The flip is separable from stage 5 by explicit ruling** — a correct-but-unselectable witness is a bounded, known cost, and these rows join an *existing* never-predict-skip class rather than minting one, so nothing new is invented to absorb them. Trigger: the compiler-closure completeness witness **greens by itself** — the counted un-quarantine event this lane already registered, and the acceptance test for the whole redesign.
4. **Drop the inlined text.** Cap deleted, `MANIFEST_INLINE_LIST_MAX` and its elision arm deleted with it. Trigger: stage 3 green across every ingest consumer, not just the closure.
5. **Restore selection eligibility.** Ref-reads become selection-eligible keyed on the content hash. Trigger: named here so stage 3's cost has a declared end; scheduling is the affected-set lane's call, not this note's.

Stages 1–2 are additive and cannot regress a consumer. Stage 3 is the behavioral flip and carries the real risk.

## 6a. Named constraints (invariants the staging depends on)

**C1 — the stage-4 read-through returns `Medium<String>`, never a bare `String`.**

Stage 4 deletes the inlined source text from `DagSourceReadWitness`. The obvious
way to do that is to delete the `source: Medium<String>` field. **That is wrong,
and it fails silently.**

`Medium<R> { carried: R, fidelity: DecodeFidelity }` is where a row's
`DecodeFidelity` actually lives (`dag/extdeps/communication/medium.dag:13`).
Deleting the field to remove the text takes the **fidelity carrier** with it, so
the row silently stops declaring `Lossless` — and the round-trip laws in this
module (`SourceAuthorityRoundTripLaw` and friends) depend on that boundary.
Nothing goes red: the laws still typecheck, they just no longer rest on a declared
fidelity. A `Lossless` boundary that quietly became undeclared is the §5
fabricated-plausible-output shape at the level of a proof obligation.

So the read-through introduced in stage 3 and made exclusive in stage 4 **must
return `Medium<String>`**. Fidelity then attaches at the moment of decode, which
is where it belongs on the meaning: fidelity is a property of a decode, not of a
location.

*Corollary (Q4, resolved 2026-07-19):* for the same reason, `SourceRef` itself
carries **no** fidelity field. A ref decodes nothing, so a fidelity on the ref
would assert what no read has established, and would stand a second fidelity
authority beside `Medium`'s (§3). `SourceRef` = path + source root + `ContentHash`.

*Why this is stated here rather than left to the implementer:* stage 4 is several
PRs downstream of stage 0. The failure is invisible at the diff — deleting one
field is the natural edit and produces no red — so the wall belongs in the
document, where the implementer meets it before the tree does.

## 7. Questions, resolved (parent rulings 2026-07-19)

1. **Same `SourceRef` as the Phase-3 effect boundary?** **Yes, and this lane owns it.** The Phase-3 boundary type is this lane's own later phase — there was no external owner to wait on. The type declaration is pulled forward to stage 0 (declaration only, not enforcement), homed with the storage-binding authority. A draft type owned here beats a dependency waited on elsewhere.

2. **Path or module?** **The ref names the STORAGE REALIZATION — path + source root + `ContentHash` — not a module, and not a coproduct fusing both.** The reasoning is this design's own separation applied consistently: the manifest is a *host-boundary artifact about files the host read*, and only host boundaries project paths, so a path-shaped ref is correct exactly there. The module binding, where one exists, is the **derived** fact the binding authority already owns — **joined, not fused**. A `ModuleRef | FileRef` coproduct would fuse two facts this lane deliberately keeps apart.

   *Consequence, now RECONCILED:* the parent design's §3 previously described the boundary `SourceRef` as "module reference, **or** an explicit typed path for genuinely-extra-graph files" — which admits a module-shaped ref. Under this ruling the ref is uniformly storage-shaped, with module identity joined through the binding rather than carried inside. Those were two readings of one type name, and one had to go before stage 0 declares it — otherwise the fork is minted at the moment of declaration, in the very type introduced to prevent forks. §3 has been updated to the uniform storage-shaped reading and the module-shaped reading withdrawn, so stage 0 declares against a single description.

3. **Is the hermeticity flip separable from stage 5?** **Yes** — bounded judgment accepted. Named in stage 3's trigger above. These rows join an existing never-predict-skip class rather than creating one.

4. **Fidelity on ref+hash rows?** **Required.** A row must declare the `DecodeFidelity` of what the ref resolves to, or the `Lossless` boundary the round-trip laws depend on is lost.
