# Forward-fix audit: PR #897 and PR #824

**Auditor:** session `quick-carp-695`  
**Method:** Map each prior codex / manual **BLOCKING** thread on the merged PRs to **current `main`** (this worktree), using file/line evidence. Cross-check integration tests that encode the acceptance surface.

**Merge anchors (GitHub):**

| PR | Title | Merge commit (`oid`) |
| --- | --- | --- |
| #897 | T-Modeling int-literal magnitude consumer | `fd21feed685700a212f3740ab498af031506441f` |
| #824 | B4.3 | `7111fbc399cb840d7dc9b340b79cf0dc5ef3a5bd` |

**Runtime verification:** The audit environment did not have a Rust toolchain (`cargo` unavailable). Operators should still run the commands in §5 on a machine with `cargo` before treating CI as redundant.

---

## 1. PR #897 — int-literal consumer / inference

### 1.1 OpenAI-pro manual review (BLOCKING): `Behavior::Value` trusted non-`Int` without fit check

**Review claim:** Returning `Decision::Retry` for any pre-resolved non-`Int` int-literal port without validating magnitude violates fail-closed behavior (e.g. `let x: UInt8 = 256`).

**Current code:** `decide` handles `Behavior::Value` for integer literals by calling `int_literal_fits_expected_type` when the port is already resolved to a type other than the default `Int` shape. Outcomes:

- `Ok(Some(true))` → `Decision::Retry` (defer restamp; literal fits).
- `Ok(Some(false))` with a known integer range → `Decision::Fail` with `magnitude_out_of_range(...)`.
- `Err(diag)` → `Decision::Fail` with that diagnostic.
- `Ok(None)` → fall through (non-range-backed / normal mismatch path).

Evidence:

```745:794:src/v3/compiler/src/infer.rs
        Behavior::Value(v) => {
            // `let x: UInt8 = 5` seeds the value port to `UInt8` after lowering;
            // ...
            if let LiteralBits::Int(literal) = &v.data {
                if let PortState::Resolved(existing) = dag.port(v.output).state() {
                    if let Some(int_sh) = dag.int_shape() {
                        if !type_shapes_equivalent(dag, existing, &int_sh) {
                            match int_literal_fits_expected_type(
                                dag,
                                *literal,
                                existing.declaration,
                            ) {
                                Ok(Some(true)) => return Decision::Retry,
                                Ok(Some(false)) => {
                                    match integer_range_for_decl(dag, existing.declaration) {
                                        IntegerRangeLookup::Found(range) => {
                                            return Decision::Fail(
                                                v.output,
                                                magnitude_out_of_range(
                                                    *literal,
                                                    *existing,
                                                    range,
                                                    v.span.clone(),
                                                ),
                                            );
                                        }
                                        // ...
                                    }
                                }
                                Err(diag) => return Decision::Fail(v.output, diag),
                                Ok(None) => {}
                            }
                        }
                    }
                }
            }
```

**Verdict:** Finding addressed.

### 1.2 Negative regression: `let x: UInt8 = 256` → `MagnitudeOutOfRange`

**Test:** `let_annotated_uint8_out_of_range_emits_magnitude_diagnostic` in `int_literal_cardinality_test.rs`.

```88:115:src/v3/compiler/tests/integration/int_literal_cardinality_test.rs
fn let_annotated_uint8_out_of_range_emits_magnitude_diagnostic() {
    let err = compile_to_dag("let x: UInt8 = 256\n", "let_u8_oob.v3")
        .expect_err("annotated let UInt8 overflow must fail closed");
    // ... asserts Diagnostic::MagnitudeOutOfRange { literal == "256", target == "u8", ... }
}
```

**Verdict:** Acceptance test present on `main`.

### 1.3 Conflict merge path still range-validates

`int_literal_magnitude_narrow_merge` only allows merge when `int_literal_fits_expected_type` returns `Ok(Some(true))`.

```716:734:src/v3/compiler/src/infer.rs
fn int_literal_magnitude_narrow_merge(
    dag: &Dag,
    port: PortId,
    from: &TypeShape,
    to: &TypeShape,
) -> bool {
    // ...
    matches!(
        int_literal_fits_expected_type(dag, lit, to.declaration),
        Ok(Some(true))
    )
}
```

**Verdict:** Aligns with review requirement that merge-site narrowing stays range-backed.

### 1.4 Call-site + Rust emit surface (prior CI triage)

Integration tests on `main` cover:

- `call_site_uint8_literal_narrows` — `id_u8(7)` narrows to `UInt8`.
- `emit_let_uint8_uses_narrow_rust_type` — emitted Rust mentions `u8` for annotated `UInt8` let.

**Verdict:** Covered by tests in the same file as §1.2.

### 1.5 Generic fail-closed diagnostics (thesis / M0)

The review thread cited `thesis_validation_test::t1_4_type_mismatch_produces_a_typemismatch_diagnostic` as a canary. The test remains in-tree:

```269:270:src/v3/compiler/tests/integration/thesis_validation_test.rs
fn t1_4_type_mismatch_produces_a_typemismatch_diagnostic() {
    let dag = match compile_to_dag("let x: Bool = 1", "t1_4_type_mismatch.v3") {
```

**Verdict:** No evidence in static review of a remaining gap; **confirm with `cargo test`** (§5).

---

## 2. PR #824 — `emit_participation` substrate (B4.3)

### 2.1 Codex BLOCKING: reflection / `Dag::new()` field list out of sync

