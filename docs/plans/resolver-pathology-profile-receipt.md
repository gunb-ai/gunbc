# Resolver pathology — cold-resolve profile receipt

**Status:** measurement doc only (no resolver refactor). Timestamped profiling receipt for ROADMAP §1 *profile the 518s resolve*. DESIGN.md remains authority.

**Owner:** vivid-carp-798. Shell entrypoint: `scripts/profile-cold-resolve.sh` (SCAFFOLD — dissolve-on: substrate-emitted resolve timings / gunbc bash-emit #5828).

---

## 1. Reproducible commands

```
unset GUNBC_RESOLVED_GRAPH_CACHE_DIR
./scripts/profile-cold-resolve.sh          # corpus + top-15
./scripts/profile-cold-resolve.sh --pair   # budget_roster vs fold_list twin
```

---

## 2. Measured top resolve offenders (warm typed_module_cache)

| Rank | ms | items | entry |
| --- | --- | --- | --- |
| 1 | 9410 | 1973 | v2/compiler/manual/dag_import_block_lexeme_stamp_test.dag |
| 2 | 6164 | 2594 | v2/test/claim/emit_host_gate/target_coverage_completeness_test.dag |
| 12 | 1556 | 2412 | complexity_gate/budget_roster_completeness_test.dag |

---

## 3. Pathological pair

**Pathological:** `budget_roster_completeness_test.dag` — cold resolve 5513ms, 2412 registry items. **Structural twin:** `fold_list_generic_instantiation.dag` — 159ms, 226 items. **Ratio:** 34.7× wall (operator ~450× targets typecheck/item work; see prose receipt).

---

## 4. Hypothesis

**`reconcile_with_typed_cache` / `typecheck_module`** — fn-typed `List<LensSubjectComplexityBudgetRow>` roster rows pulling source-bridged `v2.compiler.*` modules. Not `resolve_modules` wiring (twins share 57-module closure at ~5.5s cold).

## Dissolution trigger (DESIGN §6)

Delete when resolve-phase timings are substrate-emitted and ROADMAP 1-resolver-pathology closes with a fix PR.
