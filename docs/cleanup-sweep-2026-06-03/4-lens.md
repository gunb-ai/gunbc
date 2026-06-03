# cleanup-sweep #4: lens + workflow concerning-patterns catalog

**Slice:** `src/v4/lens/` (21 `.dag` files) + `src/v4/workflow/` (4 `.dag` files) + **ci.dag locate** (3 paths; authority is v4 workflow).

**Session:** royal-lynx-356 · **Read-only audit** (hard freeze — no code changes).

**Scale:** ~24.3k lines in slice; `src/v4/workflow/ci.dag` alone is 6,282 lines (~26% of slice).

---

## Slice — overall scariness: 🔴

`src/v4/workflow/ci.dag` dominates: triple authority (substrate + hand-synced `.github/workflows/ci.yml` + `tools/ci_affected_components` Rust mirror), ~462 digest/hash call sites, and 28 cross-domain imports. Lens side is mostly bounded modules except `testgen.dag` (2,314 lines) and `coverage.dag` (1,343 lines), which hand-roll TestClaim scheduling tables that feed CI selection.

**Slice-size note:** Recommend treating `src/v4/workflow/ci.dag` as its own future slice if edits are planned; the other 24 files are catalogable without it.

### ci.dag locations (operator flag)

| Path | Role | Scariness |
|------|------|-----------|
| `src/v4/workflow/ci.dag` | **v4 modeled authority** — pipeline AS DATA, T-21/T-24 affected-set, TestClaim roster, wave3 receipt | 🔴 |
| `dsl/gunbc/ci.dag` | v3 compiler-intent gates (`GateSource`); ci.yml transport shim | 🟡 |
| `dsl/extdeps/github/ci.dag` | GitHub workflow path facts (`.github/workflows/`) | 🟢 |

Consumers: `tools/ci_affected_components/*` (Rust mirror, parity-ratcheted), `.github/workflows/ci.yml` (live transport).

---

## Per-file (worst-first)

| Path | | bridges | towers | Note |
|------|---|---------|--------|------|
| `src/v4/workflow/ci.dag` | 🔴 | CI/YAML authority bridge (T-PB-B); live-workflow projection carriers; `NoShellScript` host transport; v2/v3 commands; wave2 shell-exception table; wave3 shadow `CiSelectionReceipt`; Rust mirror in `tools/ci_affected_components` | ~462 `combine_hash`/`content_hash`/`test_claim_*_digest`; 28-import concept sink; 79 `match` arms | Filled declarative core; transport/YAML dissolution open |
| `src/v4/lens/testgen.dag` | 🔴 | T-19 LBE/refinement scaffolds; `TestgenRunReceipt` shadow-CI bridge; NatAlgebraLawObligation⇄AlgebraLawSubject record bridge | Manual-anchor + generator scheduling tables; hand-enumerated `TestClaimCoproductVariant`→Symbol; per-law anchor classifier | Corpus producer; feeds `ci.dag` roster |
| `src/v4/lens/coverage.dag` | 🟡 | `dag_c3_surface_sugar_*` from extdeps; tight testgen coupling | Expected-vs-actual TestClaim scheduling tables | T-18 meta-lens |
| `src/v4/lens/affected_set.dag` | 🟡 | T-21 proof carriers `needs-more-work` | `content_hash` boundary folds + frontier graph fold | T-21/T-24 authority for `ci.dag` |
| `src/v4/lens/leaf_model_verification.dag` | 🟡 | Per-target extdeps facts (rust/python/go/typescript, pyright/mypy) | ~950-line fixture tables (R1/R2/R3 per target) | Data tower, not digest ladder |
| `src/v4/workflow/bootstrap.dag` | 🟡 | Placeholder `Hash` pins until T-15 B1 | Four-stage bootstrap + `fold_node` footprint | Structural wiring only |
| `src/v4/workflow/release.dag` | 🟡 | Hand-synced `release.yml`; mirrors `ci.dag` LB-P4-3213 pattern | Hand-expanded `release_pipeline` jobs | Same T-24 emission debt as CI |
| `src/v4/lens/application.dag` | 🟡 | Path-backed `DeclarationId`/`NodeId` (T-23) | — | T-23 filled; identity substrate pending |
| `src/v4/lens/structural_similarity.dag` | 🟡 | Full scaffold — `FnShapeUnrealized`, all `Unrealized` | — | Blocked on parse/resolve walk |
| `src/v4/lens/structural_resolution.dag` | 🟡 | Practice-10 witness predicates | — | Small |
| `src/v4/lens/complexity.dag` | 🟡 | Vacuous `Refined` oracle placeholder | — | Consumes `cost.dag` only (U1/U2 honored) |
| `src/v4/lens/parallelism.dag` | 🟡 | Staged `InferredFacts`; unresolved-relation predicate | — | T-13 with dissolution gates |
| `src/v4/lens/edit_locus.dag` | 🟡 | Git-diff name-only transport | — | Thin T-21 helper |
| `src/v4/lens/effect.dag` | 🟡 | Effect-enumeration coproduct interim | — | Small |
| `src/v4/lens/table_decision_tree.dag` | 🟡 | All verdicts `Unrealized` until `decision_tree_shape` | — | Substrate row for registry pointers |
| `src/v4/lens/registry.dag` | 🟢 | — | Closed `lens_registry_v0_*` data | PREFIX T-23 authority |
| `src/v4/lens/cost.dag` | 🟢 | — | SymbolicCost lattice (kernel model) | T-12 filled |
| `src/v4/workflow/cli.dag` | 🟢 | One 🟡 `GunbcTestCorpusHarnessRoute` (RR-A) | — | 131 lines; route contracts |
| **Cluster: small filled lenses** (`synthesis`, `subsumption`, `ownership`, `idempotency`, `identical_variant_payload`, `unused_parameters`, `fact_density`, `affected_set_examples`) | 🟢 | 0–1 🟡 gate each | — | No towers; safe reference surface |

