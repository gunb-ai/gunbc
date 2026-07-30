# CI invocation fixed tax — the pre-evaluation whole-corpus passes

**Status:** measurement receipt, 2026-07-29. **No behavior changes in this note** (its sibling
commit demotes the affected-set selection control; that is a scheduling change, not a fix to
anything measured here). Prose + reproduction only; DESIGN.md and the `.dag` carriers remain
authority.

**Product:** an attribution of where a `gunbc run` / `claim_batch` invocation actually spends its
wall clock, measured on the live fleet and re-derived locally by stack sampling. The headline is a
single number: **the merge-admission gate spends 108 seconds to perform 0.09 seconds of work.**

**Dissolves when** the pre-evaluation passes below are either persisted across processes or
derived from the containment tree (namespace-only resolution lane), at which point the levers in
§6 are priced against a new baseline rather than this one.

---

## 1. Why this note exists (and what it corrects)

[ci-floor-time-45-72-band-attribution.md](ci-floor-time-45-72-band-attribution.md) §9 named the
class **cold-index-per-PROCESS** and proposed per-gate pooled children as the fix. That fix
**landed** — `dag/tools/host_prelude.dag` `run_gunbc_claims` now takes `claims: List<ClaimRun>`
and passes the whole list to ONE `claim_batch` invocation via `claim_batch_claims_argv`. The
"~25 serial cold children" mechanism is **historical**; do not price levers against it.

That audit's own closing lesson — *"receipt freshness applies at the DIAGNOSIS grain, not only
dispatch"* — is the reason this note re-measures rather than re-cites. Its **rank-1 lever was
already marked STALE** and its §4 asked for a re-diagnosis of the discovery walk. §4 below is
that re-diagnosis.

---

## 2. The receipt — merge-admission gate, decomposed to the second

Run `30485116707`, ci job `90691769205`, runner `srv3-02`, green. The step is
`'git' 'fetch' '--no-tags' 'origin' 'main' && 'env' '-C' '.' 'target/release/gunbc' 'run'
'--source-root' 'dag' '--entry' 'dag/tools/merge_admission_gate.dag' '--function' 'main'`.

| interval | wall | what |
|---|---:|---|
| `20:19:29.01 → 20:20:07.80` | **38.79s** | before `resolved 159 sources` — source load |
| `20:20:07.80 → 20:20:12.08` | 4.28s | `compile.frontend` 1.31 + `normalize` 0.08 + `reconcile` 2.84 + `analyses` 0.05 |
| `20:20:12.08 → 20:21:16.97` | **64.89s** | **gap** — compile finished, declaration not yet started |
| `20:21:16.97 → 20:21:17.06` | **0.09s** | `decl=tools.merge_admission_gate::main#whole` → `ExitSuccess` |
| | **108.1s** | total |

`merge_admission_gate.dag` is 84 lines. Its `main` reads one receipt file, resolves one
`git merge-base` tree hash, compares two content hashes, and exits. That is the 0.09s. Everything
else is tax, and **103.7 of the 108.1 seconds is spent before the declaration begins evaluating.**

Ratio of wall to work: **1200:1.**

---

## 3. The tax is fixed, not entry-dependent (local probes)

Two probe modules, `--source-root dag` (1,428 `.dag` files), 4-core x86 dev box.

**Probe A** — imports only `std.process`, body is `ExitSuccess`:

```
resolved 9 sources
✓ compile.frontend 26ms   ✓ compile.normalize  3ms
✓ compile.reconcile 52ms  ✓ compile.analyses   1ms
ExitSuccess
real 1m20.574s   user 1m17.661s
```

**80.6 seconds, ~100% CPU-bound, to do nothing.** Compile is 82ms of it.

**Probe B** — same body, but additionally imports `gunbc.merge_admission { current_gate_roster_hash }`
(and never uses it): 144 resolved sources, **86.0s**.

**Control — same probe, wider source roots:** `--source-root dag --source-root src/v2` (2,677
files) measured **68.0s**, *faster* than the 1,428-file run's 77.2s on a repeat. So the tax is
**not linear in corpus file count**; run-to-run variance is ~10s and closure shape dominates.

Reading: a 16× difference in entry closure size (9 → 144 sources) moves the wall by ~7%. The cost
is not the entry.

