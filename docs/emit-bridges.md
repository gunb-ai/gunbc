> Part of: [phase1-lane3-consolidation-build-plan.md](./phase1-lane3-consolidation-build-plan.md) · [spec-field-gaps.md](./spec-field-gaps.md)

# Emitter bridges inventory (Stage 1d)

**Definition:** A **bridge** is any place Rust emission code encodes a policy the **target spec** should own: string comparisons on substrate names, hardcoded syntax, special cases for std shapes, or target-only filtering that is not already a data field in `rust.dag` / `go.dag` / `python.dag`.

**Live-state note:** the Go target's implementation moved from
`src/v3/compiler/src/emit_go.rs` to `src/v3/compiler/src/emit.rs`
behind the shared entrypoint scaffold. References to Go sites below name
the pre-move file because this inventory was the Stage 1d audit
snapshot; the bridge classifications still apply to the moved body.

**Goal:** Each bridge lists **what kills it** (dissolution) and **what spec or substrate fact replaces it**.

---

## Verification methodology (grep-friendly)

These commands are intentionally broad — they over-approximate “name-adjacent” logic so the inventory stays **complete**; manual triage marks which rows are benign (single-sourced bootstrap) vs debt.

**Policy:** Treat `rg` here as **Stage 1d reconnaissance input** to name bridges. If a pattern later becomes an enforcement gate, route it through a **lens**, **substrate** fact, or **compiler ratchet** — not permanent “CI grep” as the long-term authority for semantics (see `ROADMAP.md` on moving beyond source grep for system behavior).

**A — Tuple field probes on structural value bodies** (`(label, value)` string keys):

```bash
rg 'find\(\|\(label, _\)\| label ==' src/v3/compiler/src/emit_{rust,go,python}.rs
```

**B — `named_variant_id(dag, "TypeName", "Variant")`**: resolves enum-like declarations by **parent type name + variant label** (emit-time, not only bootstrap).

```bash
# From repo root (`gunbc/`). Per-file counts: `rg -c '…' path1 path2 path3`
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

**E — Positional payload field name `_0`** (`label == "_0"` / `label != "_0"` on conj children — tuple / anonymous single-field payload routing):

```bash
rg 'label == "_0"|label != "_0"' src/v3/compiler/src/emit_{rust,go,python}.rs
```

**Reproducible site counts** (run from repo root; paths `src/v3/compiler/src/emit_{rust,go,python}.rs`):

| Bucket | emit_rust | emit_go | emit_python | **Total** | Command / pattern |
|--------|----------:|--------:|------------:|----------:|-------------------|
| **A** `(label, _) == "…"` probes | 10 | 11 | 4 | **25** | `rg -c 'find\(\|\(label, _\)\| label =='` … |
| **B** `named_variant_id(dag,` call sites | 29 | 8 | 3 | **40** | `rg -c 'named_variant_id\(dag,'` … — matches §B above |
| **B′** `named_variant_id(` (any) | 34 | 9 | 4 | **47** | `rg -c 'named_variant_id\('` … — includes **3**× `fn named_variant_id` **definitions** (one per file) → **44** other occurrences on those lines |
| **C** `declaration_by_name("` | 9 | 1 | 1 | **11** | `rg -c 'declaration_by_name\("' …` |
| **D** `label == "Empty"\|…` | 2 | 4 | 2 | **8** | `rg -c 'label == "(Empty\|Cons\|None\|Some)"'` … |
| **E** `label == "_0"` / `label != "_0"` (positional payload) | 2 | 0 | 2 | **4** | `rg -c 'label == "_0"\|label != "_0"'` … |

**Sums for planning:** **A + B + C + D = 25 + 40 + 11 + 8 = 84** (distinct pattern families; **B** is the methodology-consistent count for `named_variant_id(dag,`). **B′** is the broader line count if you include `fn named_variant_id` headers. **E** is the **`_0` positional-payload bridge** (B13): **Rust and Python** both compare conj field labels to **`_0`**; **Go has none** under this regex. **A–D + E = 88** line occurrences across buckets (no double-count between rows).

The earlier **“≥ 86 sites”** lane estimate used a looser union/overlap mental model; **84** for **A–D** and **4** for **E** are reproducible on this tree (**emit_rust ≫ emit_go ≫ emit_python**).

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
| `emit_python.rs` `render_match_binding` (and ctor path) | `children[0].label == "_0"` when arity-1 tuple payload | **P1:** Same — positional payload as a **substrate fact**, not a string sentinel. |
| `emit_rust.rs` `render_path_body` | `child.label != "_0"` — multi-field vs single **positional** `_0` branch for `field_overrides` | **P1:** Same dissolution — **named fields vs anonymous `_0`** should be data on the variant payload Conj, not string-compare in emit. |
| `emit_rust.rs` `render_branch_pattern` | `children[0].label == "_0"` → `variant_pattern_positional` template vs field-bound pattern | **P1:** Positional vs named payload is a **pattern strategy** in spec (already partially via templates; remove `_0` string anchor). |

**Kills it:** Spec / substrate declares **which variant DeclarationIds** play which role for optional/list **and** whether a single-field payload uses **positional** vs **named** projection — emit compares **DeclarationId** roles / strategy enums, not magic labels.

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
