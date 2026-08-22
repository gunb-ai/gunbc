# Transparent-alias identity: what the exempt population actually was, and what the relation moved

Companion receipt for the change that adds `SymbolIndex.transparent_alias_rep` to
`v1.compiler.infer_env` and reads it at `v1.compiler.infer` `nominal_call_arg_brand_mismatch`.

## What class of repair this is

A phase-ordering repair, not a type-system project. The governing rule, from an outside review of
the root cause below:

> A judgment that depends on source-level declaration identity must run BEFORE normalization
> erases that identity, or normalization must carry an explicit derived identity FORWARD. It must
> never reconstruct the fact repeatedly from normalized structure.

`brand_grounds_transparently_to` is the specimen: it needs the raw leaf declaration, and
`resolve_item_types` has already replaced it. That is not a bug in the predicate -- it is a
judgment running after the fact it tests was erased, and the repair is to carry the identity
forward rather than re-derive it downstream. Re-deriving from normalized structure at every
comparison is both the forbidden shape and the shape that cost 18x on the earlier attempt; the
correctness argument and the cost argument land on the same design.

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

*(Re-measured after review 54654's tightening -- see the fail-open fix below. The numbers
below are the TIGHTENED predicate's, not the original's.)*

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

### Regen, the workload that historically regressed

The compile above is one entry's import closure. Whole-corpus REGENERATION is a different module
population (src/v1 plus dag, 132 modules planned) and is the workload the earlier 18x incident was
measured on, so the compile result does not transfer to it. One correction to the incident framing
while citing it: regen on this tree today is ~430-460s, not the ~100s of the incident era, so the
two are comparable in wall-clock -- which is why the compile figure fails to cover regen rather
than why it would have been safe to skip it.

Three alternating pairs of `claim_executor --required-regen --source-root dag --source-root src/v2`:

| pair | gen1 (relation absent) | gen2 (relation present) |
|---|---:|---:|
| 1 | 457.4 s | 428.4 s |
| 2 | 442.1 s | 428.1 s |
| 3 | 460.7 s | 428.2 s |
| **mean** | **453.4 s** | **428.2 s** |

**Arm symmetry, observed rather than assumed.** All six runs report the same five phases
completing (frontend, normalize, reconcile, analyses, emit), `planned=132 executed=132`, and BOTH
arms FAIL identically (`generated surface drift: v1_compiler_infer.rs`) with the refusal appearing
only in the final comparison after emit. A symmetric failure is a stronger control than a
symmetric success: neither arm is skipping downstream work the other pays for.

**That table is CONFOUNDED and is kept only as history.** gen1 ran FIRST in every pair, so
run-order was perfectly confounded with arm. The reviewer's arithmetic is why that matters: writing
W for the warming advantage of second position and R for the relation's true cost, the observation
is `W - R = 25s`, which does not bound R -- if W were 60s then R would be 35s, a real 8% regression
producing identical data. The confound can MASK a regression, not merely block a claim of
improvement.

### Crossed order: the measured null

Four pairs with position crossed against arm (relation-present first in pairs 1 and 3,
relation-absent first in pairs 2 and 4), on a COMMITTED tree so no arm could be perturbed mid-run,
all eight runs reporting five phases:

| | relation absent | relation present |
|---|---:|---:|
| runs (s) | 463.6 / 461.0 / 433.6 / 459.6 | 428.7 / 458.6 / 468.8 / 469.6 |
| **mean** | **454.4** | **456.5** |
| within-arm spread | 29.9 | 40.9 |

**Delta +2.0 s (+0.4%), against within-arm spreads of 30-41 s.** The effect is an order of
magnitude below what this host resolves, and now with order crossed it cannot be hidden by
position. This is the null the uncrossed table could not support.

**Two failed measurements on the way, recorded because a receipt that shows only the successful run
is not a receipt.** (1) An earlier crossed attempt was corrupted when the author edited
`src/v1/04_infer.dag` mid-run: runs 6-8 returned 56-62 SECONDS against a ~450s baseline, which
reads as a 7x speedup and was a frontend refusal that never reached emit (a `//` block inside a
function body; DESIGN 4c models module-item grain only). Caught by reading the output file rather
than the duration. (2) The shadow was first armed with `GUNBC_SHADOW_ARG_TSV`; the instrument reads
`GUNBC_SHADOW_ARG_CONFORMANCE`, so it wrote nothing -- and a missing ledger would have read as zero
`WouldDiagnose`, the exact result being hoped for. Both instruments failed TOWARD the desired
answer, which is where a fabricated result enters.

## The fail-open the first version shipped, and what caught it

Review 54654 (`claude-opus-4-7`, REQUEST_CHANGES) reported that `transparent_alias_identity_agrees`
reduced both sides to `qualified_last_segment` UNCONDITIONALLY, so that when neither name had an
entry in `transparent_alias_rep` each representative was the input name and any two homonymous
nominal types from different modules (`mod_a.Foo` vs `mod_b.Foo`) would "agree".

