# PRE-REGISTRATION — LexMatchThunk unmask A/B (registered 2026-08-22, before the B arm was built)

This file is committed **before the B arm exists**. Everything below is a commitment made without
having seen B's output; the receipt published later reports against exactly these terms, and any
departure from them is stated as a departure rather than silently re-specified. The commit that
introduces this file introduces no repair and no B-arm measurement — that is the point of it being
its own commit.

## The experiment — one variable

| arm | tree |
|---|---|
| **A** | current main emitter, `967b5bc1b92ee66250e06a7870c132b48a16b80a` — already measured, see [`e0308_partition_2026-08-22.md`](../e0308_partition_2026-08-22.md) |
| **B** | the same source sha + **only** the `LexMatchThunk` invocation repair |

Held identical across arms: source sha, entry (`src/v2/compiler/03_ingest.dag`, M=1), probe
(`curated_cargo_probe_one.sh`), `CSSL_STD_SEED_LINK=1`, empty `shim_lib_rel`, cargo manifest and
toolchain, and the emitted-file roster (177 files — a change in the roster invalidates the A/B and
is reported as such rather than absorbed).

**Acceptance for B being the declared variable and nothing else:** every
`no method named 'apply' found for struct 'Rc<LexMatchThunk>'` `E0599` site is gone in B, and no
other emitter decision is edited. If the repair cannot be made without touching a second decision,
that is reported and the A/B is re-registered — not quietly widened.

## What is registered as the expected population

**The registered population is 68 canonical E0308 sites in `src/v2_compiler_tokenize.rs`**, taken
from the 2026-08-21 partition at `2a2bd0ad59` and copied verbatim into
[`registered_masked_population.tsv`](registered_masked_population.tsv): **B3 36, T2 32**, whose
pairs are

```
18  Rc<Vector<_>>  | String            (T2)
18  Rc<Nat>        | integer           (B3)
18  Rc<Nat>        | i64               (B3)
 7  String         | Rc<Vector<_>>     (T2)
 4  Rc<Vector<i64>>| String            (T2)
 3  String         | Rc<Vector<i64>>   (T2)
```

**The 68 is a masked candidate population, not a predicted delta.** It will not land exactly, and
three named reasons are registered in advance so that none of them can be presented afterwards as
a discovery:

1. some of those sites may have changed independently while they were hidden;
2. one historical site may split into several diagnostics;
3. another may surface under a **different error code entirely** and so never appear on an E0308
   board at all.

## The prediction

**Unexplained additions = 0.** An *unexplained addition* is a newly-visible canonical E0308 site in
`src/v2_compiler_tokenize.rs` that joins to no row of the registered population under the join rule
below. Unexplained additions above zero is the **interesting** outcome — it would mean the repair
exposed something the historical partition never contained — and it is reported as its own counted
line, never absorbed into the join.

## The join rule, registered in advance

Newly-visible sites are joined to the registered population on:

- the **emitted file** (`src/v2_compiler_tokenize.rs`), and
- the **normalized expected/found relation** — module-path noise removed, elided/full spellings
  reconciled, direction-insensitive — as produced by the committed classifier
  [`e0308_classify_sites.py`](../e0308_classify_sites.py), and
- the **mechanism** its current pair classifies to.

**Never by generated line number.** A generated line is not a semantic identity and both arms
renumber freely.

**One registered limitation, stated now rather than discovered later:** the brief's stronger key —
enclosing source declaration or function — is **not recoverable for the historical roster**,
because the 2026-08-21 lane published a per-site TSV but no emitted tree, so the enclosing function
of those 68 sites cannot be reconstructed at their own ref. The B arm's own sites *do* carry it,
and the receipt will publish the enclosing function for every newly-visible site as evidence
beside the join, but the join itself can only run at pair-and-mechanism grain. This is a weaker key
than requested and it is registered as such **before** any result is known.

## How the result will be read — registered so it is not decided on the fly

- **High join rate + `unexplained=0`** → the masking hypothesis is confirmed and the successor
  population is known.
- **`unexplained > 0`** → the interesting outcome; reported as its own count with each site's
  pair, mechanism and enclosing function, and explicitly *not* folded into the join rate.
- **A joined site that arrives under a different mechanism than its historical row** is reported as
  a conversion, with both roots named — not counted as a match and not counted as unexplained.

## Reporting convention (adopted by `smart-ram-730`, 2026-08-22)

No adjusted total is invented. `315 + 68` is unit-invalid: 315 is raw coded rustc rows and 68 is
canonical sites. One headline metric plus a completeness standing:

```
A:  coded=315 | coverage=partial | mask=LexMatchThunk.apply | masked_candidates=68@historical-site-grain
B:  coded=X   | coverage=tokenize-unmasked | newly_exposed=Y | unexplained=0
```

Three movements are reported separately, and the receipt never leads with `315 → X` alone:

1. **visible board** — the coded row count in each arm;
2. **repair movement** — `LexMatchThunk.apply` `E0599` `N → 0`;
3. **exposure movement** — `Y` newly observable, `J` joined, `K` newly classified successors,
   `unexplained`.

The rise is an **exposure event, not a regression**: those sites exist in A and are unobservable
there, because a blocking error aborts the pipeline before the phase that would report them.

## Causal order this experiment protects

Unmask → fresh full board → fresh tokenize repartition → **then** repair the newly exposed roots.
No T2 or B3 repair is stacked ahead of this run; designing a repair against a diagnostic set that
does not yet exist on the current tree would be planning a successor board nobody has observed.
