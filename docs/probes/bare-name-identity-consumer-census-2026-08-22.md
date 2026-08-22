# Census: the v1 Rust emitter's bare-name identity consumers (2026-08-22)

**Session:** `smart-carp-430`. **Work item:** `node://adhoc-a09a3150-b38`.
**Carrier:** `gunbc.bare_name_identity_consumer_census` — the roster is the deliverable and lives
there; this file is the method, so that a reader can re-run the sweep rather than trust the rows.
**Nothing is repaired.** Per the sequencing rule in `gunbc.empty_decl_file_checkpoint_bypass`, the
sweep is a precondition for any refusal, not a follow-up to it.

## The proposition every row is scored against

> The resolved provider identity is the terminal fact. A spelling is an INPUT to resolution, never
> an identity a downstream consumer may reinterpret.

## Method, stated as a selector so its blind spot is visible

1. Every top-level `fn` in `v1.compiler.emit_rust`, `v1.compiler.emit`,
   `v1.compiler.infer_emit_info` and `v1.compiler.coercion` taking a name-shaped `String`
   parameter (`name`, `dag_name`, `type_name`, `tn`, `leaf`).
2. Every call in those modules passing `decl_file: ""` — the literal erasure of identity at a
   callee that accepts it. Ten sites, corpus-wide: two in `coercion`, four in `emit`, four in
   `emit_rust`.
3. For each candidate: read the body, then read every call site, and record whether a declaration
   identity expression (`type_reference_decl_file`, `decl_identity_file`) is in scope there.

**Known blind spot, stated because three rows were found outside the selector.** A name compared
against a literal *inline* — no name-shaped parameter, no `decl_file` argument — is not selected.
`is_host_optional_carrier_type`, `is_host_diagnostics_carrier_type` and `derive_variant_to_enum`
were found while reading call sites for something else. That the selector missed three rows it then
recovered by accident is evidence the selector is incomplete, not evidence that it is complete.
This is why the carrier's next-rung trigger is a lens over the emitter's own `Node` tree.

## The histogram

Every number below is **derived by a fold over the roster**, never stored — see
`census_member_count` and its siblings. Reproduce with `gunbc run --entry
dag/gunbc/bare_name_identity_consumer_census.dag --function <name>`.

| question | answer |
|---|---:|
| distinct bare-key decision sites | 12 |
| call sites across them | 55 |
| sites where the resolved identity **was available** and was not used | 11 |
| sites with an identity-keyed sibling already beside them, unused | 8 |
| sites answering from the wrong provider (or from a detected ambiguity) today | 9 |
| sites whose reach is backed by a **measured** population | 6 |
| **the interrupt cell** — measured reach AND wrong provider today | 5 |

One of the twelve is a **contrast row**, not a defect: `build_qualified_item_registry` keys on the
qualified `module.leaf`, detects a duplicate, and **refuses** through a define-and-consume marker.
It is on the roster because it is the shape every other row is missing.

## What each row is, in one line

| site | keys on | identity there? | today |
|---|---|---|---|
| `lookup_checkpoint` (empty-`decl_file` arm) | bare `dag_name` | upstream, unthreaded | table answers, roster never runs |
| `is_host_text_carrier_type` | `authored_name_at == "String"` | **at the site** | 34 measured E0308 sites |
| `is_host_optional_carrier_type` | `== "Optional"` | **at the site** | second line of 5 of the same 6 renderers |
| `is_host_diagnostics_carrier_type` | `== "Diagnostics"` | **at the site** | third line of the same 5 |
| `rust_opaque_kernel_alias_carrier` | `Json`/`Bytes`/`Symbol` | **one line earlier** | hardcodes `decl_file: ""` internally |
| `is_dag_value_type_name` | field type **name** from the summary | severed upstream | Copy answer without identity |
| `emit_json_value_extract` | authored response-field name | **at the site** | callers compute the node, pass `""` |
| `derive_variant_to_enum` | bare variant spelling | upstream, unthreaded | ambiguity **detected**, then answered |
| `add_emit_item_summary` | bare declaration name, all modules | **at the site** | silent last-write-wins |
| `build_shared_types` | bare `TypeSummary` name | **at the site** | `Rc` wrap decided by spelling |
| `rust_nominal_identity_carrier_type_eligible` | nothing — arg discarded | at the site | **constant `false`** |
| `build_qualified_item_registry` | qualified `module.leaf` | at the site | **refuses** (contrast row) |

