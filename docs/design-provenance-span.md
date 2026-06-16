# Design: Provenance Anchoring — Node Occurrence Identity + Span Index (PROV / T-8)

> **Status: DESIGN — input to the dispatched PROV build** (operator GO 2026-06-09, #4592 —
> the Mgr-SYNTH foundation lane; this doc was written while the lane was HELD and is the
> substrate-shape input the build commits to). Gated on T-8. Consumers: the three
> fixture-bound lanes — real-input affected-set (AFF), write-axis "show the correct code"
> (WRITE), synthesis IR-edit handoff (SYN, ctrl#1499 §7).
>
> Path note: the Node declaration lives at `src/v2/std/node.dag:86` (briefs citing
> `00_core.dag` are stale).

## 1. Problem (the measured verdict, restated)

`Node { kind, children }` has no source anchor. Tokens carry full spans at tokenize
(`Token { class, lexeme, file, start, end }`, `01_tokenize.dag:490`) and parse **discards
them** at the terminal boundary — the live T-8 marker at `02_parse.dag:633-636`: "terminal
node carries tok.class only — tok.lexeme, tok.file, tok.start, tok.end are staged for T-8
source-text anchoring. The node substrate has no String/Nat literal-value edge kind…". So
there is no span→node query path to expose; PROV is a build. The design questions: span
representation, every-node-vs-selective, and the index structure — plus the one hazard the
dep graph doesn't name: **what anchoring does to structural equality**.

## 2. What already exists (M9 DFS — the span carrier needs no inventing)

| Piece | Where | Role |
|---|---|---|
| `Locus = Textual { file, extent } \| NodeLocus`, `Extent = WholeFile \| ByteRange { start, end }` | `src/v2/std/diagnostic.dag:13-24` | **the span representation, already declared** — reuse, do not coin `SourceSpan` |
| Token spans (byte positions, `ByteRange`-consistent) | `01_tokenize.dag:490,502` | the producer-side facts, already flowing to the discard point |
| A4 opacity `CharOffset ≠ ByteOffset` (landed #4554) | brand relation | the offset Ints in `ByteRange` upgrade to branded `ByteOffset` when convenient — a refinement, not a blocker |
| The brand-channel architecture: opaque id stamped on Node + graph-global allocator + registry (`binding_id`, design #4579/#4587, impl #4581) | brand lane | **the playbook this design copies** for a second axis: identity-on-a-dedicated-channel, never identity-from-structure |
| `Path { steps: List<Symbol> }` | `std/node.dag` | available for path-keyed receipts in rewrite transport (§4.3) |
| `NodeArtifactProvenance` (file-level), v3 port-provenance | affected_set fixtures / v3 | **stay separate** (the measured verdict's layer 4) — different grain, different consumers |

## 3. The hazard that decides the shape: structural equality

Putting span *data* in the tree is wrong twice over:

- **Equality/identity:** structural equality is the load-bearing operation (find_witness
  candidates, fixed-point comparison, claim receipts, dedup/content-hash). Span-in-structure
  makes two occurrences of `1 + 1` unequal — unless every equality/hash consumer learns to
  skip spans, which plants a per-consumer invisible exception (the parallel-authority drift
  P2 forbids; #4564's lesson was *one* equality authority).
- **Substrate gap:** the T-8 marker is explicit that Node has no String/Nat literal-value
  edge kind — span-as-children would force that substrate extension for the wrong reason.

The content-hash side has the dual problem: identical subtrees share structural identity, so
a span table keyed by content hash cannot distinguish the two occurrences of `1 + 1` —
provenance needs **occurrence** identity, which structure by definition doesn't carry.

## 4. Design

> **Amendment (operator direction 2026-06-10, §4.5):** the index's value generalizes from
> a bare `Locus` to a forward-stamped **origin event**, so the chain of events is total
> over *every* node source — files, transforms, generators — not just parsed text. §4.1's
> anchor/equality shape and §4.2's every-node rule are unchanged; §4.3 and §4.4 are
> sharpened by it as noted inline.

### 4.1 Anchor = opaque occurrence id on Node; span data lives off-tree

Copy the brand-channel playbook for the provenance axis:

- `Node` gains one field: an opaque **occurrence id** (allocator-issued at the producing
  boundary, like `BindingIdAllocator`). The id is the *anchor*, not the data. **Every**
  node carries one — there is no legitimate absent/zero state: a producer that creates a
  node without stamping id + origin event is defective, fail-closed at that stage's own
  receipt (§4.5 totality). Synthesized nodes are not an exception — they get a fresh id
  with a `DerivedBy`/`GeneratedBy` event (§4.3).
- A per-compile **SpanIndex** carries the data: occurrence id → **`OriginEvent`** (§4.5).
  `FromSource { locus: Locus }` is the wave-1 variant — the `Locus`
  (`Textual { file, ByteRange }`) lives *inside* the event, so the diagnostic.dag carrier
  is still the span representation, but the index value is the origin event. Built where
  the T-8 marker sits — the parse terminal boundary stamps ids and records
  `FromSource` with `tok.file/start/end` instead of dropping them; interior nodes record
  the hull of their children's extents.
- **One equality rule, stated once:** occurrence ids do not participate in structural
  equality or content hashing — declared in the Node-field policy table at
  [`design-node-identity-channels.md`](design-node-identity-channels.md) (the single owner
  for Node-carried fields and equality participation), implemented only at the two equality
  authorities that doc names. This is the entire equality cost of the design, and it is one
  table row instead of N consumer exceptions. (Contrast `binding_id`, which *does*
  participate in type identity — brand is semantics, provenance is bookkeeping; opposite
  treatments, both stated in the same table.)

This sidesteps the literal-carrier substrate gap entirely (span facts are index entries, not
Node children), keeps `Node` pure for every structural consumer, and gives AFF/WRITE/SYN the
occurrence-grained handle they actually need.

### 4.2 Every node, not selective

Stamp every node. Selective stamping creates a second question ("which nodes have
provenance?") that every consumer must re-answer — ambiguity with no payoff, since ids are
a fixed-width field and the index is linear in node count. Parse stamps `FromSource`;
synthesized/derived nodes stamp `DerivedBy`/`GeneratedBy` (§4.3, §4.5) — different event
kinds, not an exemption from stamping.

### 4.3 Transport through rewrites (normalize / resolve / infer)

Stages that rebuild nodes transport ids by one of two declared moves — never silently:

- **carried**: the rewritten node is *the same occurrence* (relabel, reattach) — id rides
  along (free with a field; this is why field-anchor beats path-keying for transport: paths
  rebase on every structural edit, ids don't).
- **derived**: the node is synthesized from a set of source occurrences — it gets a fresh id
  and the index records a `DerivedBy` origin event (§4.5): the **producer identity is
  mandatory**, the `from: List<OccurrenceId>` set may be empty. A node whose chain never
  reaches source text resolves to a typed **Unanchored** verdict at query time — fail-closed,
  never a fabricated span (C-9: no plausible-placeholder loci) — but the verdict carries the
  origin chain, so "no byte-range" never means "no provenance" (§4.5).

### 4.4 The span→node query (the producer PROV's consumers call)

Two directions over the index:

- id → `OriginEvent`, and onward to `Locus`: the index lookup yields the event (§4.5);
  `FromSource` answers the span question directly, `DerivedBy` / `GeneratedBy` resolve
  through the chain to the source-text frontier (or the typed `Unanchored` verdict
  carrying the chain).
- byte position / `ByteRange` → occurrence: per-file scan for **narrowest enclosing
  extent**; 0 enclosing ⇒ typed `NoEnclosingOccurrence` (fail-closed — an edit in
  whitespace/comments resolves to nothing, honestly); ties broken by narrowest-then-deepest.
  Wave 1 is a linear fold (bounded, fine at current scale — recorded as a cost fact);
  a sorted interval structure is a later optimization on the same single representation,
  never a second one.

`Locus.NodeLocus` already exists for node-anchored diagnostics; with ids landed, a
`NodeLocus`-bearing diagnostic becomes *resolvable to source* via the index — which is
precisely WRITE's "show the correct code at the right place" and SYN's
`{locus, corrected_IR}` handoff shape.

### 4.5 Origin events — facts flow forward, from every source (operator direction 2026-06-10)

The operator's framing, now binding: provenance is a **clear chain of events from every
source, not just files**. Byte-range anchoring (the T-8 case) is one origin kind among
several, and the chain is **stamped forward** by each producer at the moment it creates a
node — where the fact is known — never reconstructed retroactively.

Concretely, the index value is an origin event, a closed coproduct (M4) over the ways a
node comes to exist in this codebase:

```
type OriginEvent
  = FromSource { locus: Locus }                                  // parse: file + ByteRange
  | DerivedBy  { producer: Symbol, from: List<OccurrenceId> }    // normalize/resolve/infer/
                                                                 //   translate/coercion derivation
  | GeneratedBy { generator: GeneratorId, from: List<OccurrenceId> } // testgen/fixture/builder
```

- **Totality is the producer obligation:** every producer that creates a node stamps an
  event; an id with no origin event is a producer defect (fail-closed at the stage's own
  receipt), not a legitimate state. The producer always knows its own identity and inputs
  at the production site — this is the "facts flow forward" discipline: stamp at the
  authoritative boundary, exactly as parse stamps byte ranges (T-8) and resolve stamps
  `binding_id` (#4581).
- **`Unanchored` becomes a query verdict, not a stored fact.** "Where did this node come
  from" is *always* answerable by walking the chain; only "what bytes" can honestly be
  unanswerable, and the `Unanchored` verdict then carries the chain (which generator, which
  stage, derived from what) instead of a dead end.
- **Reference, not merge (layer-4 separation preserved):** `GeneratorProvenance` and
  `NodeArtifactProvenance` remain separate carriers with their own grain (the measured
  verdict stands); `GeneratedBy` *references* `GeneratorId` and `FromSource` reuses `Locus`.
  End-to-end chain queryability comes from links, not from collapsing carriers.
- **Cost:** same index, richer value type; `FromSource` is the only event the §5 wave-1
  slice must populate — `DerivedBy`/`GeneratedBy` land per-stage with §4.3's transport
  receipts, each stage adding its own stamp as it adopts ids.

## 5. Consumers and minimal slice (E-10 / seesaw)

- **Consumers (named lanes, real programs):** AFF real-input selection
  (`affected_set_reading_from_git_diff` consuming byte-range edits), WRITE
  (emit-on-corrected-IR locating its output), SYN (ctrl#1499 §7 names this exact gap).
- **Minimal slice:** parse stamps ids + builds the index for one real `.dag` file;
  `TestClaim`s under `src/v2/test/claim/provenance/`:
  **green** — a real byte-range edit resolves to the node it touches, by execution (the
  PROV green criterion verbatim);
  **green** — structural equality of two identical subtrees with different ids still holds
  (the §4.1 equality rule, proven not asserted);
  **red** — position outside all spans ⇒ `NoEnclosingOccurrence`;
  **red** — synthesized node whose chain never reaches source ⇒ `Unanchored` carrying its
  origin chain (§4.5), not a fabricated locus.
- The slice deliberately stops before normalize/resolve transport (§4.3) — that lands
  per-stage with each stage's own carried/derived receipts.

## 6. Open questions — escalate, don't improvise

- **Q-P1 — id allocation scope.** Per-compile (recommended: simplest; AFF/WRITE/SYN are all
  per-compile consumers) vs stable-across-compiles (a correlation problem that
  content+path heuristics should *not* quietly solve — if cross-compile identity becomes a
  real need, it gets its own design).
- **Q-P2 — RESOLVED by the channel authority:** the equality-exclusion rule lives in the
  policy table at [`design-node-identity-channels.md`](design-node-identity-channels.md);
  the two implementing sites (the `Value::eq` authority + the `.dag` zip-fold predicate)
  cite that table, and the build PR cites both. The field-landing moment is also sequenced
  there (occurrence id lands third, only at PROV GO, copying #4581's allocator pattern with
  a distinct id space).
- **Q-P3 — interior-node extents.** Hull-of-children (recommended) vs head-token-only;
  affects narrowest-enclosing tie-breaks. Cheap to change pre-consumer, decide at build.
- **Q-P4 — `ByteRange` Int → branded `ByteOffset`** (A4): ride-along upgrade or separate
  sweep. Recommended ride-along where touched, no big-bang.

## 7. Non-goals

- No span data inside Node children; no String/Nat literal-carrier substrate extension for
  provenance purposes (the T-8 marker's literal-value question is the *lexeme* lane, not
  this one).
- No merge with `NodeArtifactProvenance` or v3 port-provenance (measured verdict, layer 4).
- No cross-compile occurrence correlation (Q-P1's bigger sibling) in wave 1.
- No IDE/LSP surface — consumers are the three named lanes; editor protocols are a later
  projection of the same index.
