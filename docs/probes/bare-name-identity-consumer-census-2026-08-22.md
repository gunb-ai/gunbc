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