| `dsl/gunbc/ci.dag` (out of slice dir) | 🟡 | v3 `gunbc.compiler` gate commands; ci.yml shim | Closed `GateSource` coproducts | Not v4 workflow authority |
| `dsl/extdeps/github/ci.dag` (out of slice dir) | 🟢 | 🟡 stem-segment validation deferred to WI-2 emitter | — | Platform paths only |

---

## Concerning patterns — unmodeled repetitive work

These are the **recurring bridges and hand-rolled towers** that appear in multiple places and should collapse to one modeled surface.

### 1. Digest/hash ladders (`combine_hash` + per-variant folds)

| | |
|---|---|
| **Pattern** | Private `combine_hash` chains over symbols, nodes, TestClaim arms, CI selection partitions, cache keys — instead of `content_hash` / structural projection on modeled carriers. |
| **Where it recurs** | **`ci.dag`** (~462 sites — dominant); `affected_set.dag` (boundary `content_hash`); `bootstrap.dag` (placeholder hash pins). |
| **One shared dissolution** | **B1 `content_hash(Node)` + whole-`TestClaim` projection** (T-15, T-21 IRT-4, T-22 / IRT-4) — receipt/cache/selection keys become derived from substrate, not hand-folded. |

### 2. CI triple authority (modeled pipeline ≠ live YAML ≠ Rust mirror)

