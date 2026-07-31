# Whole-corpus namespace census receipt — 2026-07-31

This directory makes the namespace measurement reproducible without committing its
1.9 MB raw log or the synthetic compile root. Every reported number is labelled by
its actual authority; the regex catalogue is never presented as compiler resolution.

## Pinned inputs

- Corpus: `0337fb27c039a800a1aff4b80140d6dbf027e595` (main at measurement time).
- Compiler: release build from PR 7427 commit
  `b222f079a0425a4015718175dd7f41a55b76b759`, canonical binding enabled.
- Compiler binary identity: 21,563,472 bytes, SHA-256
  `87538a75c2b56111f9ab1440336b858ee8618cd7892758f9c21c5af0655a68b6`.
- Raw stderr identity: SHA-256
  `06289db522ff4cbf1613d07219e6241fe1d92994710e2fc871ad82c3de19823f`.

The corpus and compiler are deliberately different commits. The measurement asks
how that exact compiler reads that exact corpus; neither identity may float.

## Reproduce

A normal whole-tree compile follows transitive imports and reached only 1,634 of the
2,746 declared modules, leaving 1,112 uncompiled. First create a detached worktree at
the exact corpus commit. The corpus-reading tools verify `HEAD` and refuse any other
checkout; `.` below is never the moving PR checkout:

```sh
RECEIPT_TOOLS="$(pwd)/docs/probes/namespace_census_2026-07-31"
RECEIPT_SUMMARY="$(pwd)/docs/probes/namespace_census_2026-07-31/summary.json"
git worktree add --detach /tmp/namespace-census-corpus \
  0337fb27c039a800a1aff4b80140d6dbf027e595
cd /tmp/namespace-census-corpus
mkdir -p /tmp/namespace-census /tmp/namespace-census-output
python3 "$RECEIPT_TOOLS/generate_root.py" \
  . /tmp/namespace-census/complete_population_root.dag \
  --summary-json "$RECEIPT_SUMMARY"
/path/to/pinned/gunbc compile \
  --source-root dag \
  --source-root src/v2 \
  --source-root /tmp/namespace-census \
  --entry /tmp/namespace-census/complete_population_root.dag \
  --output-dir /tmp/namespace-census-output \
  --target dag \
  > /tmp/namespace-census/pop_stdout.log \
  2> /tmp/namespace-census/pop_stderr.log
```

Compile that entry with the pinned binary, the `dag`, `src/v2`, and generated-root
source roots, and capture stderr as `pop_stderr.log`. The synthetic root forces the
compile closure to contain all 2,746 corpus modules (the compiler reports 2,747
including the synthetic root); it is a measurement scaffold and is never committed.

Then verify the raw identity and classify every diagnostic, failing closed on any
new line shape:

```sh
python3 "$RECEIPT_TOOLS/parse_diagnostics.py" \
  /tmp/namespace-census/pop_stderr.log \
  /tmp/namespace-census/parser-result.json \
  --summary-json "$RECEIPT_SUMMARY" \
  --population-json /tmp/namespace-census/population.json \
  --ambiguity-json /tmp/namespace-census/ambiguity.json
python3 "$RECEIPT_TOOLS/provider_bounds.py" \
  . /tmp/namespace-census/population.json \
  /tmp/namespace-census/provider-result.json \
  --summary-json "$RECEIPT_SUMMARY"
python3 "$RECEIPT_TOOLS/classify_ambiguity.py" \
  /tmp/namespace-census/ambiguity.json /tmp/namespace-census/ambiguity-classified.json
python3 "$RECEIPT_TOOLS/classify_ambiguity_covisibility.py" \
  . /tmp/namespace-census/pop_stderr.log \
  /tmp/namespace-census/ambiguity-covisibility.json \
  --summary-json "$RECEIPT_SUMMARY"
python3 "$RECEIPT_TOOLS/verify_receipt.py" \
  "$RECEIPT_SUMMARY" /path/to/pinned/gunbc \
  /tmp/namespace-census/complete_population_root.dag \
  /tmp/namespace-census/parser-result.json \
  /tmp/namespace-census/provider-result.json \
  /tmp/namespace-census/ambiguity-classified.json \
  /tmp/namespace-census/ambiguity-covisibility.json
```

`summary.json` is the single expected-value authority. The classifiers derive facts
without embedding receipt totals; `verify_receipt.py` compares every result, the raw
log digest, the compiler binary identity, and the generated-root population to that
one authority. Any drift is a nonzero exit.

## Classification-total witness

`COMPILER-AUTHORITATIVE`: the compiler fold reported 18,048 hard diagnostics. The
fail-closed parser partitions those same diagnostics as:

```text
17,112 unresolved-name
   324 ambiguous-variant diagnostics under the synthetic root
   594 no-field
    12 type-mismatch
     6 singleton diagnostic shapes
------
18,048 = compiler-reported hard diagnostic count
```

The log therefore contains 18,049 lines in the diagnostic section: one compiler
header plus 18,048 diagnostics. The header says how many diagnostics follow; it is
not itself a diagnostic.

`REGEX BOUND`: `provider_bounds.py` approximates declarations in two deliberately
opposite ways. Category-strict lookup over-counts one-provider references because it
mis-keys providers declared under another category. Category-agnostic lookup
over-counts two-plus references because it merges namespaces the compiler keeps
separate. Together they bracket, rather than determine, mechanical share at
60.7–81.6% and unique consumer-to-provider edges at 2,197–2,717. In particular, the
strict catalogue's 2,867 zero-provider rows are not a semantic finding: 2,663 have a
different-category declaration and only the agnostic bound's 204 remain absent.

`INFERRED GROUPING`: grouping by variant name plus unordered candidate pair maps 324
synthetic-root diagnostics to 315 decisions. The suffix heuristic then labels them A_SELF (30
decisions/34 occurrences), B_PARALLEL_TOWER (63/63), and C_TRUE_HOMONYM (222/227).
Both sums are asserted by `classify_ambiguity.py`. The grouping key and labels are
analysis choices, not compiler output. In particular, B's suffix list was selected
by inspection; reading those pairs as one axis forked per language is an inference,
not a semantic compiler judgment.

## Ambiguity instrument contamination

The 324 compiler-authoritative count is the number of ambiguity diagnostics emitted
under the synthetic all-importing root. It is **not** the ambiguity population. The
instrument forces every module into one pool, so it can report a collision between
owners that no real compilation makes mutually visible. The reporting file identifies
where one owner is declared; it does not prove both owners were visible there.

`INFERRED INSTRUMENT ANALYSIS`: `classify_ambiguity_covisibility.py` reads the exact
pinned corpus files and partitions all 324 diagnostics without residue:

```text
 42 CoVisible            reporting file names both owners
  1 BracelessUndecided   wholesale import defeats textual visibility inference
281 PoolReach            reporting file does not name one or both owners
---
324 synthetic-root ambiguity diagnostics
```

Thus 281 of 324 (86.7%) are demonstrated pool-reach artifacts. At most 42 are
candidates for genuine co-resolution. Textual co-visibility is necessary but not
sufficient, so 42 is itself only an upper bound; this instrument establishes no lower
bound on the real ambiguity population. Measuring that population requires running
against real compile closures, outside this receipt's scope.

The contamination is itself the central finding: binding and module discovery are
coupled at the wrong boundary, so a whole-corpus pool cannot measure ambiguity without
creating most of the ambiguity it reports. The receipt preserves that limitation
rather than converting an instrument artifact into a language claim.
