# Freshness System Overhaul

**Status:** Design
**Date:** 2026-03-04

## What lint_upsert actually is

`lint_upsert` is a pre-DSL monolithic command that conflates three independent concerns:

1. **Generation**: Are generated files current? (codegen, testgen, pragma)
2. **Verification**: Does the codebase compile cleanly? (clippy, test-compile, release-check)
3. **Bookkeeping**: Is the manifest up to date?

The DSL already decomposes concern #1 into independent, composable units. Each generation tool is a `func` in a `.dag` file. Each uses `content_upsert` — the atomic read-compare-write-if-different pattern. Each already reports its own freshness (`written: Bool`, `all_fresh: Bool`). The CI pipeline (`ci.dag`) already composes them with proper ordering via stage dependencies.

But the freshness system ignores all of this. It hashes 645 files into one key, decides "stale", and re-runs everything sequentially. It's a monolithic model that was never migrated to the DSL's compositional framework.

## The content_upsert pattern IS freshness

This is the key insight. Look at what `content_upsert` does:

```dag
pattern content_upsert(content: String, path: String) -> { written: Bool }
  uses fs: Filesystem(mode: ReadWrite)
{
  result = ensure(
    should_act: c => !c.matches,
    check: file_content_matches(path: path, expected: content),
    action: fs.write(path: path, content: content)
  )
  return { written: result.acted }
}
```

It reads the file. Compares content. Writes only if different. Returns whether it wrote. This is a freshness check with a conditional side effect. Running testgen on unchanged inputs reads all ~50 test files, compares them, finds them all fresh, writes nothing, and returns `all_fresh: true`.

**content_upsert is already idempotent and self-checking.** The outer freshness system is a redundant gate in front of tools that already know how to skip unnecessary work.

## What each step actually upserts

| Step | DSL model | Upsert targets | Freshness signal |
|------|-----------|---------------|-----------------|
| codegen | `tools/codegen.dag` | `target/codegen/bin/*/main.rs`, `.stamp` | `ran: Bool` |
| testgen | `tools/testgen.dag` | ~50 `generated_tests_*.rs` files | `all_fresh: Bool` |
| pragma | `tools/pragma.dag` | `clippy.toml`, allowlist, policy (3 files) | per-file `written: Bool` |
| clippy | Not DSL-modeled | Nothing (verification only) | exit code |
| test-compile | Not DSL-modeled | Nothing (verification only) | exit code |
| release-check | Not DSL-modeled | Nothing (verification only) | exit code |

The CI pipeline already knows the correct ordering:

```
ci.dag:
  codegen
    → {pragma, testgen}  (parallel, after codegen)
      → build
        → {test, lint}   (parallel, after build)
```

## Why it's slow: the double-check problem

The freshness system does this:

```
1. Hash 645 files into one key                    (~30ms)
2. Compare to manifest                            (~1ms)
3. If stale: run codegen (3s) → codegen-dag (10s) → testgen (28s) → pragma (1s)
   → clippy → test-compile → release-check
4. Each generation tool internally: read file → compare → skip write if fresh
5. Update manifest with new hash of 645 files     (~50ms)
```

Steps 3 and 4 overlap. The outer freshness system (step 3) decides "stale" and runs testgen. Testgen internally (step 4) reads each file, compares, finds everything fresh, writes nothing. 28 seconds to discover what the outer system already could have known.

The wasted work is in **tool startup**, not in the freshness logic. Each tool invocation costs:
- `cargo run` overhead: ~100-400ms (dependency resolution)
- DSL compilation: parse + typecheck + lower all imported `.dag` files
- Content generation: evaluate the DSL to produce expected content
- Content comparison: read each output file and compare

For testgen, the DSL compilation + content generation dominates (~27s of the 28s total). The actual file comparison is milliseconds.

## The real cost structure

```
testgen (28s total):
  ├── cargo run overhead:     ~200ms
  ├── DSL compilation:        ~15s   (parse + typecheck + lower tools/testgen.dag + all imports)
  ├── Content generation:     ~12s   (evaluate DSL for ~50 targets)
  └── Content comparison:     ~100ms (read ~50 files, compare, all fresh → write nothing)
```

The content comparison (the actual freshness check) is 0.3% of the cost. The other 99.7% is **reproducing the expected content** to compare against. This is the fundamental asymmetry: checking freshness requires regenerating the content, which is expensive.

## The modeling problem