---

## 4. Where it goes — live stack sampling

`gdb -p <pid> -batch -ex "bt 40"`, 24 samples at 2.5s intervals against Probe A. Every non-idle
sample landed in one of two **whole-corpus passes**, both running before the entry's 9 sources are
touched:

| pass | share of identified samples | stack |
|---|---:|---|
| **module-path index** | ~40% | `build_module_path_index_from_witness_roots` → `build_module_path_index_uncached` → `module_path_index::index::parse_module_binding` → `tokenize` |
| **bare-reference census + fn-sig inference** | ~55% | `load_sources_for_entry_with_pool` → `extend_sources_to_both_closure_fixpoint` → `both_closure_edge_index` → `build_both_closure_edge_index` → `tree_bare_census_for_root` → `pool_parse` → `tokenize`; and `build_symbol_index_census_nodes` → `census_with_resolved_fn_sigs` → `census_upgrade_binding` |
| reconcile (typed cache) | ~6% | `resolved_graph_from_sources_with_index` → `reconcile_with_typed_cache` → `build_symbol_index_qualified_fill` |

Two facts worth stating plainly:

1. **`parse_module_binding` runs the full tokenizer over every `.dag` file to read one header
   line** (`module x.y`). Sampled frames include `scan_ident`, `is_reserved_emit_sentinel`,
   `sentinel_prefix_matches`, `build_newline_index` — full lexical work per file, to extract a
   binding.
2. **`tree_bare_census_for_root` parses the whole tree again** for the bare-reference edge index,
   and drags fn-signature inference (`census_with_resolved_fn_sigs`) across it. This is the
   `#6848` bare-reference fixpoint. It is **still live** and it is a **whole-tree parse plus
   inference per invocation**.

Both caches are process-local (`MODULE_GRAPH_FACTS_CACHE` is a `thread_local!`;
`build_module_path_index` memoizes inside `LocalKey`), so they amortize *within* one process and
are paid in full by every new one.

**Scoping honesty:** the sampling above is on the local dev box. The CI decomposition in §2 shows
the same *shape* (38.8s pre-`resolved`, 64.9s post-compile pre-eval) but no sampler ran on the
runner, so the CI attribution to these two specific passes is **by analogy, not by measurement on
that host.** Confirming it needs a `perf`/gdb sample on a fleet host.

### 4.1 The passes are not the defect — the throughput is (2026-07-29, second pass)

Naming *which* passes run is not the same as explaining why they are slow, and the number that
matters is throughput. Measured on the same box:

| operation | bytes | wall | throughput |
|---|---:|---:|---:|
| read the whole `dag/` corpus (`cat` all 1,428 files) | 8,505,296 | **0.044s** cold, 0.023s warm | **193 MB/s** |
| tokenize + parse real corpus content (isolated source root, count swept 25→400 files) | 1.5–6.2 MB | 1.8–12.5s | **~0.5 MB/s** |
| tokenize + parse dense generated code (`fn` decls only, no comments) | 0.22–0.44 MB | 1.34–2.68s | **~0.16 MB/s** |

**Reading the corpus is 0.044 seconds. Parsing it is ~400–1200× slower than reading it.** A
competent tokenizer runs at 50–500 MB/s; this one runs at 0.16 MB/s on dense code.

Three hypotheses tested and **refuted**:

1. **Quadratic in file size — NO.** Holding total content constant (~4,800 `fn` decls, ~270 KB)
   and varying granularity: 8 files × 600 fns = 2.23s; 24 × 200 = 1.86s; 80 × 60 = 1.57s;
   240 × 20 = 1.45s; 800 × 6 = 1.61s. A 30× file-size increase costs ~1.5×, not ~900×. The cost
   is **linear in total bytes**, at a bad constant.
2. **Linear in file *count* — NO.** It tracks bytes, not files.
3. **The non-ASCII `is_ascii()` fallback — NOT DOMINANT.** Identical generated corpora differing
   only in one marker character inside a string literal (`-` vs `—`): 40×100 fns, ASCII 1.339s vs
   non-ASCII 1.264s; 20×400 fns, ASCII 2.681s vs non-ASCII 2.857s. The penalty is **0–7%**, not
   orders of magnitude. (35% of real `.dag` files contain non-ASCII, so the path is well
   exercised — it is just not where the time goes.)

