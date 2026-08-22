# Normalizer / discarded-fact audit — bounded inventory, calibrated instrument

**Subject:** every `normalize_*` / `unwrap_*` / `peel_*` operation in `src/v1` and `src/v2`.
**Ref:** `967b5bc1b92` (main tip; no commits since the measurement).
**Deliverable:** a table and two verdicts. **No repair was made and none is proposed here.**

## The class

A normalizer runs upstream of a consumer that **depends on anything the normalizer removed**.
The consumer reads correct in isolation and is unfixable in place.

An earlier phrasing — *reads a key, identity, annotation or source fact off the result* — is
**refuted and superseded**. It excludes shape predicates by construction, and the one confirmed
member of the class (`unwrap_single_field_product`) is found by `is_product_type`, a shape test,
reading through to a node the normalizer replaced. A class definition is an extraction pattern over
defects; that one could not express a member already in hand.

## Verdict 1 — ENUMERATION: **CLOSED**

The **26** names below are the whole population for these spellings. A bounded null over a complete
population, and it is stated first because it is the rarer result.

| probe | count |
|---|---:|
| method position (`.op(...)`) | **0** |
| declared as `let` / `data` rather than `fn` | **0** |
| bare mentions not followed by `(` | 56 — *all* classified |

The 56 account fully: 21 import-list entries, 29 inside `data`-note `String` prose, 2 module
declarations, 4 English uses of the word "normalize" in `//` comments. **No higher-order or value
position anywhere**, so no twenty-seventh operation hides outside the `fn` form.

## Verdict 2 — DEPENDENTS: **PARTIAL** (29 of 71 sites)

### Instrument calibration (required before any count below is readable)

The detector follows the bound result to the end of its enclosing `fn`, flagging identity reads,
shape predicates, structural tests and direct field access. **It reproduces the known member at both
its call sites, by the exact mechanism:**

    src/v1/05_emit_rust.dag:10635   effective.children, effective.return_cardinality, is_product_type
    src/v1/05_emit_rust.dag:10695   effective.children, is_product_type

A prior narrower version **missed** that member. No negative from an instrument that has not
reproduced a known positive is worth anything.

### Evidenced dependents — 29 sites, four operations only

| operation | evidenced | unanalysed |
|---|---:|---:|
| `normalize_access_type_node` | **19** | 3 |
| `peel_nominal_alias_identity` | 6 | 4 |
| `peel_where_refinement_base` | 2 | 0 |
| `unwrap_single_field_product` | 2 *(known member)* | 0 |

**This ranking covers the measured half only.** It is not a ranking of the class.

### Unanalysed — 42 sites, risk-sorted

| bucket | count | reading |
|---|---:|---|
| argument | 21 | traceable to a named callee; analysable later, no new machinery |
| tail-of-branch | 8 | returned via a branch arm — same escape as `returned` |
| returned | 5 | loss escapes the frame outright |
| **piped-local** | **0** | **the benign case is empty** |
| other | 8 | classifier could not place; residue, named not buried |

A returned result propagates the loss to *every* caller of the enclosing function, so producer and
consumer are never in one frame. The hypothesis that the unmeasured half is mostly benign
pipe-locals is **refuted**: there are none.

### The 13 frame-escapes, adjudicated → **2 candidates**

Test: *does the enclosing function's contract already promise a normalized result?* If yes, the
caller asked for exactly the thing that was removed and there is no loss.

| disposition | n | |
|---|---:|---|
| own recursion | 3 | `04_types.dag:708`, `complexity.dag:2066`, `:2071` — the normalizer's own descent |
| contract promises normalized | 8 | 6 interior to `03_normalize`, plus `stage_normalize` and `program_assembly_read_to_normalized_root_prepared` |
| **genuine candidate** | **2** | below |

```
src/v1/04_infer.dag:4849   peel_alias_once_for_field_access
                           inside expand_alias_chain_for_field_access
src/v2/lens/vacuity.dag:324  normalize
                           inside vacuity_witness_ingest_source_text
```

**`04_infer.dag:4849` is the best probe available** and the place any lane should start: it is
*simultaneously* an evidenced dependent (an `authored_name_at` read on the peeled node) **and** a
frame escape, inside a function about *field access* rather than about normalizing — so the contract
test does not excuse it.