## Three findings worth reading before the roster

**The short-circuit family is three predicates, not one.** The handover named
`is_host_text_carrier_type` in six renderers. Those same renderers open with a **three-predicate
preamble** — text, then `Optional`, then `Diagnostics` — each keyed on `authored_name_at`, each
returning before anything identity-keyed runs. Verified by reading the first four lines of each of
the six. The optional and diagnostics rows are filed as `ReachabilityUnmeasured`: the mechanism is
identical to the text row's, and **no second declaration spelled `Optional` or `Diagnostics` was
searched for**, so no divergence is claimed for them. Same mechanism is not the same reach, and the
brief's third condition is exactly that a big number must not recruit a neighbouring site.

**`add_emit_item_summary` is the denominator, and it is bare-keyed across the whole closure.**
`build_emit_graph_info` folds every module's items into one `Map<String, TypeSummary>` keyed on the
authored declaration name, with no collision arm — silent last-write-wins in fold order. Three
other rows (`build_shared_types`, `derive_variant_to_enum`, `is_dag_value_type_name`) are computed
*from* that map, so they inherit its key and cannot be repaired above it. If one row is the
construction move DESIGN §5 points at, it is this one: an identity-keyed summary map would let
three rows dissolve rather than be repaired.

**`derive_variant_to_enum` computes the ambiguity and then answers anyway.** On a second enum
claiming a variant spelling it writes `""` as a sentinel; `rust_qualify_type_leaf_name` reads the
sentinel and emits the **bare** variant name. That is strictly worse than never having looked: the
detection is what makes a refusal one line away, and it is spent on a widen instead. It is the same
shape `build_scope_indexes` has (`ambiguous_bare_function_names` computed, then resolves anyway).

## Two things this census deliberately does not claim

**Inertness is claimed once, and by construction rather than by inspection.**
`rust_nominal_identity_carrier_type_eligible` has the literal `false` as its whole body, so **no
input reaches an answer** — a total-function fact, decidable from the body, and stronger than an
emitted-output diff. It stays on the roster with five live call sites, because the bare eligibility
test guards a branch **ahead of** the identity-keyed grounding call: the first row added to it is a
live member with no further edit. No other row claims inertness; where reach was not measured, the
row says `ReachabilityUnmeasured` and names what would settle it.

**"Present, correct and unreachable" — the middle word is retracted, by its own author.** The
handover for the six-renderer row carried that phrase. `royal-dove-436` retracted `correct` before
this census was written: `type_realization_decision` takes `decl_file` as a parameter, and its
production key `type_reference_decl_file` falls back to **the file the reference sits in** whenever
inference did not resolve there — a location, not a declaration. So routing the six renderers to the
authority changes *which function computes the answer* and leaves *the basis of the answer*
unchanged. The carrier records this because the sibling column would otherwise read as a routing
work list, and for that row routing alone is measured to be insufficient. (The measurement that
produced the retraction ran on the pre-reconcile tree and its *count* was withdrawn; what survives
unconditionally, and what the paragraph above rests on, is the shape defect — the helper cannot
distinguish *this node is its own declaration* from *this reference was not resolved here*.)

## Rung

The **class** is *mitigatable*: nothing refuses at any row but the contrast row, and a census does
not change that. The **carrier** is lower than a reader might assume — its rows are hand-swept from
source text, so an emitter edit adding a bare-name decision moves no number here and nothing detects
the omission. What the carrier does hold by construction: a row cannot claim identity was available
without naming the expression that holds it, cannot claim inertness without carrying the constant
body, and cannot omit reachability. **Next-rung trigger:** the roster is derived by a lens over the
emitter's own `Node` tree instead of swept by hand.

## The witnesses' RED was run, not claimed

Every assertion in `bare_name_identity_consumer_census_test` is about the file beside it, which is
exactly the shape that greens for free. So the REDs were executed against a green control, on a
binary built from this tree:

| | control | mutant |
|---|---|---|
| `sibling_claim_has_an_identity_to_feed_it` | `true` | **`false`** — sibling claimed on the one row whose identity is `UnavailableNoNodeAtSite` |
| `member_has_at_least_one_call_site` | `true` | **`false`** — one row's `call_site_count` set to `0` |
| `derived_counts_bounded_by_roster` | `true` | `true` — **does not discriminate** under the zero mutant |

The third row is reported because it is the interesting one: that assertion is not violated by a
zero, which is correct and is why it is the weakest guard in the file rather than a third one. The
mutants were applied on the runner and reverted there; the tree was confirmed byte-identical
afterwards.

The six derived counts in the histogram above were read off the same binary
(`12 / 55 / 11 / 8 / 9 / 6`), so the table is a transcription of an execution, not of a reading.

## A production specimen for the `derive_variant_to_enum` row — and the correction it took

`crisp-crab-430` (namespace-cut lane) supplied one. This section was rewritten twice as the account
was falsified; what follows is the settled version, with the retractions kept because a reader who
finds only the conclusion will re-derive the errors.

**What holds, and it is a corpus property.** `Connective` is declared twice — `v1.compiler.core`
(`Conj | Disj | NoConnective | Arrow`) and `v2.std.node`
(`Atom | Conj | Disj | Arrow | Cardinality | Instantiation`) — and **both declare `Conj` and
`Disj`**. *If the two are in one pool*, they collapse to one entry in the bare-keyed map before
`derive_variant_to_enum` runs, the fold sees a single `Connective`, and the sentinel it exists to
write is never written. The ambiguity wall cannot fire because the collision was destroyed one layer
up, in the `add_emit_item_summary` row.

**Retracted: that the collapse is realized at this HEAD.** No standard invocation on main puts the
two in one pool — regen resolves `src/v1` + `dag` (`cli_run.rs` `regen_source_roots`), the required
floor resolves `dag` + `src/v2`. Neither holds both.

**Retracted: the suppression account.** This document previously argued that imports keep main quiet
by populating `already_imported_names`. That reading of the filter is correct and is **not** the
reason: the falsifier this receipt offered — *find one affected mirror whose name is not in its main
import list* — was run, and **all 26 are that one**. The decisive evidence is a positive control:
main emits `use crate::v1_std_core::Connective::{Arrow, Conj, Disj, NoConnective}`, so the synthesis
path runs on main and **answers correctly**. Nothing suppressed it; only one `Connective` was in the
pool.