**What it actually is: allocation.** 30 innermost-frame samples (`gdb -ex "bt 1"`) during a
whole-corpus parse: **15 of 18 resolved samples (~83%) were in the allocator or `memcpy`** —
`_int_malloc`, `__GI___libc_malloc`, `_int_free`, `__GI___libc_free`, `tcache_get_n`,
`malloc_consolidate`, `checked_request2size`, `__memcpy_avx512_unaligned_erms`, and
`alloc::sync::Arc<T,A>::make_mut`. The tokenizer is **allocation-bound, not algorithm-bound.**

The mechanism is visible in the seed runtime's interface, not its logic:

```rust
pub fn char_at(s: &str, pos: i64) -> String { ... String::from(bytes[pos] as char) ... }
```

`char_at` **returns a heap-allocated `String` per character**, and `string_length` /
`char_at` are called per character from `tokenize_loop`/`scan_next_token`. Add `im::Vector`
persistent collections with `Arc::make_mut` copy-on-write (sampled in `build_newline_index`,
`scan_string_body`) and every scanned character costs allocations rather than a pointer bump.

This is the **model↔realization fork** DESIGN already names, at the string layer: the modeled
value semantics (immutable `String` values, persistent vectors, character-ordinal indexing)
were transliterated into the Rust seed instead of *realized* as native byte-slice cursor
operations. The grounded operation for a tokenizer is "advance a byte cursor over a UTF-8 slice",
which is O(1), allocation-free, and **encoding-correct for ASCII and non-ASCII alike** — so the
right fix deletes the `is_ascii()` branch rather than optimizing either of its arms.

### 4.2 The `is_ascii()` fallback is a fallback pattern, and should be deleted on those grounds

Independent of its ~7% cost, the shape is the one DESIGN §5 forbids, in its performance register:

```rust
if s.is_ascii() { /* O(1) byte index */ } else { /* O(n) chars().nth(pos) */ }
```

It **degrades silently on content**. Nothing is typed, nothing is counted, nothing is located —
so the frequency of the slow arm is zero by construction, it never ranks for fixing (§6 prices by
displaced cost, and a masked cost displaces nothing), and the fact that 35% of the corpus takes
the slow arm was not knowable before this measurement. It is also a **second representation** of
"index into a string": two code paths that must agree, where the byte-cursor model needs one.
There are **11 `is_ascii()` branch sites in the seed**, 8 of them in `v1_rt.rs`
(lines 39, 253, 267, 280, 561, 584, 607, 631) — a repeated pattern, not a one-off.

Priced honestly: deleting the fallback buys ~7%; deleting *what it is a fallback for* — the
`String`-per-character interface — is the ~400× lever. They are the same edit.

---

## 5. The same disease at the per-entry grain (floor discovery)

Same job log, discovery batch. Per affected entry, the repeating cycle is:

```
20:16:17.879  [floor-drain] schedule-retention ARMED: entries=1 modules_refcounted=309
20:16:51.403  ✓ floor_disc_witness_transitive_holds (...leg=InterpretedLeg) 0.1ms
```

**33.5 seconds** between arming and verdict. Measured across four consecutive cycles in this log:
33.52s, 33.73s, 33.23s, 33.6s — tight. Per cycle the compile phases total **~0.64s**
(`frontend` ~190ms + `normalize` ~33ms + `reconcile` ~400ms + `analyses` ~11ms), and the witness
evaluation itself is **0.1–0.5 milliseconds**.

So inside the floor process the ratio is ~33s of resolution per ~0.3ms of witness. This is the
same pre-evaluation tax, paid per entry instead of per process, and it is what the audit doc's
"~971ms/group" figure has grown into on the current tree for *affected* (non-skipped) entries.

---

## 6. Ranked levers (priced against §2/§3, not the stale audit)

Ordering correction (2026-07-29, after §4.1): **caching the passes was the wrong first lever.**
A cache would preserve a 0.16 MB/s parser and hide it behind a hit rate — the §5 shape where a
mechanism stays green by not doing the work rather than by doing it correctly. DESIGN §6's
standing **bare-minimum-cost** rule is explicit that a proven cost-shape defect is *always* fixed
regardless of realized n. §4.1 is that proof. Fix the constant first; then cache what remains,
against a baseline worth caching.

