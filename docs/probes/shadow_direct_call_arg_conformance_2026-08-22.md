# Shadow census of the disabled direct-call ARGUMENT-TYPE judgment over `v2.*` (2026-08-22)

**Session:** `proud-ant-819`. **Work item:** `node://adhoc-77f52796-e22`.
**Brief:** `smart-ram-730` — measure what share of the rustc board is *source conformance
debt*: calls the `.dag` layer should have refused and did not, so the defect became Rust
that a downstream worker is now repairing.

## The adjudicated disposition — LOCKED (deep-ant-102, 2026-08-22)

**Interpretation locked, measurement dated.** The locked wording, to be used as written rather
than paraphrased:

> Zero source-conformance defects over the 97.3% of exempt direct-call relations THE PRODUCTION
> JUDGMENT ADJUDICATES, on the two named entry closures at the named ref. Every relation
> production would diagnose is transparent-alias-equivalent. The remaining 2.7% consists entirely
> of actual-position anonymous record literals that production itself declines to adjudicate, and
> this receipt makes no correctness claim over that population.

**What earned the lock rather than a hold** is a property of the residue, not of the zero: the
2.7% is **homogeneous and its membership arm was EXERCISED**, so it is a *characterized
non-observation* rather than whatever was left after subtraction. Discriminator 4 — a
representation-gap specimen surviving as `RepresentationRelationUnadjudicated` instead of being
counted as a defect — is what establishes that.

### Two grains, and they never substitute for each other

| grain | value | what it answers |
|---|---|---|
| RELATION | 115 on `03_ingest`, 111 on `00_compile`, both fully alias-equivalent | the observations |
| OPERATIONAL | **78 distinct live call sites** | the blast radius of deleting the exemption unchanged |

The two closures **overlap and must never be summed**. An earlier relay of this result said
"roughly 115 false reds"; that wording is **retired** — it silently converted a relation count
into a site count, which is how two true numbers become one false claim. A policy decision takes
the 78; a description of what the judgment would emit takes the 115/111.

### `RepresentationRelationUnadjudicated` is UNDECIDED, not INCAPABLE

Stated because a reader who takes "unadjudicated" to mean "cannot be a defect" has read a word
rather than a fact. The category records that **production's own comparison declines this relation
shape** (an anonymous record literal at an actual position) — it does **not** carry a proof that
such a relation could never become a diagnostic under a checker that did adjudicate it. Nothing in
the instrument establishes structural incapability, and this receipt does not claim it. That is
precisely why the 2.7% is excluded from the zero rather than folded into it.

### Freshness — three separate facts, part of the row rather than a footnote

Because the shadow ships as an instrument file plus a seed-wiring patch rather than entering
production — the correct anti-fork call — this receipt is:

| property | status |
|---|---|
| measurement reproducibility | **yes** — producer, ref and control are all recorded below |
| continuous enforcement | **no** — nothing runs this on any commit |
| automatic freshness | **no** — it goes SILENTLY stale the moment the corpus moves |

The next corpus change does not falsify this receipt; it means the receipt is **no longer evidence
about the new tree**. Re-run it before citing it against any ref other than the one named below.

### On the historical "104 TypeMismatch false positives"

May be cited as **qualitative corroboration** that this result lands in the same representation-gap
family at the same approximate scale. It may **NOT** be differenced against 111, 115 or 78, and it
cannot form a trend, because it records no subject, producer or root set.

### The removal order this receipt implies, stated as its terminal

Make compatibility transparent-alias-aware → run it in observation mode over `v2.*` → prove
alias-equivalent calls admit → prove a planted genuine mismatch refuses → disposition the
unadjudicated population → **then** remove the module-wide exemption. Keeping the exemption is
**today's answer and not the terminal one**: it remains a coarse module-level switch that will
also silence the next genuine mismatch, so "keep it" is a dated policy, not a resolution.

