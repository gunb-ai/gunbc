# S2 — v2 emits v2: strategic direction & decomposition

**Status:** direction doc for the strategic S2 lane (v2's own emitter). Branch `claude/s2-v2-self-emit-wbvn5z`, PR #6374. Written after rungs 1–4 landed so the remaining work can be decomposed and parallelized.

This doc is the **single authority** for the lane's direction, receipt discipline, and rung decomposition. It is reasoned from the DESIGN axioms: a target language is *rows, not a compiler fork* (§4, §7), every claim is *green by execution* with a discriminating RED (§5), and every scaffold lands with a *named dissolution trigger* (§6).

---

## 1. Objective (the milestone)

v2's own emitter — `emit = serialize_target ∘ translate`, the one-grammar-read-backward machine (DESIGN §4) — covers v2's own language surface, proven module-by-module against the v1 seed's emit of the same sources, terminating in the **byte-fixed-point self-emit** that makes `regen_stage0` unnecessary. After that, the Rust seed (`src/v1/stage0/src/*.rs`) is "one realization" of the `.dag` truth and S3 (delete `src/v1`) is mechanical.

Target language is **Rust** (confirmed: `regen_stage0` emits `src/v1/stage0/src/*.rs`; the per-module receipt compares against `gunbc compile --target rust`).

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
2. **emit→golden** (fallback, path A): `emit(node) == exact Rust text`, plus a **golden-discrimination** control (`emit(node) != wrong_golden`) and an external perturbation check (perturb a fixture field/variant ⇒ receipt flips to `false`). This is what rungs 3–4 use for variable-arity constructs. Golden is a legitimate receipt (DESIGN: "a byte-diff is the terminal receipt"); the round-trip is stronger and returns once §5 is fixed.

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

### Track B — the expression language (fn bodies; the bulk, ~5,100 fns)
Ordered by dependency; most are `[P]` once B1 (signatures) lands. The add-fn MVP already emits arrows + `x + y`, so B-lits/B-binop have a seed.
- **B1** function signature framing (params, return type, generics, `uses`).
- **B2 [→B1]** literals — int, bool, and **rope-strings** (emitted text is Cons-chain `FreeMonoid`, not `Value::Str`; use `free_monoid_to_string` at the boundary — landmine).
- **B3 [→B1]** named-argument calls; **B4 [→B1]** field access / projection (`.`).
- **B5 [→B1]** constructor expressions (record/variant construction).
- **B6 [→B1]** `if` / `else`; **B7 [→B1]** `let` bindings; **B8 [→B1]** lambdas (`fn(x) { … }`).
- **B9 [→B4,B5]** `match` — patterns + arms. The largest single sub-track; decompose per pattern kind (variant, record, binding, wildcard).
- **B10 [→B1]** binops / operator surface beyond `+`.

### Track C — decoration parity (clean Rust → v1's *exact* Rust; ~12.7k-line surface in `src/v1/05_emit_rust.dag`)
v2 currently emits **clean** Rust; the fixed point requires byte-matching the v1 seed's decorated Rust. Each `[P]` after Track A/B basics exist.
- **C1** `pub` visibility; **C2** `#[derive(…)]` attributes; **C3** `#[serde(tag = "_variant")]` on enums.
- **C4** `Rc<T>` ownership wrapping (v1's Rc-insertion rules — the SG2 use-site-ownership rows in `06_translate` are the model).
- **C5** `im_rc` collection carriers (`Vec`→`Vector`, `HashMap`, `OrdSet`) + the `use` preamble.
- **C6** the `Symbol` carrier (**landmine**: v1 special-cases `Symbol` to `pub struct Symbol(pub String)` and lowers `Symbol`-typed fields to `String`; coordinate the newtype-vs-alias decision with the tactical lane before matching).
- **C7** `v1_rt.rs` runtime shim generation; **C8** `Cargo.toml`; **C9** `lib.rs` / `main.rs` framing + `NonEmptyVec`/`NonEmptyBTreeSet`; **C10** the workspace-members region.

### Track D — the fixed point
- **D1 [→A,B,C]** emit the whole 40-file parse-pipeline closure (`s1_closure_receipt_test.dag` enumerates it) cargo-green.
- **D2 [→D1]** byte-match v1's emit over the closure (terminal byte-diff receipt).
- **D3 [→D2]** self-emit fixed point: `v2-emit(v2 sources) == committed seed`, via `src/v2/compiler/self_host.dag`'s digest/promotion harness (already scaffolded).
- **D4 [→D3]** retire `regen_stage0`; S3 (delete `src/v1`) becomes mechanical.

**Rough size:** ~35–45 small rungs remaining, dominated by Track B (expression language) and Track C (decoration parity). The invariant measure is the ~5,100 fn bodies + v1's ~12.7k-line decoration surface, not the rung count.

---

## 8. Coordination / ownership

| Lane | Owns | Must not touch |
|---|---|---|
| **Strategic (this lane)** | `src/v2/std/compilers/target_model.dag`, `src/v2/extdeps/languages/rust.dag`, `src/v2/test/claim/emit/**` | `src/v1/**`, `src/v2/compiler/0{1,2,3}_*.dag` |
| **Parse/tactical** | `src/v2/compiler/03_ingest.dag` (the §5 reparse fix), `src/v1/05_emit_rust.dag`, parse-pipeline perf | `rust.dag` grammar rows, `target_model.dag` rows |

Cross-boundary changes (e.g. the `Symbol` carrier decision C6, the reparse fix §5) are a one-message sync with the operator, not a silent edit.

**CI note:** the `emit/` receipts are green by execution but are **not yet in the discovery roster** (`witness_discovery_scan_dirs` in `dag/gunbc/ci_layer_roots.dag` lists only `dag/test/claim` and `src/v2/test/claim/manual`). Adding `src/v2/test/claim/emit` there enrolls them into tree-wide discovery — a floor-lane config change to coordinate, tracked here.

---

## 9. Definition of done (the lane)

`v2-emit(all 40 closure modules)` builds cargo-green and **byte-matches** v1's emit, terminating in the self-emit fixed point (D3). At that point `regen_stage0` is deletable and the Rust seed is one realization of the `.dag` truth.
