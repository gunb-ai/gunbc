# CI invocation fixed tax — measurement receipt for the selection-control demotion

**Status:** measurement receipt, condensed 2026-07-30 on operator review. **No behavior changes in
this note.** Its sibling commit demotes the affected-set selection control to the falsifier cadence
— a scheduling change, not a fix to anything measured here. DESIGN.md and the `.dag` carriers remain
authority; `gunbc_ci_selection_control_step_note` carries the demotion's rationale and return
trigger.

**Product:** the measured basis for that demotion, and the root cause of the invocation cost that
makes a hermetic ~80-second local suite cost ten minutes on the fleet.

**Scope discipline (operator ruling 2026-07-30).** This note had grown to 592 lines across six
investigation passes inside a behavior PR, and its ranked-lever table had gone circular: it named
byte-cursor string work as lever 1 while its own later sections had both located the real root in
one unmigrated frontend function *and* explicitly accepted "frontend construction, not byte-cursor
ops" as the correct lever name. Both are corrected below, in place. **This note does not grow
further in this PR** — follow-on work is named, not ranked as actionable. The full six-pass
investigation remains in this file's git history.

---

## 1. The receipt — a gate that spends 108 seconds to do 0.09 seconds of work

Run `30485116707`, ci job `90691769205`, runner `srv3-02`, green. Step: `git fetch --no-tags origin
main && gunbc run --source-root dag --entry dag/tools/merge_admission_gate.dag --function main`.

| interval | wall | what |
|---|---:|---|
| `20:19:29.01 → 20:20:07.80` | **38.79s** | before `resolved 159 sources` — source load |
| `20:20:07.80 → 20:20:12.08` | 4.28s | `compile.frontend` 1.31 + `normalize` 0.08 + `reconcile` 2.84 + `analyses` 0.05 |
| `20:20:12.08 → 20:21:16.97` | **64.89s** | gap — compile finished, declaration not yet started |
| `20:21:16.97 → 20:21:17.06` | **0.09s** | `decl=tools.merge_admission_gate::main#whole` → `ExitSuccess` |
| | **108.1s** | total |

`merge_admission_gate.dag` is 84 lines: read one receipt, resolve one `git merge-base` tree hash,
compare two content hashes, exit. **103.7 of the 108.1 seconds precede evaluation.** Ratio of wall
to work: **1200:1**.

## 2. The tax is fixed, not entry-dependent

Two probe modules, `--source-root dag` (1,428 files), 4-core x86 dev box.

- **Probe A** — imports only `std.process`, body `ExitSuccess`, 9 resolved sources: **80.6s**,
  ~100% CPU-bound. Compile is 82ms of it.
- **Probe B** — same body plus an unused `gunbc.merge_admission` import, 144 resolved sources:
  **86.0s**.

A 16× increase in entry closure size (9 → 144 sources) moves the wall ~7%. **The cost is not the
entry.** This is what the demotion rests on: a hermetic suite whose local wall was ~80s measured
**10m09s median on the per-PR ci job** (n=6 green runs, range 8m41s–10m32s), because the fleet pays
this fixed tax per invocation under memory pressure at width=1.

## 3. Where it goes — parsing, at a bad constant

| operation | bytes | wall | throughput |
|---|---:|---:|---:|
| read the whole `dag/` corpus (`cat` 1,428 files) | 8,505,296 | 0.044s cold | **193 MB/s** |
| tokenize + parse real corpus content | 1.5–6.2 MB | 1.8–12.5s | **~0.5 MB/s** |
| tokenize + parse dense generated code | 0.22–0.44 MB | 1.34–2.68s | **~0.16 MB/s** |

**Reading the corpus takes 0.044 seconds. Parsing it is ~400–1200× slower.** A competent tokenizer
runs at 50–500 MB/s.

Cost is **linear in total bytes at a bad constant**, not quadratic in file size: holding content
constant (~4,800 `fn` decls, ~270 KB) and varying granularity, 8 files × 600 fns = 2.23s against
800 × 6 = 1.61s — a 30× file-size increase costs ~1.5×.

