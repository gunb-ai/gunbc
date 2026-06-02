# v4 Incremental Bootstrap / CI-Perf Round-Robin Worksheet (RR-L)

> **Status:** RATIFIED FOR W2 DISPATCH — incremental-bootstrap + CI-perf cross-cut.
> **Work item:** `node://adhoc-60340772-59b` — RR-L worksheet (`eager-bat-504`) under CI Manager (`silent-crane-669`).
> **Gate:** Class 1 design closure only. Performance work may proceed when it preserves existing compiler authorities and carries equivalence receipts; timeout changes are scheduling budget, not proof of correctness.

## §10.0-adapted worksheet

```text
Migration class:        L-INCREMENTAL-BOOTSTRAP-CI-PERF
Representative failure:  Three landed fixes (#4282, #4281, #4324) reduced or exposed CI
                         wall-clock pressure, but they live across different authority layers:
                         v2 infer lookup memoization, v2 tokenizer scanning, and CI M1 probe
                         timeout budget. Without a cross-cut contract, the next worker can turn
                         "faster enough for CI" into a second compiler authority, an unsafe
                         parse/intern parallelization, or a timeout-as-green signal.
Immediate local patch:   Cache by intern ids where the meaning changes across TypeEnv updates;
                         parallelize parser/interner folds without a deterministic intern-table
                         receipt; raise CI timeouts again to hide deterministic work growth; or
                         let bootstrap reuse an artifact without proving byte/diagnostic parity.
Why forbidden:           P2/P3/P5 — performance caches must be observationally invisible, fail
                         closed, and dissolve into modeled bootstrap facts. CI budget is not a
                         verifier. The v2 stage0 compiler remains load-bearing bootstrap seed
                         territory (SELF_HOSTING §1); orchestration may sequence existing
                         authorities but must not co-author parse, infer, emit, or source facts.
DFS path:
  v2 infer/cache authority (CONSUME — do not fork):
    - src/v2/04_env.dag — LookupCache, span-only scrutinee_cache_key (#4282)
    - src/v2/04_lookup.dag / src/v2/04_infer.dag — lookup and reconcile consumers
    - src/v2/stage0/src/bin/regen_stage0.rs — stage0 regen/verify receipt
  v2 tokenizer authority (CONSUME — do not fork):
    - src/v2/01_tokenize.dag — codepoint scanning model (#4281)
    - src/v2/stage0/src/v2_compiler_tokenize.rs — generated/seed tokenizer realization
    - src/v2/compile.dag / src/v2/stage0/src/v2_compiler_compile.rs — sequential parser
      intern-table threading
  CI budget authority (CONSUME — do not fork):
    - src/v4/workflow/ci.dag — M1RustEmitProbeCommand and modeled CI signal
    - dsl/gunbc/ci_github_actions_workflow.dag — interim static Actions carrier with
      Source-SHA256 pin
    - .github/workflows/ci.yml — hand-edited transport until T-24 projection emits it
  adjacent closure (CONSUME):
    - docs/planning/v4-ci-rust-dag-shared-closure-worksheet-2026-06-01.md — #4171
      rust+dag shared closure; resolved->target emit parity remains the artifact-reuse bar
Deepest unsound boundary:
  Intern ids, memo keys, and CI artifacts can be stable within one process while not being
  semantic source authority. A cache keyed to a rebound intern id or a reused DAG artifact with
  no parity receipt can make bootstrap false-green even when standalone compile semantics drift.
Systemic fix:
  Treat perf work as observationally-equivalent rewrites over existing authorities:
  (1) cache keys cite immutable source spans or proven content hashes, not mutable context ids;
  (2) tokenizer/parser changes preserve deterministic intern-table threading unless a modeled
      merge receipt lands first;
  (3) artifact reuse consumes #4171-style byte/diagnostic parity;
  (4) CI timeout edits cite measured deterministic pressure and keep the job cap fail-closed.
Non-goals:
  - No new parser/infer/emit substrate or Rust compiler path from this worksheet.
  - No "parallel parse" implementation unless a deterministic intern-table merge authority lands.
  - No further CI timeout bump as a substitute for profiling or equivalence receipts.
  - No BuildBuddy/remote-cache policy change; runner/cache transport remains CI Manager scope.
Falsification probe:
  §4 table (R1-R9) — mandatory before any RR-L implementation PR claims PROVEN.
Metric allowed only as secondary:
  Full `src/v4 --target dag` or M1 emit wall-clock. Acceptance is semantic equivalence plus
  fail-closed CI behavior, not a particular minute count.
```

