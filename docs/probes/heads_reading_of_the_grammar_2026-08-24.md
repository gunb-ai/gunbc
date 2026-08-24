# The pool census reads declaration heads instead of discarding bodies (2026-08-24)

**Subject:** the residue `docs/probes/edge_index_tree_census_attribution_2026-08-24.md` left open —

> `parse_with_table` builds full function bodies; `census_heads_module_node` strips them 82ms
> later. Half the corpus parse is work the only consumer throws away. […] It is this class's
> next-rung trigger, it is owed under §6, and it wants its own PR — not its own excuse.

This is that PR. The attribution probe measured the defect and deliberately shipped no repair;
this change is the repair and carries its own receipt.

## What changed, stated as a reading rather than a mode

`v1.compiler.parse` gains a **declaration-heads reading of the same grammar**. `ParseContext`
carries `heads_only`; `parse_heads_with_table` is the entry that sets it. Every item head is
parsed by the same productions as before. The one difference is that a brace-delimited body of
an `fn`-shaped item is **skipped at token grain** — depth-counted over brace SHAPES, which the
tokenizer has already decided — instead of being built into a tree, and the body slot receives
the loud `CensusHeadsBodyStripped` stand-in.

Two construction choices are what make this a reading rather than a second parser, and both are
load-bearing:

- **One spelling for the body.** All four `fn`-shaped item body sites (`fn`, `func`, `pattern`,
  `interface`) route through a single `parse_item_block_body`, which selects the reading the
  context declares. The two readings cannot drift apart per call site because no call site
  chooses.
- **The census strip is KEPT, not folded into the parser.** `census_heads_module_node` still
  runs, on the heads reading's output, exactly as it ran on the full reading's. That normalizes
  the body slot — the one slot the heads reading deliberately fills differently — on *both*
  sides. The consequence is that the stand-in's exact shape is not a fact any consumer can
  depend on, and the heads reading cannot drift from the full reading through the body slot at
  all. It can only drift through the *heads*, which is precisely the surface the differential
  below measures. Deleting the strip as "now redundant" would have removed the mechanism that
  makes the two readings agree by construction and replaced it with a promise.

## Measured

Local arm64 session container, release build, roots `dag` + `src/v2`, one process,
`claim_executor --heads-reading-differential`.

| | |
|---|---|
| modules compared | **3880** |
| `divergent` (heads ≠ full, after the census strip on both) | **0** |
| `regressed` (heads refuses what full accepts) | **0** |
| `narrowed` (full refuses on body grammar, heads does not) | **0** |
| `both_refused` | 0 |

Three green runs, and the spread between them is the point rather than noise to average away:

| run | full reading | heads reading | ratio |
|---|---|---|---|
| 1 | 12175 ms | 5885 ms | 2.07x |
| 2 | 11030 ms | 5559 ms | 1.98x |
| 3 | 12882 ms | 6382 ms | 2.02x |

This is a shared container under other sessions' load, so the ABSOLUTE walls move ~15% run to
run. The RATIO does not, because both readings in a run pay the same contention over the same
modules — which is exactly what taking them in one process buys. **Quote the ratio.** The
milliseconds are evidence that it was measured, not a figure to plan against, and a single-run
absolute would have been the more confident and less true number.

**Why the two timings are comparable and a before/after pair would not be.** They are taken in
ONE process, on ONE machine, over the SAME module list, alternating per module. A before/after
figure from two separately-built binaries would have to argue away the build, the host and the
corpus state; this one has nothing to argue away.

**What the timing is NOT.** It is the PARSE term only. `tokenize` (4.16s in the attribution
probe), `build_newline_index` (0.93s) and the per-file intern-table setup sit outside both
timers and are untouched by this change, so the ~2x is a saving on the parse row, not on the
`pool_parse` row, and quoting it as the latter would be inflation. The attribution probe's
figures were taken on a remote amd64 runner and are not the same denominator as these; the two
sets are not subtracted from each other anywhere in this document.

## The RED control — the instrument can fail, measured

A green differential is worth nothing unless it can go red. One token was mutated in the
emitted mirror only (`depth + 1` → `depth + 0` in `heads_skip_block_tokens`, so a nested `{`
stops deepening the count and the skip ends at the first inner `}`), rebuilt, and re-run:

```
heads-reading-differential: compared=3880 divergent=0 narrowed=0 regressed=2820 both_refused=0
heads-reading-differential: REGRESSED dag.test.claim.witness_deferral_freeze_witness
heads-reading-differential: REGRESSED examples.cost_estimate
…
```

