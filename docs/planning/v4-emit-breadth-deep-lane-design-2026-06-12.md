# §3 Emit Breadth — DEEP Lane Design (dep-graph-2026-06-12 §3)

> **Status: PRE-ROUND-TRIP DESIGN ONLY** — no compiler-stage `.dag` edits from this artifact.
> **Work item:** `node://adhoc-f64ad6df-d11` · **Session:** `clever-moth-532` (Mgr-§3 composite).
> **Planning authority:** `gunb-ai/ctrl` `gunbc-planning/dependency-graph-2026-06-11.md` §3 +
> `dependency-graph-2026-06-12.md` frontier update.
> **Operator fork (2026-06-12):** **§3-deep, §4 thin** — deepest capacity here; §4 is one thin
> bounded lane (effect handlers + one §2 body + run loop).
>
> **Implementation GO:** sharp-fox-370 worker (`adhoc-fdd9ccb8-b54`) owns Branch emit row +
> round-trip; sharp-fox or snappy-crab-849 announces GO. Until Branch body round-trips green:
> **NO** compiler-stage `.dag` edits, **NO** emit-surface code. Same pre-gate pattern as
> sharp-ibex §2.

## 1. North star — one row per form, rows not arms

**§3 principle:** emit breadth = **N algebra rows in `target_model`**, not N×M hand-arms in
`06_translate` / `06_value_expression`. `05_emit` stays frozen (`serialize ∘ translate`).

**Standing invalidation (keystone dissolve):** any source form needing **>1**
`TargetValueExpressionKind` arm **or** multi-site `06_translate` edits **falsifies** the
keystone — **STOP and escalate to snappy-crab-849**; that is a finding, not an obstacle to
work around.

