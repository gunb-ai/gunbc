# Authored-name contract (v1 cull audit P2.9, lane H) — design only

Brief pinned at `63a464edf01fd94ea7a6ef9393f4af7b15479015`
(`git show 63a464edf01fd94ea7a6ef9393f4af7b15479015:docs/plans/v1-interface-cull-audit.md`, §3a "Cross-phase", §6 P2.9, §7 lane H).
This is a contract and a decision, not an implementation. Nothing under `src/` changes.

## 1. What exists today (read at origin/main)

`v1.std.core` `Node` carries two name-bearing fields:

- `name: String` — the **structural** name, set by the constructor (`make_*_node`).
- `ident_span: SourceSpan?` — the span of the identifier token. `default_ident_span(name, span)` is `none` for `name == ""`, else the span.

`authored_name_at(source_indices, node)` is a **projection of `ident_span`**, not a read of `name`:

| `ident_span` | file in `source_indices` | result |
|---|---|---|
| `Absent` | – | `""` |
| `Present` | yes | `source_text_at(index, span)` = `chars_to_string(index.char_codes, start, end)` |
| `Present` | no, file is `<kernel:N>` (`kernel_span`) | `N` sliced from the pseudo-file name |
| `Present` | no, otherwise (`no_span()` → `<synthetic>`, dropped files) | `node.name` |

It has ~676 call sites across `03_normalize`, `03_resolve`, `04_*`, `05_emit_*`. The measured cost (§3a: ~19 s reconcile, ~43 s emit) is one `map_get` by file path plus one `chars_to_string` slice per call, for a fact fixed at node construction.

`with_authored_identity(identity, structural)` (`v1.compiler.resolve`) builds a node whose **structural** fields (`name`, `children`, `inferred`, …) come from `structural` but whose `ident_span` comes from `identity`. This is the one place the two fields are deliberately allowed to disagree: the node *is* structurally `FieldOfFractions<Nat>` and is *spelled* as the alias. `preserve_nominal_brand_on_resolve` and the transparent-alias relation read the difference (recurring failure mode `transparent_alias_relation_read_the_brand_keeping_peel`). So `authored_name_at` and `.name` are different facts by design, and a carried value must keep them different.

## 2. The contract

**C1 — one fact, two roles, never conflated.** `authored` = the spelling at the identity position; `structural` = the name of what the node denotes. `authored_name_at` answers *authored* only. A carried value replaces the projection, never `.name`.

**C2 — the snapshot is the file's text at parse time.** The authored name is defined as the slice `[span.start, span.end)` of the source whose text the parser tokenized. Spans are indices into `char_codes` (Unicode scalars, `chars`), not bytes or UTF-16 units. Non-ASCII is therefore already correct and must stay scalar-indexed: a carried `String` is computed from the same `char_codes` slice, so the two agree byte for byte. Re-slicing later against a different snapshot is the only way they can disagree, and that is exactly the risk carrying removes.

**C3 — fixed at construction, not at use.** The carrier is computed where `ident_span` is minted (parser constructors, `kernel_span` constructors, `no_span` synthetics), from a snapshot in scope there. No consumer ever needs `source_indices` to ask a node its name.