*(Lane note, 2026-08-22: the first step above was attempted at the RESOLVE seam by this session
and refused by CI over the corpus — see guarantee-recovery row 27. It is being taken at the
COMPARISON seam by gunbc#8873, measured to clear this lane's reproduction 5 diagnostics → 1.)*

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
- **This is ONE SEAM, and it is not known how many there are.** A second seam has since been
  measured independently (`gentle-eagle-360`, gunbc#8865): a coproduct PAYLOAD inhabits a field
  declared as its parent COPRODUCT through a RECORD LITERAL — `CppHolder { subject: cpp_inner() }`
  accepted by typing and dying at runtime as `PatternMatchFailure`, against a positive control
  differing only by the wrap — **in an ordinary non-`v2` module**. A record literal in a field
  position is not the direct-call seam, so the exemption neither explains that finding nor would
  deleting the exemption close it. **Read this document's residue-zero as: the DIRECT-CALL seam
  is clean once aliases are made transparent.** It says nothing about seam two, and nobody has
  enumerated whether there is a seam three — the tracking row for that is gunbc#8868, which
  states explicitly that *two* is what has been measured, not the count.
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

## The ceiling row — UNTAKEN, and why nobody should wait for it

A guard-removal arm (exemption rewritten to `false`, whole-tree compile-clean histogram, cold,
two-arm on one tree) would be the **upper bound** on this population. **It was attempted twice
by `gentle-fox-223` and it was not obtained.** This row records that as a failed measurement,
not a pending one — an earlier revision of this document held the row open citing a
forthcoming number, and a row waiting on something that has already failed twice is worse than
a row that says plainly that no upper bound exists.

- **Attempt 1** — the remote dispatch was piped through `tail -5`; the runner streams its whole
  log at the end, so the tail kept the cleanup footer and discarded both arms.
- **Attempt 2, the informative one** — **ARM A, the unmodified baseline, was OOM-killed on the
  runner** (`Killed`, rc 137) after `compile.frontend` and `compile.normalize`. ARM B then hit
  the 45-minute remote cap with no output.

**The honest reading is that the instrument is wrong for this subject, not that the run was
unlucky.** The arm that died carried *no modification at all*, so this is not the flip
surfacing so many diagnostics that the process blew up. `compile_clean_diagnostic_histogram`
carries its own whole-tree-resolve OOM warning, and the whole-tree closure is what was
denominated over — the widest possible subject, chosen to bound a population that lives in
`src/v2`. **The lesson for anyone re-attempting it: do not denominate over the whole tree for
this.** A narrower closure — this document's per-entry shape, or a per-entry sweep — is what
can actually be taken.

Two properties of that instrument are worth keeping on record regardless, because they explain
why its absence costs less than it appears to:

- **It could not have partitioned.** It has exactly two outcomes — a diagnostic appeared or it
  did not — so every representation-gap false positive would have landed in it as though it
  were a defect. It bounds; the adjudication above is what makes a bound mean anything, and the
  adjudication is the half that survived.
- **It is a different unit.** It counts DIAGNOSTICS produced with the guard removed; this
  counts RELATION ROWS classified `WouldDiagnose`. One relation can produce no diagnostic if a
  sibling arm already refused the call, so the two were never going to agree numerically.

## A population NEITHER arm covers, named because nobody owns it

The 521 unadjudicated relations are skipped by *production*, upstream of the guard — so a
guard-removal arm cannot see them either. If hidden source debt exists at the argument seam,
that is where it is, and reaching it needs a third instrument that judges an anonymous record
literal against its formal. No lane currently owns that.

## Conclusion — a refuted hypothesis, and what its refutation buys

This lane was staffed on a specific hypothesis: that a material share of the rustc board on
emitted v2 Rust is **source conformance debt** the `.dag` layer should have refused, and that
some of the board therefore belongs to a different repair at a different layer. **Two
independent measurements killed it.** That is a result, not an absence of one, and it pays
for two questions that were open this morning:

1. **The lanes grinding emitted Rust are working the correct layer.** No part of the board
   reclassifies out from under them. A reclassification would have moved work; its absence
   confirms the current allocation instead of leaving it assumed.
2. **The exemption is hiding exactly the representation-gap class it was introduced for, and
   nothing else** — at the direct-call seam, which is the only seam it gates. Its stated justification was never measured against the live corpus. It is
   now: 115 and 111 candidates, 100% transparent aliases, zero residue. There is no hidden
   population of real defects behind the guard on these closures — which also means the
   argument-type wall, once the alias relation is grounded, can be restored without a
   source-repair campaign in front of it.

Both of those were live open questions. Neither is now, and neither could have been settled
by the guard-removal arm alone, because that instrument cannot tell a false positive from a
defect.

**Read the zero against the locked wording at the top of this document, not on its own.** It is
zero over the 97.3% *the production judgment adjudicates* — not over the population. The excluded
2.7% is characterized, not unknown, and it is `RepresentationRelationUnadjudicated` in the sense
of UNDECIDED rather than INCAPABLE. And the receipt is reproducible but not continuous and not
automatically fresh: it is evidence about one named ref and stops being evidence the moment the
corpus moves.

## Artifacts

- `would_diagnose_ingest03.tsv` — all 115 candidate relations (03_ingest arm), full row shape.
- `would_diagnose_compile00.tsv` — all 111 candidate relations (00_compile arm).
- `shadow_arg_conformance.rs.instrument` + `instrument_seed_wiring.patch` — the instrument,
  carried as a patch rather than merged. A shadow judgment living in the seed would be a
  permanent second authority reading the same facts, with no owner for its dissolution
  (ruling, `smart-ram-730`, 2026-08-22).
- `summarize.py`, `adjudicate.py`, `join_board.py`, `README.md` — the producer and how to
  re-run it, including the behaviour-preservation control to run first every time.
