# Inner cost lanes — scoping

Scoping only. No code lands from this note. It exists so the two remaining CI-cost lanes can be
picked up without re-deriving what they are, and so the broad "why is CI slow?" investigation stays
closed (operator ruling 2026-07-30: every subsequent PR carries only its own discriminating
before/after receipt).

These are the lanes left after the workflow-graph cleanup. The workflow work removed *duplicated
process startup*; these two are about *the work itself* — what the frontend does per byte, and what
resolve does per entry.

---

## Lane A — frontend construction

**LANDED.** `v1.compiler.tokenize` `process_escapes_loop` now consumes the pre-decoded code points.
The measured result and the two corrections this lane produced are recorded below; the scoping that
follows is kept because Lane B is still open and the two share this note.

**Contained slice: `process_escapes_loop`.**

`v1.compiler.tokenize` `process_escapes_loop`. It was the one function in its file still walking a
raw `String` by index while the rest of the file had migrated to the pre-decoded `SourceRef` /
`source_chars` idiom. That is the whole defect: the surrounding file already had the fix.

### The cost shape, read off the primitives

Both primitives it called are branch-on-ASCII (`v1_rt` `char_at`, `string_length`):

| call | ASCII | non-ASCII |
|---|---|---|
| `char_at(s, pos)` | `is_ascii()` scan then `bytes[pos]`, **O(n)** | `is_ascii()` scan then `s.chars().nth(pos)`, **O(n)** |
| `string_length(s)` | `is_ascii()` scan then `s.len()`, **O(n)** | `s.chars().count()`, **O(n)** |

**Correction (2026-07-31, measured).** An earlier version of this table read the ASCII column as
O(1) and concluded "ASCII input: O(n), non-ASCII input: O(n²)". That is wrong, and the error was in
reading the fast path as free: both primitives *begin* with `s.is_ascii()`, which scans the whole
string on **every call**, so the ASCII branch is O(n) per call too. A per-character loop over either
primitive is therefore quadratic **regardless of encoding**. Measured directly on `v1_rt::char_at`
over a pure-ASCII string, per-character loop: n=4,000 1.50ms, 8,000 5.66ms, 16,000 22.03ms, 32,000
86.88ms — ~3.9× per doubling. Non-ASCII is not a different asymptotic class, only a worse constant
(`nth` walks as well). This matters beyond this slice: it means every surviving `char_at`-indexed
loop in the corpus is quadratic, not just the ones handling non-ASCII text.

`process_escapes_loop` called `string_length` on the *same unchanging source* on every iteration —
several times per iteration across its branches — and `char_at` at up to four offsets per iteration.

Second, smaller correction: the defect was not confined to `process_escapes_loop`. `scan_string_body`
had already walked `source_chars` correctly, then `join`ed the characters into a `String` **purely so
that `process_escapes` could re-split it** — a decompress→recompress round trip across the
`StringScanResult` boundary. The fix had to cross that boundary to be real.

### The fix

Migrate the loop onto `source_chars` (the pre-decoded array the rest of the file already builds), so
both `char_at` and `string_length` disappear from the escape path entirely, and drop the
`Rc<Vec<String>>` accumulator. That removes the quadratic *and* the per-character allocation in one
motion — the accumulator-copy class DESIGN §6 already rules is always fixed regardless of realized n,
because "n is small here" is not a time-stable fact.

Landed shape: `StringScanResult.content` carries `List<Int>`, so `scan_string_body` stops joining and
`process_escapes` stops re-splitting; the accumulator is code points converted once by
`chars_to_string`; the escape table compares code points, which is what the rest of the file already
did (`ch == 61 && next_ch == 62` for `=>`).

**Measured, one string literal, non-ASCII with escapes (before → after):**

| literal chars | before | after | speedup |
|---|---|---|---|
| 2,769 | 7.79 ms | 0.90 ms | 8.6× |
| 11,019 | 41.34 ms | 1.44 ms | 28.7× |
| 44,019 | 583.9 ms | 5.95 ms | 98.1× |

