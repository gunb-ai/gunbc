# Transparent-alias identity: what the exempt population actually was, and what the relation moved

Companion receipt for the change that adds `SymbolIndex.transparent_alias_rep` to
`v1.compiler.infer_env` and reads it at `v1.compiler.infer` `nominal_call_arg_brand_mismatch`.

## The question this had to answer before anything was written

`v1.compiler.infer` `module_skips_direct_call_arg_check` switches the direct-call argument-TYPE
judgment off for every `v2.*` caller. gunbc#8864 measured what that silence contains — 115
would-be diagnostics at 78 sites on the `src/v2/compiler/03_ingest` closure, all reducing to a
transparent type alias, residue zero — and concluded that deleting the exemption today would
bury real defects under known-false refusals.

What #8864 did not establish is WHY each row is there, and a repair has to be aimed at the
mechanism, not the count. That is what this probe adds.

## Instrument

#8864's report-only shadow, unchanged in its verdict path — every outcome still comes from
calling `direct_call_arg_type_mismatch`, the predicate production calls — plus one extra TSV
column, `why`, populated only on a `WouldDiagnose` row (`why_column.patch`, applies to
`docs/probes/shadow_direct_call_arg_conformance_2026-08-22/shadow_arg_conformance.rs.instrument`).
It records which of the predicate's three disjuncts fired, both directions of
`brand_grounds_transparently_to`, whether each authored name resolves through
`lookup_type_by_name` in the caller's env, and the two authored names.

The instrument stays out of the tree, per the `smart-ram-730` ruling #8864 records: a shadow
judgment merged into the seed is a second authority reading the same facts with no owner for its
dissolution.

Reproduction is #8864's README verbatim, with this patch applied on top of its instrument file,
against these two binaries:

- **gen1** — the tree with the `.dag` relation NOT applied (the committed authority).
- **gen2** — the same tree with the relation applied, its emitted mirrors installed from
  `claim_executor --required-regen`'s candidate tree, and the instrument's one seam hunk
  re-applied afterwards (regen overwrites the mirror the hunk lives in).

Both arms carry the instrument, armed or unarmed identically, so nothing in a comparison between
them is attributable to the instrument's own cost.

## What the `why` column found: one mechanism, not a mixed population

All 115 `WouldDiagnose` rows on the `03_ingest` closure are the same row:

```
nominal=true container=false kernel=false bg_fa=false bg_af=false lookup_formal=true lookup_actual=true
```

Every one fires in `nominal_call_arg_brand_mismatch`, and every one has **both** directions of
`brand_grounds_transparently_to` returning false **even though both authored names resolve** in
the caller's environment. That combination is the whole finding, and it rules out the two
explanations one would otherwise reach for first: the names are not unresolvable, and the guard
that exists precisely to excuse this pair is being asked and is answering no.

The reason it answers no is structural. `brand_grounds_transparently_to` recognises an alias only
when the binding it looks up is still the RAW leaf declaration — `connective == NoConnective`,
zero children, `inferred` present. By the time inference asks this question the binding has been
through `resolve_item_types`, so `type Hash = Fnv1a64Structural` is bound as the RESOLVED
structural node: a `Conj` record wearing the brand name. The leaf test can never hold, so the
guard can never fire, for any alias whose right-hand side is not a bare kernel primitive.

That is why the answer is a precomputed relation over the census rather than more peeling at the
seam. The fact needed — what a declaration aliases — exists only in the raw declaration, which
the census already walks and which the judgment seam no longer has.

The pairs behind the 115 rows:

| formal ← actual | rows |
|---|---:|
| `Fnv1a64Structural` ← `Hash` | 91 |
| `Hash` ← `Fnv1a64Structural` | 1 |
| `Node` ← `ResolvedTree` / `ParseTree` / `CoreNode` / `TargetNodeTree` / `RuntimeTarget` | 16 |
| `ResolvedTree` / `CoreNode` ← each other, and ← `Node` | 7 |

The `Hash`/`Fnv1a64Structural` pair appearing in both directions is a corroborator that does not
depend on any classifier: a genuinely unequal pair does not disagree with itself.

## What the relation moved

Same entry, same root set, same instrument, one binary apart.

| arm | rows | Compatible | WouldDiagnose | Unadjudicated |
|---|---:|---:|---:|---:|
| gen1 (relation absent) | 20527 | 19884 | **115** | 528 |
| gen2 (relation present) | 20527 | 19999 | **0** | 528 |

