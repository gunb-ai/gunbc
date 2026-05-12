# R3 CI Layer 2 — Per-test path-conditional gating worker brief

**Status:** DIRECTOR-SCAFFOLD (authored under standing authority per `feedback_director_mgr_energy_input` 2026-05-11; awaiting Verification Mgr finalization). **Bridge-debt** with named dissolution trigger.

**Owner:** worker TBD on dispatch. **Coordinator:** R3 Verification Mgr (`clever-tern-670`). **Authority chain:** PM ratification (msg_45457c77 + GitHub inbox c4425726922) → Director ratification (msg_a77c7f42; routing Verification Mgr per parallel-representation-debt coherence) → Verification Mgr finalization → auto-spawn dispatch.

**Authority (cite-and-execute):**
- **Operator escalation**: Brian's CI-up-to-1-hour framing at gunbc#846 2026-05-11 ("their CI is up to 1 hour ... not happening" — re v3 ratchet attempt context)
- **Layer 1 prior art**: PR #2718 `ci(layer1): skip v3 job on docs-only PRs via changes-filter` — the `changes` job mechanism this brief EXTENDS (not parallels)
- **Bridge-debt → dissolution lifecycle (R4-bounded, NOT R3 close)**: per `docs/design-affected-set-lens.md:3` ("**Status**: R4 wishlist (R4.B Introspect-lens saturation lane)") + `:354` ("§5. CI integration sketch (**deferred to R4 full delivery**)") + `:366` ("**Out of scope here**: implementation of the CI integration. The prototype demonstrates the lens output; the CI integration is R4 full-delivery work"). Layer 2's hand-authored path-mapping table dissolves when **R4.B's CI integration delivery** lands the affected-set lens consumer in the `changes` job (currently no ROADMAP authority for a gate named `ci_uses_provable_minimal_affected_set_selection`; that name was a Director-tier placeholder and has been removed per Brian's P5 catch at PR #2719 c#4426351828). **R4 owner**: R4.B saturation lane (no concrete dispatch yet; lane is wishlist per `docs/design-affected-set-lens.md:3`). **R3-tactical framing**: Layer 2 is an R3-cycle CI mitigation bridge whose dissolution lifecycle is bounded by R4 lens-CI delivery, NOT by R3 close. Acknowledged explicitly per Brian's BLOCKING #1 absorption 2026-05-12.
- **Post-dissolution selection semantics (canonical 2-step join per design §5)**: when the lens ships, per-group `skip_*` flags compute via the canonical join:
  - **NodeRef intersection**: `(group.testclaim_references ∩ lens.affected_node_refs) ≠ ∅` (per `docs/design-affected-set-lens.md:359`: "intersect aggregate affected-set with TestClaim references")
  - **Dimension intersection**: `(group.dimensions ∩ lens.changed_dimensions) ≠ ∅` (per same line: "selection keeps TestClaims whose asserted-dimensions intersect with changed-dimensions")
  - **Run condition**: both ≠ ∅ ⇒ run; either ∅ ⇒ skip. Canonical formula: `run = (refs ∩ nodes) ≠ ∅ ∧ (group.dims ∩ changed.dims) ≠ ∅`; equivalently `skip = ¬run = (refs ∩ nodes) = ∅ ∨ (group.dims ∩ changed.dims) = ∅`.
  - **Bridge coarseness acknowledgment**: Layer 2's bridge operates at file-path-regex level, NOT TestClaim-reference level. The `required_paths_regex` column is a path-side proxy for the NodeRef-intersection step; bridge over-approximates the canonical (runs MORE tests than canonical would). That's fail-closed-safe direction but structurally coarser — the brief MUST be explicit about this gap. Per Brian's BLOCKING #2 absorption 2026-05-12: the surviving post-dissolution schema MUST include `group.testclaim_references: Set<NodeRef>` (or equivalent group-to-NodeRef-membership) as a third column, NOT just `(group_name, dimensions)`.
- **Polarity check**: carrier name is `skip_*`; CI consumer wires `if: skip_<group> != 'true'` (run when skip=false). Skip-form is canonical: NodeRef-empty OR dim-empty ⇒ unaffected ⇒ skip; both non-empty ⇒ affected ⇒ run. Set-intersection semantics per locked-design §2 union — NOT singular `.contains()` membership.
- **Per-dimension structural target**: `docs/design-affected-set-lens.md` §2 (affected_set defined as union over `Set<Dimension>`) — every Layer 2 path-mapping entry MUST carry a `dimensions:` field of type `Set<Dimension>` (members drawn from `value | cost | complexity | effect | refinement`). Single-element sets like `{cost}` are valid for single-dimension groups; multi-dim consumers (e.g., LBP demonstration reading both `complexity` + `cost`) get expanded sets. Prevents schema divergence from future lens output AND silent-skip on multi-dim consumers when only the non-primary dim changes.
- **Slow-test inventory sources** (per PM pre-stage at #828 c4425726922):
  - (a) `scripts/slow-test-exemptions.txt` — curated >2s ratchet exemption list. **Mgr-finalization MUST recompute live count** via `grep -v "^#" scripts/slow-test-exemptions.txt | grep -v "^$" | wc -l` at finalization (count fluctuates per hot-fix arcs — e.g., #2723 added cuts; PM template citation at PR #2721 of "78" is historical and stale). Do not cite snapshot integers in this brief — they rot between authoring and Mgr-fill (cursor BLOCKING #9832 absorption 2026-05-12: prior in-brief fixed-count wording had already drifted by review time).
  - (b) `/tmp/v3-test-timings.log` — empirical per-test wall-time captured by every CI run via `--report-time` (consumed by `scripts/check-test-timeout.sh`, wired at `.github/workflows/ci.yml:405-424`)
  - (c) NEW per-group required-paths mapping (the deliverable; bridge-debt artifact)

---

## §0. Scope

Extend PR #2718's `changes` job with per-test-group `skip_*` boolean outputs derived from the same `changed`-files list and per-group required-paths regex. Wire `v3` STEP-level `if:` predicates to skip individual test groups whose path-dependencies are unaffected by the PR's diff.

**Not in scope**: a parallel CI gate mechanism; new path-classification source-of-truth (must reuse PR #2718's `gunbc-quick` `changes` job).

**Single source of truth invariant**: all path classification flows from one `changes` job. Layer 1 produces `code: bool`; Layer 2 grows additional outputs (`skip_cost_lens: bool`, `skip_emit_target: bool`, `skip_parser_grammar: bool`, ...) on the same job — each output named `skip_<group_name>` matching the per-group table's `group_name` column verbatim per §1 naming convention. No second `gunbc-quick` job. No `actions/cache`-fork. No parallel diff mechanism.

## §1. Mechanism (extend PR #2718, do not parallel)

**Layer 1 baseline** (per PM pre-stage):
```yaml
# .github/workflows/ci.yml (post-#2718)
changes:
  runs-on: gunbc-quick
  timeout-minutes: 3
  outputs:
    code: ${{ steps.diff.outputs.code }}
  steps:
    - <git diff origin/main...HEAD>
    - <grep -v '^(docs/.*|[^/]+\.md)$'>
    - <emit code=true if any file remains, else false; push events short-circuit to true>
```

**Layer 2 extension**: same `changes` job grows per-group outputs:
```yaml
changes:
  outputs:
    code: ${{ steps.diff.outputs.code }}
    skip_cost_lens: ${{ steps.classify.outputs.skip_cost_lens }}
    skip_emit_target: ${{ steps.classify.outputs.skip_emit_target }}
    skip_parser_grammar: ${{ steps.classify.outputs.skip_parser_grammar }}
    skip_complexity_lens: ${{ steps.classify.outputs.skip_complexity_lens }}
    # ... per-group entries per Mgr-fill inventory
    # naming convention: skip_<group_name> where <group_name> matches the
    # per-group table's group_name column verbatim (no abbreviation).
  steps:
    - <existing diff step>
    - id: classify
      run: |
        # for each group in per-group-required-paths-table:
        #   skip_<group_name> = "true" if (changed files ∩ required_paths) is empty else "false"
        # push events short-circuit all skip_* to "false"
```

`v3` job consumes via STEP-level `if:`:
```yaml
v3:
  needs: [changes]
  if: ${{ needs.changes.outputs.code == 'true' || github.event_name == 'push' }}
  steps:
    - <fmt + clippy — always run when v3 runs>
    - name: cost-lens integration
      if: ${{ needs.changes.outputs.skip_cost_lens != 'true' }}
      run: cargo test -p v3-compiler --test integration cost_lens
    - name: emit-target integration
      if: ${{ needs.changes.outputs.skip_emit_target != 'true' }}
      run: cargo test -p v3-compiler --test integration emit
    # ... per-group test invocations
    # IMPORTANT (per openai-pro P3 BLOCKING #9749 absorption): the positional
    # argument to `cargo test` after `--test integration` is libtest's
    # test-name SUBSTRING filter, NOT a glob. `cost_lens` matches all tests
    # whose name CONTAINS "cost_lens"; `cost_lens_*` would be treated as a
    # literal substring (with the `*`) and match zero tests — silent skip.
    # NEVER use shell-glob syntax (`*`, `?`, etc.) in the positional filter.
    # If precision beyond substring is needed, use `--exact` with the
    # specific test name OR script-side enumeration via cargo metadata.
```

**Job-level `code` flag retained**: docs-only PRs still skip the entire `v3` job (~67min → 0min). Layer 2 makes the granularity finer for code PRs whose changed-paths affect only some groups.

## §2. Inventory sources + per-group table shape

Per-group mapping table (the deliverable):

```
(group_name, dimensions, required_paths_regex, testclaim_references, test_pattern)
```

**`testclaim_references` column added per Brian's BLOCKING #2 absorption 2026-05-12**: the canonical 2-step selection algorithm per `docs/design-affected-set-lens.md:359` requires `Set<NodeRef>` membership AND `Set<Dimension>` intersection. Dropping the NodeRef step (as my prior post-dissolution sketch did) silently violates Facts Flow Forward. Each group entry MUST compute a `testclaim_references: Set<NodeRef>` value (union of TestClaim references across the group's tests; sourced from `tests/dag/*` TestClaim authorities at Mgr-fill time). Bridge tier proxy is `required_paths_regex` (file-path-coarse); post-dissolution proxy is `testclaim_references` (NodeRef-precise).

Where:
- `group_name` — short identifier (e.g., `cost_lens`, `emit_target`, `parser_grammar`)
- `dimensions: Set<Dimension>` — non-empty subset of `{value, cost, complexity, effect, refinement}` per `docs/design-affected-set-lens.md` §2 union semantics. Single-element sets `{cost}` are valid for single-dim groups; multi-dim consumers MUST list all dimensions they read (e.g., LBP demonstration: `{complexity, cost}`). Empty set is invalid — escalate per §7.
- `required_paths_regex` — regex over changed file paths; if no changed file matches, skip this group
- `test_pattern` — `cargo test` positional **test-name substring filter** (NOT a glob; libtest's filter is substring-based per `cargo test --help`). Example: `cost_lens` matches all tests whose name contains the literal substring "cost_lens". **Never use shell-glob syntax** (`*`, `?`) in this field — those characters would be treated as literal substring characters and silently match zero tests (fail-open per openai-pro P3 BLOCKING #9749). If precision beyond substring is needed, use `--exact` with the specific test name or enumerate via cargo metadata.

**Inventory derivation** (Mgr-fill from 3 sources):

(a) **`scripts/slow-test-exemptions.txt`** — start with the **current live count** of active >2s entries (Mgr MUST re-run `grep -v '^#' scripts/slow-test-exemptions.txt | grep -v '^$' | wc -l` at finalization time; do NOT cite stale snapshot integers from this brief or any earlier reference — the count fluctuates per hot-fix arcs). Each entry already has citation discipline; group by `_test.rs` file-area prefix. **Fail-closed completeness invariant** (per openai-pro BLOCKING #9779 absorption 2026-05-12): under-inventory is the fail-open shape — every active exemption MUST appear in either a per-group `required_paths_regex` row OR the harness/shared-infra full-run bucket. No exemption left unclassified.

(b) **`/tmp/v3-test-timings.log` empirical** — last N CI runs aggregated → top-K slowest groups by file-area. Cross-validates (a) and surfaces non-exempted slow tests.

(c) **NEW per-group required-paths mapping** — for each group, hand-author the required-paths regex by examining which `src/v3/*` files the group's tests transitively depend on. This is the bridge-debt artifact; dissolves when the affected-set lens lands.

**Shared-infrastructure full-run fail-closed bucket** (per codex P3 BLOCKING finding 2026-05-12 review #9744): per-group regexes by themselves are NOT sufficient — a PR that changes shared test infrastructure or selection machinery OUTSIDE `src/v3/*` (e.g., `.github/workflows/ci.yml`, `scripts/*`, harness code, `Cargo.toml`/`Cargo.lock`, `rust-toolchain.toml`, `.cargo/config.toml`) would be classified as "unaffected" for every per-group regex and silently skip tests whose behavior actually changed. That's the fail-open boundary class P3 forbids.

**Mechanism**: the `changes` job MUST gate ALL `skip_*` flags to `false` (force full-run) when any changed file matches the shared-infrastructure regex:

```
^(\.github/.*|scripts/.*|(.*/)?Cargo\.(toml|lock)|rust-toolchain\.toml|\.cargo/.*|(.*/)?build\.rs|src/v3/compiler/tests/integration/common/.*|src/v3/compiler/tests/integration/sg0_census_test\.rs|src/v3/compiler/tests/integration/test_runner_test\.rs|src/v3/compiler/tests/integration/t_pb_b_1_dag_runner_test\.rs|src/v3/compiler/tests/integration/integration\.rs|src/v3/compiler/tests/integration\.rs)$
```

**Harness/test-selection-machinery arms** (per openai-pro P3 BLOCKING #9749 absorption): the regex includes the named harness-code class explicitly — `tests/integration/common/*` (shared test utilities), `sg0_census_test.rs` (census authority), `test_runner_test.rs` (runner framework), `t_pb_b_1_dag_runner_test.rs` (suite enumeration framework), and the integration test entry points. Worker MUST add any new harness-class file to this regex before merging the file. **A harness-class file MUST never appear in a per-group `required_paths_regex` — it always triggers full-run.**

**Crate-local build metadata** (per openai-pro P3 BLOCKING review on PR #2725 absorption 2026-05-12 — parity fix to #2719): the regex MUST match `Cargo.toml`/`Cargo.lock`/`build.rs` at ANY depth, not just workspace-root. The project has crate-local manifests + build scripts (e.g., `src/v3/compiler/Cargo.toml`, `src/v3/compiler/build.rs` per `CODING.md:319`); a root-only anchored regex (`^Cargo\.(toml|lock)$`) would miss these and silently skip tests for crate-local manifest/build-script changes — fail-open boundary class P3 forbids. The `(.*/)?` non-capturing optional path prefix on those alternates above matches both root-level (e.g., `Cargo.lock`) AND any-depth crate-local (e.g., `src/v3/compiler/Cargo.toml`, `src/v3/compiler/build.rs`).

Equivalently in step output: `force_full_run = (any changed file ∈ shared-infrastructure regex)`; when `force_full_run = true`, all per-group `skip_*` outputs short-circuit to `false`. Composes with the `code=true|false` Layer 1 gate (docs-only PRs already skip everything via Layer 1; this constraint applies only to code PRs).

The shared-infrastructure regex MUST be hand-authored at the **changes job level**, not delegated to per-group regexes — every group entry's regex covers ONLY its own `src/v3/*` deps; the full-run trigger is the join-point that catches inter-group / cross-cutting changes. This is fail-closed by construction: a missing per-group regex entry doesn't matter when shared-infra changes; everything runs.

**[Mgr-fill]**: validate the shared-infra regex against representative recent PRs that touched `.github/workflows/ci.yml` / `scripts/*` / `Cargo.lock` and confirm those PRs would have `force_full_run = true`.

**Starting template — PM pre-staged Mgr-fill reference doc**: `docs/briefs/r3-ci-layer-2-pm-prestaged-mgr-fill-template.md` (open as **PR #2721** at https://github.com/gunb-ai/gunbc/pull/2721; **NOT yet landed on `main`** — verified via codex BLOCKING #9754 absorption 2026-05-12: `git ls-tree origin/main -- ...` returned no blob; file lives only on PR #2721's branch until merge). 220 lines.

**Authority caveat per codex P1/P2 catch**: this brief cites the template as the inventory data attachment, but the cited path will be unresolvable on a worker checkout of `main` until PR #2721 merges. Verification Mgr finalization MUST coordinate the merge sequencing: either (a) merge PR #2721 first so the template is on `main` before Mgr-fill starts, OR (b) read the template from PR #2721's branch (e.g., `gh pr view 2721 --repo gunb-ai/gunbc` or checking out `origin/<2721-branch>`) until it merges. Cross-link: PR #2721 author is PM (`deep-wolf-155`); merge coordination is PM-routable.

The PM template provides (per PR #2721 review-state at sha `262f42d7d` — post fix):
- **PM-grouped `scripts/slow-test-exemptions.txt` entries** (PM template snapshot was 78 entries at template authoring time; live count grows — Mgr re-verifies via `wc -l` at finalization, NOT this stale historical reference) grouped into 9 clusters (A–I) by module prefix. The 9-cluster taxonomy survives even as new entries land; Mgr maps new entries to existing clusters or escalates if a new cluster surface emerges.
- **`(test_pattern, dimensions, required_paths_regex)` skeleton table** with every row carrying a `dimensions:` field of type `Set<Dimension>` per locked-design §2 union semantics (PM template post-fix at `dedcf69a4`)
- **Pilot recommendation: Cluster B** (Lane 2 Stage 2d symbolic cost — high confidence, single-element set `{cost}`, ~6 tests). Note: my §6 recommendation was `cost_lens` first — these converge; Cluster B IS the cost-lens family with singleton `{cost}` dimensions.
- **12 `[Mgr-fill]` placeholders** marking where consumer-tracing exceeded PM bandwidth (substrate-lens deps, R3-V L4/L7, R1C-E `.dag` wrapper, free-consequences cross-target). These are the Mgr-tier sub-classification decisions.

Inline sketch (illustrative — defer to the PM template for the actual starting inventory). Note `dimensions` column is `Set<Dimension>`; singletons shown as `{cost}`, multi-dim consumers as `{complexity, cost}`:
```
cost_lens          | {cost}              | ^(src/v3/lenses/cost\.dag|src/v3/std/algebra\.dag|src/v3/compiler/src/lens_cost_.*\.rs)$       | cost_lens
complexity_lens    | {complexity}        | ^(src/v3/lenses/complexity\.dag|src/v3/compiler/src/lens_complexity_.*\.rs)$                    | complexity_lens
lbp_demonstration  | {complexity, cost}  | ^(src/v3/lenses/(cost|complexity)\.dag|src/v3/compiler/src/.*lbp.*\.rs)$                       | lbp  # multi-dim
emit_target        | {effect}            | ^(src/v3/extdeps/.*|src/v3/compiler/src/emit/.*|src/v3/compiler/src/omni_shape_.*\.rs)$          | emit
parser_grammar     | {refinement}        | ^(src/v3/parser/.*|src/v3/compiler/src/parser.*\.rs|src/v3/compiler/src/lower.*\.rs)$            | parser
# ... etc per Mgr-fill
# test_pattern is libtest SUBSTRING filter — no globs; `cost_lens` matches every test name containing "cost_lens".
```

**[Mgr-fill]**: full per-group table — exhaustive coverage of current `scripts/slow-test-exemptions.txt` entries (count fluctuates per hot-fix arcs; Mgr re-runs `grep -v "^#" ... | grep -v "^$" | wc -l` at finalization rather than relying on stale snapshot integers in this brief) grouped + empirical top-K from timings log + per-group required-paths regex tested against representative diffs. **Also required per Brian's BLOCKING #2 absorption**: each group entry must compute `testclaim_references: Set<NodeRef>` (union of TestClaim references across group's tests) for the canonical 2-step selection join post-dissolution. Bridge-tier proxy is `required_paths_regex`; post-dissolution proxy is `testclaim_references` populated from `tests/dag/*` TestClaim authorities.

## §3. Per-dimensions structural target — `feedback_parallel_representation_debt` prevention (set semantics per locked-design §2)

The Layer 2 path-mapping is bridge-debt by design. The dissolution is the affected-set Introspect-lens (canvas PR #2713 by `clever-tern-670`, locked-design `docs/design-affected-set-lens.md`). When the lens lands, the dissolution is:

```yaml
# Post-dissolution (after R4.B CI integration delivery per design-affected-set-lens.md §5)
# NOTE per Brian's BLOCKING #1 absorption: no current ROADMAP gate name; R4.B is the
# owning lane (wishlist status per design doc :3). Mgr re-cites a concrete gate ID
# once R4.B ROADMAP authority lands the actual gate.
changes:
  steps:
    - id: lens
      run: |
        cargo run -p v3-compiler --bin affected_set_lens -- \
          --pr-diff origin/main...HEAD \
          --output /tmp/affected.json
        # lens output: {affected_node_refs: Set<NodeRef>, affected_dimensions: Set<Dimension>, per_dim: {dim: Set<NodeRef>}}
    - id: classify
      run: |
        # Canonical 2-step selection per docs/design-affected-set-lens.md:359:
        # 'intersect aggregate affected-set with TestClaim references; keep
        # TestClaims whose asserted-dimensions intersect with changed-dimensions'.
        # Per-group skip_* = NOT (NodeRef-intersection AND dim-intersection).
        for group in cost_lens emit_target parser_grammar ...; do
          dims_json="${group_dimensions[$group]}"    # e.g., '["cost"]' or '["complexity","cost"]'
          refs_json="${group_testclaim_refs[$group]}"  # Set<NodeRef> union over group's TestClaims
          # run iff BOTH (refs ∩ affected_node_refs) ≠ ∅ AND (dims ∩ affected_dimensions) ≠ ∅
          # skip = ¬run = either intersection is ∅
          # (Brian BLOCKING #2 absorption: dimension-only check would silently drop NodeRef-step
          # and violate Facts Flow Forward per design §5 canonical algorithm.)
          if jq -e --argjson dims "$dims_json" --argjson refs "$refs_json" \
               '(.affected_dimensions | any(. as $d | $dims | contains([$d])))
                and (.affected_node_refs | any(. as $n | $refs | contains([$n])))' \
               /tmp/affected.json; then
            echo "skip_$group=false" >> $GITHUB_OUTPUT
          else
            echo "skip_$group=true" >> $GITHUB_OUTPUT
          fi
        done
```

**The `(group_name, dimensions, testclaim_references)` mapping survives the dissolution** (corrected per Brian's BLOCKING #2 absorption + cursor internal-consistency catch 2026-05-12) — only the `required_paths_regex` column gets retired (replaced by `testclaim_references: Set<NodeRef>` as the NodeRef-precise version of group-tests-membership). **Both** `dimensions: Set<Dimension>` AND `testclaim_references: Set<NodeRef>` columns survive as lens consumers per the canonical 2-step selection join at `docs/design-affected-set-lens.md:359`. For this to work, **every Layer 2 path-mapping entry MUST have a `dimensions:` field of type `Set<Dimension>` AND a `testclaim_references:` field of type `Set<NodeRef>`, with members sourced from the lens enum + `tests/dag/*` TestClaim authorities exactly**.

This is the parallel-representation-debt prevention. If Layer 2's group-classification diverges from the lens's dimension axis (e.g., Layer 2 groups by file-area but lens groups by dimension), dissolution becomes a schema-migration rather than a column-retirement.

**Why `Set<Dimension>` not `Dimension`** (per PM caught semantic violation 2026-05-11 via codex RC on template PR #2721, fixed at `dedcf69a4`): `docs/design-affected-set-lens.md` §2 defines `affected_set` as a **union** over `Set<Dimension>`, not single-match. A multi-dim consumer (e.g., LBP demonstration reading both `complexity` + `cost`) declared with singular `dimension: cost` would be silently skipped when only `complexity` changes — a fail-open violation against P3. The set type makes the union semantics structurally faithful.

**Hard constraint**: no group entry without a `dimensions:` field of type `Set<Dimension>` with members from the lens enum. Single-element sets like `{cost}` are valid for single-dim groups. Empty set is invalid. If a group doesn't fit any of the 5 dimensions cleanly, escalate to Coordinator — that's a substrate-shape question, not a Layer 2 design choice.

**Polarity invariant** (per PM caught inversion 2026-05-11 via openai-pro RC #9721 + cursor APPROVE_WITH_COMMENTS on #2725 2026-05-12 catching dimensions-only residual): the carrier name is `skip_<group>`. CI consumer wires `if: skip_<group> != 'true'` (i.e., RUN when `skip` is false). **Post-dissolution, the dissolution formula is the CANONICAL 2-STEP JOIN per `docs/design-affected-set-lens.md:359` — BOTH NodeRef AND dimension intersections required**:

```
run  = (group.testclaim_references ∩ lens.affected_node_refs) ≠ ∅
       AND (group.dimensions ∩ lens.changed_dimensions) ≠ ∅
skip = ¬run = (refs ∩ nodes) = ∅  OR  (dims ∩ changed_dims) = ∅
```

**The dimensions-only form `skip = (dims ∩ changed_dims) = ∅` (i.e., using ONLY the dimension intersection clause from the canonical conjunction) is INCOMPLETE** — it silently drops the NodeRef-intersection step `(refs ∩ nodes) = ∅`, violating Facts Flow Forward (catch #9 absorption). Both inversion (`skip = (∩ ≠ ∅)` instead of `= ∅`) and dimension-only collapse are fail-open bug patterns.

**Bridge-tier note**: at bridge stage (pre-dissolution), per-group `skip_*` derives from `(changed-files ∩ required_paths_regex) = ∅` — a path-side proxy for the canonical 2-step (over-approximates: bridge runs MORE tests than canonical because regex coverage > NodeRef precision). Bridge-tier skip-form remains canonical (empty intersection) but operates on a coarser carrier than post-dissolution.

Any acceptance-criterion / YAML example / formula citation in this brief or its Mgr-fill output MUST use the canonical skip-form (empty intersection) AND, post-dissolution, the full 2-step conjunction — never invert + never collapse to dimensions-only.

## §4. Hard constraints

1. **Single source of truth for path classification** — the `changes` job (one job, one diff, one classifier). NO parallel `gunbc-quick` job; NO duplicate `git diff` invocation; NO per-group diff fork.
2. **STEP-level `if:` on `v3`, not separate jobs** — keeps `v3`'s `needs:` graph and required-check name stable. `self_host_ratchet` `if:` widening from PR #2718 remains unchanged.
3. **No new `actions/cache` keys or workflow-tier infrastructure** — Layer 2 is path-regex + boolean output; nothing more.
4. **Bridge-debt acknowledgment in every PR**: each PR landing a Layer 2 group must include in body: "Bridge-debt; lifecycle bounded by R4.B Introspect-lens saturation lane CI integration delivery (per `docs/design-affected-set-lens.md` §5). NOT R3 close-blocking. When R4.B CI integration lands, lens output replaces `required_paths_regex` column and bridge retires."
5. **`dimensions: Set<Dimension>` field on every group entry** — non-empty subset of the lens enum per locked-design §2 union semantics. Single-element sets valid for single-dim groups; multi-dim consumers MUST list all dimensions they read. Substrate-shape questions on dimension assignment escalate.
   **Polarity invariant** (post-cursor catch on #2725 review 2026-05-12): post-dissolution `run = (refs ∩ affected_node_refs) ≠ ∅ AND (dims ∩ affected_dimensions) ≠ ∅`; `skip = ¬run = either intersection ∅`. Bridge-tier proxy `skip = (changed-files ∩ required_paths_regex) = ∅` over-approximates canonical (runs more tests; fail-closed-safe). **Two fail-open bug patterns to reject in review**: (a) inversion `skip = (∩ ≠ ∅)` instead of `= ∅`; (b) dimension-only collapse `skip = (dims ∩ changed_dims) = ∅` (using ONLY the dimension intersection clause, dropping the NodeRef-intersection step from the canonical conjunction). Carrier name matches contract: `skip_*` flag is true when group is unaffected — i.e., **either** lens-join input is empty (`(refs ∩ nodes) = ∅` OR `(dims ∩ changed_dims) = ∅`). The "both lens-join inputs empty" framing is **stricter than canonical** and would itself be a fail-closed (run when canonical says skip) bug if implemented literally — reject in review.
6. **No closure-allowed carve-outs**: Layer 2's lifetime is bounded by the affected-set lens dissolution. If a group can't be path-classified accurately, it stays in the `code=true` full-run bucket (no special carve).
7. **Push events short-circuit to full-run** — `github.event_name == 'push'` bypasses ALL skip_* flags (run everything on main). Matches Layer 1.
8. **Hand-Rust budget: zero**. Layer 2 lives entirely in `.github/workflows/ci.yml` + an optional path-mapping data file (e.g., `scripts/ci-path-classification.yaml` or inline in the workflow).
9. **Shared-infrastructure full-run fail-closed bucket** (P3 invariant; per codex BLOCKING #9744 + openai-pro BLOCKING #9749 absorption). Per §2 mechanism: `force_full_run = (any changed file matches shared-infra regex)` short-circuits all per-group `skip_*` to `false`. The regex covers at minimum: `.github/*`, `scripts/*`, `Cargo.{toml,lock}`, `rust-toolchain.toml`, `.cargo/*`, `build.rs`, AND the **harness/test-selection-machinery class** — `tests/integration/common/*`, `sg0_census_test.rs`, `test_runner_test.rs`, `t_pb_b_1_dag_runner_test.rs`, and integration test entry points. Per-group regexes cover ONLY their own `src/v3/*` deps; the full-run trigger is the join-point that catches inter-group / cross-cutting changes. **Never collapse the full-run trigger into per-group regexes** — that's the structural fail-open shape. **A harness-class file MUST never appear in a per-group `required_paths_regex`** — its appearance in a changed-file list always triggers full-run.
10. **`test_pattern` is libtest SUBSTRING filter, NOT a glob** (per openai-pro BLOCKING #9749 absorption). The positional argument to `cargo test ... --test integration <pattern>` is libtest's test-name substring filter. `cost_lens` matches all tests whose name CONTAINS "cost_lens" literally. Glob characters (`*`, `?`) are treated as literal substring characters and would silently match zero tests. **Never use shell-glob syntax in the `test_pattern` field.** If precision beyond substring is needed, use `--exact` with the specific test name OR enumerate via cargo metadata. Self-test: a worker should be able to copy the `test_pattern` value verbatim into `cargo test -p v3-compiler --test integration <value>` and have it run a positive number of intended tests.

## §5. Acceptance

The Layer 2 PR-set is acceptable when:

- `changes` job grows per-group `skip_*` outputs (no parallel job created)
- `v3` step-level `if:` predicates wired for each group
- Per-group path-mapping table cites all 3 inventory sources (a)(b)(c)
- Every group entry has `dimensions: Set<Dimension>` field (non-empty subset of `docs/design-affected-set-lens.md` §2 enum); set semantics preserve multi-dim consumer fidelity per locked-design union
- **Every group entry has `testclaim_references: Set<NodeRef>` field** (per canonical 2-step join at `docs/design-affected-set-lens.md:359` + Brian's BLOCKING #2 absorption + codex BLOCKING #9780 absorption 2026-05-12) — populated from `tests/dag/*` TestClaim authorities at Mgr-fill time. NodeRef-precise version of group-tests-membership; survives the dissolution alongside `dimensions`. Bridge-tier proxy is `required_paths_regex` (file-path-coarse); post-dissolution, `testclaim_references` replaces the regex column in the 3-column surviving schema. **Dimensions-only acceptance closeout is rejected**: P2 facts-flow-forward requires both lens-join inputs.
- **Polarity check passes**: every YAML / formula / acceptance-text reference to the dissolution formula uses canonical skip-form `skip = (∩ = ∅)` or equivalent run-form `run = (∩ ≠ ∅)`. Inverted form `skip = (∩ ≠ ∅)` is the fail-open bug pattern; reject in review.
- **Shared-infrastructure full-run check passes** (P3 fail-closed): `force_full_run = (any changed file matches shared-infra regex)` is implemented at the `changes` job level; when true, all `skip_*` outputs short-circuit to `false`. Self-test: a PR touching ONLY `.github/workflows/ci.yml` or `Cargo.lock` or `scripts/check-test-timeout.sh` or `src/v3/compiler/tests/integration/common/cached_compile.rs` (harness-class) MUST run all test groups (not just the ones whose `src/v3/*` regexes coincidentally match). Harness class explicitly verified: any change to `sg0_census_test.rs` / `test_runner_test.rs` / `t_pb_b_1_dag_runner_test.rs` / `tests/integration/common/*` triggers full-run.
- **`test_pattern` substring-filter check passes** (P3 fail-closed): no `test_pattern` value contains glob characters (`*`, `?`); each value, when substituted into `cargo test -p v3-compiler --test integration <value>` and executed locally, runs a positive number of intended tests. Self-test: pilot wave PR validates this empirically — run the cost-lens group's `test_pattern` and confirm `running N tests` output shows N ≥ expected group cardinality.
- Self-test (4 cases): (a) docs-only PR triggers Layer 1 (entire `v3` skip — `code=false`); (b) a code PR touching only `src/v3/lenses/cost.dag` triggers ONLY the cost-related test groups (cost-dimension groups run; other-dimension groups skip); (c) a code PR touching only `.github/workflows/ci.yml` triggers ALL test groups via `force_full_run` (P3 fail-closed for shared infra); (d) a `push` to main runs everything
- PR body explicitly states bridge-debt + dissolution trigger
- `self_host_ratchet` required-check name remains green via existing PR #2718 `if:` widening
- No new hand-Rust files; no SG-0 census changes
- Empirical validation against `/tmp/v3-test-timings.log` last-N runs: median per-group skip-rate captured + expected CI-time savings projected

## §6. Decomposition (Mgr-fill)

Recommended split (subject to Mgr judgment + PM pre-staged Cluster B recommendation):

- **Pilot wave** (1 cluster; ~2 hours): **Cluster B (Lane 2 Stage 2d symbolic cost)** per PM template recommendation — high confidence, singleton `dimensions: {cost}`, ~6 tests. Converges with my prior `cost_lens`-first recommendation; Cluster B IS the cost-lens family in the PM grouping. Validates per-group `skip_*` output + STEP-level `if:` mechanism on `v3` + set-typed `dimensions` invariant against locked-design §2.
- **Class wave** (5-8 clusters; ~1 day): parallel-dispatch per remaining PM Clusters A/C-I from empirical timings. Each cluster's PR is bounded; reviewer can verify required-paths regex against test deps + `dimensions:` set assignment against lens enum (must include ALL dimensions the consumer reads, not just primary).
- **`[Mgr-fill]` placeholder resolution** (12 entries per PM template): per-entry escalation as consumer-tracing surfaces substrate-lens deps / R3-V L4/L7 / R1C-E `.dag` wrapper / free-consequences cross-target shapes. These may bundle with the corresponding cluster waves or stand alone.
- **Long-tail wave** (remaining groups + edge-case path-regex tuning): per-group escalation if path-classification accuracy issues surface.

## §7. STOP and escalate

Escalate via dashboard-message to Verification Mgr (`clever-tern-670`) if:

- A test group can't be cleanly assigned a non-empty `Set<Dimension>` from the locked-design §2 enum (substrate-shape question, not a Layer 2 design choice). **Do NOT default to a singleton `{primary}` to bypass this** — that's the fail-open multi-dim collapse pattern §3 Polarity invariant + §4 hard constraint #5 explicitly forbid. If the group genuinely reads multiple dimensions and you can't enumerate them confidently, escalate.
- The `changes` job exceeds 3-minute cap once Layer 2 classification logic added (mechanism-shape question)
- Required-paths regex authoring produces false-negatives (test skipped that should have run) in self-test — escalate to widen regex, NOT to disable group classification
- `self_host_ratchet` `if:` widening breaks when interacting with per-step `if:` — coordinate with PR #2718 author for the predicate composition
- Layer 2 dissolution shape (when R4.B CI integration lands) doesn't match the `(group_name, dimensions, testclaim_references)` schema per canonical 2-step join — substrate-shape question, escalate to Substrate Mgr coordinator
- A group consumer reads multi-dim but it's unclear which dimensions are load-bearing — escalate to Coordinator for dimension-set assignment (do NOT default to singleton `{primary}`; that's the fail-open shape PM caught in template review)

Do not push a workaround PR for any of these.

## §8. Bridge-debt + dissolution path

**This brief produces a bridge.** Per BridgeLedgerZero discipline + `feedback_bridge_debt_window_cadence`, every bridge has a named dissolution trigger:

- **Bridge**: per-group `required_paths_regex` table (hand-authored, file-path-substring-based)
- **Dissolution trigger** (R4-bounded per Brian BLOCKING #1 absorption): R4.B Introspect-lens saturation lane delivers CI integration per `docs/design-affected-set-lens.md` §5 ⇒ affected-set lens output replaces `required_paths_regex` column. **No current ROADMAP gate name exists** for this dissolution; the prior placeholder `ci_uses_provable_minimal_affected_set_selection` was Director-tier speculation and has been removed throughout this brief. Mgr re-cites concrete gate ID once R4.B authority lands one.
- **Surviving artifact post-dissolution** (corrected per Brian's BLOCKING #2 absorption 2026-05-12): `(group_name, dimensions, testclaim_references)` mapping — BOTH `dimensions: Set<Dimension>` AND `testclaim_references: Set<NodeRef>` columns survive as lens consumers per the canonical 2-step selection join (`docs/design-affected-set-lens.md:359`). Only the `required_paths_regex` column retires; `testclaim_references` replaces it as the NodeRef-precise version of group-tests-membership. Surviving schema is 3 columns, not 2.

Cite the dissolution path in every Layer 2 PR body. When the gate lands, a single follow-up PR retires the bridge and the brief is done.

---

**Mgr-finalization checklist** (before flipping to PRE-AUTH DISPATCH-READY):

- [ ] Complete per-group inventory (sources (a)+(b); table column (c)) — recommend PM pre-staged skeleton if/when available
- [ ] Per-group `dimensions: Set<Dimension>` assignments validated against `docs/design-affected-set-lens.md` §2 union semantics — multi-dim consumers MUST list all dimensions they read (no silent-skip on non-primary dim changes)
- [ ] **Per-group `testclaim_references: Set<NodeRef>` populated** (per Brian BLOCKING #2 absorption + codex non-blocking improvement #9817; required for canonical 2-step join post-dissolution; sourced from `tests/dag/*` TestClaim authorities at Mgr-fill time; P2 facts-flow-forward gate)
- [ ] Per-group `required_paths_regex` tested against 3-5 representative recent PRs for false-positive/false-negative rate
- [ ] Pilot wave selection (recommend `cost_lens` first per §6)
- [ ] Coordination ack from PR #2718 author on `self_host_ratchet` `if:` interaction shape

**End of scaffold.** Director-tier shape established; Verification Mgr fills inventory + per-group regex + pilot selection + dispatch.

---

## Authority footer

- **Operator directive**: gunbc#846 c4425420798 (CI mitigation urgent escalation 2026-05-11)
- **Layer 1 PR**: gunbc#2718
- **PM ratification**: gunbc#828 c4425726922 (GitHub inbox fallback — dashboard-message service flap)
- **Director ratification**: dashboard-message msg_a77c7f42 (Verification Mgr routing per `feedback_parallel_representation_debt` coherence)
- **Locked design**: `docs/design-affected-set-lens.md` (canvas at gunbc#2713)
- **Phase 3 scaffold pattern reference**: `docs/briefs/r3-v-cluster-m-84-class-reflected-dag-bulkport-worker.md` + `docs/briefs/r3-v-cluster-m-84-class-generic-dimreport-bulkport-worker.md` (Director scaffold-fill pattern; commit landed via PR #2708 squash)
