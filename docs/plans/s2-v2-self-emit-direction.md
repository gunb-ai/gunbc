# S2 — v2 emits v2: strategic direction & decomposition

**Status:** direction doc for the strategic S2 lane (v2's own emitter). Branch `claude/s2-v2-self-emit-wbvn5z`, PR #6374. Written after rungs 1–4 landed so the remaining work can be decomposed and parallelized.

This doc is the **single authority** for the lane's direction, receipt discipline, and rung decomposition. It is reasoned from the DESIGN axioms: a target language is *rows, not a compiler fork* (§4, §7), every claim is *green by execution* with a discriminating RED (§5), and every scaffold lands with a *named dissolution trigger* (§6).

---

## 1. Objective (the milestone) — REVISED 2026-07-08 (operator: drop byte-fixed-point)

v2's own emitter — `emit = serialize_target ∘ translate`, the one-grammar-read-backward machine (DESIGN §4) — covers v2's own language surface, emitting the **cleanest principled Rust** and proven **by execution**: each self-emitted module compiles cargo-green and is **behaviorally equivalent** to the v1 seed on a discriminating corpus. **Byte-identity with the seed is explicitly NOT the goal** (operator, 2026-07-08) — the fixed point is a *quality drag* that would force v2 to reproduce v1's hacky/rusty warts to match bytes, cementing poor decisions (§1: spending future time to preserve the present seed's accidents); dropping it *raises* the bar. Self-discipline (§1–§7) is fully retained — "impure" means *honestly-fenced*, never *sloppy*.

The seed (`src/v1/stage0/src/*.rs`) is "one realization" of the `.dag` truth and shrinks across a **typed self-host frontier**: each module is *self-emitted* (green-by-execution) or *seed-retained* (a declared row with a reason + migration trigger — countable, prioritizable, **never** a silent escape hatch, §5; the `DecodeFidelity`/`Lossless` boundary, §4). "v2 obviously works" once the self-emitted set compiles + runs + links against the retained seed; S3 (delete `src/v1`) becomes mechanical as the retained set drains to zero on its triggers.

Target language is **Rust**. The per-module receipt is: **emitted module compiles cargo-green + behavioral-equivalence to the seed on a discriminating corpus** (NOT `emit(node) == seed bytes`).

---

## 2. The mechanism (why this is rows, not a fork)

Every construct v2 can emit is a set of **grammar-relation rows** in `src/v2/extdeps/languages/rust.dag`, consumed by the shared machinery in `src/v2/compiler/06_translate.dag`:

- `grammar_relation_row_for_emitted(rules, emitted)` selects a production row for an emitted `Node` (the **emit** direction);
- `serialize_concrete_syntax_tokens_to_source(target, tokens)` renders the row's tokens to source text;
- `grammar_relation_row_forward_token_selection(tokens, rules, start_lhs)` reparses source back to the emitted `Node` (the **reparse** direction, in `03_ingest.dag`).

A new construct = new **formal productions + lex rules + a target model** in `rust.dag`. No edit to the fold, no compiler fork (DESIGN §4: *N rows, not N×M adapters*; §7: *a wall is a row*). The emitted-node shapes are **target-agnostic** production trees — the same shape emits to DAG, Rust, TypeScript, etc.; only the productions/lex (keyword spellings, punctuation) differ per target.

**Template to copy:** `dag_type_decl_structural_*` in `extdeps/languages/dag.dag` (the live DAG type-decl round-trip) is the canonical fixed-arity template. `rust_struct_decl_structural_*` / `rust_enum_decl_structural_*` / `rust_generic_enum_decl_structural_*` in `rust.dag` are the Rust adaptations landed so far.

---

## 3. Receipt discipline (non-negotiable)

Every rung lands with a `*_test.dag` receipt under `src/v2/test/claim/emit/` that is **green by execution** and carries a **discriminating RED**. Run one with:

```
gunbc run --source-root src/v2 --source-root dag \
  --entry src/v2/test/claim/emit/<rung>_test.dag --function <fn> --claim-run
```

Two receipt tiers, in preference order:

1. **Normalized round-trip** (preferred): `emit(node) → source → reparse → structurally-equal node`. Immune to formatting; this is DESIGN's "normalized round-trip, not golden strings." **Currently available only for fixed-arity constructs** — see §5.
2. **emit→golden** (fallback, path A): `emit(node) == exact Rust text`, plus a **golden-discrimination** control (`emit(node) != wrong_golden`) and an external perturbation check (perturb a fixture field/variant ⇒ receipt flips to `false`). This is what rungs 3–4 use for variable-arity constructs. Golden is a legitimate **per-construct dev receipt** for a single rung's emit; the round-trip is stronger and returns once §5 is fixed. (This is a *construct-level* correctness receipt only — the **terminal self-host oracle** is behavioral-equivalence on a discriminating corpus, §9 / Track D2, **not** a byte-diff over the corpus, per the revised §1.)

**Refusals are the worklist.** Every construct a rung does *not* cover is surfaced as an **executed** `grammar_relation_row_for_emitted` rejection with a count — never a fabricated skip (DESIGN §5). The refusal list *is* the backlog for the next rungs.

**Hard rules (review bar):**
- `.dag` files have **no comment syntax** in this dialect and **no multi-line `data =`**; documentation lives in names + this doc. `.dag` source must be **ASCII-only** (the lexer rejects non-ASCII).
- A receipt that only type-checks or greps is not done. "Done" = a real consumer green by execution + a RED that goes red when the behavior is wrong.

---

## 4. What has landed (rungs 1–4)

| Rung | Construct | Grammar (`rust.dag`) | Receipt |
|---|---|---|---|
| 1 | records → `struct` (fixed-2) | `rust_struct_decl_structural_*` | round-trip → now golden |
| 2 | nullary coproduct → `enum` (fixed-2) | `rust_enum_decl_structural_*` | round-trip → now golden |
| 3 | **variable-arity** structs & enums | recursive `*_suffix` productions | emit→golden |
| 4 | **generic + payload** enum → `enum Witness<C> { Holds { value: C }, … }` | `rust_generic_enum_decl_structural_*` | emit→golden; **witness.dag module fully emitted** |

**Milestone reached:** all three `witness.dag` declarations (2 records + the `Witness<C>` coproduct) emit byte-exact — the seed module is closed (`witness_coproduct_emit_test.dag :: witness_module_fully_emitted_holds`).

**Emitted so far (byte-exact clean Rust):**
```
struct StructuralPropertyWitness { property: Symbol, evidence: Node }
struct Artifact { kind: ArtifactKind, id: Symbol, file_path: String }
enum ExecutionMode { Hermetic, Wet, Record }
enum ArtifactKind { SourceFile, GeneratedSource, … WitnessBundle }   // 7 variants
enum Witness<C> { Holds { value: C }, Violates { diagnostic: Diagnostic } }
```

---

## 5. The one cross-lane dependency (reparse recursion)

**Symptom:** `grammar_relation_row_forward_token_selection` (`src/v2/compiler/03_ingest.dag`) rejects any grammar whose production list contains a **recursive alternative + an ε (empty) alternative** for one LHS — i.e. a variable-length list. The **emit** side handles the identical grammar fine.

**Reproducer** (verified): with `variant_list_tail → , ident variant_list_tail | ε`, `grammar_relation_row_for_emitted` emits `enum Mode { A, B, C }` byte-exact (Accepted), but `grammar_relation_row_forward_token_selection` on those tokens returns Rejected. Root: forward-selection needs one token of lookahead to choose the recursive vs ε alternative (next token `,` ⇒ recurse; `}`/`>` ⇒ stop).

**Ownership:** `03_ingest.dag` is in the parse/tactical lane (the forbidden `0{1,2,3}_*.dag` set for the emit lane). **This is the operator-assigned parallel task.**

**Dissolution trigger:** when forward-selection gains lookahead-based alternative selection (FIRST/FOLLOW or next-token disambiguation), the round-trip receipt returns for all arities. At that point, convert the rung 3–4 golden receipts back to round-trip and delete the "reparse blocked" notes.

---

## 6. Consolidation debt — RESOLVED (rung 7)

