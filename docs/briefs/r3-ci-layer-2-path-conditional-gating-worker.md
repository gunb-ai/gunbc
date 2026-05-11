# R3 CI Layer 2 — Per-test path-conditional gating worker brief

**Status:** DIRECTOR-SCAFFOLD (authored under standing authority per `feedback_director_mgr_energy_input` 2026-05-11; awaiting Verification Mgr finalization). **Bridge-debt** with named dissolution trigger.

**Owner:** worker TBD on dispatch. **Coordinator:** R3 Verification Mgr (`clever-tern-670`). **Authority chain:** PM ratification (msg_45457c77 + GitHub inbox c4425726922) → Director ratification (msg_a77c7f42; routing Verification Mgr per parallel-representation-debt coherence) → Verification Mgr finalization → auto-spawn dispatch.

**Authority (cite-and-execute):**
- **Operator escalation**: Brian's CI-up-to-1-hour framing at gunbc#846 2026-05-11 ("their CI is up to 1 hour ... not happening" — re v3 ratchet attempt context)
- **Layer 1 prior art**: PR #2718 `ci(layer1): skip v3 job on docs-only PRs via changes-filter` — the `changes` job mechanism this brief EXTENDS (not parallels)
- **Bridge-debt → dissolution trigger**: `docs/design-affected-set-lens.md` §5 (affected-set Introspect-lens R3 close-blocking gate) — when `ci_uses_provable_minimal_affected_set_selection` gate lands, Layer 2's hand-authored path-mapping table dissolves and per-group `skip_*` flags become `affected_dimensions.contains(group.dimension)`
- **Per-dimension structural target**: `docs/design-affected-set-lens.md` §2 enum (`value | cost | complexity | effect | refinement`) — every Layer 2 path-mapping entry MUST carry a `dimension:` field matching this enum to prevent schema divergence from future lens output
- **Slow-test inventory sources** (per PM pre-stage at #828 c4425726922):
  - (a) `scripts/slow-test-exemptions.txt` — 78 active entries (curated >2s ratchet exemption list)
  - (b) `/tmp/v3-test-timings.log` — empirical per-test wall-time captured by every CI run via `--report-time` (consumed by `scripts/check-test-timeout.sh`, wired at `.github/workflows/ci.yml:405-424`)
  - (c) NEW per-group required-paths mapping (the deliverable; bridge-debt artifact)

---

## §0. Scope

Extend PR #2718's `changes` job with per-test-group `skip_*` boolean outputs derived from the same `changed`-files list and per-group required-paths regex. Wire `v3` STEP-level `if:` predicates to skip individual test groups whose path-dependencies are unaffected by the PR's diff.

**Not in scope**: a parallel CI gate mechanism; new path-classification source-of-truth (must reuse PR #2718's `gunbc-quick` `changes` job).

**Single source of truth invariant**: all path classification flows from one `changes` job. Layer 1 produces `code: bool`; Layer 2 grows additional outputs (`skip_lens: bool`, `skip_emit: bool`, `skip_parser: bool`, ...) on the same job. No second `gunbc-quick` job. No `actions/cache`-fork. No parallel diff mechanism.

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
    skip_lens: ${{ steps.classify.outputs.skip_lens }}
    skip_emit: ${{ steps.classify.outputs.skip_emit }}
    skip_parser: ${{ steps.classify.outputs.skip_parser }}
    skip_cost: ${{ steps.classify.outputs.skip_cost }}
    # ... per-group entries per Mgr-fill inventory
  steps:
    - <existing diff step>
    - id: classify
      run: |
        # for each group in per-group-required-paths-table:
        #   skip_<group> = "true" if (changed files ∩ required_paths) is empty else "false"
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
      run: cargo test -p v3-compiler --test integration cost_lens_*
    - name: emit-target integration
      if: ${{ needs.changes.outputs.skip_emit != 'true' }}
      run: cargo test -p v3-compiler --test integration *_emit_*
    # ... per-group test invocations
```

**Job-level `code` flag retained**: docs-only PRs still skip the entire `v3` job (~67min → 0min). Layer 2 makes the granularity finer for code PRs whose changed-paths affect only some groups.

## §2. Inventory sources + per-group table shape

Per-group mapping table (the deliverable):

```
(group_name, dimension, required_paths_regex, test_pattern)
```

Where:
- `group_name` — short identifier (e.g., `cost_lens`, `emit_target`, `parser_grammar`)
- `dimension: Dimension` — value | cost | complexity | effect | refinement (load-bearing — matches future lens output enum)
- `required_paths_regex` — regex over changed file paths; if no changed file matches, skip this group
- `test_pattern` — `cargo test` arg pattern selecting the group's tests

**Inventory derivation** (Mgr-fill from 3 sources):

(a) **`scripts/slow-test-exemptions.txt`** — start with the 78 active >2s entries. Each entry already has citation discipline; group by `_test.rs` file-area prefix.

(b) **`/tmp/v3-test-timings.log` empirical** — last N CI runs aggregated → top-K slowest groups by file-area. Cross-validates (a) and surfaces non-exempted slow tests.

(c) **NEW per-group required-paths mapping** — for each group, hand-author the required-paths regex by examining which `src/v3/*` files the group's tests transitively depend on. This is the bridge-debt artifact; dissolves when the affected-set lens lands.

**Starting template** (PM pre-staged skeleton to be attached if/when available):
```
cost_lens          | cost       | ^(src/v3/lenses/cost\.dag|src/v3/std/algebra\.dag|src/v3/compiler/src/lens_cost_.*\.rs)$       | cost_lens_*
complexity_lens    | complexity | ^(src/v3/lenses/complexity\.dag|src/v3/compiler/src/lens_complexity_.*\.rs)$                    | complexity_lens_*
emit_target        | effect     | ^(src/v3/extdeps/.*|src/v3/compiler/src/emit/.*|src/v3/compiler/src/omni_shape_.*\.rs)$          | emit_target_*
parser_grammar     | refinement | ^(src/v3/parser/.*|src/v3/compiler/src/parser.*\.rs|src/v3/compiler/src/lower.*\.rs)$            | parser_grammar_*
# ... etc per Mgr-fill
```

**[Mgr-fill]**: full per-group table — exhaustive coverage of `scripts/slow-test-exemptions.txt` 78 entries grouped + empirical top-K from timings log + per-group required-paths regex tested against representative diffs.

## §3. Per-dimension structural target — `feedback_parallel_representation_debt` prevention

The Layer 2 path-mapping is bridge-debt by design. The dissolution is the affected-set Introspect-lens (canvas PR #2713 by `clever-tern-670`, locked-design `docs/design-affected-set-lens.md`). When the lens lands, the dissolution is:

```yaml
# Post-dissolution (after ci_uses_provable_minimal_affected_set_selection gate)
changes:
  steps:
    - id: lens
      run: |
        cargo run -p v3-compiler --bin affected_set_lens -- \
          --pr-diff origin/main...HEAD \
          --output /tmp/affected.json
    - id: classify
      run: |
        # per-group skip_* derived from lens output, not path-regex
        for group in cost_lens emit_target parser_grammar ...; do
          dim="${group_dimension[$group]}"
          if jq -e ".affected_dimensions | contains([\"$dim\"])" /tmp/affected.json; then
            echo "skip_$group=false" >> $GITHUB_OUTPUT
          else
            echo "skip_$group=true" >> $GITHUB_OUTPUT
          fi
        done
```

**The `(group_name, dimension)` mapping survives the dissolution** — only the `required_paths_regex` column gets retired (replaced by lens-provided per-dimension affected-set). For this to work, **every Layer 2 path-mapping entry MUST have a `dimension:` field matching the lens enum exactly**.

This is the parallel-representation-debt prevention. If Layer 2's group-classification diverges from the lens's dimension axis (e.g., Layer 2 groups by file-area but lens groups by dimension), dissolution becomes a schema-migration rather than a column-retirement.

**Hard constraint**: no group entry without a `dimension:` field matching the lens enum. If a group doesn't fit one of the 5 dimensions cleanly, escalate to Coordinator — that's a substrate-shape question, not a Layer 2 design choice.

## §4. Hard constraints

1. **Single source of truth for path classification** — the `changes` job (one job, one diff, one classifier). NO parallel `gunbc-quick` job; NO duplicate `git diff` invocation; NO per-group diff fork.
2. **STEP-level `if:` on `v3`, not separate jobs** — keeps `v3`'s `needs:` graph and required-check name stable. `self_host_ratchet` `if:` widening from PR #2718 remains unchanged.
3. **No new `actions/cache` keys or workflow-tier infrastructure** — Layer 2 is path-regex + boolean output; nothing more.
4. **Bridge-debt acknowledgment in every PR**: each PR landing a Layer 2 group must include in body: "Bridge-debt; dissolves when `ci_uses_provable_minimal_affected_set_selection` gate lands and lens output replaces `required_paths_regex` column."
5. **Dimension field on every group entry** — no group without `dimension: <Dimension>` field. Substrate-shape questions on dimension assignment escalate.
6. **No closure-allowed carve-outs**: Layer 2's lifetime is bounded by the affected-set lens dissolution. If a group can't be path-classified accurately, it stays in the `code=true` full-run bucket (no special carve).
7. **Push events short-circuit to full-run** — `github.event_name == 'push'` bypasses ALL skip_* flags (run everything on main). Matches Layer 1.
8. **Hand-Rust budget: zero**. Layer 2 lives entirely in `.github/workflows/ci.yml` + an optional path-mapping data file (e.g., `scripts/ci-path-classification.yaml` or inline in the workflow).

## §5. Acceptance

The Layer 2 PR-set is acceptable when:

- `changes` job grows per-group `skip_*` outputs (no parallel job created)
- `v3` step-level `if:` predicates wired for each group
- Per-group path-mapping table cites all 3 inventory sources (a)(b)(c)
- Every group entry has `dimension: <Dimension>` field matching `docs/design-affected-set-lens.md` §2 enum
- Self-test: a docs-only PR still triggers Layer 1 (entire `v3` skip — `code=false`); a code PR touching only `src/v3/lenses/cost.dag` triggers ONLY the cost-related test groups; a `push` to main runs everything
- PR body explicitly states bridge-debt + dissolution trigger
- `self_host_ratchet` required-check name remains green via existing PR #2718 `if:` widening
- No new hand-Rust files; no SG-0 census changes
- Empirical validation against `/tmp/v3-test-timings.log` last-N runs: median per-group skip-rate captured + expected CI-time savings projected

## §6. Decomposition (Mgr-fill)

Recommended split (subject to Mgr judgment + PM pre-staged skeleton availability):

- **Pilot wave** (1-2 groups; ~2 hours): pick the highest-signal pair (e.g., `cost_lens` + `parser_grammar` — large slow-test surface, well-bounded path-dependencies). Validates the per-group `skip_*` output + STEP-level `if:` mechanism on `v3`.
- **Class wave** (5-8 groups; ~1 day): parallel-dispatch per top-K groups from empirical timings. Each group's PR is bounded; reviewer can verify required-paths regex against test deps.
- **Long-tail wave** (remaining groups + edge-case path-regex tuning): per-group escalation if path-classification accuracy issues surface.

**Pilot ordering recommendation**: start with `cost_lens` because the affected-set lens canvas (PR #2713) is also `cost_lens`-aware; validates the dimension-mapping invariant against locked-design §2 enum directly.

## §7. STOP and escalate

Escalate via dashboard-message to Verification Mgr (`clever-tern-670`) if:

- A test group can't be cleanly assigned a single `Dimension` (substrate-shape question, not a Layer 2 design choice)
- The `changes` job exceeds 3-minute cap once Layer 2 classification logic added (mechanism-shape question)
- Required-paths regex authoring produces false-negatives (test skipped that should have run) in self-test — escalate to widen regex, NOT to disable group classification
- `self_host_ratchet` `if:` widening breaks when interacting with per-step `if:` — coordinate with PR #2718 author for the predicate composition
- Layer 2 dissolution shape (when affected-set lens lands) doesn't match the `(group_name, dimension)` schema — substrate-shape question, escalate to Substrate Mgr coordinator

Do not push a workaround PR for any of these.

## §8. Bridge-debt + dissolution path

**This brief produces a bridge.** Per BridgeLedgerZero discipline + `feedback_bridge_debt_window_cadence`, every bridge has a named dissolution trigger:

- **Bridge**: per-group `required_paths_regex` table (hand-authored, file-path-substring-based)
- **Dissolution trigger**: gate `ci_uses_provable_minimal_affected_set_selection` lands ⇒ affected-set Introspect-lens output replaces `required_paths_regex` column
- **Surviving artifact post-dissolution**: `(group_name, dimension)` mapping — the dimension column remains as the lens consumer; only path-regex column retires

Cite the dissolution path in every Layer 2 PR body. When the gate lands, a single follow-up PR retires the bridge and the brief is done.

---

**Mgr-finalization checklist** (before flipping to PRE-AUTH DISPATCH-READY):

- [ ] Complete per-group inventory (sources (a)+(b); table column (c)) — recommend PM pre-staged skeleton if/when available
- [ ] Per-group `dimension:` assignments validated against `docs/design-affected-set-lens.md` §2 enum
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
