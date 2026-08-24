# Re-measure at `1ed02057a5` — the board is 123 of 305, and six R1 blocks were repaired

**A board is a measurement at a ref, not a constant.** This file exists because the partition's
headline was being quoted as though `128` were a property of the corpus. It was true at
`98b18cdc81e` / `907f19c2cc` and is false now.

## The measurement, and its provenance

| field | value |
|---|---|
| ref | **`1ed02057a5fac683893afcbb427fa8933cc0f2a4`** (main, 35 commits past `907f19c2cc`) |
| observed HEAD | `1ed02057a5…` — equals requested; `PROBE_EXPECT_BASE_SHA` armed |
| binary | `PRE_STAMP=ABSENT` → key miss → **rebuilt from this tree** |
| emitted roster | 177 files, 503 gunbc diagnostics, 0 blocking |
| raw log | `03_ingest_1ed02057a5.cargo.log` (published beside this file) |
| classifier | `docs/probes/e0308_classify_sites.py`, unchanged |

```
coded rows  305      E0308  123      E0277  18
```

Independently reproduces `quiet-eagle-429`'s reading, which was taken at a *different* ref
(`6a59c1a549`) on a separately built binary. Two refs, two binaries, same figures — and the emit side
is byte-identical across both (177 / 503 / 0), so the instrument is continuous and the movement is
on the rustc side alone.

## The partition at this ref — stated as *N of M at SHA*

**E0308 = 123 of 305 coded rows at `1ed02057a5`**, folding to **148 canonical sites**.

| root | sites | | root | sites |
|---|---|---|---|---|
| R1 | **28** *(was 34)* | | C | 6 |
| R2 | 24 | | B2 | 6 |
| D | 16 | | ELEM-COLL | 5 |
| T3 | 14 | | W | 5 |
| A-clone | 12 | | DIAG | 3 |
| ARG-ORDER | 11 | | BOX-WRAP | 1 |
| B3 | 10 | | **RESIDUE** | **7** |

## Identity-grain join against the published 154

Join key: **file + normalized expected/found relation + mechanism**. Never a generated line number —
generated lines move for reasons unrelated to the diagnostic, and a line-keyed join manufactures churn.

```
OLD 154   NEW 148
STILL          148
FIXED            6
NEWLY_VISIBLE    0
MOVED_CODE       0   (vacuous within E0308: nothing entered the bucket)
UNCLASSIFIED     0
```

**The six FIXED, named individually** — all `R1` (bare ↔ `Rc` wrap):

```
x4  src/v2_lens_complexity_accumulator_copy_analyze.rs   Rc<LetBinding>        -> LetBinding
x1  src/v2_std_artifact.rs                               Rc<Refined<Rc<Artifact>>> -> Refined<Rc<Artifact>>
x1  src/v2_std_artifact.rs                               Rc<Refined<_>>        -> Refined<_>
```

**This is a repair that landed, not a partition failure.** `R1` is not resurrected to keep the count
whole — it stands at 28 with six of its blocks gone. Every other cluster is unchanged, and **residue
is still 7**, so the fail-closed condition holds: no root went unnamed and nothing became unclassified.

## What is deliberately not claimed

- **No cause is attributed.** Five E0308 and five E0277 blocks disappearing together is the shape a
  reference-layer repair would produce, and #9025 / #8984 are plausible candidates — but that is a
  hypothesis relayed from `smart-ram-730`, not a measurement, and it is recorded here as unverified
  rather than quoted as a finding.
- **`123` is not adopted as a new constant.** It is this ref's number. The next lane to need it
  re-measures at its own ref, or quotes this one *with* its SHA.
- **E0277's fall from 23 to 18 is outside this partition's bucket.** It is noted because the two
  moved together, not because this instrument measured its mechanism. A per-bucket classifier cannot
  see arrivals or departures in another code; that requires the separate histogram diff.

## The correction this file embodies

Every prior statement of this board's share — including in the merged receipt — is a fraction whose
denominator was a moment. From here the partition is quoted as **"N of M at SHA"**. A fraction
without a ref is not a smaller fraction; it is an unverified one.