The speedup grows with n because the quadratic term is gone: after the change, 2× the input costs
2.01× then 2.05× the time.

### Oracle

Byte-identical token streams over the corpus is the *equivalence* half. It is not sufficient on its
own — it is satisfied by not changing anything — so it needs a discriminating input beside it: a long
string literal with non-ASCII content and escapes, which separates the two implementations
asymptotically, plus a RED that a wrong escape decode changes the output. Both halves execute, per
DESIGN §5: a typecheck and a grep are not consumers.

Discharged by `src/v1/stage0/tests/tokenize_escape_receipt.rs`, executed on both sides of the change:

- Equivalence, corpus grain: `regen_stage0 --verify` reports `regen_divergence_count=0`. Regen
  re-tokenizes all of `src/v1` and `dag` and emits the seed; a byte-identical seed across a
  2-generation fixed point is the corpus token-stream equivalence, established by execution rather
  than asserted.
- Equivalence, decode grain: `escape_decode_table`, `malformed_hex_escape_declines_rather_than_fabricating`,
  `unknown_escape_passthrough_is_retained` — green against the pre-migration seed *and* after.
- Separation: `escape_cost_is_linear_in_literal_length` asserts a **ratio** (4× input must not cost
  ≥8× time) so it does not encode one machine's speed. Against the pre-migration seed it reds at
  **14.2×**; after, it passes. That RED was observed by running it, not predicted.

  It ships **`#[ignore]`d — a benchmark, not a gate** (review 45416). A wall-clock assertion can
  fail correct code when the larger run is the one that catches contention, and gating correctness
  on timing is against hermetic-first test discipline. The deterministic alternative does not
  rescue it: the tree's only work counter (`v1_rt::take_text_lookup_chars_walked`) is behind the
  non-default `text_lookup_work_counter` feature and does not instrument `char_at`/`string_length`
  at all, so a counter-based test would be `#[cfg(feature = ...)]` and equally non-gating while
  additionally changing a core primitive.

  The distinction that matters: the oracle was **discharged by execution** in the landing PR — red
  observed before, green after — which is what DESIGN §5 asks for. What is deferred is the standing
  regression *guard*, and the repo's native form for that is a structural lens over the `Node` tree,
  as `v2.lens.complexity_accumulator_copy` is for the copied-accumulator class. Such a lens would
  also cover the ~30 raw-index sites the audit below found, which a per-function test never could —
  so it is the better instrument, not merely the substitute.

### Beyond the slice

The named generalization is `SourceCursor` / `TokenBuilder` / `FrozenTokenStream` and enforceable
generated-boundary cost rules — the point being that a *rule* makes the next such function unwritable
rather than found later. That is downstream of the slice, not a prerequisite for it, and it should be
priced against a second measured instance rather than authored speculatively (the purity trap, §6).
Task #3 tracks auditing the v2 parser for the same class; if it is clean, the generalization is worth
less and should say so.

