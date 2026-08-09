# Import-strip measurement — reproducer and receipts

The machinery behind §15 of
[the witness-discovery cascade diagnosis](../import-strip-witness-discovery-cascade-diagnosis.md)
and behind [`import-strip-residual-ledger.tsv`](../import-strip-residual-ledger.tsv).

It exists because a 3,088-row ledger reads as authoritative, and a number a
future worker cannot regenerate or perturb is not a measurement — it is a claim
(DESIGN §5: green by execution, plus a discriminating input). Everything below
regenerates from a named tree.

**This is a registered scaffold, not production machinery.** Two Python scripts
reading `.dag` text are exactly the "second representation" the substrate exists
to replace: the classification is a `Node`-tree read that a lens should perform.
**Dissolution trigger:** when B2 lands `OrdinaryLoadedCompilationClosure`
production, the residual ledger becomes a projection of the loader's own
accepted-binding/provider output — at which point these scripts, the receipts
below, and this README delete together. Until then the strip cannot be measured
any other way, because the thing being measured is what happens when the
substrate's own resolution is removed.

## Tool identity

- `gunbc` release binary built from the measured head itself
  (`CTRL_BUILD_BYPASS_SHIMS=1 cargo build --release --bin gunbc`).
- It was REBUILT mid-lane, and that is load-bearing rather than incidental. The
  first measurement used a binary from `1eadad4af25`; main then landed #8062,
  which changes `src/v1` binding behaviour. A binary older than the substrate it
  judges reports a compiler that no longer exists — and it did: the heal entry
  passed locally under the old binary while failing in CI under the new one.
  Rebuild whenever `src/v1` moves, and do not reuse a binary across a main merge
  without checking.
- Measured base commit and corpus hash: [`receipts/subject-tree-hash.txt`](receipts/subject-tree-hash.txt).

## Regenerate everything

From a clean checkout of the measured base, with `$G` the `gunbc` binary and
`$W` a scratch directory:

```sh
# 0. two copies of the subject tree: one control, one to strip
mkdir -p $W/control $W/stripped
cp -r dag src $W/control/
cp -r dag src $W/stripped/

# 1. strip every import declaration, brace-depth-aware, with a per-file manifest
python3 docs/plans/import-strip-measurement/strip_imports.py \
    $W/stripped/dag $W/stripped/src/v2 \
    --manifest $W/stripped-file-manifest.tsv
# expect: stripped 16382 import declarations across 2583 files
grep -rc '^import ' $W/stripped/dag $W/stripped/src/v2 --include='*.dag' | grep -v ':0' | wc -l
# expect: 0   (zero residue — a partial strip reads downstream as corpus failure)

# 2. the known-positive control, then the stripped reading
(cd $W/control  && $G compile --source-root dag --source-root src/v2 \
    --dependency-pool-index primary-precedence --target dag \
    --output-dir $W/o-control  > $W/control-diagnostics.log 2>&1)
(cd $W/stripped && $G compile --source-root dag --source-root src/v2 \
    --dependency-pool-index primary-precedence --target dag \
    --output-dir $W/o-stripped > $W/stripped-diagnostics.log 2>&1)
grep 'hard diagnostic' $W/control-diagnostics.log $W/stripped-diagnostics.log
# expect: control 10, stripped 3098   (both exit 1 — see the note below)

# 3. the ledger, with the reconciliation printed
python3 docs/plans/import-strip-measurement/classify_residual.py \
    $W/stripped-diagnostics.log $W/stripped \
    docs/plans/import-strip-residual-ledger.tsv --control-count 10

# 4. the declaration census (full; only its duplicate slice is committed)
grep -rhoE '^(fn|func|type|data|service) +[A-Za-z_][A-Za-z0-9_]*' \
    dag src/v2 --include='*.dag' | awk '{print $2}' | sort | uniq -c | sort -rn
```

Both compiles exit **1**, control included, because the control's 10
pre-existing annotation-grain diagnostics are themselves hard. Exit status does
not discriminate between the two trees; only the count and the per-name join do.

## The reproducer claim is itself verified

"Regenerable" was asserted here once while the two scripts were absent from the
tree — `.gitignore`'s repo-wide `*.py` rule dropped them silently (review 50719
caught it). Both now carry negation entries beside the existing exemptions, and
the claim is checked the way the rest of this lane's claims are — by execution
against tracked content only:

```sh
git archive HEAD | tar -x -C $W/src          # ONLY what a fresh clone gets
python3 $W/src/docs/plans/import-strip-measurement/strip_imports.py …
python3 $W/src/docs/plans/import-strip-measurement/classify_residual.py …
```

Result: the manifest and `import-strip-residual-ledger.tsv` are reproduced
**byte-identically**, and the classifier prints the reconciliation
(`3,098 = 10 + 3,088`, ledger rows 3,088, OK). A reproducer that cannot be run
from a clean checkout is a receipt with a story attached, not a reproducer.

## What in the receipts is NOT byte-reproducible

The compile logs carry wall-clock lines (`compile.reconcile done in 5 minutes`),
so a re-run reproduces the diagnostics exactly and the timings never. Compare
receipts with those lines filtered:

```sh
diff <(grep -v 'done in' new.log) <(grep -v 'done in' receipts/control-diagnostics.log)
```

This is not a caveat added to excuse a mismatch — it is how the stamp below was
checked. Everything the measurement asserts (diagnostic text, counts,
reconciliation, manifest, ledger) is byte-reproducible; only the durations are
not, and nothing derives from them.

## Receipts

| file | what it is |
| --- | --- |
| `receipts/control-diagnostics.log` | raw unstripped compile output (the known-positive) |
| `receipts/stripped-diagnostics.log` | raw stripped compile output |
| `receipts/stripped-file-manifest.tsv` | per-file count of import declarations removed (2,579 rows) |
| `receipts/declaration-census-duplicates.tsv` | every multiply-declared name (the actionable slice) |
| `receipts/summary.txt` | disposition totals and the reconciliation identity |
| `receipts/subject-tree-hash.txt` | corpus hash + measured commit |

## What the scripts deliberately do not do

- **They never observe the loader.** `provider_in_loaded_closure`,
  `intended_provider` and `accepted_binding` are `unobserved` on every ledger
  row. Whether a provider entered the closure, and which declaration a reference
  accepted, are exactly the open clause E/F questions; they are not derivable
  from diagnostic text. Filling them would fabricate the measurement the lane
  exists to make.
- **They never treat an index miss as absence.** The declaration index reads
  line-anchored declarations and `|`-per-line variant tags. It does not see
  inline `type X = A | B` variants, interpreter builtins, `std.primitives`
  rows, algebra templates, or per-target emit vocabulary — and DESIGN's
  determinism open thread records why a complete primitive denominator is not
  assemblable today (the roster is forked across five authorities). So a miss
  produces a typed sub-reason (`variant_owner_unindexed`,
  `ordinary_callee_unindexed`, …), never "this name does not exist".
- **They assign no cause.** Every disposition that would require knowing *why* a
  reference failed carries `_unobserved` in its name.