Two of the eight excused rows were initially mis-sorted as candidates because the test was applied as
a **prefix regex** rather than as the concept: `stage_normalize` and
`program_assembly_read_to_normalized_root_prepared` both promise a normalized result in their names
without starting with `normalize_`. Matching a spelling where a concept was meant is the same failure
the class itself is about.

## The class is wider than normalizers — normalizers are where it was found

**A fifth instance surfaced inside the reporting of this audit.** An interim report named the 13
frame-escapes by **basename**, and `complexity.dag` exists in *both* `src/v1` and `src/v2/lens` — so
`complexity.dag:2066` collapsed two real files. A basename is **a shorter spelling substituted for a
discriminating identity**, which is the same shape as the bare-keyed summary map collapsing two
declarations that share a name, and as `type_reference_decl_file`'s `String` collapsing four states
into two spellings. The prefix regex above is a third: it collapsed `stage_normalize` and
`program_assembly_read_to_normalized_root_prepared` into non-members by matching a spelling where a
concept was meant.

So the class generalises:

> **Any layer that substitutes a shorter spelling for a discriminating identity, and any consumer
> downstream that needed the distinction.**

Instances today span five layers, and only the first is where anyone would have gone looking:
the emitter (`unwrap_single_field_product`, the summary map, `decl_file`), an extraction pattern (a
character class without digits turning `v1` into `v`), adjudication tooling (the prefix regex), and a
report (the basenames).

That framing predicts where to look next **with no new instrument**: wherever this repository
shortens a name to report, key, group or match on it. Extending the enumeration to cover the wider
class is deliberately **not** in scope here — it is a different subject.

## Known members

| member | status |
|---|---|
| `unwrap_single_field_product` → `emit_shell_return` | **REPAIRED**, 60 → 0 |
| emit summary map — bare authored name collapses two declarations before the ambiguity guard | open |
| `type_reference_decl_file` — `String` collapses four states into two spellings | open |
| use-line `StructRepr` filter — candidates dropped with no counted diagnostic | open |
| `peel_where_refinement_base` ×2 (`04_infer:1706`, `:2592`) | open, **legibility not wrongness** |

The two `peel_where_refinement_base` rows are a *diagnostic naming a subject the user did not write*:
the peel is **correct** for the judgment — a refined `String` should be judged and should find methods
as `String` — and the loss surfaces only in the report, where `MethodNotFound { receiver_type: … }`
names the base. Same class as a refusal hardcoding `"v2 self-compile"` on the generic compile path.
**Both are wrong-subject; only that one misdescribes what was operated on.** Construction fix, shared
shape: judge on the base, report on the authored node, carry both rather than replacing one with the
other.

## What this audit does not establish

- The 42 unanalysed sites are **unmeasured, not clean**. Closing them needs the expression tree
  rather than a bound variable name.
- The evidenced ranking orders visible sites; with 13 frame-escapes located (2 surviving
  adjudication) the class may be larger than it suggests.
- No site here has been read for whether the discarded fact is *actually* depended upon — only that a
  dependent exists in frame. Each candidate still needs its executed discriminator.

---

## CORRECTION 2026-08-22 — Verdict 1 is RESCOPED (not withdrawn)

Verdict 1 was published as ENUMERATION CLOSED. That word is false. But the
first revision of this correction withdrew the verdict outright, and that was
an over-correction in the opposite direction: it deleted an executed result
because the sentence carrying it was too wide.

**The distinction that fixes it: completeness of the SEARCH WITHIN a found set
is not completeness of the FINDING of that set.** Two different claims were
unified by one word.

- **STANDS — a bounded, executed null over the 26 found names.** No method
  position, no `let`/`data` binding, and no higher-order use hides an operation
  among them: 0 method positions, 0 let/data, and all 56 bare mentions
  classified (21 imports, 29 data-note prose, 2 module decls, 4 `//` comments).
  That was measured, it discriminates, and nothing below touches it.
- **FALSE — that the 26 names were all the names.** The name-finding itself was
  never checked for completeness, and it was not complete.

Verdict 1 is therefore restated as: **SEARCH CLOSED OVER A COMPILER-SCOPED
FOUND SET; FOUND SET NOT CLOSED.** The scope is stated below rather than left
implied, which is the whole of what the original verdict omitted.