**2820 of 3880 modules go red on a one-character mutation**, and the command exits 1. The
mutation was then reverted by re-copying the regen candidate, rebuilt, and the differential
re-taken green (`compared=3880 divergent=0 narrowed=0 regressed=0`, exit 0) — the mutated
mirror is not in the diff, and the restored tree is the one measured, not merely assumed
restored. The mutated run's own timings are not quoted anywhere: it bails out of most bodies
early, so its numbers measure the bug rather than the reading.

The mutation landing in `regressed` rather than `divergent` is itself informative and is the
reason both columns exist. An early-ending skip leaves the token stream mid-body, so the next
item fails to parse and the heads reading *refuses* — it does not quietly return a shorter head
list. That is the fail-closed behaviour one wants, but it is not something to assume: the two
columns are separate so a future mutation that DOES produce a silently shorter head list has
somewhere to land.

## The self-host fixed point holds

The parser is `.dag` authority with an emitted stage0 mirror, so a parser change that emits a
mirror the changed compiler would not re-emit is not a change to the parser — it is a fork.
`claim_executor --required-regen` was run after the mirror was installed:

```
required-regen: elapsed_ms=583524 first_generation_equal=true planned=134 executed=134 declared_divergent=1 [main.rs]
exit=0
```

`first_generation_equal=true` over all 134 planned outputs. The one declared divergence
(`main.rs`) is pre-existing and unrelated. So the compiler built from this mirror re-emits this
mirror: the heads reading is at the self-host fixed point, not merely compiling.

## The one refusal this reading does not make — declared, and countable

The heads reading refuses an unterminated body and every malformed item HEAD, exactly as the
full reading does. It does **not** refuse a well-braced but ungrammatical body, because those
tokens are never handed to the expression grammar.

**Why this is a scope correction rather than a hole.** The required run's `.dag` parse sweep
full-parses `src/v1`, `dag` and `src/v2` and owns exactly the question *is the corpus
grammatical*. The pool census answering it a second time — as a side effect of building
declaration heads — was one fact with two authorities (§3), and it was the expensive authority:
it coupled every pool-derived resolve in the process to the grammaticality of every body in the
corpus, and paid a whole-corpus body parse to do it.

**What is honestly given up.** For source roots the parse sweep does not cover, that refusal is
no longer taken anywhere. Today the sweep's roots are a superset of the pool roots the required
run uses, so the live population is empty; an invocation naming some other root is not covered
by either authority. The measurement above is what makes this *observable* rather than asserted:
`narrowed` is its own column, counted per run, currently `0` — DESIGN §5 requires a degradation
to be counted rather than absorbed, and a column reading zero today is how a future nonzero
becomes visible instead of silent.

## Rung

The class is *a parse whose only consumer discards half its output*. It sat at **mitigatable** —
nothing was wrong, it was merely paid — and this change does not move it up a safety rung,
because it is a COST repair and saying otherwise would be the inflation §4b(1) names.

What the change does carry a rung for is the new surface it introduces: *the heads reading
agrees with the full reading*. That sits at **mechanically preventable** — the differential is a
real, re-runnable wall with a discriminating RED and a positive control, and it is
`claim_executor --heads-reading-differential`, not a number quoted from a run nobody can repeat.
It is **not** *structurally guaranteed*, and the honest reason is that the agreement is enforced
by a check rather than by construction on the heads: the census strip makes the BODY slot
unable to disagree, but nothing structurally prevents a future edit to the skip from consuming
one token too many.

**It is not enrolled in the required run, and this is a deliberate scope statement, not an
oversight.** Reading 3880 modules twice is precisely the cost the heads reading exists to
remove; paying it on every push would spend more than the repair saves. Its **next-rung
trigger** is a form of the check whose cost is proportional to the diff rather than the corpus —
the same shape the affected-set work wants generally — at which point enrolling it becomes
cheap enough to be ordinary.

## What is NOT claimed

- **No claim about `pool_parse`'s whole row.** This measures the parse term, in isolation, on a
  different machine and architecture from the attribution probe. The end-to-end effect on a
  required-floor run is not measured here and no figure for it appears in this document.
- **No claim that the corpus contains a body the full reading refuses.** `narrowed=0` says the
  opposite: every module in `dag` + `src/v2` parses clean both ways today. The declared
  narrowing is about what WOULD happen, and it is carried as a counted column rather than as a
  prediction.
- **`src/v1` was not in the differential's roots.** The roots measured are `dag` + `src/v2`,
  which are the roots the required run gives the pool. A run over other roots is a different
  denominator and would need its own receipt.