Two distinct paths, and conflating them was one of this note's own errors (§6.2). **The ordinary
scan path is allocation-bound and linear. The string-literal path is allocation-bound *and*
algorithmically quadratic.** §4 is the latter.

## 4. Root cause — the `.dag` migration is right; one function did not come along

`src/v1/01_tokenize.dag` carries `SourceRef` with a **pre-decoded code-point array**, and the main
scan indexes it directly (`source_code_point`, `source_len`, `source_substring`,
`source_scan_while`). That is the cursor-style, encoding-agnostic design — O(1) per step, no ASCII
special case. **The `.dag` move landed and is correct**, and emission is faithful.

One function stayed on the old interface:

```
fn process_escapes_loop(source: String, pos: Int, acc: List<String>) -> String {
  if pos >= string_length(s: source) { join(acc, separator: "") }
  else { let ch = char_at(s: source, pos: pos) ... list_push(acc, ch) ... }
}
```

It takes `source: String`, not `SourceRef`/`source_chars`. Two defects, per character:

1. `string_length` and `char_at` each rescan the whole literal (`s.is_ascii()`, then byte index or
   `chars().nth(pos)`) ⇒ **O(L²) per literal**; the non-ASCII arm is `chars().nth(pos)`, O(pos).
2. `list_push(acc, ch)` appends into an `im::Vector`, so each push is copy-on-write through
   `Arc::make_mut` — the allocator frames that dominate sampling.

It is called **unconditionally on every string literal**, from all six `scan_string` arms.

**Why this corpus loads it so hard.** 30% of corpus bytes (5,841,277 of 19,732,620) sit inside
string literals; 38 files carry a literal over 2,000 chars, the largest 14,178
(`dag/gunbc/design_document.dag`), then 10,813 (`ci_spec.dag`), 7,752 (`ci_materialization.dag`).
15 of the 20 largest files contain non-ASCII, because they are prose notes with em-dashes. **The
repo's documentation-as-`data ..._note: String` convention is precisely what feeds the unmigrated
path** — including this correction, written into a carrier note.

**Why no wall caught it.** `01_tokenize.dag` appears in no lens roster, so its recursive `list_push`
accumulator never met the `complexity_accumulator_copy` class. The `text_lookup_work_counter`
instrumentation exists and its cost model is honest — it records `take_len` for ASCII and
`start + take_len` for non-ASCII, naming the quadratic directly — but it is behind a cargo feature
enabled only for `src/v1/tests`, and is called from `substring` only, never from `char_at`. **The
counter watches the migrated path; the quadratic lives in the unmigrated one.**

## 5. Corrected lever ordering

The prior ranked table put byte-cursor string realization first. That predated the root-cause
section, and was never re-ordered even after the review correction that explicitly renamed the
lever. The honest ordering:

1. **Frontend construction.** Migrate `process_escapes_loop` onto the `SourceRef`/`source_chars`
   interface the rest of its own file already uses, and replace the `list_push` accumulator with the
   fold idiom. This is the super-linear term; it is one function; the target interface already
   exists. Oracle: byte-identical token streams over the corpus, plus a discriminating case with
   `\x` escapes and non-ASCII. Then point the counter at `char_at` and roster `01_tokenize.dag` in
   the complexity lens, so the next unmigrated interface reds.

   The **full** lever is typed source/token/text construction — `SourceCursor`, `TokenBuilder`,
   `FrozenTokenStream`, structural token text carrying a span rather than an owned `String` — of
   which a byte cursor is one part. `Rc<Token>`, `Rc<ScanResult>`, `make_token`'s text clone, and
   the `Rc<im::Vector>` accumulator are separate allocation sources the `char_at` framing hid.
2. **Merge-admission stamp + gate fusion** — ~108s/run of pure process tax (§1), independent of (1)
   and compounding with it. Implemented via an explicit `OnWalkSuccess` finalizer, not a
   dependency-aware scheduler change.