**That specific scenario is NOT reachable, and this note says so rather than banking the credit.**
`nominal_call_arg_brand_mismatch`'s third conjunct is already
`qualified_last_segment(formal_name) != qualified_last_segment(actual_name)`, so a pair whose names
share a last segment never reaches the new conjunct at all. A later review by `smart-ram-730`
raised the same reading and then withdrew it on exactly this ground; the withdrawal is correct, and
the mechanism is verifiable at `v1.compiler.infer` `nominal_call_arg_brand_mismatch`. An earlier
revision of this section asserted the fail-open was live; it was not, and a receipt that overstates
what a review caught is the same fabrication class the receipt exists to prevent.

The tightening below is therefore a HARDENING, not the repair of a live hole: it makes the
predicate self-standing instead of relying on a distant conjunct for its safety, which is worth
having because the distant conjunct is not part of this function's contract and could move.

The fix requires the agreement to be licensed by an ALIAS EDGE rather than by spelling: at least
one side must actually chase through `transparent_alias_rep`, and an unchased pair refuses, since
the caller has already established the names differ. The last-segment reduction survives only for
the chased case, where it is load-bearing -- an alias target is the authored RHS name and may be
bare where the other side is qualified.

**A DIFFERENT fail-open survives the tightening, at representative grain, and it is open.**
`smart-ram-730` located it: the guard excludes pairs whose NAMES share a last segment, but not
pairs whose REPRESENTATIVES do. Given `a.Foo = x.Bar` and `b.Baz = y.Bar` -- two aliases with
distinct names onto two distinct types whose names both end in `Bar` -- conjunct three passes
(`Foo` != `Baz`), both sides chase, both representatives end in `Bar`, and a genuine brand mismatch
is suppressed. The chased-edge requirement does NOT close this: both sides did chase. The
population is not obviously empty -- this run's floor prints
`[floor-bare-name-ambiguity] scopes_affected=961 of 1339 names_total=87040`, so same-last-segment
names are ordinary here -- and it is not measured yet.

**Also open: the enrolled arms cannot discriminate it.** All five witness arms are single-module,
so a change that widened the relation to full name-collapse would keep every one of them green.
That is a coverage gap in this PR's own evidence, named here rather than left for a reader.

**Residual beyond that.** For a census-AMBIGUOUS bare name the reduction can still equate two
distinct declarations -- the open hole DESIGN 4b already names on the source->`.dag` acceptance
path, not one this relation introduces.

**All four arms were re-run against the tightened predicate before merge was requested**, because
"it cannot have moved them" is reasoning, not evidence. They hold: 20527 rows, 19999 `Compatible`,
0 `WouldDiagnose`, 528 unadjudicated -- identical to the pre-tightening measurement; the planted
`String`-at-an-`Int`-formal still refuses as exactly one row with `kernel=true nominal=false`; and
production still emits `0 blocking / 503 advisory`. A result that survives a constraint not in
force when it was first produced is strictly stronger than the original.

## Scope of the diagnostic control, stated because it was nearly overclaimed

`0 blocking / 503 advisory` on both binaries is CLOSURE-SCOPED. It covers the
`src/v2/compiler/03_ingest.dag` import closure and nothing else. The corpus-scoped evidence for
this change is its CI run, not this figure.

The distinction is not hypothetical. gunbc#8879 attempted the same semantics at the resolve seam,
carried six enrolled witnesses green by execution including a discriminating RED and a destruction
control, and its corpus run then reddened 38 diagnostics -- among them
`expected Product(Hash), got Product(Fnv1a64Structural)`, the alias family behind 92 of the 115
relations measured here, located in `dag/gunbc/scm/object_store.dag` and `repository_envelope`.
**Those files are not in the closure measured above.** An author-written fixture set is a sample of
the author's hypothesis; the corpus is the only instrument that has caught anything in this area.

## Two lessons from this lane's own measurement failures

**More runs of a confounded design converge on a confident wrong answer.** Uncrossed said -25s (the
relation faster), two crossed pairs said +9.6s (slower), four crossed pairs say +2.0s. Every one of
those is arithmetically correct on its own data; only the last answers the question, and the
difference is entirely DESIGN rather than precision. Crossing the order changed the sign, not the
error bars.

**An instrument that fails toward your hypothesis is not rare here -- it is the DEFAULT failure
direction**, because the failure modes that produce empty or short output are the ones that look
like success. Three in this session: a shadow armed on the wrong env var (no ledger reads as zero
`WouldDiagnose`), a frontend refusal that never reached emit (56s against a ~450s baseline reads as
a 7x speedup), and a source edit landing mid-run. All three were caught by reading the artifact
rather than the metric, and none by noticing the number looked wrong.

## Corpus residue: adopting a number that is not this probe's

`silent-badger-817` measured what remains when `module_skips_direct_call_arg_check` is actually
deleted: **285 blocking type mismatches against main, 67 with this change merged -- 218 of 285
cleared (76%)**.

This receipt's own "residue zero" is true of the `03_ingest` closure and FALSE of the corpus, where
it is 67. The two numbers had different SUBJECTS, not different values, and leaving both for a
reader to reconcile would have been the mixed-grain failure. Of the 67, 48 are `CoreNode` declared
identically in two modules (`v2.compiler.00_compile` and `v2.compiler.self_host`) -- one alias onto
one target, authored twice -- which a name-keyed relation excludes conservatively. Admitting a bare
name whose declarations are UNANIMOUS is sound and is deliberately left to its own PR: it is a
distinct semantic claim and deserves to be argued on its own rather than bundled here.

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