**Anti-cement precedent (#4623):** TypeScript went **through the descriptor** (`TargetValueExpressionProjection` + operator catalog on `TargetModel`); the enumerated hand token list (`ts_mvp1_concrete_tokens`) is marked dissolve-on, not re-expanded. A second target must **never** re-introduce per-language hand-lists in compiler code or parallel token tables.

**P2 audit rule:** every PR touching compiler stages must sweep **all** stage `.dag` files —
including `emit_host.dag` (past audit missed it by only checking `06_translate`). Emit-surface
diffs in load-bearing stages route through snappy-crab-849 for signing authority.

## 2. Emit-coverage census — BY EXECUTION

Witnesses run 2026-06-12 on `target/release/gunbc run --source-root src/v4 --entry … --function … --claim-run` at main tip (`b7411556ef`). **Do not infer green from code.**

### 2.1 Today's proof shape vs today's colors

**Keystone proof shape (operator brief):** add **signature + body** → TypeScript **end-to-end**
(producer-rooted `Arrow.body` → `emit()` → authority source string, discriminating mutates).

| Witness | Entry | claim-run | Tier |
|---------|-------|-----------|------|
| `mvp1_ts_emit_add_fn_accepts_holds` | `mvp1_typescript_add_translate.dag` | **true** | signature-only (hand-built `InferredTree` fixture root) |
| `comprep_ts_bodied_emit_add_fn_accepts_holds` | `comprep_add_body_emit_typescript.dag` | **false** | signature **+ body** (producer-rooted) |
| `comprep_ts_bodied_emit_operand_swap_discriminates` | same | **false** | body perturbation red |
| `comprep_ts_bodied_emit_catalog_minus_discriminates` | same | **false** | catalog perturbation red |
| `comprep_ts_bodied_emit_missing_catalog_rejects` | same | **true** | fail-closed (missing catalog → Rejected) |
| `comprep_eval_by_execution_keystone_holds` | `comprep_eval_by_execution.dag` | **false** | add body **eval** |
| `comprep_branch_eval_by_execution_keystone_holds` | `comprep_branch_eval_by_execution.dag` | **true** | Branch body **eval** |

**Honest summary:** **signature-tier** TS emit is green by execution; **body-tier** add eval and
body-tier TS emit are **red** in this environment. Fail-closed operator-catalog refusal is green.
Branch eval is green; Branch emit and round-trip have **no** claim-run witness yet (§2 lane).

### 2.2 Which source forms emit today (execution truth)

| Source form | Parses | Produces body | Eval | Emit TS | Notes |
|-------------|--------|---------------|------|---------|-------|
| `fn add(…) -> … { x + y }` signature only | ✅ | fixture only | — | ✅ fixture tree | `mvp1_ts_emit_*` |
| `fn add(…) { x + y }` body (producer) | ✅ | ✅ `produce_mvp1_add_arrow_with_body` | ❌ | ❌ | #4623 consumer red |
| `fn pick(b) { if b then … else … }` | ✅ | ✅ source-bridged | ✅ | ❌ | fan-in (ii) pending |
| `let` / `match` / general call | grammar exists | ❌ | ❌ | ❌ | §2+ breadth |
| Rust / Python / Go MVP signature | fixtures | ❌ body | — | grammar-inverse signature only | no value projection row |

### 2.3 What does NOT emit (named)

- Any form without a landed `TargetValueExpressionKind` + per-target projection row.
- Branch `if`-then-else body tokens (no `TargetValueExprConditional` kind yet).
- Callable / method apply bodies (`TargetValueExprCallableApply` absent).
- `Bind` / `Loop` bodies (kinds absent; §2 producers open).
- Host execute of emitted TS (`run_emit_host` — only TS routes through `run_host_process` descriptor; rust/python/go substrate rows fail-closed; eval intercept in `emit_host_eval.rs`).

## 3. Breadth ladder — ordered source forms (post-Branch)

Each rung = **exactly one** `TargetValueExpressionKind` + **one** projection fold arm + **one**
serialize arm. Nested/block variants (B2/B3) use **recursive projection on the same kind**, not
new kinds (`design-comprep-m0-branch-mapping.md` §12.1 tripwire).

| Order | Source form (`dag.dag`) | One algebra row | Producer gate | Eval fan-in | Emit fan-in | Round-trip |
|-------|-------------------------|---------------|---------------|-------------|-------------|------------|
| L0 | infix primitive (`+` add keystone) | `TargetValueExprPrimitiveApply` | ✅ wave-1 | ❌ red | ❌ red | — |
| L1 | `dag_production_if_then_form` | `TargetValueExprConditional` | ✅ M0-B1 | ✅ green | sharp-fox (ii) | sharp-fox (iii) → **§3 GO** |
| L2 | cond = param ref (B1a) | same `Conditional` | B1a | follow-on | same row | — |
| L3 | `dag_production_if_block_form` | same `Conditional` + stmt scaffold | Bind producer | §2 | same row | — |
| L4 | nested else `if` (B3) | recursive `Conditional` | M0-B1 + Bind | recursive eval | recursive serialize | — |
| L5 | `dag_production_let_expr` | `TargetValueExprBindLet` | §2 Bind | §2 | one row | — |
| L6 | bounded `loop` | `TargetValueExprLoop` | §2 Loop | §2 | one row | — |
| L7 | callee `Transform` (non-primitive) | `TargetValueExprCallableApply` | callee path | §2 | one row | — |
| L8 | record / list literals | leaf kinds or new substrate decision | wave-4 | — | escalate if >1 row | — |
| L9 | `dag_production_match_expr` | desugar → L1 (M0-B6) **or** escalate Q-B1 | pattern authority | — | **forbidden** second kind without ruling | — |

**Tripwire examples that MUST escalate:**

- Splitting `TargetValueExprMatch` off `Conditional` for Bool match (violates one-row rule).
- Second `06_translate` matcher parallel to `project_arrow_body_to_value_expression`.
- Re-adding `ts_mvp1_concrete_tokens`-style enumerated body tokens per target (#4623 regression).

## 4. Rust-second-target plan — through the descriptor

Rust is the **second Shape-A target** after dag home + TS vetting consumer. Plan lands **only**
as `extdeps/languages/rust.dag` descriptor data + `TestClaim` witnesses — **zero** new
`run_emit_host_rust` hand-edits, **zero** compiler if-chain arms.

### 4.1 What Rust adds (data only)

| Descriptor | Location | Purpose |
|------------|----------|---------|
| `rust_value_expression_projection()` | `rust.dag` | `binding_ref_form`, `primitive_apply_form` (infix-only M0), `callable_apply_form` shell; `conditional_form` with fan-in (ii) |
| `rust_operator_realizations_catalog_node` | `rust.dag` | `OpAdd` → `InfixToken { ^rust_token_plus }` |
| `rust_mvp1_value_projection_bundle_node` | `rust.dag` | edge `target_model_edge_value_expression_projection` |
| `rust_host_transport_mvp1_descriptor()` | `rust.dag` | **HostTransportDescriptor** row (mirror `ts_host_transport_mvp1_descriptor`) — compile/run/stdout codec as data |
| `rust_mvp1_runtime_row` (T-22) | `rust.dag` on `TargetModel` | promoted typed row replacing authority-text if-chain lookup |

### 4.2 What Rust dissolves (never re-expands)

| Incumbent | Dissolution trigger | Anti-pattern |
|-----------|---------------------|--------------|
| `emit_host.dag` `runtime_value_parse` if-chain (`:215-229`) | `TargetModel.runtime_row` generic lookup (#4674) | 4th/5th `else if authority_source_text == …` arm |
| `emit_host.dag` `run_emit_host` if-chain (`:284-299`) | descriptor fold + `run_host_process` only | per-target `run_emit_host_rust` substrate stub growth |
| `emit_host_eval.rs` `try_dispatch_emit_host_*` | substrate Callable owns dispatch | name-string intercept per target |
| `tools/emit_host_runner` near-duplicate bodies | one primitive + descriptor rows | copy-paste spawn/capture per lang |
| SG-0 `emit_host_bridge.rs` / `emit_host_eval.rs` | D1/D2 in §5 | hand-Rust eval hook |
| Enumerated body token tables in extdeps | derived projection output | `rust_mvp1_concrete_tokens` parallel |

**T-22 mark (authoritative):** `emit_host.dag:216` and `:289` — *promote typed `runtime_row`
onto `TargetModel`; replace per-target if-chains + `emit_host_eval.rs` mirror; **forbidden:**
adding a 5th per-target arm without dissolution.*

Rust second-target work **extends the descriptor fold**, it does not add a fifth hand-list entry.

### 4.3 Sequencing

```text
sharp-fox: Branch fan-in (ii) emit + (iii) round-trip on dag
  → Rust descriptor PR (extdeps + claim-run only; P2 sweep includes emit_host.dag)
    → TS body emit greens (producer path) — same PR or immediate follow-on
      → Rust value projection attach + body emit claim-run
        → T-22 runtime_row promotion PR (dissolves if-chains, not extends them)
          → python/go descriptor clones (no new compiler arms)
```

## 5. §5 census-drain ordering

**Rule (INVARIANTS / ROADMAP §0):** self-host counter goes down by **migration** — repoint
consumer onto fold/model, delete hand Rust in the **same PR** — **never** by stuffing Rust into
templates or cementing per-target logic to satisfy a ratchet.

### 5.1 Emit-breadth-enabled drain (after body emit exists)

| Order | SG-0 / bridge path | Migration replacement | §3 rung that unlocks |
|-------|-------------------|----------------------|----------------------|
| D1 | `emit_host_eval.rs` | `TargetModel.runtime_row` + generated eval transport | L0 body emit + T-22 row |
| D2 | `emit_host_bridge.rs` | same | D1 |
| D3 | `r1c_e_emit_gates.rs` + bin | `.dag` claim runner | L1 emit receipts |
| D4 | `boundary_emit_gates.rs` + bin | T-PB-B claim execution | breadth claims |
| D5 | `post_emit_verifier.rs` | `run_target_verification` substrate | host descriptor green |
| D6 | `emit_rust.rs`, `emit/*.rs` | v4 `serialize ∘ translate` per module | convergence ladder B |
| D7 | `regen_*_emit.rs` | `compiler.dag` emits stage0 | stage C fixed point |

### 5.2 What NOT to drain early

- Do not delete SG-0 emit entries before the **executed** claim-run witness replaces the capability.
- Do not shrink census by moving logic into `06_translate` string templates (graft detection: file grows).
- `hand_maintained_src` seed (v2 binary) drains at stage C only — emit breadth accelerates D1–D6, not the seed.

## 6. §4 thin lane (operator decided)

§4 runs thin: landed effect handlers + one §2 body + run loop → first `dag run program.dag`.
**Falsifier:** one real effect needing bespoke per-effect Rust glue → escalate (breaks #4609 fabric claim). Mgr-§3 does not deepen §4.

## 7. Child dispatch (post-GO only)

| # | When | Owner hint |
|---|------|------------|
| W0 | Branch round-trip green | sharp-fox `adhoc-fdd9ccb8-b54` (in flight) |
| W1 | Rust descriptor + claims | extdeps worker; P2 sweep all compiler stages |
| W2 | TS body emit red → green | producer→emit path; no hand token list |
| W3 | T-22 runtime_row | model PR; dissolves emit_host if-chains |
| W4 | python/go descriptor clone | after Rust proof |

## 8. Non-goals

- No compiler-stage edits until GO.
- No parallel-ledger docs; marks stay authoritative.
- No second `TargetValueExpressionKind` for Branch family.
- No per-target hand-lists (#4623 regression).

## 9. Escalation

`dashboard-ops escalate` when blocked; `dashboard-message send --to snappy-crab-849` for rulings.

Escalate immediately if:

- Any ladder rung requires >1 kind or multi-site translate.
- Implementation proposes a 5th `emit_host` per-target arm before #4674.
- Body emit is ratcheted green without claim-run witness.

## 10. Acceptance (this artifact)

- [x] Emit-coverage census by execution (§2)
- [x] Breadth ladder with one-row invalidation (§3)
- [x] Rust-second-target through descriptor + T-22 dissolve plan (§4)
- [x] §5 census-drain ordering (§5)
- [x] §3-deep / §4-thin fork recorded (§6)
- [ ] Implementation GO — blocked on Branch body round-trip (`adhoc-fdd9ccb8-b54`)