3. **Cross-process module-path index cache** — sequence *after* (1), or it cements the current
   constant. Key on declared-input content; byte-identical cached-vs-cold is the purity oracle.
4. **Bare-reference census** (`tree_bare_census_for_root`, the `#6848` class) — the deepest
   structural fix, dissolving via the namespace-only resolution lane. (1) shrinks its constant
   without removing the pass.

Byte-cursor realization of the string primitives is **downstream of (1)**, not ahead of it: once the
unmigrated caller is gone, the remaining `char_at` sites sit in the migrated cursor path where they
are O(1), and re-realizing them prices against a different baseline.

On the `is_ascii()` split: it is **not** load-bearing for the ordinary path (0–7% non-ASCII), which
never calls `char_at` — but on the path that does, the penalty is **46% at L=4,000 and 111% at
L=64,000**, growing with literal length as an O(pos) `chars().nth` predicts. Both numbers are
needed; either alone misleads. The conclusion is unchanged: bankrupt the interface rather than
optimize either arm.

## 6. Corrections this note made to its own passes

Recorded rather than silently edited, because load-bearing errors that are only superseded later
still mislead whoever reads the earlier section:

1. **"Not corpus-linear; shape-dependent" — superseded.** That read compared two whole-corpus runs
   whose ~10s variance swamped the effect; the controlled sweep shows linear-in-bytes. The original
   observation (2,677 files measuring faster than 1,428) stands as data, explained by content
   density, not file count.
2. **"`char_at` is quadratic in the ordinary tokenizer scan" — retracted.** All six `char_at` sites
   in `01_tokenize.dag` are `process_escapes_loop` (3) plus `all_hex_upper_in_range` /
   `sentinel_prefix_matches` / `sentinel_suffix_matches`. The ordinary scan converts once through
   `chars(source)` and indexes `SourceRef.source_chars`. The dense-code benchmark has no meaningful
   literal content and still runs ~0.16 MB/s, so `char_at` cannot explain the ordinary path at all.
   Stated because the code still *reads* quadratic and the next reader will form the same hypothesis.
3. **"Allocation-bound, not algorithm-bound" was too broad.** Corrected in §3.
4. **Two probe configurations were measuring a parse abort, not a parse.** `//` comment lines before
   the first item declaration panic `for_each_parsed_module_binding`, returning in ~80ms having
   parsed nothing — briefly appearing to show "comments are cheap" and "non-ASCII is free". Every
   timing above is guarded by an explicit parsed/PARSE-ABORT check. **Any future probe here must
   assert the parse succeeded before its wall clock means anything.**

## 7. Probe discipline, and one contributed split

The same word "resolve" names two different-shaped costs, so every probe must record binary SHA,
execution leg (native vs interpreted), source roots, and corpus identity. A later floor diagnosis
put ~96% of resolve wall in `reconcile_assembly` with the closure loader at ~1ms; a single-entry
cold leg measured here is shaped differently — `load=34651.6ms parse=1400.1ms resolve=24.6ms
typecheck=13077.7ms reconcile_assembly=3252.6ms`, i.e. **load dominates**. Both are real and
separate: **cold process starts are a source-universe problem; pooled floor entries are a
graph-major assembly problem.** Recorded rather than reconciled.

This note's measurements: release `gunbc`/`claim_batch` built from this branch, native leg,
`--source-root dag --source-root src/v2`, 4-core x86 dev box.

## 8. Reproduction

```
# §1 — CI decomposition (any green ci job)
#   read the merge-admission step timestamps: git fetch -> "resolved N sources"
#   -> compile.* -> "started decl" -> ExitSuccess

# §2 — the fixed tax, locally
time ./target/release/gunbc run --source-root dag --entry <probe>.dag --function main

# §3 — throughput: compare `cat` of all dag/*.dag against a parse over the same
#   bytes, asserting the parse did not abort
```

**Dissolves when** the pre-evaluation whole-corpus passes are either persisted across processes or
derived from the containment tree (namespace-only resolution lane), at which point §5's ordering is
repriced against a new baseline rather than this one.
