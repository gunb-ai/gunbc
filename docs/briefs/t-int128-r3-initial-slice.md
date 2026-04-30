# T-Int128 (R3) — initial audit + first shippable slices

**Dispatch:** Director inbox (#1130), lane thread #1142. **R3 framing:** [`docs/r3-structure.md`](../r3-structure.md) lane **T-Int128** (Evaluator-independent). **Sibling context (int-lit magnitude, R2 sub-lane):** [`docs/briefs/t-substrate-cardinality-int-lit-worker.md`](t-substrate-cardinality-int-lit-worker.md) — carrier widening and `LitInt` reshaping are explicitly deferred there to this lane.

## 1. Audit — current integer / literal / range surface

| Authority | Current fact |
|-----------|--------------|
| [`dsl/std/integer.dag`](../../dsl/std/integer.dag) | `Int8`–`Int64`, `UInt8`–`UInt64`; `type Int = Int64`, `type UInt = UInt64`. **No** `Int128` / `UInt128`. |
| [`dsl/std/bit.dag`](../../dsl/std/bit.dag) | `Word128` **exists**: `type Word128 { bytes: List<Byte> }` (distinct from opaque `Word64`-style carriers — modeling note in `integer.dag` still describes Int as `OrderedRing<Word64>`). |
| [`src/v3/std/substrate.dag`](../../src/v3/std/substrate.dag) | `LitInt(Int)` with `Int` the std default (Int64). |
| [`src/v3/std/tokenize.dag`](../../src/v3/std/tokenize.dag) (per cardinality brief) | `IntLit(Int)`; source narrowing at token time. |
| [`dsl/extdeps/languages/rust/primitives.dag`](../../dsl/extdeps/languages/rust/primitives.dag) | `TargetCarrier` closes at `Word64Carrier` (no `Word128Carrier`). Pilot `rust_pilot_primitives`: **8** `IntegerPrimitive` rows (`i8`–`i64`, `u8`–`u64`) with String-decimal `range_*`. |
| [`src/v3/compiler/src/int_literal_ranges.rs`](../../src/v3/compiler/src/int_literal_ranges.rs) | Range validation walks pilot list; `EXPECTED_INTEGER_ROWS = 8`; allowed carriers are Byte/Word16/Word32/Word64 only; `std_word_carrier_to_target_carrier_variant_ty` maps std `Byte`..`Word64` → `TargetCarrier` (no `Word128`). Interval math uses host `i128` for decimal bounds; **literal payload** in the DAG remains `LiteralBits::Int(i64)`. |
| [`src/v3/compiler/src/dag_scalar_generated.rs`](../../src/v3/compiler/src/dag_scalar_generated.rs) / infer | `LiteralBits::Int(i64)`; reconciliation still defaults wide int literals toward `Int64` (`int_shape()`), per cardinality brief citations. |
| R3 acceptance (structure doc) | `tier2_int128_overflow_proven`, `int_lit_full_int128_word128_consumer` — overflow story + full magnitude consumer for literals. |

**Non-goal for this lane:** invent a **parallel** numeric magnitude carrier. The approved end state is the **same** substrate path: std integer types + extdeps `TargetCarrier` + optional `LitInt` / token widening in lockstep with modeling — not a second “shadow” int literal representation.

## 2. STOP + PING (do not proceed silently)

1. **`dsl/std/substrate.dag`** — any change to `LitInt(...)` or its parameter type is **substrate.dag-adjacent / cross-program**; ping Substrate / Director before editing (align with [`t-substrate-cardinality-int-lit-worker.md`](t-substrate-cardinality-int-lit-worker.md) STOP notes).
2. **Default `Int` / `UInt` aliases** — do not retarget `Int = Int128` without an explicit language decision (host ergonomics, literals, inference).
3. **Grammar / tokenizer** — surface syntax for literals beyond `i64` (single-token `i64::MIN`, wider decimal magnitudes) is a **separate** slice from DSL type declarations; STOP if the parser/tokenize contract is unclear.
4. **DB-8 / self-host fixed point** — any bootstrap regen that moves declaration ids or stage0 output must satisfy the fixed-point discipline in the cardinality brief (`self_host_fixed_point`).

## 3. First shippable slices (recommended order)

**Slice A — std types only (smallest DSL fact):** Add to [`dsl/std/integer.dag`](../../dsl/std/integer.dag):

- `type Int128 = OrderedRing<Word128>`
- `type UInt128 = Semiring<Word128>`

*Caveat:* touches the live std module; may require **bootstrap regen** and a large `bootstrap_generated*.rs` delta — treat as one focused PR with tests + fixed-point check, not a drive-by.

**Slice B — Rust target pilot + compiler validation (no substrate.dag):**

1. Extend `TargetCarrier` with **`Word128Carrier`** in `primitives.dag` (and pilot-scope comments / Rust Reference citations as needed).
2. Append `IntegerPrimitive` rows for **`i128`** / **`u128`** with correct String-decimal inclusive bounds.
3. Regenerate or hand-sync, per file headers: `src/v3/compiler/src/bootstrap_generated.rs`, `bootstrap_generated_without_parse_surface.rs`.
4. Update [`int_literal_ranges.rs`](../../src/v3/compiler/src/int_literal_ranges.rs): `std_word_carrier_to_target_carrier_variant_ty`, `allowed_carriers`, `EXPECTED_INTEGER_ROWS`, and any error strings that assume “through Word64 only”.
5. Update [`src/v3/grounding_pilot/src/lib.rs`](../../src/v3/grounding_pilot/src/lib.rs) mirror (`RUST_PILOT_PRIMITIVES`, `TargetCarrier` enum) — file header states hand sync until T-Ground-Engine consumes `.dag` directly.

**Slice C — syntax + literal carrier (blocked on modeling / Director):** `tokenize.dag`, `LiteralBits`, inference narrowing to `Int128` / pilot rows — lands **after** A+B unless Director explicitly sequences otherwise.

**Slice D — R3 TestClaim gates:** Author `tier2_int128_overflow_proven` and `int_lit_full_int128_word128_consumer` in the appropriate `verification` / lane `.dag` once predicates are implementable without stubs.

## 4. Suggested test sequence after Slice B

1. `cargo test -p v3-compiler -- int_literal` (or the narrowest existing integration module that exercises `validate_rust_pilot_integer_primitives` / range routing).
2. `cargo test -p v3-compiler -- grounding_pilot` if present; else workspace filter for pilot primitives.
3. Full `cargo test --workspace --exclude v2-compiler-tests` + `cargo clippy --all-targets -- -D warnings`.

## 5. Disposition for Director

- **Today’s outcome:** audit captured above; **first concrete PR** = **Slice A** or **Slice B** depending on whether Substrate Manager prefers std-type-first vs extdeps-pilot-first (B unlocks range facts for `i128`/`u128` in the same authority list as existing pilots).
- **Explicit blocker:** Slice C requires grammar + `LitInt` / `LiteralBits` agreement — **STOP** until that contract is written (this brief does not specify token grammar).