---

## §1 Landed Evidence Map

| Artifact | Landed state | RR-L disposition |
| --- | --- | --- |
| #4282 `v2 perf: fix the O(n^2) in reconcile / type inference` | MERGED — lookup/reconcile memoization; `scrutinee_cache_key` is span-only because intern ids can be rebound across `TypeEnv` updates | **Consume as the cache-key law**: source span/content identity is acceptable; mutable context identity is not |
| #4281 `v2 perf: optimize tokenizer codepoint scanning` | MERGED — integer codepoint scanning avoids per-character `String` allocation; unsafe parallel parse was dropped | **Consume as tokenizer law**: hot-path rewrites are fine when token positions and intern threading stay deterministic |
| #4324 `ci(wave1): bump M1 emit probe step timeout 20m -> 35m` | MERGED — M1 v4 full-tree Rust emit probe step budget matches bootstrap step budget, job remains capped at 60m | **Consume as budget law**: timeout can track measured deterministic work; it cannot certify correctness |
| #4171 CI Rust+DAG shared closure | MERGED — resolved->target emit helper and parity receipt for shared DAG reuse | **Prerequisite pattern** for any future bootstrap artifact reuse |

## §2 RR-L Authority Contract

### 2.1 Cache and Memoization

Performance caches in v2 stage0 paths are admissible only when their key is immutable with respect to the semantic fact being cached. For source-authored nodes, span/content identity is stable; intern ids are stable only inside the intern-table version that authored them.

**Accepted pattern:** `span_cache_key(SourceSpan)` or a content hash whose authority is already declared.

**Rejected pattern:** memoizing scrutinee/type facts by intern id when `TypeEnv` can rebind the id to a different semantic environment.

### 2.2 Tokenizer / Parser Incrementality

Tokenizer improvements may change representation and allocation behavior, not token stream meaning. Parser improvements must preserve deterministic intern-table threading for parser-authored dotted module/import names until a modeled merge authority exists.

The #4281 parallel parse attempt is the negative receipt: parallel work that cannot preserve intern-id authority is out of scope, even if it improves wall-clock.

### 2.3 CI Budget and Bootstrap Reuse

CI timeout increases are scheduling changes. They must cite a deterministic pressure source, update all three live carriers when applicable (`ci.yml`, static `dsl/gunbc` carrier, and `src/v4/workflow/ci.dag`), and preserve fail-closed job caps.

Artifact reuse, incremental bootstrap, and shared compile closures require #4171-style parity: standalone path and shared/incremental path produce identical diagnostics and emitted bytes on a fixed slice before CI can trust the reused artifact.

## §3 Implementation Lanes

| Lane | Allowed work | Required receipt |
| --- | --- | --- |
| L.1 infer lookup/reconcile perf | More cache warming, lookup indexes, or resolved-type memoization inside existing v2 infer authority | Regen verify plus a targeted equivalence test that fails when cache key identity is wrong |
| L.2 tokenizer/source scan perf | Allocation reductions and source scanning rewrites inside tokenizer authority | Token position/slice tests and regen verify |
| L.3 incremental bootstrap artifact reuse | Reuse previously produced artifacts only through existing emit/bootstrap authorities | Byte-for-byte emitted artifact equality and diagnostics equality, following #4171 |
| L.4 CI scheduling budget | Timeout/build-profile adjustments for measured deterministic work | Modeled CI smoke tests plus static Actions pin update; no correctness claim from timeout alone |

## §4 Falsification Table (Implementation PROVEN)