**Audit result (task #3, 2026-07-31): the v2 parser is clean, and the generalization is worth less
than it looked — but the class is not extinct, and it is bigger than the parser.**

`v2.compiler.tokenize` and `v2.compiler.parse` contain **zero** `char_at` / `string_length`
occurrences. v2's `String` is `FreeMonoid<Char>`, a cons list; the tokenizer consumes it through
`fold_source` / `string_head` / `string_tail`, and `list_tail` is `Cons { tail: t } => TailFound`,
i.e. O(1) with structural sharing. There is no position to re-index and no length to re-count, so the
defect cannot be written in that shape. A `SourceCursor` abstraction would therefore be modelling a
problem v2's carrier already dissolved — it should not be authored for the parser's sake.

What the audit *did* find is that the raw-index idiom survives at 28 sites across `src/v2`, none of
them in the parser. The concentration is in shell-emission validators —
`v2.compiler.emit_orchestration` (`char_at` loops over a path, a binder, a test operand) and
`extdeps.languages.bash_orch_if` — each pairing a per-character `char_at` loop with a hoisted
`string_length`. Under the corrected cost table above these are quadratic, not linear: the earlier
reading would have excused them as "ASCII, so O(1) per call", which is exactly the reasoning the
measurement refutes. `v1.compiler.tokenize` itself still has two more in `sentinel_prefix_matches` /
`sentinel_suffix_matches`.

None of these were touched here: they are short-input call sites, so the realized cost is small
today, and DESIGN §6's bare-minimum-cost rule says a proven cost-shape defect is fixed regardless of
realized n — which makes them real work, not non-work. They are named rather than folded in because
the honest lever is now a different one: **the cheapest fix is `char_at` / `string_length`
themselves.** Removing the per-call `is_ascii()` scan (or caching the decode) makes all 30 sites
linear at once, without an abstraction and without touching 30 call sites — a `v1_rt` change whose
authority is `v1.runtime_rust`. That is the second measured instance the "price it against one"
condition asked for, and it points at the primitive rather than at `SourceCursor`.

---

## Lane B — union resolve

**This is a program, not a PR.** Saying so is the useful part of the scoping.

### The measurement

One entry resolve, measured this session on `dag/tools/floor_effect_gate_witness.dag`:

```
[resolve] 107422ms (528 modules, 12067 resolved items in closure)
[resolve-split] load=38491.9ms parse=5026.3ms resolve=184.0ms normalize=331.3ms
                typecheck=74677.6ms parent_envs=7.4ms reconcile_assembly=4418.3ms
```

**These are inclusive counters, not an exclusive partition, and the difference matters before anyone
ranks work by them.** `load + typecheck` alone is 113169.5ms against a 107422ms total, and all
displayed components sum to roughly 115% — so the phases overlap or nest, and no component's share
of the total can be quoted as its exclusive cost. An earlier draft of this note did exactly that
("typecheck is 69%, load is 36%"); those figures are withdrawn.

What survives the correction is the ordering claim, which does not depend on an exclusive partition:
the step literally named `resolve` is 184ms against a 107422ms whole, so whatever the overlap
semantics, it is not where the time goes. The cost is in loading and typechecking, and the lane name
is a nickname for the whole entry-graph construction. Anyone attacking "resolve" by its name will
optimize the wrong thing.

Deriving an exclusive partition — or documenting which counters nest inside which — is the first
thing the measurement slice below should produce, because the union case has to be priced against
real per-phase attribution rather than against inclusive totals.

### The claim

The floor reconstructs a `ResolvedGraph` **per entry**. The affected-set corpus batch selects
hundreds of entries whose closures overlap heavily — 528 modules for one entry, against a corpus
where a typical main-push selects ~600 entries. Resolving the *union* of the selected entries once,
then serving each entry from it, is the same work performed once rather than per-entry.

### Why it is a program

It is not a local optimization with a local oracle:

- It changes the retention profile, which is the axis the floor already fails on. DESIGN's v1
  run-stability thread records the root cause as *retention, not footprint* — multiplicative
  inductive-field duplication along the import DAG — and the eviction work (M2) is explicitly shelved
  behind measured triggers. A union resolve holds *more* live at once by construction, so it collides
  with exactly the constraint that is already binding.
- Its acceptance is fleet memory, not wall time: same verdicts, no material increase in cgroup peak,
  hard backoff, or throttle-wall, at the arm64 slot's real budget. That verdict cannot be produced in
  a dev container with a different memory regime.
- It interacts with the shared-index and memo work already in flight (`walk_memo`, the entry-closure
  memo landed at #6999), which serve part of the same purpose and must not be forked.

### Sequencing

Do not open this until the floor's retention story has a measured floor to build on. The honest first
step is a *measurement*, not a change: instrument how much of the per-entry resolve cost is genuinely
shared across the selected set, so the prize is known before the architecture moves. If the overlap
is smaller than assumed, the program is worth less and the memo lane already captured most of it.

---

## What is deliberately not here

No ranked lever table. The previous attribution document's ranked table went circular — it named
byte-cursor work as lever 1 while its own later sections had located the root in frontend
construction — and that circularity is why the investigation was closed. These two lanes are named,
priced, and sequenced; the next receipt should come from a PR that changes one of them, not from
another census.
