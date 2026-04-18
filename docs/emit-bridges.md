> Part of: [phase1-lane3-consolidation-build-plan.md](./phase1-lane3-consolidation-build-plan.md) · [spec-field-gaps.md](./spec-field-gaps.md)

# Emitter bridges inventory (Stage 1d)

**Definition:** A **bridge** is any place Rust emission code encodes a policy the **target spec** should own: string comparisons on substrate names, hardcoded syntax, special cases for std shapes, or target-only filtering that is not already a data field in `rust.dag` / `go.dag` / `python.dag`.

**Goal:** Each bridge lists **what kills it** (dissolution) and **what spec or substrate fact replaces it**.

---

## Verification methodology (grep-friendly)

These commands are intentionally broad — they over-approximate “name-adjacent” logic so the inventory stays **complete**; manual triage marks which rows are benign (single-sourced bootstrap) vs debt.

**A — Tuple field probes on structural value bodies** (`(label, value)` string keys):

```bash
rg 'find\(\|\(label, _\)\| label ==' src/v3/compiler/src/emit_rust.rs \
  src/v3/compiler/src/emit_go.rs src/v3/compiler/src/emit_python.rs
```

**B — `named_variant_id(dag, "TypeName", "Variant")`**: resolves enum-like declarations by **parent type name + variant label** (emit-time, not only bootstrap).

```bash
rg 'named_variant_id\(dag,' src/v3/compiler/src/emit_{rust,go,python}.rs
```

**C — `declaration_by_name("...")`**: absolute name anchors (bootstrap hooks, tests, **OrderedRing**).

```bash
rg 'declaration_by_name\("' src/v3/compiler/src/emit_{rust,go,python}.rs
```

**D — Literal variant labels in branching** (`Empty`, `Cons`, `None`, `Some`, …):

```bash
rg 'label == "(Empty|Cons|None|Some)"' src/v3/compiler/src/emit_{rust,go,python}.rs
```

**E — Payload shape** (`_0` tuple field):

```bash
rg 'label == "_0"' src/v3/compiler/src/emit_python.rs
```

**Reproducible site counts (2026-04, this worktree):**

| Bucket | Count | Command / pattern |
|--------|------:|-------------------|
| `(label, _) == "…"` structural probes | **25** | `rg 'find\(\|\(label, _\)\| label ==' emit_{rust,go,python}.rs` |
| `named_variant_id(` occurrences | **47** | `rg 'named_variant_id\(' emit_{rust,go,python}.rs` (includes **3** `fn named_variant_id` definitions — ~**44** call sites) |
| `declaration_by_name("` | **11** | `rg 'declaration_by_name\("' emit_{rust,go,python}.rs` |
| `label == "Empty"\|"Cons"\|"None"\|"Some"` | **8** | `rg 'label == "(Empty\|Cons\|None\|Some)"' emit_{rust,go,python}.rs` |
| **Ordered sum (categories overlap minimally)** | **≥ 86** | Matches the lane estimate (**emit_rust ≫ emit_go ≫ emit_python**). |

*Notes:* `named_variant_id` is **shared infrastructure** — it is still a bridge relative to the thesis “substrate ids only,” because **parent/variant strings** cross the boundary until the walker caches **DeclarationId** handles earlier. Dissolution is “resolve once at index time; emit only typed ids.”

---

## Bridge catalog (by theme)

### B11 — Target realization filtering by `rust_*` name prefix

| Where | What | Dissolution |
|-------|------|-------------|
| `RealizationIndexes` construction comments + tests (`emit_rust.rs` ~5394+) | Explains / asserts `data rust_* : …` surface vs `language: rust_language` | **Already** the model: filtering is by `language` ref, not string prefix in hot path. Keep **documentation + grep test** as guard; no extra spec field unless a future target violates the pattern. |
| Historical B11 concern | `name.starts_with("rust_")` style filtering | **Not present** in emit hot paths today; if reintroduced, replace with **`language` field** on realizations. |

**Kills it:** CI / invariant grep (E-6 style) + always filter by `LanguageSpec` id.

---

### B12 — Bootstrap path prefixes (`is_bootstrap_file`)

| Where | What | Dissolution |
|-------|------|-------------|
| `emit_rust.rs`, `emit_go.rs`, `emit_python.rs` — `is_bootstrap_file` | Hardcoded `dsl/std/`, `src/v3/std/`, `src/v3/spec/`, … prefixes to skip compiler’s own types in user emission | **P2:** `stdlib_path_prefixes: Vec<String>` (or substrate `WellKnownSourceRoots`) on `LanguageSpec` / `TargetExecutionModel`. |

**Kills it:** Spec-declared roots; walker reads list.

---

### B13 — Std sum-type **variant labels** in control flow (Python + Go; Rust list patterns)