**The settled account is co-residency**, and on the namespace-cut branch it was caused by `src/v2`
being added to `regen_source_roots` — which this tree records as a standing invariant in the other
direction (*"src/v2 is not a regen root and never will be, because stage0 IS the v1 seed and a seed
that reached into src/v2 would depend on the successor it bootstraps toward"*).

**That branch then found its root cause, and it is worth recording here because it prices the whole
class.** The root list was step 3 of a cascade: four references in `src/v1/05_emit_rust.dag` were
qualified to `v2.std.node` by a bulk qualification pass, when `v1.std.core` declares those same names
(`Arrow`, `Bind`) and main spells all four **bare**. Those four pulled `v2.std.node` into the seed
closure; regen then refused with `undefined variable 'v2'`; the root was added to make the refusal go
away; that made the two declarations co-resident; the bare-keyed registry answered last-write-wins;
26 mirrors got a wrong `pub use`; **10,334 of 10,821 errors (95.5%) sit in those 26 files.** The
refusal at step 3 was the fail-closed boundary working, read as an obstacle — DESIGN §5's
author-side absorbing fallback, arrived at honestly.

**What that does and does not say about the emitter.** It does *not* say the emitter is fine: the
bare-keyed registry answers last-write-wins with no way for a caller to distinguish that from a
unique answer, and it detonates for anyone who puts a homonym in a pool **by any route**. What the
episode measures is *how cheaply that condition is met* — one element appended to a `Vec`, by an
author who believed he was fixing a resolution error. That is a reach fact about this whole census,
not only about that row.

**Reach therefore rests on the collision half, which is unaffected.** `Refused` (three coproducts)
and `Unknown` (three more) are co-resident in `dag` + `src/v2`, the pool the floor resolves on every
PR, so the sentinel *is* written and *is* read back as a bare name there.

**The corollary is larger than the specimen.** Whether this fold sees one entry or two is a function
of the **invocation**, not of the module graph — so every identity answer downstream of the map is
invocation-relative. That is the finding worth carrying out of this exchange, and it belongs to
`crisp-bat-769`, who reached it from the pool side.

**One negative result carried from that lane, because it prevents a wrong generalisation:**
`NamedEdgeTargetLookup` and `Edge` in the same use-line are **not** explained by this mechanism —
`Absent` is declared by six modules so the wall genuinely fires for it, and `Edge` is a record that
cannot reach the variant arm at all. One emitted use-line, at least two distinct causes. The census
claims this row for `Connective` only.

**And a refinement to the row's repair note, from the same lane:** re-keying the map does not by
itself deliver the refusal, because `derive_variant_to_enum` **scans** the whole map rather than
looking one entry up. A better key hands it two entries where it had one; it still has to be taught
that two entries sharing a spelling is the refusing case. A scan is not repaired by a key.

## A third instance, from the same branch, landing on two rows

`crisp-crab-430` carried their branch 10,821 → 1,801 → 395 errors. The second fix is this census's
`build_shared_types` row, measured far more sharply than the row had it: of **21 `shared_types`
membership tests, only 7** passed an already-leaf-reduced expression. A dotted spelling missed the
set and rendered **unwrapped**, while a leaf-reduced site rendered the same type **wrapped** — the
emitted tree disagreeing with itself about one type *in both directions*. `source_indices` went
343 wrapped / 371 unwrapped → **714 / 0**, the shape main already holds, which is the discriminator
rather than the raw drop.

**The row's `call_site_count` stays at 2 and is deliberately not 21.** That column counts calls to
the *deciding function* across the whole roster; 21 counts the membership tests *consuming* its
output. Both are true here and only one is what the column means — re-defining it for one row would
silently break every comparison in the histogram. The 21 lives in the row's measurement text.

**And a second way `lookup_checkpoint`'s key misses**, added to that row: the comparison against
`cp.dag_name` is *exact*, so a qualified spelling cannot match a bare-keyed row at all — the table is
**missed**, not mis-answered. That lane completed the leaf reduction that closes it, **measured it,
found it inert on their corpus (no emitted change outside its own mirror), and reverted it** rather
than land an inert change inside a "1,801 → 395" narrative. Recorded here as a *measured-inert
repair* rather than as an open item, precisely so the next reader who notices the exact comparison
does not re-derive and re-land it.

**The `shared_types` result now carries a same-binary control**, which it did not when first
relayed. The concern was real and that lane raised it against itself: their bootstrap host predates
the identity machinery by thirteen commits on exactly `coercion.dag` and `05_emit_rust.dag`, so "an
older, more main-like emitter produced the consistency" was a live alternative explanation for the
714/0. Two arms, same binary vintage, same tree, same roots, one difference:

| arm | wrapped / unwrapped |
|---|---|
| bootstrap binary **with** the leaf-reduction | 714 / 0 |
| same binary **without** it (control) | 343 / 371 |

The control reproduces the original split exactly, so the old emitter alone does not produce main's
shape and the leaf-reduction is what closes it. The row's second measurement is controlled.

**And the `Nat` mechanism I recorded here an hour ago is RETRACTED by its author, before anyone built
on it.** I wrote it as two live hypotheses — identity-unknown versus identity-known-and-refused,
identical output, opposite repairs. That framing is wrong in a way that matters: the measurement was
taken on a tree emitted by a binary that **lacks `type_reference_decl_file` entirely**, so the
instrument cannot exhibit the mechanism it was offered as evidence for, and neither hypothesis is
supported by it. What survives is only the **observation** — 119 errors, `Measure<Q, S, Nat>` where
main emits `Measure<Q, S, i64>` — which is exactly what an emitter with no `decl_file` threading
produces whether or not a resolution defect exists. The four-cell classification is still the right
instrument and **cannot be run on that host**, because the function it would instrument is not there;
it is gated on that tree compiling, not on anyone's willingness. This is the instrument-vintage trap,
and it is recorded here rather than quietly deleted because I had already written the superseded
version into this receipt.