| # | lever | displaces | risk | notes |
|---|---|---|---|---|
| 1 | **Realize string primitives as byte-cursor ops** — delete `char_at -> String`, `string_length`'s rescan, and the 11 `is_ascii()` branches | targets the ~400–1200× gap; ~83% of samples are allocator | medium | the model↔realization fork at the string layer. Oracle: byte-identical token streams on the corpus, plus a non-ASCII discriminating case. Encoding-correct by construction, so the fallback disappears rather than being optimized. |
| 2 | **Fuse merge-admission stamp + gate** | ~108s/run today, ~2×83s of process tax | low | independent of lever 1 and compounding with it; after lever 1 both approach seconds. |
| 3 | **Cross-process module-path index cache** | ~40% of what remains *after* lever 1 | low | pure function of file contents ⇒ content-addressable. DESIGN §5 cache-impurity rule: key on declared-input content; byte-identical cached-vs-cold is the purity oracle. **Sequence after lever 1**, or it cements the current constant. |
| 4 | **Bare-reference census (`tree_bare_census_for_root`)** | ~55% of every invocation, and the §5 per-entry walk | high | the `#6848` class. Dissolves via the namespace-only resolution lane (`build_module_path_index` becomes a projection of `v2.compiler.source_authority`). Still the deepest structural fix; lever 1 shrinks its constant without removing the pass. |
| 5 | **`regen` → `ci` `needs:` edge** | ~8.5 min of PR critical path | low | independent of the above; `ci` consumes only `build`'s artifact. Counter-argument: it is a deliberate fail-fast before the 34-min floor, and unsequencing it starts two memory-heavy cold processes concurrently on a fleet already at width=1. |

Note the interaction: **every lever multiplies with invocation count**, so reducing process count
is only worth doing once the per-process constant is fixed, or it optimizes the wrong term.

---

## 7. Workflow-level timings this note was derived from

359 `ci.yml` `pull_request` runs across 37 branches, 2026-07-29 05:07–22:32 UTC, plus 30 `main`
push runs. 8 runs instrumented at step grain.

| | n | median | range |
|---|---:|---:|---|
| green PR run | 31 | **39.5 min** | 32.5–52.6 |
| `main` push run | 25 | 48.4 min | 44.9–54.3 |
| failed run | 81 | 9.6 min | p90 34.6 |
| cancelled (superseded) | 230 | 10.8 min | — |

Critical path is `build → regen → ci` (`heal_generated_artifacts` runs parallel off `build` and
never gates; `deploy_dashboard_srv1` is skipped on PRs). Step medians across the 6 green
instrumented runs:

| job / step | median | min | max |
|---|---:|---:|---:|
| **build** job | 2m44s | 1m04s | 7m54s |
| **regen** job | 8m33s | 1m39s | 9m19s |
| *heal* (parallel) | 2m20s | 2m13s | 2m37s |
| **ci** job | **33m56s** | 29m23s | 36m23s |
| ↳ `gunbc ci` floor | 21m44s | 18m53s | 23m58s |
| ↳ affected-set selection control | 10m09s | 8m41s | 10m32s |
| ↳ merge-admission gate | 1m52s | 1m35s | 1m57s |
| ↳ setup + checkout + artifact | 0m11s | — | — |

The `ci` job is 75% of the critical path; setup/checkout/artifact overhead is 11 seconds total.
There is no scaffolding to trim — the wall is the invocation tax in §2–§5.

Two workflow-level observations recorded without being acted on here: **230 of 359 runs (64%)
were cancelled by supersede, burning 2,514 of 5,007 runner-minutes (50.2%)**, median survival
10.8 min into a ~40-min pipeline; and one branch consumed 58 runs in the window. Separately,
`gunbc-ci-auto-heal` pushed 8 regeneration commits and 85 skew-remedy merge commits across all
954 remote branches in 3 days — each push restarts CI on its branch. Neither is an invocation-tax
problem; both are workflow-shape questions.

---

## 8. Reproduction

