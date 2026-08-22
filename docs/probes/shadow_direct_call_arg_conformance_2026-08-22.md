# Shadow census of the disabled direct-call ARGUMENT-TYPE judgment over `v2.*` (2026-08-22)

**Session:** `proud-ant-819`. **Work item:** `node://adhoc-77f52796-e22`.
**Brief:** `smart-ram-730` — measure what share of the rustc board is *source conformance
debt*: calls the `.dag` layer should have refused and did not, so the defect became Rust
that a downstream worker is now repairing.

## The mechanism, scoped narrowly

`v1.compiler.infer` gates ONE judgment on the caller's module name:

```
let arg_compat_diags = if module_skips_direct_call_arg_check(module_name: scope.module_name) { [] }
                       else { direct_call_arg_mismatch_diags(...) }
```

and `module_skips_direct_call_arg_check` is now a bare three-character prefix test on
`"v2."` (the `v1.compiler.*` arm was deleted as dead — a thirteen-character slice compared
against a twelve-character literal, so it never returned true).

**What this does NOT mean.** The exemption reaches the direct-call ARGUMENT-TYPE judgment
and nothing else. In the same match arm `direct_call_structured_application_mismatch_diags`
runs unconditionally, and the direct-call SHAPE wall (unknown label, positional
surplus/deficit, duplicate binding) runs over every module including the compiler's own —
`direct_call_shape_wall_note` states explicitly why the exemption's reason does not reach
labels. Constructor sealing, record-literal checking, return conformance, method calls and
field access are all outside it. Any reading of this document as "v2 has no type checking"
is false.

## Instrument

`src/v1/stage0/src/shadow_arg_conformance.rs` — a REPORT-ONLY shadow. It does not
reimplement compatibility: every verdict comes from calling
`v1_compiler_infer::direct_call_arg_type_mismatch`, the same predicate production calls.
What it re-walks is only the per-formal plan production also walks, and the two control-flow
arms production takes before reaching the predicate are mirrored explicitly and LABELLED.