The declaration-grammar fork is paid down. The five special-case grammars (`rust_struct_decl_structural_*`, `rust_generic_struct_decl_*`, `rust_enum_decl_structural_*`, `rust_generic_enum_decl_*`) collapsed into **two general authorities** — `rust_struct_general_decl_*` and `rust_enum_general_decl_*` — over **shared sub-productions** (`rust_decl_shared_field_productions` / `_generic_productions` / `_variant_productions`, one definition each). One authority per decl-kind; the generic/non-generic and nullary/payload forms are **multiple productions per LHS selected by arity** (not ε-alternation), so emit disambiguates without needing the §5 reparse fix. The `field`/`type_expr`/`qualified_name` productions are now defined once and reused by struct fields *and* variant payloads.

Five emit receipts + a probe collapsed into one `decl_emit_consolidated_test.dag` (10 covered constructs across both grammars — nullary enums 2/3/7, generic-payload `Witness<C>`, structs arity 1/2/3, generic structs — plus RED, executed empty-record / empty-enum refusals counted 10/2, and the witness.dag-module-closed check). Type aliases stay their own grammar/test (a distinct decl kind).

**Remaining consolidation note:** type aliases could later fold their `qualified_name`/`type_expr` onto `rust_decl_shared_field_productions` too; low priority. Every *new* declaration construct (Track A) now extends the two general grammars, not a new fork.

---

## 7. Remaining work, decomposed for parallelism

Rungs are **additive rows** and mostly independent — the point of the row mechanism is that they parallelize. Below, `[P]` = parallelizable now; `[→X]` = depends on X. Each rung = one `rust.dag` grammar + one `emit/` receipt (green + RED + refusal count).

### Track A — declaration level (`type` / `data` / module framing)
- **A1 [P]** general enum authority (consolidate §6) — optional generics + optional payloads.
- **A2 [P]** generic parameters on `struct` (`struct Foo<T> { … }`) — mirror `rust_generic_enum` params onto the struct grammar.
- **A3 [P]** generic *instantiation* in field/variant types: `List<T>`, `Map<K, V>`, `Optional<T>`, `Outcome<T>` (type-expr with a generic-apply tail; the SG2 type-expression projection in `06_translate.dag` already models this shape — wire it into the decl grammars).
- **A4 [P]** type aliases (`type Symbol = …`, `type GeneratorId = Symbol`).
- **A5 [P]** empty/marker records (`struct Foo {}` / unit variants) — the 0-field / 0-variant ε cases.
- **A6 [→A1..A5]** `data` declarations (`data x: T = <value-expr>`) — needs the value-expression subset (Track B constructors + literals).
- **A7 [P]** module + import framing, `use` statements, `uses`/effect clauses on signatures.

### Track B — the expression language (fn bodies; the bulk, ~5,100 fns) — see §10 for the scoped decomposition
Superseded by §10 below. Scoping (rung-7 follow-up) found the compositional body-emission engine already exists and is target-agnostic; Rust just isn't wired in. Read §10, not this stub.

