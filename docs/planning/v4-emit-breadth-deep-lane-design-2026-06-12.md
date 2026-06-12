# §3 Emit Breadth — DEEP Lane Design (dep-graph-2026-06-12 §3)

> **Status: PRE-ROUND-TRIP DESIGN ONLY** — no compiler-stage edits from this artifact.
> **Work item:** `node://adhoc-f64ad6df-d11` · **Session:** `clever-moth-532` (Mgr-§3 composite).
> **Implementation GO:** fires when Branch body **fan-in (iii) round-trip** lands (byte-identical
> or quotient-honest per target declaration); fan-in (ii) emit + Q-B2 infer row are immediate
> predecessors. Fan-in (i) eval is **GREEN** (#4730 + #4738 lazy-arm).
>
> This doc is the §3-deep spawn package: emit-coverage census, ladder of source forms,
> Rust-second-target descriptor plan, §5 census-drain ordering, and the operator §3 vs §4 fork
> frame. Marks + claim rows remain authoritative for live state (P5 ledger rule).

## 1. North star — rows not arms

**§3 principle (ROADMAP):** emit breadth = **N data rows**, not N×M hand-arms. Every construct
lands as:

| Layer | Authority | Consumer |
|-------|-----------|----------|
| Substrate kind | `TargetValueExpressionKind` (+ template twin) in `std/compilers/target_model.dag` | `06_value_expression` projection fold |
| Per-language row | `TargetValueExpressionProjection` + operator catalog on `TargetModel` bundle | `extdeps/languages/<lang>.dag` |
| Serialize | derived `FormalProduction` rows (bidir §4.1) | frozen `05_emit` (`serialize ∘ translate`) |
| Proof | `TestClaim` body perturbation receipts | claim runner / `--claim-run` |

**Anti-pattern (INVARIANTS P2 receipt):** PR #4627 — literal target atoms and hand-inlined token
layouts in `06_value_expression.dag`. Dissolving those into realization rows is a blocking
precondition of this lane (`docs/modeling-discipline.md` emit/template dissolution).

**Coupling:** COMPREP wave-3 ("bodies through translate/emit",
`design-computation-representation.md` §3) and the emit ladder's **value-expr tiers (T4+)**
are the same work — not parallel tracks. This design names the breadth census and sequencing;
`design-value-emit-schema.md` names the schema; `design-comprep-m0-branch-mapping.md` §12
names the Branch fan-in (ii) row.

## 2. Hard gates (do not implement §3-deep past)

| Gate | State | Unblocks |
|------|-------|----------|
| **§1 keystone** (#4699) | ✅ merged | translate fold repoint |
| **COMPREP M0-B1 eval** (#4730) | ✅ green | fan-in (ii) emit design-sign |
| **Q-B3 lazy-arm** (#4738) | ✅ green | Bind/Loop spine eval without eager-arm poison |
| **Fan-in (ii) emit** | 🟡 design endorsed (§12); impl diff pending | Branch body tokens by execution |
| **Fan-in (iii) round-trip** | not started | **§3-deep implementation GO** (this doc's downstream) |
| **Q-B2 infer Branch row** | absent in `04_infer.dag` | lands **with** fan-in (ii) — widened design-sign (§12.10) |
| **Grammar-first A (#4462)** | open | `06_translate` mode-2 region (SG-2 GAP-1/GAP-2) |
| **T3 type projection** (`snappy-owl-682`) | in progress | at-scale Q1 ~55-arm dissolution (`q1-projection-dissolution-prestage`) |

**PRE-ROUND-TRIP constraint:** this artifact does not authorize edits to the four load-bearing
pipeline stages (`emit/lower/infer/parse`) beyond what fan-in (ii)/(iii) PRs already carry.

## 3. Emit-coverage census

Two orthogonal axes: **substrate Behavior coverage** (producer × projection × eval × emit) and
**v2 long-tail checklist** (dissolution target, not vocabulary to copy).

### 3.1 Behavior × value-tier matrix (substrate axis)

Closed source domain: `Behavior = Value | Transform | Branch | Loop | Bind` (`std/node.dag`).

| Behavior arm | Producer (COMPREP) | `TargetValueExpressionKind` | Eval | Emit (value tier) | Round-trip |
|--------------|-------------------|----------------------------|------|-------------------|------------|
| **Value** literals + refs | wave-1 (add keystone) | leaves + `BindingRef` ✅ | ✅ primitive path | 🟡 TS skeleton only (`comprep_add_body_emit_typescript.dag`) | signature-tier only |
| **Transform** primitive | wave-1 ✅ | `PrimitiveApply` ✅ | ✅ `call_primitive` + add keystone | 🟡 TS manual claims; no dag/rust body emit | — |
| **Transform** callable | not landed | `CallableApply` ❌ (planned §4.1) | ❌ | ❌ | — |
| **Branch** Bool if-else | M0-B1 ✅ (#4730) | `Conditional` ❌ (fan-in ii) | ✅ lazy-arm (#4738) | design endorsed §12; impl pending | gates §3-deep |
| **Branch** block / nested | M0-B2/B3 catalogued | same `Conditional` (recursive) | 🟡 gated on Bind producer | fan-in ii+ | follow-on |
| **Bind** | §2 next | `BindLet` ❌ | ❌ | ❌ | — |
| **Loop** | §2 next | `Loop` ❌ | partial skeleton | ❌ | — |

**Kinds landed today (9 of 13 planned wave-3 set):** symbol/bool/char/string literals, Rc/Box
refs, `BindingRef`, `PrimitiveApply`. **Missing kinds (4):** `CallableApply`, `Conditional`,
`BindLet`, `Loop` — each lands with producer + template twin (E-10).

### 3.2 Per-target value projection row census

`TargetValueExpressionProjection` + operator catalog (`target_model.dag` T6 carriers).

| Target | Projection row | Operator catalog | Body emit receipt | Notes |
|--------|----------------|------------------|-------------------|-------|
| **dag** (home) | ❌ | ❌ | RTADD semantic only (`mvp1_dag_add_round_trip.dag`) | signature/type-expr tier; honest label |
| **typescript** | ✅ `ts_value_expression_projection` | ✅ `OpAdd` infix | 🟡 manual (`comprep_add_body_emit_typescript.dag`, fold types) | design-vetting consumer per `design-value-emit-schema.md` §6 |
| **rust** | ❌ | ❌ | signature fixture only (`rust_mvp1_*`) | **second-target plan §5** |
| **python** | ❌ | ❌ | grammar-inverse signature | — |
| **go** | ❌ | ❌ | grammar-inverse signature | — |
| **cpp** | ❌ | ❌ | T3 type row in flight | value tier blocked on T6 template |
| java, kotlin, swift, … | ❌ | ❌ | extdeps staged | outside MVP tranche |

**Approximate breadth counter (not a ratchet):** 4 missing kinds × 5 MVP targets ≈ **20**
value-tier row bundles to land or defer — analogous to Q1's ~55 type-tier arms
(`q1-projection-dissolution-prestage` §1). Exact tracking: inline marks + claim rows.

### 3.3 v2 `ExprData` long-tail map (dissolution checklist)

v2 `ExprData` (~22 arms, `v2_std_core.rs`) → v4 landing bucket. **Not** a substrate import —
coverage tracker only.

| v2 `ExprData` arm | v4 bucket | Wave / gate |
|-------------------|-----------|-------------|
| `ExprLiteral` | Value leaves | ✅ |
| `ExprVar` | `BindingRef` | ✅ wave-1 |
| `ExprBinOp` | `PrimitiveApply` + `CanonicalOperation` | ✅ wave-1 (OpAdd only) |
| `ExprUnaryOp` | `PrimitiveApply` | catalog arm + producer |
| `ExprCall` | `CallableApply` | COMPREP callee path |
| `ExprMethodCall` | `CallableApply` + `MethodIdent` shape | post-callee |
| `ExprIf` | `Conditional` | fan-in (ii) M0-B1 |
| `ExprMatch` | `Conditional` or desugar → B1 (M0-B6) | M0-B5 escalates Q-B1 |
| `ExprLet` | `BindLet` | COMPREP Bind row |
| `ExprBlock` | statement scaffold + `Bind` chain | M0-B2 |
| `ExprLambda` | declaration + body (arrow) | wave-4 breadth |
| `ExprRecordLit` / `ExprListLit` | aggregate literals (value kinds TBD) | wave-4 |
| `ExprFieldAccess` | projection / accessor row | wave-4 |
| `ExprCast` | coercion witness surface | bidir coupled |
| `ExprStringInterp` | lex + value kind | lex-layer |
| `ExprForEach` / `ExprIndex` / `ExprSlice` | surface-specific rows | wave-4 |
| `ExprReturn` | statement scaffold (Q-V2) | function-decl production |
| `ExprError` | diagnostic carrier | not emit tier |
| `NoExprData` | structural placeholder | N/A |

### 3.4 Honest relabeling (already required)

Per `design-computation-representation.md` §5.1: dep-graph T0 / RTADD / T1 rows must say
**signature/type-expr-tier** until body emit receipts exist. "emit(add)" language oversells
until `TargetValueExprPrimitiveApply` projects from a real `Arrow.body` sub-DAG on the MVP
target model bundle.

## 4. Ladder of source forms

The ladder has **two coupled rails** — type-expr (T0–T3) and value-expr/body (T4–T7) — sharing
one `TargetModel` bundle and one `serialize ∘ translate` spine.

```text
T0  signature-only fixtures (rust_mvp1_emitted_add_fn)     ✅ honest at type tier
T1  multi-target signature emit (ctrl#1489 D1)              ✅ grammar-inverse
RTADD semantic round-trip on fixture tree                   ✅ (#4544); not text round-trip
T2  type-expr projection rows (SG-2 / Q1 ~55 arms)          🟡 rust/ts rows; MVP attach open
T3  translate fold = row interpreter (Q1 anti-fold)         🟡 gated (snappy-owl-682)
─── COMPREP / value-expr rail ───
T4  Value leaves + BindingRef + PrimitiveApply kinds        ✅ substrate
T5  body producer wave-1 (add) + wave-2 (Branch…)           🟡 B1 eval ✅; B1 emit (ii)
T6  per-target value projection + operator catalog          🟡 TS only
T7  body ingest = forward row interpreter (bidir §6)        prep; execution gated on T6
T8  text round-trip (source → tree → source)                fan-in (iii) keystone → §3-deep GO
```

### 4.1 Source-form rungs within T5–T8 (COMPREP × emit)

Ordered by **producer dependency** (not language count). Each rung names the `dag.dag`
production anchor and the discriminating receipt shape.

| Rung | Source form (`dag.dag`) | Behavior / kind | Eval receipt | Emit receipt | Round-trip |
|------|-------------------------|-----------------|--------------|--------------|------------|
| **R0** | `add` body infix | `Transform` + `PrimitiveApply` | `comprep_eval_by_execution` ✅ | TS manual 🟡 | — |
| **R1** | `dag_production_if_then_form` | `Branch` + `Conditional` | `comprep_branch_eval_by_execution` ✅ | fan-in (ii) | fan-in (iii) |
| **R1a** | cond = param ref | same | `pick(b)` follow-on | with B1a | — |
| **R2** | `dag_production_if_block_form` | `Branch` + Bind arms | Bind producer | B2 block `then_token` Optional | post-R2 |
| **R3** | nested `dag_production_if_expr` | recursive `Conditional` | recursive eval | nested serialize | post-R2 |
| **R4** | `let` / `bind` productions | `Bind` + `BindLet` | §2 Bind row | kind + row | — |
| **R5** | `loop` / bounded recur | `Loop` + `Loop` kind | §2 Loop row | kind + row | — |
| **R6** | `match` (Bool desugar) | desugar → R1 | M0-B6 policy | same as R1 | optional |
| **R7** | `match` (general) | Q-B1 edge discipline | escalate | `TargetValueExprMatch`? | — |

**DEEP lane scope:** R0–R3 + Rust-second-target descriptors (§5) + census drain (§6). R4+ stay
§2 manager ownership until Bind/Loop producers green.

### 4.2 Discriminating receipt pattern (every rung)

1. **Eval:** perturb condition or operand → outcome flips (`--claim-run`).
2. **Emit:** perturb arm operand or operator catalog entry → emitted token stream changes
   (structural, both arms projected — §12.6).
3. **Round-trip:** `source → parse → … → serialize` equals source up to declared quotient
   (`target_model_edge_fidelity_quotient`); perturb identifier → red.

## 5. Rust-second-target descriptor plan

**Ordering rule (bidir §7, `design-value-emit-schema.md` §6):** home language (`dag`) first for
round-trip semantics; **Rust second** for the first non-self Shape-A target with real body
emit — mirrors "rust_mvp1 rows (first non-self target)" after dag RTADD.

### 5.1 Why Rust before Python/Go

| Factor | Rust | Python / Go |
|--------|------|-------------|
| Type-tier projection | ✅ `rust_type_expression_projection` (SG-2) | ❌ grammar-inverse only |
| Statement context (Q-V2) | expression-bodied — simpler value scaffold | statement prefix/suffix required |
| MVP roster / ctrl#1489 | primary Shape-A anchor | follow T6 template after Rust proof |
| v2 dissolution pressure | `emit_rust.rs` in SG-0 census | sibling targets |

Python/Go value rows **copy the Rust template** after Rust body emit greens — not parallel
invention.

### 5.2 Rust descriptor bundle (lands in one extdeps PR per rung)

Author in `src/v4/extdeps/languages/rust.dag` — **no** `06_value_expression.dag` edits in the
descriptor-only PR (same Phase-A discipline as Q1 T3 template).

1. **`rust_value_expression_projection()`** — fields per `design-value-emit-schema.md` §4.2:
   - `binding_ref_form`: `{ ident_token: ^rust_token_ident }`
   - `primitive_apply_form`: infix-only for M0 (`open`/`close`/`separator` absent or ignored
     per row policy — Rust add is `x + y`, not call-shaped)
   - `callable_apply_form`: reserved for callee path (catalog empty until CallableApply)
   - `conditional_form`: lands with fan-in (ii) — `{ if_token, then_token, else_token }`
2. **`rust_operator_realizations_catalog_node`** — `OpAdd` → `InfixToken { ^rust_token_plus }`.
3. **`rust_mvp1_value_projection_bundle_node`** — named edge
   `target_model_edge_value_expression_projection` on MVP target model (attach gate separate
   from SG-2 model until roster says green).
4. **Golden fixtures** — parallel to TS manual claims:
   - `rust_comprep_add_body_emitted_tokens` — body segment for `fn add(x: i32, y: i32) -> i32 { x + y }`
   - `rust_comprep_branch_if_then_else_emitted_tokens` — fan-in (ii) keystone
   - perturbed twins: operand swap; `+` → `-` catalog edit
5. **Claim module** `src/v4/test/claim/manual/comprep_*_rust.dag` — structural + perturbation;
   roster promotion only when `project_arrow_body_to_value_expression` consumes the row
   (fan-in ii merge).

### 5.3 Rust-specific shapes (no new substrate kinds for M0)

| Concern | Plan |
|---------|------|
| Infix-only `OpAdd` | `TargetOperatorShape::InfixToken` — no `primitive_apply_form` parens |
| Block-bodied `if` | M0-B2 uses `Optional<Symbol>` for `then_token` absence (§12.9B) — Rust expr-bodied M0 ignores |
| Ownership (Q-V4) | out of scope — `ReferenceRcNew` leaves unchanged; no ownership in value rows yet |
| MVP1 attach | `claim_sg2_mvp1_projection_absent` pattern — new `claim_rust_mvp1_value_projection_attached` when bundle edge lands |

### 5.4 Sequencing

```text
fan-in (ii) merge (dag target, Conditional kind)
  → Rust descriptor PR (extdeps + manual claims only)
    → fan-in (iii) round-trip (dag, then rust)
      → §3-deep worker dispatch (translate consumer wiring if not already in ii)
        → python/go value rows (T6 template clone)
```

## 6. §5 census-drain ordering

§5 self-hosting (`get-off-v3` caller census → zero hand-maintained Rust) drains **downstream of
substrate migration**, not ahead of it (`ROADMAP` §0 same-PR rule). Emit breadth removes
**emit-shaped** SG-0 / `hand_maintained_src` entries in this order:

### 6.1 v3 SG-0 `EXPECTED_HAND_AUTHORED_NON_TEST` (emit-class)

| Order | Path | Capability | Dissolution trigger |
|-------|------|------------|---------------------|
| **D1** | `emit_host_eval.rs` | T-22 per-target eval dispatch | `TargetModel.runtime_row` generic lookup (#4674); generated eval calls `emit_host` transports |
| **D2** | `emit_host_bridge.rs` | host runner bridge | same as D1 + `emit_host_runner` deletion |
| **D3** | `r1c_e_emit_gates.rs`, `r1c_e_emit_gates` bin | R1C-E emit gate checks | `boundary_emit_gates.template.dag` / claim runner sole authority |
| **D4** | `boundary_emit_gates.rs`, bins | class-5 boundary gates | T-PB-B / `PB-Runtime-External-Toolchain-TestClaims` |
| **D5** | `post_emit_verifier.rs` | post-emit host verify | substrate `run_target_verification` |
| **D6** | `emit_rust.rs`, `emit_rust_bin_shim.rs`, `emit_rust_roundtrip_fixtures.rs` | v3 Rust emitter | v4 `serialize ∘ translate` parity on compiler module slice |
| **D7** | `emit.rs`, `emit/rust_target.rs`, `emit/python_target.rs` | v3 multi-target emit core | v4 module emission + convergence ladder row (self-host stage B) |
| **D8** | `regen_*_emit.rs`, `regen_bootstrap_emit.rs` | stage0 regen shims | `compiler.dag` emits stage0; fixed-point stage C |
| **D9** | `emit/collection_ops_method_contract.rs` | method template gate | Shape B / `pb_method_template_projection` dissolution |

**Rule:** one SG-0 line per PR; capability must be replaced in the **same PR** (ROADMAP §0).
Do not ratchet census down by deleting tests without modeled replacement.

### 6.2 v4 `hand_maintained_src` (self-host stage A list)

From `design-self-host-fixed-point.md` §4 — emit-independent list land, emit-dependent drain:

| Order | Entry class | Drains when |
|-------|-------------|-------------|
| **H1** | v3 harness residuals (`v4_t15_self_host_fixed_point_harness_test.rs`) | PB-Runtime deferral |
| **H2** | T-22 bridge rows (ROADMAP bounded bridges) | D1/D2 complete |
| **H3** | v2 stage0 seed binary | stage C fixed point + operator pin rotation |
| **H4** | workflow CI shims (`release.yml` etc.) | YamlStatic emission lane |

Emit breadth (§3) primarily accelerates **D1–D7** and **H2**; it does not shortcut **H3**
(whole-compiler fixed point).

### 6.3 Convergence ladder coupling (stage B)

Per `design-self-host-fixed-point.md` §5: each compiler module row in `ConvergenceLadder`
turns green when `stage0(module)` ≡ pinned reference. Emit tiers T5–T6 add ladder rows
**in module dependency order**:

1. `06_value_expression` (body projection fold)
2. `06_translate` (serialize arms only — fold already migrated)
3. `compiler/*` leaves per module graph
4. whole `compiler.dag` (stage C)

## 7. Operator fork decision frame (§3 vs §4)

**Context (ROADMAP):** after §2 lands, operator picks **§3 + self-host breadth** vs **§4
runnable I/O demo**. Both depend on §2; the pick sets deepest staffing.

### 7.1 Readiness snapshot (2026-06-12)

| Lane | Ready now | Blocked on |
|------|-----------|------------|
| **§3 DEEP** | M0-B1 eval ✅; fan-in (ii) design endorsed; Rust descriptor plan §5 | (ii) impl + (iii) round-trip |
| **§2 tail** | Bind/Loop producers, M0-B2/B3 | COMPREP §2 manager |
| **§4 I/O** | effect handlers, run-loop | `std/effects` + scheduler substrate |

### 7.2 Option A — §3-first (recommended default)

**Staff:** fan-in (ii) → (iii) → Rust T6 descriptor → Q1 type attach → D1–D4 census drain.

**Upside:** unblocks §5 self-host; dissolves T-22 bridges; convergence ladder rows; aligns with
dep-graph edge `§3 → §5`.

**Cost:** no demo-able external I/O program until §4 starts.

### 7.3 Option B — §4-first

**Staff:** effect handler modeling, run-loop, minimal `main` with stdout.

**Upside:** external demo narrative; exercises runtime scheduler.

**Cost:** §5 remains blocked; emit/host bridges stay; risk of cementing Rust scheduler before
body emit rows land (INVARIANTS: model before implement).

### 7.4 Recommendation

**§3-first** unless operator needs a public I/O demo before self-host metrics move. Rationale:

1. Branch round-trip is already on the critical path and is the explicit **§3-deep GO** gate.
2. §5 is structurally downstream of §3 (`ROADMAP` §3 → §5).
3. §4 effect modeling is largely orthogonal substrate work — can run as a **thin parallel
   modeling lane** without forking implementation staffing away from fan-in (ii)/(iii).

**Hybrid:** operator may parallelize **§4 modeling-only** (effect carriers, no compiler-stage
touches) while §3 implementation proceeds — same PRE-ROUND-TRIP discipline as this doc.

## 8. Child dispatch map (post-GO)

When fan-in (iii) greens, spawn workers in this order:

| # | Worker title | Scope | Touches load-bearing? |
|---|--------------|-------|---------------------|
| W1 | Fan-in (ii) carryover if not merged | `TargetValueExprConditional` + Q-B2 infer + TS/dag emit claims | yes — design-sign required |
| W2 | Rust T6 descriptor + manual claims | `rust.dag` extdeps only | no |
| W3 | Rust body emit roster promotion | attach value projection to `rust_mvp1_target_model` | yes — translate consumer |
| W4 | `pick(b)` / B1a eval + emit | param-ref condition | yes |
| W5 | M0-B2 block `if` | Bind producer dependency | yes — gated §2 |
| W6 | T-22 dissolution slice | D1/D2 per #4674 | yes — escalate if row missing |

## 9. Non-goals (this design)

- No edits to `05_emit.dag`, `06_translate.dag` fold regions, `04_infer.dag`, or `02_parse.dag`
  from this doc alone.
- No `TargetValueExprMatch` / fifth Branch kind without Q-B1 resolution.
- No bit-identical round-trip claims unless target quotient declares it.
- No Shape B format emit (Branch D).
- No census ratchet without same-PR capability replacement.

## 10. Escalation triggers

Escalate to program coordinator (`sharp-fox-370`) if:

- Fan-in (ii) requires >1 `TargetValueExpressionKind` for M0 Branch family (tripwire §12.1).
- Rust descriptor needs a substrate kind not in `design-value-emit-schema.md` §4.1.
- Implementation proposes literal target atoms in `compiler/` (P2 receipt).
- T3/Q1 type-tier work blocks Rust value attach unexpectedly (bundle edge conflict).
- Operator fork choice H2 — §4 implementation staffing before (iii) lands.

## 11. Acceptance (this artifact)

- [x] Emit-coverage census (§3)
- [x] Ladder of source forms (§4)
- [x] Rust-second-target descriptor plan (§5)
- [x] §5 census-drain ordering (§6)
- [x] Operator fork frame + recommendation (§7)
- [ ] Implementation GO — **blocked on fan-in (iii) round-trip**