| Where | What | Dissolution |
|-------|------|-------------|
| `emit_python.rs` `render_list_branch` | `v.label == "Empty"` / `"Cons"` | **P0/P1:** Declare **list constructor tags** on the **list template** decl (substrate metadata) or a **`MatchStrategy`** field in spec — emit compares **DeclarationId** roles, not strings. |
| `emit_go.rs` `render_optional_branch` | `variant.label == "None"` / `"Some"` | Same — optional template should expose **sentinel variant ids**. |
| `emit_go.rs` `render_vector_list_pattern_branch` | `Empty` / `Cons` arms | Same as Python list branch. |
| `emit_rust.rs` vector list pattern | `Empty` / `Cons` | Same. |
| `emit_python.rs` `render_branch_condition` | `variant_name == "None"` for optional | Name bridge — optional **DeclarationId**-backed display or spec rule. |
| `emit_python.rs` `render_match_binding` | `children[0].label == "_0"` for payload tuple shape | **P1:** Tuple projection spec or substrate **positional payload** fact — avoid magic `_0` string. |

**Kills it:** Spec / substrate declares **which variant DeclarationIds** play which role for optional/list; walker never compares user-facing names.

---

### B14 — `algebra_field_for_operator` → **`OrderedRing` by name**

| Where | What | Dissolution |
|-------|------|-------------|
| `emit_{rust,go,python}.rs` — `canonical_operator_field` | `dag.declaration_by_name("OrderedRing")` when algebra Conj walk fails | **P0:** Canonical fallback algebra as a **typed substrate marker** (or explicit spec field **`fallback_algebra: DeclarationRef`**) — no string lookup. |
| Same helper | `children.iter().find(|field| field.label == field_label)` | Acceptable **if** `field_label` is **only** `OperatorKind::algebra_field_name()` (single source) — document as **invariant**, not arbitrary user string. |

**Kills it:** Bootstrap publishes **`CanonicalOrderedRing` marker**; emit uses `DeclarationId`, not `"OrderedRing"` text.

---

### B15 — Hardcoded Rust attributes / derives

| Where | What | Dissolution |
|-------|------|-------------|
| `emit_rust.rs` `render_type_declaration` | Prepends `"#[derive(Clone, Debug)]\n"` before struct/enum templates | **P0:** `TypeDefinitionSyntax` gains **attribute template list** or `record_derive` carriers (see `spec-field-gaps.md` §2). |

---

### B16 — Substrate accessor miss error strings

| Where | What | Dissolution |
|-------|------|-------------|
| `emit_rust.rs` `render_substrate_accessor` | Error text references `rust_language` and `rust.dag` paths | **P1:** Templated diagnostic in spec or generic “missing `SubstrateAccessorBinding` for active `LanguageSpec`” with **typed** language display name. |

---

### B17 — Anonymous lambda name filter

| Where | What | Dissolution |
|-------|------|-------------|
| All three `emit_*_with_mode` | `name.starts_with("__anon_lambda_")` filters declarations | **P2:** Substrate marks anonymous lambdas with a **flag** on `Declaration` instead of name prefix (cleaner for non-Rust targets). |

---

### B18 — `CallableStrategy` label parse (`emit_go.rs`)

| Where | What | Dissolution |
|-------|------|-------------|
| `emit_go.rs` — parses callable strategy from **string label** (`"ListEmpty"`, `"ListCons"`, …) | Maps via `named_variant_id` | **P2:** Same as other `named_variant_id` sites — **pre-resolve at index build** so emit matches on **DeclarationId** only. |

---

### B19 — Half B pessimistic revisit items (explicitly in build plan §5)

| Item | What | Dissolution |
|------|------|-------------|
| Copy-type lens | User sum types conservatively non-`Copy` | **copy-type lens** in dedicated DAG; walker reads facts (`spec-field-gaps` §6). |
| `OwnedConstructLastUse` | Clone vs move | **ownership lens** + template-aware walker (build plan §5.B). |
| `Behavior::Loop` | Go/Python fail-closed; tests `#[ignore]` | **Loop** `BehaviorRealization` per target + shared semantics (`spec-field-gaps` §7). |

These are **not** all separate “name bridges,” but they are **behavior bridges** until specs + lenses land.

---

## Substrate support today

| Bridge class | Substrate ready? | Note |
|--------------|------------------|------|
| Typed `DeclarationId` realizations | **Yes** | Core M1(3) story — bridges are **string remnants** at index/emit edges. |
| `LanguageSpec` discrimination | **Yes** | Use `language` refs — B11 largely closed. |
| List / optional **variant roles** | **Partial** | Need explicit metadata or spec fields (B13). |
| Canonical algebra fallback | **Partial** | `OrderedRing` name is the gap (B14). |
| Rust derive attributes | **No** | Hardcoded (B15). |

---

## Suggested dissolution priority (for P2)

1. **B14** + **B15** + **Loop** (B19) — unblock “spec-complete” Rust/Go/Python.  
2. **B13** — remove cross-target string variant compares.  
3. **B12**, **B16**, **B17**, **B18** — polish and de-string remaining bootstrap.

---

## Related docs

- [emit-functions-inventory.md](./emit-functions-inventory.md) — per-function classification.  
- [spec-field-gaps.md](./spec-field-gaps.md) — spec fields that subsume bridges.  
- [phase1-lane3-consolidation-build-plan.md](./phase1-lane3-consolidation-build-plan.md) §5 — Half B revisit items tied to bridges.