### Track C — decoration (REVISED 2026-07-08 — mostly DISSOLVED)
Byte-matching v1's *exact* decoration is **no longer required** (operator: drop byte-fixed-point). v2 emits only the decorations Rust **requires to compile and behave correctly**, **derived from the model, not matched to v1**: the derives actually used, `Rc<T>` where ownership demands it, `serde` where a wire format is consumed, the `im_rc` carriers the runtime needs. **C6 (Symbol carrier) stops being a landmine** — pick a representation that compiles and behaves, do not reverse-engineer v1's `pub struct Symbol(pub String)`. The C1–C10 items below are retained **only insofar as compilation/behavior needs them** (e.g. a `#[derive]` that a trait bound requires is in; a cosmetic `pub`/attribute that only matches v1 is out). The invariant measure is no longer v1's 12.7k-line surface.
- **C1** `pub` visibility; **C2** `#[derive(…)]` attributes; **C3** `#[serde(tag = "_variant")]` on enums.
- **C4** `Rc<T>` ownership wrapping (v1's Rc-insertion rules — the SG2 use-site-ownership rows in `06_translate` are the model).
- **C5** `im_rc` collection carriers (`Vec`→`Vector`, `HashMap`, `OrdSet`) + the `use` preamble.
- **C6** the `Symbol` carrier — **no longer a landmine** (byte-matching v1 dropped, per §116): v1 uses `pub struct Symbol(pub String)` with `Symbol`-typed fields lowered to `String`, but v2 need not reproduce that. Pick the cleanest newtype-or-alias that compiles and passes the behavioral-equivalence receipt.
- **C7** `v1_rt.rs` runtime shim generation; **C8** `Cargo.toml`; **C9** `lib.rs` / `main.rs` framing + `NonEmptyVec`/`NonEmptyBTreeSet`; **C10** the workspace-members region.

### Track D — "v2 works" (REVISED 2026-07-08 — byte-diff → behavioral-equivalence oracle)
- **D1 [→A,B,C]** emit the whole 40-file parse-pipeline closure (`s1_closure_receipt_test.dag` enumerates it) cargo-green.
- **D2 [→D1] REPLACED.** The terminal receipt is a **behavioral-equivalence oracle**, not a byte-diff: the self-emitted module compiles cargo-green **and** produces the same outputs as the v1 seed on a **discriminating corpus** (green-by-execution + a RED that goes red when the emitted behavior is wrong, §5). This is cheaper and more honest than the byte-diff, and it is the §5 replacement for byte-identity's lost correctness-oracle role.
- **D3 [→D2] REPLACED by the typed self-host frontier.** No byte-fixed-point. Instead: the self-emitted set links against the seed-retained set; each seed-retained module is a declared boundary row (reason + migration trigger). `src/v2/compiler/self_host.dag`'s harness is repurposed from digest/promotion to **frontier bookkeeping + behavioral-equivalence checks**.
- **D4 [→D3]** retire `regen_stage0` once the self-emitted set covers the closure with a green behavioral-equivalence receipt; S3 (delete `src/v1`) follows as the seed-retained set drains to zero on its triggers.

**Rough size (REVISED 2026-07-08):** dominated by Track B (expression language); Track C is reduced to the **compile/behavior-required** decorations (no longer v1 decoration parity — see the revised Track C header). The invariant measure is the ~5,100 fn bodies reaching a green **behavioral-equivalence** receipt over the self-emitted closure — **not** v1's ~12.7k-line decoration surface, and not the rung count.

---

## 8. Coordination / ownership

| Lane | Owns | Must not touch |
|---|---|---|
| **Strategic (this lane)** | `src/v2/std/compilers/target_model.dag`, `src/v2/extdeps/languages/rust.dag`, `src/v2/test/claim/emit/**` | `src/v1/**`, `src/v2/compiler/0{1,2,3}_*.dag` |
| **Parse/tactical** | `src/v2/compiler/03_ingest.dag` (the §5 reparse fix), `src/v1/05_emit_rust.dag`, parse-pipeline perf | `rust.dag` grammar rows, `target_model.dag` rows |

Cross-boundary changes (e.g. the `Symbol` carrier decision C6, the reparse fix §5) are a one-message sync with the operator, not a silent edit.

**CI note:** the `emit/` receipts are green by execution but are **not yet in the discovery roster** (`witness_discovery_scan_dirs` in `dag/gunbc/ci_layer_roots.dag` lists only `dag/test/claim` and `src/v2/test/claim/manual`). Adding `src/v2/test/claim/emit` there enrolls them into tree-wide discovery — a floor-lane config change to coordinate, tracked here.

---

## 9. Definition of done (the lane) — REVISED 2026-07-08

The self-emitted module set builds **cargo-green** and is **behaviorally equivalent** to the v1 seed on a discriminating corpus (green-by-execution, §5), linked against the seed-retained set across the typed self-host frontier. **Byte-identity is dropped as a requirement** (operator, 2026-07-08); the behavioral-equivalence oracle is its §5 replacement. `regen_stage0` retires when the self-emitted set covers the parse-pipeline closure with a green behavioral-equivalence receipt; S3 (delete `src/v1`) follows as the seed-retained set drains to zero on its migration triggers. Each seed-retained module remains a declared, counted boundary row — the frontier is honest and prioritizable, never a silent hatch.

---

## 10. Track B — expression bodies: scoping result & decomposition (post-rung-7)

### 10.1 The finding (reframes Track B)

The compositional body-emission **engine already exists and is target-agnostic**, and is validated end-to-end on **TypeScript, Python, C++, and Go** for all 6 behaviors + record-construct + field-access. **Rust is not wired into it.** The Rust `add` MVP is a *dead-end monolithic golden* — `TypeNode{Arrow}(i32,i32,i32)` + a flat baked 18-token spine + `rust_source_text` — that composes nothing and does not scale to real bodies.

So Track B is **not** "build a compositional emitter" (it exists) and **not** "add rows to a live Rust path" (there is none). It is: **wire Rust into the existing value-expression subsystem and supply Rust token synthesis per expression form, copying the 4 reference targets.**

Key machinery (all present):
- **Node model** (`std/node.dag`): expressions are `ComputationNode { behavior }`, `Behavior = Value | Transform | Branch | Loop | Bind | Match` (6, not 5); record-construct is `TypeNode{Conj}` gated at value position. Edge discipline: Value 0 / Transform ≥1 / Branch 3 (as arrow body) / Bind 3 / Loop LoopBoundEdges.
- **Projector** (`std/compilers/target_model.dag`): `target_project_arrow_body_to_value_expression` (~`:11298-11352`) dispatches each behavior → `handle_transform` / `target_project_field_access` / `target_project_branch_bool_if_else` / `target_project_bind_let` / `target_project_loop` / `target_project_match` / `target_project_record_construct`, producing a `TargetValueExpression`. `TargetValueExpressionKind` (~`:491-511`) enumerates **18 emit-able forms** (symbol, bool/char/string literals, Rc/Box ref, binding-ref, primitive/effect/callable apply, closure, conditional, bind-let, loop, record-construct, field-access, match).
- **Bodied-arrow path** (`compiler/06_translate.dag`): `translate_bodied_arrow_preserve_producer_ir` (`:534`, identity — keeps the ComputationNode body) → serialize via `target_serialize_bodied_arrow_from_model` (`:871-917`): fetch signature **scaffold**, project body → value-expression, synthesize body tokens, splice. Gated by `target_value_expr_arrow_has_value_expression_body`.
- **Source→node lowering** (`std/compilers/body_lowering.dag`, target-agnostic): `lower_binary_infix` → `Transform`, plus Branch/Loop/Bind/Match constructors.
- **Reference target** to copy: `ts_value_expression_projection` (`extdeps/languages/typescript.dag:527`), wired via edge `target_model_edge_value_expression_projection` (`:1455`); bodied scaffold attached by `grammar_relation_row_attach_bodied_scaffold` (`target_model.dag:~2630-2695`). Rust has **zero** `value_expression` references today.
- **Reference tests** to mirror (each *discriminates* — operand/op swap changes output, proving composition): `add_body_emit_typescript_test` (Transform/binop), `branch_if_then_else_emit_test`, `match_bool_emit_test`, `bind_emit_test`, `loop_emit_test`, `field_access_emit_test`, `record_construct_emit_test`, `fold_call_closure_emit_test`, `typescript_effect_io_emit_test`.

### 10.2 Decomposition

**B0 — wire Rust into the value-expression engine [GATING, serial, by lead]. LANDED.** Author `rust_value_expression_projection` (mirror `ts_value_expression_projection`), wire `target_model_edge_value_expression_projection` into the Rust bundle, attach the bodied-arrow signature scaffold, and supply Rust token synthesis for the *minimal* set to emit a **composed** `fn add(x: i32, y: i32) -> i32 { x + y }` — body built from a `ComputationNode{Transform}` (`+`) over two binding-refs, **discriminating** on operand-swap (`y + x`) and op-swap (`x - y`) like `add_body_emit_typescript_test`. This retires the monolithic `rust_mvp1_*` add golden as the body template. Receipt: `src/v2/test/claim/emit/rust_body_add_emit_test.dag`. **Everything below depends on B0.**

*Landed (this PR):* `rust_value_expression_projection` (binding-ref + primitive-apply wired to real Rust tokens, block mode `ValueProducing` for Rust's expression-tail block; all other forms `^rust_token_unwired_*` placeholders — each is a subsequent B-rung's to wire), `rust_operator_realizations_catalog_node` (`op_add → InfixToken{rust_token_plus}`), `rust_bodied_arrow_statement_scaffold` (`derive_bodied_arrow_scaffold` over signature-with-open + block-close), the two new bundle edges (`value_expression_projection`, `operator_realizations`), and `dag_binding_param_x/y` spellings. Bodied vs flat is decided by input node shape (`translate_type_expression_tree` short-circuits a value-expr-bodied arrow to `translate_bodied_arrow_preserve_producer_ir` — identity, no facts read), so the change is additive: the signature-only golden path (`rust_add_emit_translate_test`, `emit_host_add_equals_eval_test`) is unchanged. Receipt green by execution with a built-in RED: operand-swap producer emits `{ y + x }` (≠ base), minus catalog emits `{ x - y }` (≠ base, proving the operator comes from the catalog not a fixed golden), empty catalog → `Rejected` (fail-closed op_add lookup). The `rust_mvp1_*` flat golden row remains as the signature scaffold source; its flat-token path is superseded as the *body* template.

Per-form token synthesis (each = Rust arm of one `TargetValueExpressionKind` + a Rust `*_emit_test` mirroring the named TS reference; all `[P]` after B0):
- **B1 [P]** literals (Value): int, bool, char, **rope-string** (`FreeMonoid` Cons-chain → `free_monoid_to_string` at the boundary — landmine).
- **B2 [P]** binding-ref (a bare variable).
- **B3 [P]** primitive apply / binops (Transform): the operator surface (`+ - * / == && ...`).
- **B4 [P]** callable apply (Transform): function/method calls (named-arg model → Rust call syntax).
- **B5 [P]** field access (Transform-gated): `x.field` — mirror `field_access_emit_test`.
- **B6 [P]** conditional (Branch): `if c { t } else { e }` — mirror `branch_if_then_else_emit_test`.
- **B7 [P]** bind-let (Bind): `let x = v; …` (Rust block/`let-in`) — mirror `bind_emit_test`.
- **B8 [P]** loop (Loop) — mirror `loop_emit_test`. (Watch the open `body-lowering` FLAG thread in DESIGN — loop termination facts; park if it bites.)
- **B9 [P] but large]** match (Match) — mirror `match_bool_emit_test`; then decompose per pattern kind (variant / record / binding / wildcard) into sub-rungs.
- **B10 [P]** record-construct (`TypeNode{Conj}` at value position): `Name { field: v }` — mirror `record_construct_emit_test`.
- **B11 [P]** closure (Arrow body at value position): `|x| …`.
- **B12 [P]** Rc/Box reference forms (overlaps Track C ownership — coordinate).