```bash
# §2 — CI decomposition (any green ci job)
#   read the merge-admission step timestamps: git fetch -> "resolved N sources"
#   -> compile.* -> "started decl" -> ExitSuccess

# §3 — the fixed tax, locally
cargo build --release --bin gunbc
mkdir -p dag/zzprobe && cat > dag/zzprobe/min.dag <<'EOF'
module zzprobe.min
import std.process { ProcessExit, ExitSuccess }
func main() -> ProcessExit { ExitSuccess }
EOF
time ./target/release/gunbc run --source-root dag --entry dag/zzprobe/min.dag --function main
rm -rf dag/zzprobe   # do not leave probes in tree

# §4 — attribution
./target/release/gunbc run --source-root dag --entry dag/zzprobe/min.dag --function main & PID=$!
for i in $(seq 1 24); do gdb -p $PID -batch -ex "bt 40" 2>/dev/null \
  | grep -oE 'build_module_path_index_uncached|tree_bare_census_for_root|census_with_resolved_fn_sigs'; sleep 2.5; done
```

---

## 9. Provenance

- 2026-07-29, session `pr-timing-analysis`. Workflow/step timings from the GitHub Actions API
  (359 runs; 8 job-level pulls). §2 from job `90691769205` log, re-derived to the second.
  §3/§4 measured locally by execution and by gdb stack sampling.
- Corrects [ci-floor-time-45-72-band-attribution.md](ci-floor-time-45-72-band-attribution.md)
  §9 on the child-spawn mechanism (pooling landed) and supplies the §4 re-diagnosis that doc
  asked for.
- Related: [floor-shared-compute-memoization.md](floor-shared-compute-memoization.md),
  [namespace-resolution-design.md](namespace-resolution-design.md),
  [v1-run-stability-throughline.md](v1-run-stability-throughline.md).

---

## 10. Corrections to this note's own first pass

Recorded rather than silently edited, because two of them were load-bearing:

1. **"Not corpus-linear; shape-dependent" (§3) — superseded.** That read came from comparing two
   whole-corpus runs whose variance (~10s) swamped the effect. The controlled sweep in §4.1 shows
   cost is **linear in total bytes** at ~0.16–0.5 MB/s. The original observation (2,677 files
   measuring faster than 1,428) stands as *data* and is explained by content density, not file
   count.
2. **A quadratic-tokenizer hypothesis — refuted by execution.** `char_at`'s per-call
   `s.is_ascii()` and `chars().nth(pos)` look O(n²) in file size. The constant-total sweep says
   otherwise: 30× file size costs 1.5×. Stated because the code still *reads* quadratic and the
   next reader will form the same hypothesis.
3. **Two probe configurations in an earlier draft were measuring a parse abort, not a parse.**
   `//` comment lines placed before the first item declaration panic
   `for_each_parsed_module_binding`, so those runs returned in ~80 ms having parsed nothing, and
   briefly appeared to show "comments are cheap" and "non-ASCII is free". Every timing in §4.1
   is guarded by an explicit parsed/PARSE-ABORT check. Any future probe here must assert the
   parse succeeded before its wall clock means anything.

---

## 11. Root cause: the `.dag` migration is right, one function did not come along

§4.1's "linear, not quadratic; non-ASCII costs ~7%" is **superseded**. Those probes used generated
files containing *no string literals*, which exercises only the migrated scan path. Re-measured
with one string literal per file, 20 files, warmed:

| literal len | bytes | ASCII | KB/s | non-ASCII | KB/s |
|---:|---:|---:|---:|---:|---:|
| 4,000 | 80,953 | 0.126s | 627 | 0.184s | 430 |
| 8,000 | 160,953 | 0.304s | 517 | 0.506s | 311 |
| 16,000 | 320,953 | 0.851s | 368 | 1.422s | 220 |
| 32,000 | 640,953 | 2.605s | 240 | 5.462s | 115 |
| 64,000 | 1,280,953 | 9.360s | 134 | 19.705s | 63 |

Per-doubling ratios approach 4× (2.41 → 2.80 → 3.06 → 3.59): **quadratic in string-literal
length**, with non-ASCII a consistent **~2.1×** on top. Throughput collapses 627 → 134 KB/s.

### The model is not the problem

