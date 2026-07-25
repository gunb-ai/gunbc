# `.dag` note-prose census — what the long strings are, and what that prices

**Status:** census complete, measured 2026-07-25. No code migrated. This document is the
*reading*; `docs/plans/dag-note-prose-census.py` is the authority — every number below
re-derives with `python3 docs/plans/dag-note-prose-census.py` from the repo root.

**Scaffold, with a dissolution trigger:** both files delete when the typed carriers land and
the annotation-budget lens counts *rows*. A lexical census over prose is superseded the moment
the rows are typed — that is the whole point of the migration it prices.

---

## 0. The one-paragraph answer

The five-class model in the brief is right about *shape* and wrong about *proportion*, and the
correction changes the sequencing. A note is indeed an anemic serialization of several typed
things — **69% of prose bytes sit in rows that mix two or more classes**, which is the finding
that matters. But history is **not** a third of the bytes; it is **7–18%** depending on
threshold, and the mass that can be deleted today is **under 1%**. The dominant class is one
the brief's model does not name: **specification-and-norm** — present-tense statements of what
a declaration *is* and *requires* — at **≥40% of bytes** and rising once the unclassified
residue is folded in. So the payoff is not a deletion. It is that the largest class is
`StandingIntent` + type material, i.e. the enforcement-intent lane's first consumer, exactly as
the brief guessed — but the *event log* earns its place on the **append rate**, not the stock.

---

## 1. Population — three tiers, not one

Measured over 2,530 `.dag` files. "Prose" = a String value ≥ 200 bytes; below that a String row
is a tag, path, or name, not prose (8,681 such rows, 302.7 KiB, excluded).

| tier | all rows | prose rows | prose bytes |
|---|---|---|---|
| `data <n>: String = "…"` | 2,355 (819.7 KiB) | 1,071 | **764.8 KiB** |
| typed struct-field values | 7,111 (325.7 KiB) | 190 | **91.0 KiB** |
| `List<String>` elements | 498 (21.2 KiB) | 22 | **8.1 KiB** |
| **total** | | **1,283** | **863.9 KiB** |

Of the prose `data` rows, 820 (628.7 KiB) are `*note*`-named. The brief's "915 rows / ~718KB"
was close on the note tier and, as it suspected, an undercount overall — the true figure is
**863.9 KiB**, and the undercount has two sources the brief named plus one it did not:

- **typed fields** (91.0 KiB) — as predicted. Concentrated in `reason` (47.4 KiB, 78 rows),
  `text` (22.2 KiB), `dissolve_on` (4.1 KiB).
- **`List<String>` elements** (8.1 KiB) — **missed by any `: String` scan**, which is where the
  brief's own `MacAddress` example lives (`dag/extdeps/dhcp/v4.dag:23`). Small in mass,
  decisive in meaning — see §5.

Largest single row: `gunbc_ci_floor_batch_wall_budget_note`, 8,278 B (the batch-budget note),
matching the brief. Four more exceed 3 KiB.

**Concentration:** 50% of prose bytes live in **82 of 645** prose-bearing files (13%).
`dag/gunbc` (302.6 KiB) and `src/v2` (301.9 KiB) hold 70% between them. The top file is
`dag/gunbc/ci_layer_roots.dag` at 25.3 KiB. This is not diffuse — a migration can start where
the mass is and cover half the problem in ~80 files.

---

## 2. Six classes, not five

Classified at **sentence** grain, because the note is the wrong unit — a note is a mixture, and
the sentence is what becomes a row. 3,841 sentences.

| class | sentences | bytes | share | migrates to |
|---|---|---|---|---|
| **SPEC_NORM** | 1,429 | 341.3 KiB | **39.6%** | a type, or a lens |
| XREF | 569 | 139.7 KiB | 16.2% | a citation edge |
| EVENT | 426 | 106.6 KiB | 12.4% | an event-log row (ages out) |
| RATIONALE | 346 | 88.7 KiB | 10.3% | *irreducible prose* |
| RULING | 148 | 38.0 KiB | 4.4% | a ruling-register row |
| RECEIPT | 124 | 33.1 KiB | 3.8% | a measurement row |
| UNCLASSIFIED | 799 | 114.1 KiB | 13.2% | — (≈ all missed SPEC_NORM) |