**Net-new (not modeled on any target — design first):**
- **BN1** generic parameters on `fn` signatures (`fn f<T>(...)`), and generic instantiation in value position — no target models this yet.
- **BN2** `uses`/effect clauses on signatures — `effects.dag` exists but is unwired into signatures; `TargetValueExprEffectApply` exists on the value side (see `typescript_effect_io_emit_test`). Design the signature-level surface.

### 10.3 Recommended sequencing

1. **B0 first, serially** (I do it) — it re-establishes the fn-body path and is the dependency root; it also validates the Rust wiring against the TS reference before any fan-out.
2. Then a **parallel batch** of the highest-frequency body forms: **B1 (literals), B3 (binops), B6 (conditional), B7 (let)** — these cover most fn bodies and are mutually independent. The pilot brief template (worktree base = lane branch; absolute source-roots with the shared binary) applies; each agent copies the named TS reference test + the TS projection arm.
3. Then **B4/B5/B10/B9/B11** as a second batch; **B9 (match)** likely wants its own sub-decomposition.
4. **BN1/BN2** (generics, effects) are design-first and coordinated — not fire-and-forget.

### 10.4 Revised size note

Track B is **smaller and safer** than §7's original estimate: the engine, node model, projectors, body-lowering, and 4 reference implementations already exist, so B is ~1 gating rung (B0) + ~12 token-synthesis rungs that copy existing references + 2 net-new design rungs — not "build an emitter." The real-inferred-body path (`translate` preserves bodied arrows; serialize composes) is already supported, so this track also carries Rust toward the D-track **behavioral-equivalence milestone** ("v2 works"), not just fixtures.
