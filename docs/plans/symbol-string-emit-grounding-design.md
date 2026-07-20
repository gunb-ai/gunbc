# Symbol/String emit grounding — the 82% E0308 lever (Gate 1)

Owner lane: `emit_representation_mismatch` (v1_deletion_plan.dag `^emit_representation_mismatch`).
Owns (s2-self-emit-brief §5): `src/v2/std/compilers/target_model.dag`, `src/v2/extdeps/languages/rust.dag`,
new claims under `src/v2/test/claim/emit/`. Must NOT touch `src/v1/**` (tactical lane) or `src/v2/compiler/0{1,2,3}_*.dag`.

## The fork (measured, not guessed)

The v2 emitter realizes the two "text" carriers **inconsistently**:

- `Symbol` (`v2.std.node`, opaque kernel) → native Rust `String` — `rust_target_atom_realization_symbol`
  (`rust.dag:1920`): `type_form` spelling `^rust_target_atom_type_spelling_string`, `value_form`
  `ValueSymbolToOwnedString` (`^atom` literal → `"...".to_string()`, a native `String` value).
- modeled `String` = `FreeMonoid<Char>` (`std.string_type`) → **`Vec<Char>`**, via the unconditional
  `rust_free_monoid_collection_primary_choice` → `TargetCollectionReprVec` (`rust.dag:1573`). There is **no**
  `Char`-element special case and **no** String atom-realization row.

So a `Symbol`-typed value (native `String`) meeting a modeled-`String` position (`Vec<Char>` / rope) is
`expected String, found Vec<..>` — E0308. This is the same seam the v1 seed emitter already grounds:
`rust_corpus_repr(has_v1_seed) = if has_v1_seed { HostNative } else { FaithfulFreeMonoid }`
(`src/v1/04_infer.dag:7715`). The green whole-corpus regen picks **HostNative** (`type String = std::string::String`,
`is_host_text_carrier_type` collapses `FreeMonoid<Char>`/`List<Char>`/`String` → native `String`, plus native
string-op impls `string_head`/`string_tail`/… — Root B, #5597/`e23f0f7ba7`). Isolated per-module emit picks
**FaithfulFreeMonoid** (`type String = Rc<FreeMonoid<Char>>`) and forks against every `.to_string()` Symbol value.

## Reframe (important, from committed receipts)

- **Gate 1 whole-emitter regen is already GREEN** (`regen_stage0 --emit-fresh` → 0 errors after the 1-line libc
  peel; `confidence_probe_gate1_*` on commit `1847af7296`). It emits HostNative (has_v1_seed=true) → no fork.
- The **~4400 E0308** the milestone sizes is the **Gate-2 per-module isolated CSSL probe** (FaithfulFreeMonoid,
  has_v1_seed=false). The committed report calls those counts *"closure-denominated, not distinct fix sites"*
  and the earlier dominant class (#6775/#6776 Rc-ownership wrap-decision) has since **landed**, leaving the
  Symbol/String representation collision as the current residual.
- The **type-pair histogram** behind "Symbol/String ~2100 = 82%" was a throwaway probe on the quiet-bee/#6924
  lane — **never committed**. There is no durable type-pair receipt today. Rebuilding one is a named deliverable.
- Full E0308 reproduction on `main` is **blocked**: the deep modules currently refuse with `E0425 cannot find
  type FreeMonoid` (import-closure), which masks E0308 (`first_error`). This is the `PlanDependency`
  (`emit_import_closure_root` + `generic_t_rendering` before COMPLETION). A **controlled minimal fixture** with a
  tiny import closure sidesteps the blocker and is the discriminating receipt we can run NOW.

## The grounding (generalize #5428: native form == modeled form)

Ground modeled `String` → native Rust `String` in the v2 emit path so the isolated/self-emit realization matches
the HostNative parity target and the `Symbol`/`.to_string()` native-String carrier. Symbol is already grounded and
already matches v1's settled carrier (`type Symbol = String`, confirmed in regenerated output) — no carrier ask
outstanding. Interpreter side largely exists (`free_monoid_to_string`, `StringRealizationStraddle` fail-closed
backstop in `v1_interpreter.rs`).

### Slice decomposition (each priced by the refusal bucket it zeroes, RED kept live)

1. **Type layer.** `FreeMonoid<Char>` (and the `String` alias) → native `String` in the v2 emit realization —
   a `Char`-element grounding at the collection-repr choice (the analog of `is_host_text_carrier_type`), NOT a
   blanket `FreeMonoid<T>` change. Discriminating receipt: emit a `String`-typed signature/field → `String`, not
   `Vec<Char>` (before: Vec — RED; after: String — green).
2. **Op layer.** `FreeMonoid<Char>` operations used on `String` (`string_head`/`string_tail`/`string_is_empty`,
   `Empty`/`Cons` construction) get native-String realizations, mirroring Root B's `rust_host_string_op_fn_emit`.
   A module that does rope-ops on `String` needs this or it emits native-String code expecting rope ops.
   Refusal-bucketed: an uncovered String op is a typed, located refusal naming the op (§5), never fabricated Rust.
3. **Cargo receipt.** A controlled fixture module (Symbol field + String field + a `^atom` Symbol value + a
   String value) whose emitted Rust cargo-compiles green — the by-execution bar (§5), the discriminating RED that
   goes red before the grounding.
4. **Durable type-pair receipt** (rebuild the thrown-away probe): extend the histogram tooling to bucket rustc
   errors by (expected, found) type pair so the Symbol/String share is committed and reproducible once
   import-closure clears. This is the plan's `antagonize` mechanism ("histogram dropping toward 0 as each wall lands").

## Open coordination point

FaithfulFreeMonoid is a genuine v1-emitter mode (isolated emit without the seed). The v2 emit path has no such
mode — grounding String→native there is unconditional (single authority), matching the HostNative parity target.
Whether the v1-emitter FaithfulFreeMonoid *String* dual-representation is retained or deleted is the **tactical
lane's** call (`src/v1/05_emit_rust.dag`); this lane only grounds the v2 rows and must emit the SAME carrier
(native `String`) so parity can hold.

Dissolution trigger for this doc: the grounding rows land in `target_model.dag`/`rust.dag` with the cargo receipt
green; this sketch then dissolves into the carrier marks + the committed emit claims (no parallel ledger).