It changes no compilation or emission behaviour: rows go to a sidecar written once at the
compile boundary, so the diagnostics total the canary board is read beside does not move
(this was the deciding constraint — an advisory-diagnostic transport would have perturbed
one of the board's own reported columns).

It fails toward `ComparisonUnavailable`, never toward silence: every formal of every direct
call reaching the seam produces exactly one row.

Outcome vocabulary, deliberately conservative — `WouldDiagnose` is a CANDIDATE source
conformance defect and is not promoted to a defect here:

```
Compatible | WouldDiagnose | ComparisonUnavailable { cause } | RepresentationRelationUnadjudicated { cause }
```

## Subject, ref, producer

| | |
|---|---|
| subject | `src/v2/compiler/03_ingest.dag` closure — the canary the E0308 partition was taken on |
| roots | `dag`, `src/v2`, `src/v1`, `--dependency-pool-index primary-precedence` |
| ref | `90986d19469397098ddaa799dfc9e9087541cbf4` |
| producer | local arm64 `gunbc` built from that tree, `--target rust`, sidecar via `GUNBC_SHADOW_ARG_CONFORMANCE` |
| wall | 6m18s |
| M | **1**. Every figure here is "at M=1, 03_ingest" |

## Behaviour preservation, by control rather than assertion

The armed run reports **177 files emitted, 503 diagnostics, 0 blocking** — the same counts as
the unarmed baseline taken on the same ref *before the instrument existed*. The board's own
diagnostics column does not move, which was the deciding constraint on the transport.

## Population

20,526 relation rows, one per actual/formal pair reaching the direct-call seam.

| | rows | Compatible | WouldDiagnose | ComparisonUnavailable | Unadjudicated |
|---|---:|---:|---:|---:|---:|
| **exempt** (`v2.*` callers, 94 modules) | 19,426 | 18,790 | **115** | 0 | 521 |
| **judged** (non-exempt, 37 modules) | 1,100 | 1,093 | 0 | 0 | 7 |

The 115 sit at **78 distinct call sites** across 9 `v2.*` modules.

## The answer: zero

All 115 `WouldDiagnose` relations were adjudicated **mechanically** against the corpus's own
transparent type aliases (`type A = B`, 88 read from `src/v2` + `dag` + `src/v1`). A relation
counts as a representation gap exactly when the formal's and the actual's type names reduce to
the same base. **115 of 115 reduce. Residue: 0.**

| reduced base | relations | the aliases involved |
|---|---:|---|
| `Fnv1a64Structural` | 92 | `src/v2/std/node.dag` — `type Hash = Fnv1a64Structural` |
| `Node` | 23 | `type ParseTree = Node`, `type ResolvedTree = Node`, `type CoreNode = Node`, `type TargetNodeTree = Node`, `type RuntimeTarget = Node` |

So on this canary the exemption is silencing **exactly the representation-gap class it was
introduced for, and not one source conformance defect**. The share of the rustc board that is
source conformance debt at this seam is **0 of 235 E0308 sites, 0 of the 509-error coded board**.

**Stated at the headline rather than in a footnote, because a bare "0%" reads stronger than
what was measured: zero source conformance debt at this seam, over the 97.3% of exempt
relations this instrument can see.** The other 2.7% — 521 relations — are unadjudicated
because *production itself* skips an anonymous record literal at an actual position and the
shadow preserves that skip rather than inventing a judgement. Source debt could hide there.

**Denominator warning — this board is not the canary series and the two must not be
differenced.** The curated-probe series (`docs/probes/curated_cargo_probe_one.sh`,
`scratchpad/boards/sites_629252b6df.tsv`) reports 339 coded rows at ref `629252b6df` and 336 at
this document's ref, on a *different root set*. Both the ref and the closure differ from the run
above, so "339 → 509" is a splice of two instruments, not a movement. Report against the header
of whichever run you are reading.

**A corroborating tell independent of the classifier:** the `Fnv1a64Structural`/`Hash` pair
appears in **both directions** — 91 relations with `Fnv1a64Structural` formal and `Hash`
actual, 1 with the reverse. A genuinely unequal pair does not disagree with itself; a carrier
whose identity is decided by a transform applied at some positions and not others does.

## The join to the board, at the only key both instruments honestly share

The board's per-site key is `(file, line, col)` in **emitted Rust**; the shadow's is
`(caller module, caller decl, callee, formal index)` in **`.dag` source**. There is no line
correspondence and manufacturing one would be the join-by-generated-line-number failure. The
emitted FILE is a pure function of the caller module, so file grain is the strongest shared
key. At that grain:

- 5 emitted files carry both a board site and a shadow candidate (39 of 235 board sites);
- **196 of 235 board sites are in files with no shadow candidate at all** and therefore cannot
  be source conformance debt at this seam under any join;
- and since every shadow candidate is adjudicated a representation gap, the intersection
  contributes **0** as well.

The file-grain join is reported because it bounds the answer from a second direction, not
because it was needed: the adjudication already returns 0.

## Discriminators

| # | what it proves | result |
|---|---|---|
| 1 | shadow and production agree exactly on a NON-exempt call | the same source in `v1x.probe.shadow_control` yields production `type mismatch: expected 'Primitive(String)', got 'Primitive(Int)'` (blocking) and shadow `WouldDiagnose` at the same relation |
| 2 | a valid `v2.*` call returns `Compatible` | `control_valid` → `Compatible` |
| 3 | a PLANTED `v2.*` mismatch is located | `v2.probe.shadow_planted` → 2 `WouldDiagnose` rows (`String <- Int`, `Int <- String`) while production emits **0 diagnostics** — the exemption's silence and the shadow's sight, in one run |
| 4 | a representation-gap specimen is preserved, never counted as a source bug | 521 exempt relations carry `RepresentationRelationUnadjudicated{anonymous_record_literal_at_actual_position}`; and the 115 candidates are adjudicated gaps rather than promoted |

Discriminator 3 is the one that makes the zero readable: an instrument that returns 0 because
it is blind returns 0 on a planted defect too. This one does not.

## What this does NOT establish

- **Two entries, both M=1.** 03_ingest's and 00_compile's closures. These are overlapping
  closures, not a partition, so their counts may not be summed — and neither is a corpus
  board.
- **A coverage ceiling, stated rather than glossed.** 521 exempt relations (2.7%) are
  unadjudicated because *production itself* skips an anonymous record literal at an actual
  position; the shadow preserves that skip rather than inventing a judgement. Source debt could
  hide there and this measurement cannot see it.
- **Nothing about the method/pipe seam**, constructor sealing, record-literal checking, return
  conformance, or field access. The exemption does not reach them and neither does this census.
- **`WouldDiagnose` is not promoted to `SourceDefect` anywhere in this document.** On this
  canary the promotion would have been wrong 115 times out of 115.
- **Not comparable, by unit, to a guard-removal count.** A flip-off arm counts DIAGNOSTICS
  produced when the exemption is deleted; this counts RELATION ROWS classified `WouldDiagnose`.
  One diagnostic can arise from one relation, and one relation can produce none if a sibling arm
  already refused the call. Expect the same order of magnitude, never agreement; a wide
  divergence is first a unit question, not a finding.

## The consequence for the exemption

The roadmap node `compiler-source-exemption-removal` names, as its first slice, "the fresh
adjudication run with its failure classification, before any relation change". This is that
run, and it says the relation change comes first: deleting
`module_skips_direct_call_arg_check` **today** would red 78 live call sites with 115 false
refusals, every one of them a transparent alias the compatibility relation declines to peel
across a module boundary. Ground the alias-identity relation and the exemption's stated reason
dissolves on its own; delete it before that and the corpus pays for a wall that is wrong.

It also **adjudicates the roadmap's unlocated "104 TypeMismatch false positives"**: the claim
is now located and corroborated at the same order of magnitude — 115 relations at 78 sites,
100% false positive — with the caveat that the historical claim recorded no subject, so this
is a same-order match rather than a reproduction.

## Second arm — a different entry, published beside the first and never merged into it

| | |
|---|---|
| subject | `src/v2/compiler/00_compile.dag` closure, same roots, same ref, same binary |
| emitted | 175 files, 503 diagnostics, 0 blocking |
| wall | 5m59s |

**This is a DIFFERENT closure, not a wider one, and the numbers say so rather than the
adjective:** 175 emitted files against 177, 19,251 exempt relations against 19,426. It
overlaps 03_ingest heavily and is not a superset — `v2.compiler.ingest`'s 4 candidates are
absent here. Treat it as a second sample, never as a bound on the first.

| | rows | Compatible | WouldDiagnose | ComparisonUnavailable | Unadjudicated |
|---|---:|---:|---:|---:|---:|
| exempt | 19,251 | 18,620 | **111** | 0 | 520 |
| judged | 1,100 | 1,093 | 0 | 0 | 7 |

**111 of 111 reduce to one base. Residue: 0.** 92 to `Fnv1a64Structural`, 19 to `Node` — the
same two families, in the same eight modules, led by `v2.compiler.eval` at 79. Two entries,
two independent adjudications, zero residue between them.

The `judged` column is byte-identical across both arms (1,100 / 1,093 / 0 / 7), which is the
expected signature of a shared non-`v2.*` sub-closure and a second, incidental check that the
instrument is deterministic across runs.

## The ceiling row — held open, deliberately unfilled

A guard-removal arm (exemption rewritten to `false`, whole-tree compile-clean histogram,
cold, two-arm on one tree) is the **upper bound** on this population and is owned by
`gentle-fox-223`. **As of this document it has not landed**, so the row cites a pending
measurement rather than a number of mine — filling it with anything derived here would be
the fabricated-plausible-output failure, since a ceiling I compute myself is not a ceiling.

Two things are known about it in advance and both are recorded so the eventual number is
read rather than inherited:

- **It cannot partition.** That instrument has exactly two outcomes — a diagnostic appeared
  or it did not — so every representation-gap false positive lands in it as though it were a
  defect. It bounds; the adjudication above is what makes a bound mean anything.
- **It is a different unit.** It counts DIAGNOSTICS produced with the guard removed; this
  counts RELATION ROWS classified `WouldDiagnose`. One relation can produce no diagnostic if
  a sibling arm already refused the call. Same order of magnitude is the agreement to look
  for; a wide divergence is first a unit question, not a finding.

The falsifiable prediction sent to that lane before its run finished: its delta on this
closure should be dominated by the same two alias families and led by `v2.compiler.eval`.
A family in the delta that is **not** an alias pair would be the first evidence this
document's zero is closure-dependent.

## A population NEITHER arm covers, named because nobody owns it

The 521 unadjudicated relations are skipped by *production*, upstream of the guard — so a
guard-removal arm cannot see them either. If hidden source debt exists at the argument seam,
that is where it is, and reaching it needs a third instrument that judges an anonymous record
literal against its formal. No lane currently owns that.

## Artifacts

- `would_diagnose_ingest03.tsv` — all 115 candidate relations (03_ingest arm), full row shape.
- `would_diagnose_compile00.tsv` — all 111 candidate relations (00_compile arm).
- `shadow_arg_conformance.rs.instrument` + `instrument_seed_wiring.patch` — the instrument,
  carried as a patch rather than merged. A shadow judgment living in the seed would be a
  permanent second authority reading the same facts, with no owner for its dissolution
  (ruling, `smart-ram-730`, 2026-08-22).
- `summarize.py`, `adjudicate.py`, `join_board.py`, `README.md` — the producer and how to
  re-run it, including the behaviour-preservation control to run first every time.