**The sixth class is the finding.** The brief's five classes were read off the *largest* notes,
which skew historical — the biggest rows really are budget-bounce logs. But the bulk of the
corpus is present-tense statements of what a declaration means and what it requires:

> "Interior directory candidates (basename has no '.') are descended; dotted basenames are
> skipped as non-dag files."
> "Never treat `partition_fold() -> []` as a refusal signal — that compat wrapper collapses
> `PartitionFoldRefused` to `[]`."

Neither is a ruling, event, receipt, cross-reference, or rationale. The first is **spec**; the
second is an **invariant**. Both are live, both are prose, and neither expires — but neither is
history either, so neither is addressed by an event log.

### The mixing that proves the model

| distinct classes in one note | notes | bytes |
|---|---|---|
| 0–1 | 622 | 266.6 KiB |
| 2 | 455 | 326.9 KiB |
| 3 | 154 | 165.2 KiB |
| 4+ | 52 | 105.2 KiB |

**661 notes / 597.3 KiB — 69% of prose bytes — fuse two or more classes in one String.** That
is the anemic serialization, measured. It is also why deletion is hard by hand: the operator's
recurring labor is re-deriving live-vs-dead *within* a paragraph, not between paragraphs.

---

## 3. History is 7–18%, and deletable-now is under 1%

The brief guessed a third of the bytes were pure history. Measured, it is not:

| measure | rows | bytes | share |
|---|---|---|---|
| history-class sentences (event/receipt, no live force) | 277 | 61.3 KiB | **7.1%** |
| notes ≥60% history sentences | 122 | 65.8 KiB | 7.6% |
| notes ≥40% history sentences | 225 | 157.6 KiB | 18.2% |
| **crisp deletable, hand-verified** | **5** | **5.1 KiB** | **0.6%** |

The bias runs *against* the guess, not for it: hand-verification (§6) shows EVENT is
**over**-detected, so 12.4% is an upper bound.

### The crisp deletable set

Two genres are lexically decidable and hand-checkable one by one:

- **fired dissolution** — a scaffold-debt row whose trigger already fired; records a completed
  transition, binds nothing.
- **superseded dated snapshot** — a dated refresh note that a later-dated sibling in the *same
  file* replaces. Pure append-forever residue.

The gated detector emits **3 rows / 3.0 KiB**, all 3 verified true by reading:

| bytes | row |
|---|---|
| 1691 | `src/v2/compiler/self_host/frontier_probe_types.dag:101` `honest_frontier_refresh_2026_07_22_note` — superseded by the `_07_23_` sibling **in the same file** |
| 1151 | `src/v1/04_env.dag:164` `ancestry_cache_sharing_dissolution_trigger` — "DISSOLVED…", a completed optimization + its proof |
| 249 | `src/v2/test/claim/long/…/accumulator_copy_fold_analysis_test.dag:36` `projection_call_residue_note` |

Hand-reading the *ungated* 7 candidates found 2 more genuinely deletable
(`cssl_emit_artifact_sanitize_scaffold_debt`, `derived_projections_landed_note`) — hence 5 rows
/ 5.1 KiB as the honest total. `src/v1/04_env.dag` and `dag/std/effect_grant.dag` are
load-bearing per DESIGN §7; **nothing here has been deleted** — this is a proposal list for an
operator call, not an applied change.

**Why the gate is load-bearing, and why this must not be automated:** the ungated detector was
71% precise (5/7). Both false positives share one mode — *a sub-part dissolved while the row
still binds*:

- `corpus_scan_scaffold_note` records a sub-check dissolving, then states its **own un-fired**
  "Dissolution trigger: witness realization…". Deleting it would delete a live trigger.
- `placement_grain_witness_note` records a 2026-07-11 dissolution as the *reason a synthetic RED
  control exists*. Deleting it strands the test's purpose.

Requiring zero live-force markers drops exactly those two. It also drops two true positives
(`"was DEFERRED … trigger fired"` reads as live), so the gate trades recall for precision —
correct for a deletion proposal, and the residue is hand-extended, never auto-applied.

---

## 4. The rate is the pain, not the stock

The brief's claim that notes grow back was checkable, so I checked it. **Caveat: this worktree
is a shallow clone** — 80 commits spanning 2026-07-22 → 2026-07-23 — so the multi-month growth
claim is *not measurable here*. Within the available window:

| | notes | bytes |
|---|---|---|
| `8331811bee` (2026-07-22) | 627 | 464 KiB |
| `7a087e8178` (2026-07-23) | 804 | 626 KiB |

**+177 notes, +162 KiB across 80 squash-merged PRs — ~2 KiB of note prose per PR.** Note-row
additions to deletions run **227 : 50 ≈ 4.5 : 1**.

That ratio *is* the brief's thesis measured: prose cannot expire, so appends outrun deletions
by 4.5×. It reframes the sequencing. The stock of deletable history (5 KiB) would not repay a
carrier. The **flow** does: at ~2 KiB/PR with a 4.5:1 append ratio, the corpus re-accumulates
the entire crisp-deletable set roughly every three PRs. A hand-deletion pass is a treadmill; a
typed row with mechanical expiry is the only thing that changes the derivative.

---

## 5. The target model already exists in tree

21 rows (7.6 KiB) already use a hand-serialized three-field format:

```
closure:<id>|<rationale>|trigger:<condition>
```

e.g. `dag/extdeps/dhcp/v4.dag:24` —
`"closure:decompose_mac_address_octets|MacAddress is a branded NonEmptyStr hiding six grounded
octets; …|trigger:extdeps/network/mac.dag lands with parse_mac_address witness"`

Someone independently converged on a **record with a stable id, a rationale, and an expiry
condition, serialized into a String because String was the only medium available on the
carrier.** That is the brief's diagnosis reproduced from the inside, and it is the strongest
available evidence that the model is right and the medium is wrong. It is also a ready-made
first migration target: 21 rows with an already-agreed field structure, needing a type rather
than a design.

---

## 6. Honesty bound

Detection is lexical because the input is unstructured English — that is the disease being
priced, not a modeling choice (DESIGN §4: a heuristic is never necessary *in a closed system*;
this measurement is over prose, which is precisely the un-closed residue).

Verified on a 35-sentence stratified sample, read by hand against its label:

| class | correct | notes |
|---|---|---|
| RATIONALE, SPEC_NORM, XREF | 5/5 each | |
| RECEIPT | 4/5 | an instruction ("re-run the receipt whenever…") is not a measurement |
| EVENT | 3/5 | "Wave 1 Gate 1 Stage D" is spec; "deferred until #6775" is an *open* item |
| RULING | 3/5 | incidental "operator's" fires it |
| UNCLASSIFIED | 0/5 | all five are missed SPEC_NORM |

**≈70% primary-class precision, with a known directional bias:** SPEC_NORM is **understated**
(the 13.2% unclassified residue is almost entirely missed SPEC_NORM, so its true share is
≈50%), EVENT and RULING are **overstated**. Both biases push the same way — they *strengthen*
§0's conclusion rather than soften it. Treat §2's shares as ±10pp, and §3's history figure as
an upper bound. Nothing here is a proof; it is a measurement with its error named.

---

## 7. What this prices

1. **Do not plan a deletion pass.** There is no third of the bytes to reclaim — there is 0.6%.
   The 5 rows in §3 are worth landing as a *demonstration* of the collapse (one live row + one
   event row each), not as cleanup.
2. **The event log is justified by §4's rate, not §3's stock.** Build it because it makes expiry
   mechanical at 4.5:1 append pressure, and say so — justifying it on reclaimed bytes would be
   pricing it wrong (DESIGN §6: denominate the benefit in displaced cost; here the displaced
   cost is the operator's recurring re-derivation, which recurs every ~3 PRs).
3. **The largest class is the enforcement-intent lane's material.** ~40–50% of bytes is
   spec/norm — invariants stated in prose that a lens could hold, and semantics that belong in a
   type. This is the first consumer the brief predicted, and it is bigger than the ruling
   register.
4. **Cheapest first migration: the 21 `closure:…|trigger:` rows** (§5). Field structure already
   agreed, mechanically parseable, and it converts the corpus's own invented format into the
   real one.
5. **Then the concentration head** (§1): 82 files hold half the mass.

### Explicitly not claimed

- That any note *should* be deleted — §3 is a proposal list; the two false positives are why.
- That the growth rate generalizes beyond a 2-day, 80-commit window (§4's caveat).
- That the class shares are precise to better than ±10pp (§6).