| | |
|---|---|
| **Pattern** | Same semantics authored in substrate, hand-synced `.github/workflows/ci.yml`, and `tools/ci_affected_components` — plus wave2 **shell-exception table** listing steps not yet emitted from `CiUpsertStep`. |
| **Where it recurs** | **`ci.dag`** + `release.dag` (same hand-sync pattern for `release.yml`); `dsl/gunbc/ci.dag` (v3 gate transport). |
| **One shared dissolution** | **`project_github_actions` / YamlStatic emission** from `ci_pipeline` + `release_pipeline` (T-PB-B, T-24, gates #98/#100) — single projection authority; delete parallel transport. |

### 3. Practice-10 coproduct `match` towers (hand arm discrimination)

| | |
|---|---|
| **Pattern** | Hand-enumerated `match` over closed coproducts (`TestClaim`, `CiCommand`, `ReleaseCommand`, generator concepts) with 🟡 dissolve-on markers instead of reflection. |
| **Where it recurs** | **`testgen.dag`** (`TestClaimCoproductVariant`→Symbol, manual-anchor table); **`ci.dag`** (command properties, selection); **`release.dag`** (mirrors `ci_command_authority_ok`); **`coverage.dag`** (expected generator arms). |
| **One shared dissolution** | **`TestClaim` / command coproduct reflection** (T-19 codegen, LB-P4-3213) — arm-key→symbol and property helpers generated from substrate, not duplicated per consumer. |

### 4. TestClaim corpus scheduling tables (closed vocab as functions)

| | |
|---|---|
| **Pattern** | Large literal tables scheduling generators, manual anchors, law obligations, and coverage expectations — same closed vocabulary re-listed per lens. |
| **Where it recurs** | **`testgen.dag`** (2.3k lines); **`coverage.dag`**; consumed by **`ci.dag`** roster selection + shadow receipt. |
| **One shared dissolution** | **T-19 testgen codegen from substrate** + **registry-as-data schedule rows** (`nat_declared_algebra_law_obligations`, generator registry) — one schedule authority, lenses project slices. |

### 5. Live-workflow / host-transport bridge carriers

| | |
|---|---|
| **Pattern** | Interim types bridging modeled `ci_pipeline` to GHA: raw `if_condition` strings, `CiLiveWorkflowStepSignal`, `NoShellScript`, phase1 nat-semiring shell script, M1 probe signal rows. |
| **Where it recurs** | **`ci.dag`** (densest); **`cli.dag`** (`GunbcTestCorpusHarnessRoute` parallel to CI). |
| **One shared dissolution** | **`ci_job_scheduled_by_policy` + emitted step env from pipeline rows** (ci-bankruptcy Tier-0, T-38) — delete string/`Symbol` transport carriers. |

### 6. Path-/Symbol-backed identity (declaration handles)

| | |
|---|---|
| **Pattern** | `DeclarationId { path: Path }`, `DeclId = Symbol`, section refs — identity as paths/symbols without containment evidence. |
| **Where it recurs** | **`application.dag`**; **`structural_similarity.dag`**; **`affected_set.dag`** (via `application` ancestor walk). |
| **One shared dissolution** | **Declaration/node identity substrate** (T-23/T-10) — typed IDs + structural containment evidence from parse/resolve. |

### 7. Scaffold lenses (`Unrealized` / `needs-more-work` producers)

| | |
|---|---|
| **Pattern** | Substrate rows exist but producers fail-closed until another walk lands — consumers reference typed authority that always returns `Unrealized`. |
| **Where it recurs** | **`structural_similarity.dag`**, **`table_decision_tree.dag`**, **`affected_set.dag`** (proof carriers). |
| **One shared dissolution** | **Producer-stage walks** (`decision_tree_shape`, parse/resolve structural index, B-4/B-5 proof arms on `AffectedSet`) — one graph pass feeds multiple lenses. |

### 8. Per-target fixture tables (leaf model verification)

| | |
|---|---|
| **Pattern** | Repeated R1/R2/R3 compile/run/build fixture rows per language — closed tables over extdeps language facts. |
| **Where it recurs** | **`leaf_model_verification.dag`** only (large), but pattern mirrors testgen's table style. |
| **One shared dissolution** | **`LeafModelFixture` registry-as-data** in std/extdeps — lenses select slices, not re-author tables. |

---

## Recurring patterns (summary)

1. **Dual/triple CI authority** — modeled `ci.dag` vs `ci.yml` vs Rust tools; wave2 exception table documents the gap.
2. **TestClaim selection ahead of full eval** — testgen schedules; ci narrows by hash frontier + shadow receipts; T-38 structural slice closed, runtime verdict CI open.
3. **Practice-10 predicate dissolution** — hand `match` over coproducts pending reflection primitives.
4. **Placeholder hash/identity** — bootstrap pins, CI receipt keys, Path-backed declaration handles.
5. **Scaffold `Unrealized` lenses** — substrate reserved, producer walks not landed.

---

## Missing-substrate map

| Hand-roll / bridge | Recurs in | Shared surface that dissolves it |
|--------------------|-----------|----------------------------------|
| `combine_hash` receipt partitions, cache keys, claim selection | `ci.dag`, `affected_set.dag`, `bootstrap.dag` | `content_hash` / `Projection` on whole `TestClaim` + `TestClaimRun` persistence (T-15 B1, T-21 IRT-4, T-22, F11c) |
| GHA `if:` / step env / M1 probe signals | `ci.dag`, `cli.dag` | `project_github_actions` / YamlStatic from `ci_pipeline` (T-PB-B #98/#100) |
| Shell floor + `detect-affected-components.sh` | `ci.dag`, Rust tools | `CiUpsertStep` runtime authority + `ci_job_scheduled_by_policy` (T-24 Tier-0) |
| Manual-anchor + coproduct arm tables | `testgen.dag`, `coverage.dag`, `ci.dag` | T-19 codegen + `TestClaim` coproduct reflection |
| `NatAlgebraLawObligation` ⇄ `AlgebraLawSubject` record bridge | `testgen.dag` | Consume `std/nat` obligation rows directly; delete parallel record shape |
| Path/`Symbol` declaration handles | `application.dag`, `structural_similarity.dag` | Declaration/node identity substrate (T-23/T-10) |
| Exclusion/proof scaffold carriers | `affected_set.dag` | B-4/B-5 effect receipts + proof arms on `AffectedSet` |
| Symbolic bootstrap hash pins | `bootstrap.dag` | T-15 B1 merkle `content_hash` per stage |
| Hand-synced release YAML | `release.dag` | YamlStatic `release_pipeline` (mirror CI dissolution) |
| `lens_registry_v0_*` closed lists | `registry.dag` (healthy) | M2 fn-def scan extends registry-as-data (already on-model) |

---

## Escalation

None. Findings align with documented gates (T-24, T-PB-B, T-19, T-22). Audit only — no code changes under freeze.