**What was wrong.** The enumeration ran over a compiler-centric file scope
and the verdict was published corpus-wide. Re-deriving the same rule against
`origin/main` returns 32 operations against the audited 26. Six were never
considered; none of the 26 are dead.

    normalize_cost_expr_to_symbolic   src/v2/lens/cost/expr.dag
    normalize_size_expr_to_bound      src/v2/lens/cost/expr.dag
    normalized_grain_tree             src/v2/test/claim/long/mandatory_tag_gate_witness_test.dag
    normalized_of                     src/v2/test/claim/long/accumulator_copy_compile_gate_test.dag
    normalizes                        src/v2/test/claim/long/dag_repeat_spine_wellformed_test.dag
    normalize_outcome                 src/v2/test/claim/body_lowering/{declaration_structure_preserved,statement_let_bind}_test.dag

**Staleness is NOT the cause, and this matters for the remedy.** The audit
was performed against a worktree 171 commits behind main (merge-base
`a6ca6882d18`). That is a real, separately-reported defect. It is not this
one: five of the six missed operations were present at the audit's own head
and were missed anyway. Only `normalize_outcome` is new in those 171 commits.
So re-running the audit on a fresh tree would have corrected 1 of 6. The
enumeration rule was narrower than the class it claimed to close, and no
amount of rebasing repairs that.

**How they escaped, measured.** Audited call-site distribution was 17 in
`src/v2/compiler`, 1 in `src/v2/lens`, 3 in `src/v2/test`. All six missed
operations live in `src/v2/lens/cost/` and `src/v2/test/claim/`. The scope
was compiler-shaped; the verdict was not.

**Is the missed scope inert?** No, though the honest reading is weaker than
it first looks. `normalize_size_expr_to_bound` contains
`SizeConst { value: _ } => unit_cost()`, which discards a constant's value —
on-subject shape for this audit. For a symbolic *complexity* bound, collapsing
constant size to O(1) is plausibly correct by design, so this is NOT reported
as a member of the class. It is reported as what it is: an unanalysed
operation, of the right shape, inside a scope the audit declared closed
without looking at it. The two `lens/cost` operations are production code;
the four test-file operations may or may not be in-class once adjudicated.

**Verdict 2 (DEPENDENTS PARTIAL) is unaffected in direction** — it was
already partial (29 evidenced / 42 unanalysed) — but its denominator is now
known to be a subset, so 29/71 is a ratio over the compiler scope only and
must not be quoted as a corpus figure.

**What survives unchanged.** The 13 frame-escape sort, the 2 contract-test
candidates (`src/v1/04_infer.dag` `peel_alias_once_for_field_access` inside
`expand_alias_chain_for_field_access`; `src/v2/lens/vacuity.dag`), and the
calibration receipt are all statements about operations that do exist and were
analysed. They are narrowed by this correction, not falsified — with the
caveat that the three v1 files they were read in (`05_emit_rust.dag`
+1076/-217, `04_infer.dag` +197/-12, `04_types.dag` +80/-20) differ from main,
so their line-anchored positions need re-derivation before citation.
`src/v2/compiler/03_normalize.dag` and `src/v1/04_env.dag` are byte-identical
to main and need no re-derivation.

**Class.** This is OBSERVED ON vs CLAIM ABOUT: the measurement named its
subject, the verdict did not, and "CLOSED" is precisely the word that carries
a scope claim it never earned. A completeness verdict must state the
population it closed over in the same sentence that closes it.

The reusable form, because this class has a specific shape rather than being
general carelessness: **any search-based completeness claim has two independent
completeness questions, and only one of them is usually measured.** Did the
search exhaust the found set, and did the finding exhaust the population. The
first is the one an executed probe naturally answers; the second is the one
the word "closed" is heard as answering. When only the first is measured, the
verdict is not wrong — it is a true statement about a set whose boundary was
never established, and it should be written with that boundary in its own
sentence.

**Also recorded: the convenient explanation was refuted rather than used.** The
audit tree was 171 commits stale, which is a true fact, an available cause, and
the wrong one. Had it been accepted, the repair would have been a rebase and
re-run, which corrects 1 of 6 and reproduces the same scope gap on the next
pass. The check that separated them — were the missed operations present at the
audit's own head — cost one command and inverted the remedy.

— smart-ram-730
