# Whole-corpus namespace census receipt — 2026-07-31

This receipt makes the namespace measurement reproducible without committing its
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
- Generated synthetic-root identity: SHA-256
  `69f80839ecf0d9edf2f43210311a781396f365b7c64dbfd4e63c93b2363530c6`.

The corpus and compiler are deliberately different commits. The measurement asks
how that exact compiler reads that exact corpus; neither identity may float.

## Reproduce

`namespace_census_reproduction_steps` in `gunbc.namespace_census_receipt` is the
ordered workflow authority. The shell below is a `SCAFFOLD` projection of those typed
intents. It dissolves when the canonical orchestration-to-bash emitter (bash-emit
#5828) renders that step sequence; the model remains and the hand-authored block is
deleted.

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
python3 "$RECEIPT_TOOLS/provider_scenarios.py" \
  . /tmp/namespace-census/population.json \
  /tmp/namespace-census/provider-result.json \
  --summary-json "$RECEIPT_SUMMARY"
python3 "$RECEIPT_TOOLS/classify_ambiguity.py" \
  /tmp/namespace-census/ambiguity.json /tmp/namespace-census/ambiguity-classified.json
python3 "$RECEIPT_TOOLS/classify_ambiguity_textual.py" \
  . /tmp/namespace-census/pop_stderr.log \
  /tmp/namespace-census/ambiguity-textual.json \
  --summary-json "$RECEIPT_SUMMARY"
python3 "$RECEIPT_TOOLS/verify_receipt.py" \
  "$RECEIPT_SUMMARY" /path/to/pinned/gunbc \
  /tmp/namespace-census/complete_population_root.dag \
  /tmp/namespace-census/parser-result.json \
  /tmp/namespace-census/provider-result.json \
  /tmp/namespace-census/ambiguity-classified.json \
  /tmp/namespace-census/ambiguity-textual.json
```

`summary.json` is the single expected-value authority. The classifiers derive facts
without embedding receipt totals; `verify_receipt.py` compares every result, the raw
log digest, the compiler binary identity, and the exact generated-root digest to that
one authority. Any drift is a nonzero exit.

The receipt distinguishes four evidence grades: `COMPILER-AUTHORITATIVE`,
`REPRODUCIBLE TEXTUAL CLASSIFICATION`, `REGEX SENSITIVITY SCENARIO`, and
`HUMAN OR INFERRED GROUPING`. Only the first is compiler semantics.

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

`REGEX SENSITIVITY SCENARIO`: `provider_scenarios.py` runs two declaration-catalogue
approximations. The category-agnostic scenario reports 60.7% apparent single-provider
rows and 2,197 unique apparent consumer-to-provider edges. The category-strict
scenario reports 81.6% and 2,717 respectively. These are scenarios, not bounds:
either can fall on either side of semantic resolution. Both include declarations
that may not be structurally visible and can miss multiline forms, aliases, and
grounded operations. Strict lookup can turn true-many into apparent-one; agnostic
lookup can turn true-one into apparent-many; either can turn a real provider into
zero. Neither executes namespace admissibility or containment. Consequently these
figures must not size a migration population. The scripts also fail closed unless
every apparent-single row maps to exactly one consumer module; both unmapped and
duplicate mapping counts are asserted to be zero.

`HUMAN OR INFERRED GROUPING`: grouping by variant name plus unordered candidate pair maps 324
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

`REPRODUCIBLE TEXTUAL CLASSIFICATION`: `classify_ambiguity_textual.py` reads the exact
pinned corpus files and partitions all 324 diagnostics without residue:

```text
 42 BothOwnerNamesTextual       reporting file text names both owners
  1 BracelessImportTextPresent  reporting file contains a brace-less import
281 BothOwnerNamesNotTextual    reporting file text does not name both owners
---
324 synthetic-root ambiguity diagnostics
```

This text predicate constrains semantic visibility in neither direction. Declarations
may be visible through imported constructors, functions, aliases, or a module surface
without either owner name appearing in the reporting file. Conversely, both names may
appear in notes, strings, or unrelated annotations without being visible at the
failing occurrence. Therefore 42 is not an upper bound, 281 is not a demonstrated
artifact population, and this classification cannot justify or avoid a projection
mechanism.

The only authoritative ambiguity population comes from the compiler fold over real
compile closures. No homonym-renaming wave may be dispatched from this receipt's
textual classifications. Measuring real closures belongs to later work; this durable
historical instrument receipt does not block P1 or P2 implementation.