The denominator is identical, the unadjudicated population is untouched, and `Compatible` grew by
exactly the 115 rows that stopped being `WouldDiagnose`. No row changed category in any other
direction.

## The planted control, without which the table above is unreadable

A blind instrument reports zero on a real defect too. Two arms guard against that.

**Planted, inside the exempt population.** Two functions appended to `src/v2/std/node.dag` — a
`fn planted_take_int(n: Int)` and a caller passing a `String` — then the same run, then reverted.
The gen2 shadow reports exactly one `WouldDiagnose`:

```
v2.std.node | planted_pass_string -> planted_take_int | Primitive(Int) <= Primitive(String) | exempt
```

with production emitting zero blocking diagnostics, which is the exemption doing exactly what it
does. The row is in `would_diagnose_planted_ingest03.tsv`, and its `why` column reads
`kernel=true`, not `nominal=true` — a genuine kernel-type mismatch fires a different disjunct
from the alias class entirely, so the relation is not merely failing to reach it.

**A three-way fixture, judged rather than shadowed.** The same three shapes in one ordinary
(non-exempt) module, so the real production judgment runs and emits real diagnostics:

| call | gen1 | gen2 |
|---|---|---|
| `Handle` actual at a `Payload` formal, `type Handle = Payload` | refused | **accepted** |
| `String` actual at an `Int` formal | refused | refused |
| `IntHandle` actual at a `String` formal, `type IntHandle = Int` | refused | refused |

The third row is the boundary case worth stating on its own: `IntHandle` IS a transparent alias,
so the relation must agree it is `Int` — and must still refuse it where a `String` is declared. A
relation that peeled to "some ground type" and stopped comparing would turn that row green. This
fixture is enrolled permanently as `dag/test/claim/transparent_alias_identity_witness_test.dag`.

## Cost

The bar, declared before the relation was written: **the relation must not be visible above host
noise on a whole-closure compile.** The failure this bar exists to prevent is on the record — a
previous attempt at the same problem called `peel_nominal_alias_identity` per comparison and moved
whole-corpus regeneration from ~100 s to beyond 31 minutes. The design that satisfies it is the
precomputation itself: the chain is chased once per declared name during census construction and
the seam does a map lookup, so the per-comparison work is O(1) and the per-corpus work is one pass
over declarations the census already walks.

Measured by alternating A/B on one host, both binaries carrying the instrument unarmed, same
entry (`src/v2/compiler/03_ingest.dag`), same roots, same output mode:

| pair | gen1 (relation absent) | gen2 (relation present) | delta |
|---|---:|---:|---:|
| 1 | 379.2 s | 388.1 s | +2.3% |
| 2 | 403.2 s | 372.3 s | −7.7% |
| 3 | 393.7 s | 395.7 s | +0.5% |
| **mean** | **392.0 s** | **385.3 s** | **−1.7%** |

The sign flips between pairs and the mean moves the wrong way for a regression, so the honest
reading is not "the relation is free" but **"the relation is smaller than this instrument can
resolve on this host"** — run-to-run spread within one arm (gen1 spans 24 s) exceeds the gap
between arms (6.7 s). Three pairs on a shared, contended container is what could be measured here;
a tighter figure needs a quiet host, and the claim being made is only that nothing on the order of
the earlier 18× regression is present.

Both arms emitted identically throughout — `0 blocking error(s), 503 advisory diagnostic(s)` on
all six runs — so the two binaries were doing the same work, not one of them short-circuiting.

## What is NOT claimed

- **The exemption is not deleted, and this probe does not authorise deleting it.** It establishes
  that the measured obstacle on this closure is gone. The removal is its own change, with its own
  fresh adjudication run.
- **Two closures at one entry is not a corpus board.** This probe re-ran `03_ingest` only, the
  same entry #8864 measured first, and inherits that arm's own stated gap: 528 relations remain
  `RepresentationRelationUnadjudicated` because production itself skips an anonymous record
  literal at an actual position. Neither #8864 nor this change adjudicates that population, and
  the two arms here agree on it exactly, which is the most that can be said.
- **One judgment.** The direct-call SHAPE wall, structured application, constructor sealing,
  return conformance, method calls and field access are all outside the exemption and outside
  this measurement.
- **The record-literal seam is not closed.** gunbc#8865 shows a coproduct payload inhabiting its
  parent coproduct field through a record literal in an ordinary non-`v2` module — not this seam,
  not gated by this exemption. The relation is readable from that seam unchanged, because it is
  keyed on declaration name and lives on the `SymbolIndex` every judgment already carries, but
  reading it there is that lane's change, not this one.