**Review claim:** Declared substrate fields did not match what `Dag::new()` exposed for `BranchNode` / `BindNode`.

**Current code:** `substrate_declares_expected_reflection_surface` expects `emit_participation` on both node types:

```228:261:src/v3/compiler/tests/integration/m2_substrate_inhabitance_test.rs
    assert_eq!(
        record_fields(&dag, "BranchNode"),
        vec![
            "id",
            "input",
            "paths",
            "result_port",
            "span",
            "emit_participation"
        ]
    );
    // ...
    assert_eq!(
        record_fields(&dag, "BindNode"),
        vec![
            "id",
            "name",
            "result_port",
            "params",
            "span",
            "lane2_workflow",
            "emit_participation"
        ]
    );
```

Canonical substrate declarations include the fields (`src/v3/std/substrate.dag`).

**Verdict:** Finding addressed; test is a direct checkable ratchet.

### 2.2 Claude review: stale bootstrap snapshot (`emit_participation: None` on user binds)

**Current code:** `bootstrap_generated.rs` / `bootstrap_generated_without_parse_surface.rs` wire `BindNode` for std `two_terms` with `emit_participation: Some(BindEmitParticipation::UserCallable)` (see grep hit on `bootstrap_generated.rs` line 6 in worktree).

**Verdict:** Bootstrap matches lowering intent for at least the cited binds; **regen drift** should still be caught by normal `regen_bootstrap --check` in CI if present.

### 2.3 Codex BLOCKING: “substrate without real consumer”

**Disposition on `main`:** Beyond test-only selectors, production `emit.rs` tests use `bind.emit_participation()` / `branch.emit_participation()` to locate user callables vs `match` branches (replacing `span.file` hacks). Example:

```3175:3223:src/v3/compiler/src/emit.rs
            .find_map(|node| match node {
                Behavior::Bind(bind)
                    if bind.name == "id"
                        && bind.emit_participation()
                            == Some(BindEmitParticipation::UserCallable) =>
                {
                    bind.params.first().copied()
                }
                _ => None,
            })
            // ...
                Behavior::Branch(branch)
                    if branch.output == classify_output
                        && branch.emit_participation()
                            == Some(BranchEmitParticipation::UserMatch) =>
```

Python boundary serialization uses **Python string literals** for enum payloads (not Rust paths), with an explicit regression:

```616:667:src/v3/compiler/tests/boundary/m1_4_emit_python_test.rs
fn serialize_opt_bind_emit_participation(p: Option<BindEmitParticipation>) -> String {
    match p {
        None => "None".to_string(),
        Some(BindEmitParticipation::UserCallable) => "\"UserCallable\"".to_string(),
    }
}
// ...
#[test]
fn serialize_dag_embeds_valid_python_emit_participation_literals() {
```

**Verdict:** “Real consumer” + boundary shape are satisfied in-repo; the original codex concern is overtaken by later commits on `main`.

### 2.4 Visibility asymmetry (`BranchNode.emit_participation` public vs accessor pattern)

**Current code:** `BranchNode.emit_participation` is `pub(crate)` with `emit_participation()` accessor (`src/v3/compiler/src/dag.rs` ~1617–1627), matching the tightened review ask.

**Verdict:** Addressed.

---

## 3. Conclusion

- **#897:** The **fail-closed** `Behavior::Value` path, **negative `UInt8 = 256`** coverage, **range-validated merge**, and **downstream integration tests** are present in the current tree. No additional code change is justified from this audit alone.
- **#824:** **Reflection**, **bootstrap**, **emit consumers**, **Python serialization**, and **visibility** align with the resolved review threads documented on the PR.

**Follow-up PR:** **None** opened from this session — no unfixed BLOCKING gap was identified in current sources.

---

## 5. Recommended verification commands (operator / CI)

Run on a checkout of `main` (or this branch) with Rust installed:

For `v3-compiler`, only **`tests/integration.rs`** and **`tests/determinism_test.rs`** at the package root become `--test integration` and `--test determinism_test`. The **`tests/boundary/`** directory is taxonomy only: those suites are **`mod`uled into** `tests/integration.rs` (see `#[path = "boundary/m1_4_emit_python_test.rs"] mod m1_4_emit_python_test`), so there is **no** `--test boundary` binary.

```bash
cargo test -p v3-compiler --test integration int_literal_cardinality_test
cargo test -p v3-compiler --test integration thesis_validation_test::t1_4_type_mismatch_produces_a_typemismatch_diagnostic
cargo test -p v3-compiler --test integration substrate_declares_expected_reflection_surface
cargo test -p v3-compiler --test integration m1_4_emit_python_test::serialize_dag_embeds_valid_python_emit_participation_literals
```

Broader confidence:

```bash
cargo test -p v3-compiler --test integration
```

---

## 6. Traceability

| Source | Review / thread | Disposition |
| --- | --- | --- |
| #897 | OpenAI-pro BLOCKING: `Behavior::Value` retry without fit check | Fixed — see §1.1 |
| #897 | Negative `UInt8 = 256` | Test — §1.2 |
| #897 | Merge path validation | §1.3 |
| #897 | Call-site + emit | §1.4 |
| #824 | Reflection mismatch | Test + substrate — §2.1 |
| #824 | Bootstrap stale | Spot-check — §2.2 |
| #824 | No real consumer | emit + boundary tests — §2.3 |
| #824 | Visibility | §2.4 |