The monolithic `lint_upsert` key conflates two different questions:

1. **"Have my inputs changed?"** — Can be answered in O(N_inputs) with mtime/hash checks. Cheap.
2. **"If inputs changed, what do I need to re-run?"** — Currently answered with "everything." Expensive.

The DSL already provides the information to answer question 2 precisely:

- **Module graph** (from `ModuleGraph` in daglang-resolve): Which `.dag` files does each tool import? If `services/cargo.dag` changed but `tools/pragma.dag` doesn't import it, pragma doesn't need to re-run.
- **Output paths** (from `extract_output_paths()`): What files does each tool produce? These are the content_upsert targets.
- **Pipeline stages** (from `ci.dag`): What ordering constraints exist between tools?

None of this is connected to the freshness system. The freshness system uses a hand-coded file list and a single key.

## What the right model looks like

### Each tool is its own freshness unit

Instead of one `build:lint_upsert` entry covering 645 files, each DSL tool gets its own entry covering only its import closure:

| Tool | Input set (from module graph) | Approx. files |
|------|------------------------------|---------------|
| codegen | `tools/codegen.dag` → `extdeps/gunbc.dag` → `std/resources.dag` | ~8 `.dag` files |
| testgen | `tools/testgen.dag` → `std/patterns.dag` → `std/types.dag` | ~10 `.dag` files |
| pragma | `tools/pragma.dag` → `std/patterns.dag` → `config/clippy_policy.dag` | ~8 `.dag` files |

The module graph gives us the exact file set. Not a glob. Not 645 files. The precise transitive import closure — the files the compiler actually reads.

### The check is: "did any file in my import closure change?"

For testgen with ~10 `.dag` input files:
- stat 10 files: ~0.3ms
- If all mtimes < last run: **fresh, skip** (0.3ms total)
- If any mtime newer: hash the changed files, compare to stored hash
- If hash unchanged (touch without edit): **fresh, skip**
- If hash changed: re-run testgen

This is O(import_closure_size), not O(all_tracked_files). For most tools, the import closure is 5-15 files.

### But the import closure is only .dag files

True. The generation tools also depend on Rust code:
- codegen depends on `core/codegen/src/cli_gen.rs` (the template generator)
- testgen depends on `core/codegen/src/testgen/` (the test generator)
- pragma depends on `core/codegen/src/makegen/shared.rs` (the DSL evaluator)

These Rust dependencies form a second input set. But cargo already tracks whether these crates need recompilation. When `core/codegen/src/cli_gen.rs` changes, `cargo run --bin gunbc-codegen` recompiles the crate (~3s) before running. This is cargo's own freshness system working correctly.