`src/v1/01_tokenize.dag` (483 lines, `module v1.compiler.tokenize`) carries `SourceRef` with a
**pre-decoded code-point array**, and the entire main scan indexes it directly:

```
fn source_code_point(source: SourceRef, pos: Int) -> Int { source.source_chars[pos] }
fn source_len(source: SourceRef) -> Int { count(source.source_chars) }
fn source_substring(source: SourceRef, start: Int, end: Int) -> String { chars_to_string(...) }
fn source_scan_while(source: SourceRef, start: Int, pred: fn(Int) -> Bool) -> Int { ... }
```

That is the cursor-style, encoding-agnostic design — O(1) per step, no ASCII special case. **The
`.dag` move landed and is correct.** Emission is faithful: the model's 6 `char_at` sites map
one-to-one onto the generated `.rs`.

### One function stayed on the old interface

```
fn process_escapes(raw: String) -> String { process_escapes_loop(source: raw, pos: 0, acc: []) }

fn process_escapes_loop(source: String, pos: Int, acc: List<String>) -> String {
  if pos >= string_length(s: source) { join(acc, separator: "") }
  else { let ch = char_at(s: source, pos: pos) ... list_push(acc, ch) ... }
}
```

It takes `source: String`, **not** `SourceRef`/`source_chars`. Two defects, per character:

1. `string_length(s: source)` and `char_at(s: source, pos)` each rescan the whole literal
   (`s.is_ascii()`, then either byte index or `chars().nth(pos)`) ⇒ **O(L²) per literal**, and the
   non-ASCII arm is `chars().nth(pos)` — O(pos), no SIMD — which is the measured 2.1×.
2. `list_push(acc, ch)` appends into an `im::Vector` (`v1_rt.rs` line 6 aliases
   `Vector as Vec`), so each push is copy-on-write through `Arc::make_mut` — the allocator and
   `Arc::make_mut` frames in §4.1's sampling.

It is called **unconditionally on every string literal**, from all six `scan_string` arms.

### Why this corpus loads that one path so hard

**30% of corpus bytes (5,841,277 of 19,732,620) sit inside string literals.** 38 files carry a
literal over 2,000 chars. Worst offenders — all non-ASCII, because they are prose notes with
em-dashes:

| literal | file bytes | file |
|---:|---:|---|
| 14,178 | 96,843 | `dag/gunbc/design_document.dag` |
| 10,813 | 60,549 | `dag/gunbc/ci_spec.dag` |
| 7,752 | 29,352 | `dag/gunbc/ci_materialization.dag` |
| 6,968 | 97,919 | `dag/gunbc/ci_layer_roots.dag` |
| 5,963 | 44,209 | `dag/gunbc/ci_workflow.dag` |

15 of the 20 largest files contain non-ASCII. The repo's documentation-as-`data ..._note: String`
convention is precisely what feeds the unmigrated path, and every whole-corpus pass (§4) pays it
again.

### Why no wall caught it

- `01_tokenize.dag` **appears in no lens roster** — it surfaces only as a `NameResolutionGap`
  frontier row in a plan doc. A recursive `list_push` accumulator is exactly the
  `complexity_accumulator_copy` class, and the lens does not reach this module.
- The `text_lookup_work_counter` instrumentation **exists and is itself emitted from `.dag`**
  (`rt_text_lookup_work_counter`), and its cost model is honest — it records `take_len` for ASCII
  and `start + take_len` for non-ASCII, naming the quadratic directly. But it is behind a cargo
  feature enabled only for `src/v1/tests`, and `record_substring_chars_walked` is called from
  **`substring` only** (`v1_rt.rs:287,291`), never from `char_at`. **The counter watches the
  migrated path; the quadratic lives in the unmigrated one.**

### Fix shape

Give `process_escapes_loop` the same interface the rest of the file already uses — `SourceRef` +
range over `source_chars`, or fold escape handling into `scan_string_body`'s existing single walk
— and replace the `list_push` accumulator with the fold idiom. Oracle: byte-identical token
streams over the corpus, plus a discriminating case with `\x` escapes and non-ASCII. Then point
the counter at `char_at` and roster `01_tokenize.dag` in the complexity lens, so the next
un-migrated interface reds instead of being measured a year later.