**C4 — propagation.**
- Constructors (`make_*_node`, the 04_types kernel nodes): authored = the name slice they already receive as `name`/`name_span`; kernel nodes: authored = the kernel name.
- Rebuilds that copy a node (`Node { …n… }`, normalize, resolve, infer rewrites): authored is copied with `ident_span` — same field group, same rule, so a copy cannot split them.
- `with_authored_identity`: authored comes from `identity` (alongside `ident_span`), everything else from `structural`. This is the single sanctioned divergence and stays the only one.
- Synthetic nodes with `ident_span = Present{no_span()}`: authored = `name` (today's fallback), made explicit.

**C5 — missing source and kernel.** Today a missing snapshot silently falls back to `node.name`. In the carried form missing-source cannot arise at read time (C3). Where a construction site genuinely has no snapshot it must say so by constructing the `<synthetic>`/`<kernel:>` variants, not by omission. Refusal, not fabrication: a node with a span but no obtainable text at construction is a typed construction error, not `""`.

**C6 — what must stay identical.** For every node the current pipeline reaches, carried authored name == `authored_name_at` today (including `""` for absent `ident_span`, and `node.name` for unknown files). Outputs and refusals unchanged; this is checkable per node by a differential over the corpus before the projection is deleted.

## 3. Decision: carry it, in the identity position's field group

`authored_name_at` becomes a **carried value**, not a cached projection. Reasons in the repo's terms:

- DESIGN §2 step 1: the repetition is authored duplication (the same fact demanded ~676 call sites × every phase), and its least common ancestor is the node's construction. Carry the first value; do not add a cache (preamble: "never add a cache").
- A memo keyed by `(file, span)` would add a provider with an identity (file, span) and a retention story, to hide the signal. It also cannot express the `with_authored_identity` divergence without keying on the node, which is the carried-value design with extra steps.
- A single field, `authored_name: String`, replaces the `ident_span`-as-text role. **`ident_span` stays** (diagnostics, `node_name_span`, spans for edits); only its text-projection role moves.

Where: on `Node`, adjacent to `ident_span` and set by the same constructors. Not on `SourceSpan` (a span is position, not content) and not in a side table (`source_indices` is exactly the per-call lookup being removed).

## 4. Relation to P2.8 (typed-graph identity)

P2.8 extends `OccurrenceProjected` to inferred/rebuilt type nodes and keys on immutable node state + traversal role. The authored name is **immutable node state at the identity position** and so belongs in that key's state side, not its role side. But the brief is explicit that P2.9 does not wait for P2.8: the carried field does not change any occurrence ID; it only moves where a derived-from-`ident_span` string is computed. P2.8 later reads the field for free. No ordering constraint beyond that.

## 5. Open points for the operator (not decided here)

1. Adding a field to `Node` changes every positional/record `Node { … }` construction (~the 25+ literal sites in `04_types.dag` alone) and the stage0 `Node` mirror — lane Z regenerates mirrors, but the **literal-site edit is large**. Alternative with the same contract and a far smaller blast radius: make the carrier `ident_span`'s payload (`SourceSpan?` → `Ident?` holding span + authored text). Recommendation: the `Ident?` payload form, since it removes the case where one is updated without the other. This reshapes every `ident_span` reader (`node_name_span`, `default_ident_span`), so it still needs an operator call before any lane implements it.
2. `Node` is load-bearing (DESIGN names the substrate types); both forms touch it. Per the project-spirit rule this raises the bar and wants a model PR in `std`/seed before a pipeline lane migrates.
3. Not measured here: the sum of the ~19 s + ~43 s is a call-count claim from §3a, not re-profiled; the contract does not depend on the number.

## 6. Verification plan for the implementer

- Differential: for every node reachable in the self-host closure, assert carried == `authored_name_at` (RED control: a rebuild that drops the field, and a `with_authored_identity` that takes it from `structural`).
- Non-ASCII specimen: an identifier with a non-BMP scalar earlier in the file than the name.
- Kernel and `<synthetic>` specimens, and an alias brand-keeping peel (`FieldOfFractions<Nat>` spelled as its alias).
- Required routes `//gunbc/instruments:self-host` and `//gunbc/instruments:v2-native-cli` unchanged.

## PURPOSE test (§5b.3, `gunbc.v1.v1_maintenance_standing v1_seed_standing`)

Serves the v2 self-host program by (a) removing the ~19 s reconcile and ~43 s emit demand on the required job once implemented, and (b) deleting a source-snapshot threading (`source_indices` as a parameter of ~676 sites) so less seed surface exists for v2 to carry. This PR itself is design only and changes no behavior.