| ID | Probe | Receipt |
| -- | ----- | ------- |
| R1 | Cache keys do not use mutable context ids for facts whose meaning changes across `TypeEnv` updates | Diff review plus a test analogous to #4282's rebound-intern failure |
| R2 | Synthetic nodes bypass source-span cache unless a stable content identity is declared | Unit/integration test for synthetic-node cache miss |
| R3 | Tokenizer emits identical token kind, span, and authored slice sequence before/after rewrite | Token position test on representative `.dag` fixtures |
| R4 | Parser/interner changes preserve deterministic intern-table threading | Stage0 regen verify and a dotted module/import fixture |
| R5 | Shared or incremental bootstrap artifact path matches standalone path diagnostics and emitted bytes | Fixed-slice parity test, #4171 form |
| R6 | CI timeout change updates `.github/workflows/ci.yml`, `dsl/gunbc/ci_github_actions_workflow.dag` Source-SHA256, and `src/v4/workflow/ci.dag` together | `gunbc_ci_github_actions_workflow_pins` / matches tests |
| R7 | Timeout increase cites measured deterministic work and keeps the containing job fail-closed | PR body names measurement and job cap |
| R8 | No new hand-Rust parser/infer/emit authority lands to satisfy a perf metric | Diff review over `src/v2/stage0` and `.dag` authority files |
| R9 | Wall-clock metric is secondary: a slower but equivalent/fail-closed path is accepted over a faster unproven path | PR review checklist cites semantic receipt before timing |

## §5 Forbidden Patterns

| Pattern | Why forbidden |
| ------- | ------------- |
| Memo key based on intern id across `TypeEnv` updates | Reintroduces #4282 semantic drift |
| Parallel parser fold without deterministic intern-table merge receipt | Repeats the dropped #4281 unsafe shape |
| Timeout bump as sole fix for CI failure | Scheduling budget is not correctness authority |
| Reusing M1 DAG/bootstrap artifacts without byte/diagnostic parity | Bootstrap can false-green |
| Editing only `.github/workflows/ci.yml` for modeled CI facts | Splits CI authority from `ci.dag` and the static carrier pin |
| Hand-authored emit/lower/infer/parse shortcut for perf | Load-bearing stage0 authority fork |

## §6 Landing Order

```text
1. RR-L merged (this doc) — CI Manager may dispatch Class 2 perf/CI workers.
2. L.1/L.2 implementation PRs: cache/tokenizer perf with semantic equivalence receipts.
3. L.3 implementation PRs: incremental bootstrap or artifact reuse only after #4171-form parity.
4. L.4 CI budget PRs: only for measured deterministic work, with modeled CI carrier updates.
5. Follow-up dissolution: replace hand-Rust/string CI smokes when workflow/TestClaim projection executes the modeled facts.
```

## §7 Handoffs

- **CI Manager (`silent-crane-669`)**: owns L.4 scheduling budget and runner/cache transport decisions. RR-L does not authorize remote-cache policy changes.
- **Branch E / Bootstrap**: owns self-host/bootstrap-as-data design. RR-L may consume bootstrap receipts but must not invent a `.dag` source regenerator or stage replacement.
- **Source Authority / Branch H**: owns canonical `.dag` source serialization. Incremental bootstrap may not treat JSON IR, emitted Rust, or CI artifacts as source authority.
- **v2 stage0 perf workers**: consume #4282/#4281 laws; preserve regen verify and equivalence tests.

## §8 Modeling DFS Arbiter Checklist

- [x] Single-authority: caches and CI budget consume existing parse/infer/emit/bootstrap/CI authorities.
- [x] #4282 cache-key lesson recorded: span/content keys, not mutable intern-context identity.
- [x] #4281 tokenizer lesson recorded: allocation rewrite OK; unsafe parallel parser/interner shape rejected.
- [x] #4324 timeout lesson recorded: budget tracks deterministic pressure; it does not prove correctness.
- [x] #4171 parity form required for future artifact reuse.
- [x] Falsification R1-R9 accepted.
- [x] **READY-FOR-WORKER-DISPATCH** (RR-L Class 1 closure — implementation workers may proceed under §3).

---

## Related Artifacts

- gunb-ai/gunbc#4282 — v2 infer/reconcile memoization perf fix
- gunb-ai/gunbc#4281 — v2 tokenizer codepoint scanning perf fix
- gunb-ai/gunbc#4324 — CI M1 emit probe timeout 20m -> 35m
- gunb-ai/gunbc#4171 — CI Rust+DAG shared closure worksheet implementation
- `src/v2/04_env.dag` — `LookupCache`, `scrutinee_cache_key`
- `src/v2/01_tokenize.dag` — tokenizer model
- `src/v4/workflow/ci.dag` — modeled CI facts
- `dsl/gunbc/ci_github_actions_workflow.dag` — interim static Actions carrier
- `.github/workflows/ci.yml` — hand-edited transport until T-24 workflow projection
