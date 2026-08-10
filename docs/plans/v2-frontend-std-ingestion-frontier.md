# The v2 frontend's std-ingestion frontier — measured rows

**Status: measurement receipt, not a plan.** Produced 2026-08-10 from the CI2-0 lane (stopped
the same day), at worktree HEAD `e588031201`, with instruments built from that branch tree.
Nothing here depends on CI2 context: the subject is *which std substrate modules the v2
frontend can ingest, and at which stage the ones it cannot fail*. It bears on the self-host
frontier and on anything that assumes `dag/std/**` + `src/v2/std/**` are ingestible.

## How it was measured

Each module's source text was driven through the canonical v2 frontend fns in sequence —
`v2.compiler.tokenize` `lex_walk_artifact` (rules `dag_lex_rules`) → `v2.compiler.parse`
`parse_module` (grammar `dag_grammar`) → `v2.compiler.normalize` `normalize` — and the typed
rejection reasons read off the resulting `Outcome`, deduplicated per module. A separate probe
read the pre-`well_formed` projection (`normalize_node` + `v2.compiler.namespace_graft`
`namespace_graft_collect_body_edges`) to see which declarations survive the graft.

The subject population is the 15-file import closure of one ordinary witness
(`src/v2/test/claim/manual/cross_tree_constructor_binding_test.dag`), discovered with the
executor's own `discover_source_root_reads_for_entry` — so it is a real closure, not a
hand-picked list. **11 of 15 members reject.** Four accept: `dag/std/primitives.dag`,
`src/v2/std/collection.dag`, `src/v2/std/witness.dag`, and the entry itself.

## The rows

| module | stage | reason |
| --- | --- | --- |
| `dag/std/types.dag` | **lex** | `lex_walk_artifact` Rejected |
| `src/v2/std/node.dag` | **parse** | `parse_module` Rejected |
| `dag/std/content_hash.dag` | **parse** | `parse_module` Rejected |
| `dag/std/algebra.dag` | normalize | `body_lowering_reason_wrapper_retained_emitted` (many) + `normalize_reason_post_normalize_not_well_formed` |
| `src/v2/std/logic.dag` | normalize | same pair |
| `src/v2/std/optional.dag` | normalize | same pair |
| `src/v2/std/diagnostic.dag` | normalize | same pair |
| `dag/std/occurrence_identity.dag` | normalize | same pair |
| `src/v2/std/live_tree.dag` | normalize | `body_lowering_reason_wrapper_retained_emitted` + `namespace_graft_body_dissolved_refused` |
| `dag/std/error_primitives.dag` | normalize | same pair as `live_tree` |
| `src/v2/std/algebra.dag` | normalize | **reason not captured** — this member rejects, but the per-module reason probe was run over 10 of the 11 and this is the one it did not cover |

Three observations, each independently useful:

**The frontier is spread across every frontend stage, not concentrated in one gap.** Lex,
parse, and normalize each reject at least one std module, so "make v2 ingest std" is not a
single capability.

**`content_hash.dag`'s parse wall is attributed**: the `where`-refinement type form
(`type H = String where lower_hex_16`) — an isolated probe of that form alone is Rejected.
DESIGN's hex construction walls ride on that form. `types.dag`'s lex wall and `node.dag`'s
parse wall are **not** attributed; for `types.dag` the gap is narrower than "non-ASCII",
because isolated probes of `§` (U+00A7) and `—` (U+2014) inside string literals both pass.

**The normalize rejections are a self-contradiction, not a missing capability.** Normalize
*retains* wrapper declarations for bodies it cannot lower — the declared
`lowered | wrapper-retained{cause}` frontier, whose contract is "counted, corpus stays green"
— and then its own module-grain `well_formed` gate *rejects the tree those retentions live
in*. The retention arm and the gate cannot both hold on these five modules. Anything relying
on the retention frontier to keep un-lowered bodies non-blocking should verify against these
rows before assuming it holds at module grain.

## Open, deliberately unattributed: behaviour differs by POSITION

Same instruments, same route, opposite outcomes:

- The **entry** module carries all its declarations through the graft *with* wrapper-retained
  grammar-shaped subtrees inside them, and final normalize **accepts** it.
- **Member** modules with same-class retained content **reject** at the same gate.

And within a graft-surviving module, declarations without a lowered (fold-family) anchor
dissolve *individually*: `dag/std/algebra.dag`'s graft projection retains
`boolean_algebra_templates` but loses `list_append` and `FreeMonoid`; `src/v2/std/logic.dag`'s
retains six node-builder fns but loses the `Bool` type declaration. (The *total*-dissolution
case is refused loudly by `namespace_graft_body_dissolved_refused`, landed at `e588031201`;
this partial-drop case is not covered by that guard and was explicitly deferred by its
witness.)

**The discriminating variable between entry-position and member-position treatment is not
named.** Probing stopped by directive, not by resolution. A compiler behaving differently by
position with no named reason will silently decide later questions, so it is recorded as an
open question rather than left as folklore.

## Reproducing

The instruments were scratch host bins (deliberately never committed) that loaded an
interpreted probe harness and called the canonical fns per file. Reproducing costs one small
bin plus a probe module; the fns named above are the whole interface, and the expensive part
is the fixed per-invocation corpus load (~80s), not the per-file work (~2-3s), so any repeat
should batch its files into one invocation.