The question is whether the *output* of codegen changes when `cli_gen.rs` changes. Usually yes (template changed → generated code changes). But sometimes no (refactor that doesn't change output). This is where **early cutoff** helps: run codegen, hash its output, if unchanged → downstream tools stay fresh.

### Generation vs. verification: different concerns, different triggers

Generation steps (codegen, testgen, pragma) should run when DSL files change. Their inputs are `.dag` files.

Verification steps (clippy, test-compile, release-check) should run when Rust files change. Their inputs are `.rs` and `Cargo.toml` files.

These are different file sets with different change frequencies. A `.rs` edit should not trigger testgen. A `.dag` edit should not trigger release-check (unless generation changed `.rs` output, in which case cargo handles it).

Currently both sets are merged into one 645-file key. Separating them is the highest-value change.

### Verification is cargo's job, not ours

clippy, test-compile, and release-check are literally cargo commands. Cargo already:
- Tracks per-crate fingerprints
- Skips compilation when inputs haven't changed
- Handles incremental compilation

Running `cargo clippy --workspace` when nothing changed takes ~2s (cargo checks fingerprints, finds everything fresh, exits). The freshness system's attempt to pre-screen this adds overhead (645-file hash) without value — it can't be more precise than cargo's own per-crate tracking.

For interactive tool use, verification should be deferred entirely. The user running `gunbc-gist` doesn't need a clean clippy pass — they need their tool to work. CI enforces verification.

## The abstraction

```
Every DSL tool is a content_upsert composition.
Every content_upsert is self-checking (read → compare → conditional write).
The DSL compiler knows the import closure (precise input set per tool).
The CI pipeline knows the ordering constraints.

Freshness = "have any files in this tool's import closure changed since last run?"
This is answerable in O(import_closure) time with mtime checks.
For a typical tool: O(10 files) = O(1ms).
```

The freshness system's job shrinks from "decide if the whole repo is stale and re-run everything" to "check each tool's import closure and skip tools whose inputs are unchanged." The tools themselves handle the rest via content_upsert.

## Concrete changes

### 1. Per-tool freshness entries derived from module graph

After compilation, record `(tool_name, source_files, source_digest)` where `source_files` comes from `ModuleGraph`. This is a one-line change to `CompileReceipt` — expose the file list alongside the digest.

The manifest gains per-tool entries:
```json
{
  "freshness:testgen": {
    "source_files": ["dsl/tools/testgen.dag", "dsl/std/patterns.dag", ...],
    "source_digest": "abc123...",
    "output_hash": "def456...",
    "created_at": 1772671933123
  }
}
```

### 2. Replace monolithic stale check with per-tool checks

```
check_and_plan_freshness() currently:
  hash(645 files) → stale? → run all 7 steps

check_and_plan_freshness() proposed:
  for each generation tool:
    stat(tool.source_files) → any mtime newer? → mark tool stale
  return only stale tools as steps
```

### 3. Early cutoff between steps

After codegen runs, hash its output files. If output unchanged from previous run, mark codegen-dag/testgen/pragma as "upstream unchanged" and skip them even if their own inputs changed. (Their inputs include codegen's output via the pipeline dependency.)

### 4. Remove verification from interactive path

Generated binaries call generation-only freshness. `make check` and CI run the full verification chain. This removes clippy + test-compile + release-check (~50s) from every interactive tool invocation.

### 5. Direct binary invocation

Replace `cargo run -p gunbc-app --bin gunbc-testgen` with `target/debug/gunbc-testgen`. Saves ~100-400ms per step. Trust `make install` for binary freshness.

## Measurements

### File inventory

| Metric | Value |
|--------|-------|
| Tracked files in current freshness glob | 645 |
| Typical tool import closure | 5-15 `.dag` files |
| testgen output files | ~50 `generated_tests_*.rs` |
| pragma output files | 3 config files |

### Operation timings

| Operation | Wall time |
|-----------|-----------|
| stat 10 files (typical import closure) | ~0.3ms |
| stat 645 files (current approach) | ~15ms |
| `git rev-parse HEAD` + `git status` | ~10ms |
| Full DSL compilation (testgen) | ~15s |
| Content generation (testgen, ~50 targets) | ~12s |
| Content comparison (testgen, ~50 files) | ~100ms |
| `cargo run` overhead (warm) | ~200ms |
| `cargo clippy --workspace` (nothing to do) | ~2s |

### Expected improvement

| Scenario | Current | After | Why |
|----------|---------|-------|-----|
| No changes (fresh) | ~15ms | ~5ms | Same tier-0 git signal check |
| Non-DSL `.rs` change | 45-120s | ~5ms | Not in any tool's import closure; skip all generation |
| Single `.dag` change (not imported by tool) | 45-120s | ~5ms | Not in this tool's import closure |
| Single `.dag` change (imported by tool) | 45-120s | 1-28s | Only affected tool re-runs |
| Compiler crate change | 45-120s | 30-40s | All tools re-run (cargo recompiles, outputs change) |

The common case (editing `.rs` files during development) goes from minutes to milliseconds.

## Open questions

1. **Rust crate dependencies in the input set.** The import closure only covers `.dag` files. When `core/codegen/src/testgen/render_rust.rs` changes, testgen's output changes, but the `.dag` files didn't change. Options: (a) include Rust crate source files in the input set, (b) include the binary mtime as an input (if the binary was recompiled, assume outputs may change), (c) rely on early cutoff (codegen/testgen re-run, output hash comparison catches unchanged cases).

2. **Tool discovery cache interaction.** `tool_discovery.rs` already has a `DiscoveryCache` keyed on `source_digest`. Should the freshness system use this cache, or maintain its own? They track the same thing (source file hashes) at the same granularity (per-tool).

3. **Testgen's dynamic input set.** testgen's targets come from `discover_testgen_targets()` at runtime (via inventory). The target list itself is an input — if a new testgen target is registered, testgen needs to re-run. This is outside the `.dag` import closure.

4. **Concurrent tool invocations.** If two tools run simultaneously and both determine codegen is stale, they'd both try to run codegen. Need a file lock or deduplication.