This does not touch §4's two whole-corpus passes — the corpus is still parsed twice per
invocation. It removes the super-linear term; levers 3 and 4 remain.

---

## 12. The complexity lens: drivable per-file, infeasible corpus-wide

Asked whether the repo's own complexity lens can be run manually over `src/v1` and `src/v2`
entirely. Mechanically yes; at current cost, no.

### Driving it

`v2.lens.complexity_accumulator_copy.roster_gate` accepts an arbitrary live-tree path:

```
file_gate(path, refusal_ceiling)   file_suspect_count(path)   file_refusal_count(path)
```

driven per the recipe already recorded in `offline_roster_gate_claim_batch_recipe`:

```bash
claim_batch --source-root dag --source-root src/v2 \
  --entry <a *_test.dag naming a test fn over file_suspect_count> \
  --functions <csv> --claim-run --wet
```

The probe module used here was **deliberately not committed**: any `*_test.dag` under a source
root is picked up by CI's discovery walk, so landing it would add live-tree witnesses at
~3 min each to every PR.

### Result (execution)

`file_suspect_count(path: "src/v2/compiler/01_tokenize.dag") == 0` → **FAIL**.

The lens finds copied-accumulator suspects in the v2 tokenizer, confirming §11's static reading of
`lex_repeat_step` / `lex_delimited_step` (`list_append(left: state.lexeme, right: consumed)` once
per character ⇒ O(L²)). **Not obtained:** exact suspect/refusal counts, and the corresponding
`src/v1/01_tokenize.dag` figures — the bracketing run was stopped before completion. Re-run with
the recipe above to pin them.

### Cost, and why the roster is two files

Measured on `src/v2/compiler/01_tokenize.dag` (556 lines): resolve 34,208 ms, **witness
184,828 ms (3 m 05 s)**, total wall 4 m 42 s — to analyze **one** file.

| scope | files | serial extrapolation |
|---|---:|---:|
| `src/v1` + `src/v2` | 1,298 | **~67 h (2.8 days)** |
| whole corpus | 2,726 | ~140 h (5.8 days) |
| `src/v1` + `src/v2`, 16-way | 1,298 | ~4.2 h |

Linear extrapolation **understates** it: cost is superlinear in file size, the mean file is 237
lines, and the largest is `src/v2/std/compilers/target_model.dag` at 12,961 lines — ~500× the
measured file under quadratic scaling.

This is the unstated reason the roster gate is operator-ruled **OFFLINE** with a two-file roster
(`dag/std/change.dag` ceiling 13, `dag/std/render_repeat_string_bootstrap.dag` ceiling 0). Two
files out of 2,726. The scope was cut to what the cost allowed, and that reduction is not
legible as a coverage gap anywhere in the tree.

### The self-referential finding

`ingest_findings(path)` parses its target with the **v2 parser, interpreted under v1** — so every
file the lens audits pays *both* quadratics at once: v1's `char_at` ordinal indexing (§11) and
v2's `list_append` accumulator. **The complexity lens cannot be run over the corpus because of the
complexity defect it exists to detect.**

That settles sequencing. Fixing the two parser quadratics is not only a CI-time win; it is what
makes whole-corpus complexity enforcement affordable, which is what then polices everything else.
The lens is currently pinned at two files by a cost it is itself designed to flag.

### Two coverage gaps, named

1. **`Unclassifiable` does not gate.** `accumulator_copy_compile_gate` rejects on `Poly2Suspect`,
   but routes `Unclassifiable` refusal causes through the **Accepted** diagnostics channel —
   typed, located, counted, never gating (by design, as the undecidable residue). If
   `list_append(left: acc, …)` is not a registered combiner it lands there, which would explain
   how the v2 tokenizer compiles today while the lens still finds something in it. Which bucket
   it falls in is exactly what the stopped bracketing run would have shown.
2. **`src/v1/*.dag` never reaches the gate.** The gate is enrolled in v2's
   `always_required_root_lenses`, but the v1 seed is compiled by v1 to Rust via `regen_stage0`,
   not through v2's compile door — so `src/v1/01_tokenize.dag`, which carries the §11 quadratic,
   is structurally outside enforcement. Hypothesis from module topology, not yet executed.
