# R3 Bug-Fix Worker Brief — u128 grounding-pilot Rust mirror sync

**Owner**: Substrate Mgr (warm-wolf-698 / gunbc#2068) lane scope; worker dispatch via Substrate Mgr standing authority OR direct PM (deep-wolf-155 / gunbc#846) dispatch.
**Authority parent**: gpt-5-5-pro reflective analysis on `main@b09e0c8` Finding 1; PM dispatch at gunbc#846 #issuecomment-4413207527 (operator authorized 2026-05-09).
**Priority**: HIGHEST — concrete drift bug; thesis-validating (the bridge has already drifted, proving "must stay in sync" comments don't enforce).

---

## §0. Problem statement

The `.dag` substrate has u128 declared in three places:
- `dsl/extdeps/languages/rust/primitives.dag:274-277` — `IntegerPrimitive { target_name: "u128", algebra: SemiringAlgebra, carrier: Word128Carrier, ... }`
- `dsl/std/integer.dag:57-62` — `type UInt128 = Semiring<Word128>`
- `src/v3/spec/rust.dag:167-174` — `data rust_u128: TypeRealization`

But the Rust mirror at `src/v3/grounding_pilot/src/lib.rs:6-8` still says:
```
// (T-Int128 Slice B1 added i128; u128 deferred to B2 pending interval
// representation widening — see int_literal_ranges.rs.)
```

And the actual Rust list at `src/v3/grounding_pilot/src/lib.rs:256-265` jumps `u64 → bool` with no u128.

The file's own comment at `dsl/extdeps/languages/rust/primitives.dag:29-37` admits:
> "the engine does NOT yet read this `.dag` file directly. It mirrors the structural facts authored here as Rust constants..."

**The bridge has already drifted.** The "must stay in sync" convention isn't a contract.

## §1. Required outcome

Substrate authority wins; eliminate the drift.

## §2. Fix options

PM recommends **Option B** for speed (smaller PR; bridge survives until structural dissolution); **Option A** for proper dissolution (eliminates the bridge entirely).

### Option A — proper dissolution (preferred per `feedback_isomorphism_or_generation_for_mirrors`)

Make the grounding pilot **consume `rust_pilot_primitives` structurally** — read primitive rows from `.dag` declarations rather than mirroring as Rust constants. Eliminates the "must stay in sync" convention entirely.

Scope:
- Modify `src/v3/grounding_pilot/src/lib.rs` to load `rust_pilot_primitives` from compiled DAG at startup
- Delete `RUST_PILOT_PRIMITIVES` and `DAG_PILOT_TYPES` Rust constants
- Update `dsl/extdeps/languages/rust/primitives.dag:29-37` comment to reflect new state ("engine reads `.dag` declarations directly")
- Cementing test pinning that pilot's primitive set matches `.dag` rust_pilot_primitives `target_name` set

### Option B — pragmatic ratchet (faster)

Add `u128` to `RUST_PILOT_PRIMITIVES` Rust constant + cementing test that compares `.dag` `target_name` set against Rust constant set. Test fails if they ever diverge again.

Scope:
- Add `u128` IntegerPrimitive entry to `RUST_PILOT_PRIMITIVES` (mirror `dsl/extdeps/languages/rust/primitives.dag:274-277` row)
- Update Rust comment at `src/v3/grounding_pilot/src/lib.rs:6-8` to remove "u128 deferred" claim
- Add cementing test (`.dag` TestClaim form preferred per cross-cutting constraints): pins `.dag rust_pilot_primitives.target_name` set ⊆ Rust `RUST_PILOT_PRIMITIVES.target_name` set; fail-closed if either set has entries the other lacks

## §3. Files (expected scope)

**Option A**:
- `src/v3/grounding_pilot/src/lib.rs` (read `.dag` directly; delete Rust constants)
- `dsl/extdeps/languages/rust/primitives.dag` (update comment)
- `src/v3/compiler/tests/integration/` or fixture dir (cementing test)

**Option B**:
- `src/v3/grounding_pilot/src/lib.rs` (add u128 entry + comment update)
- `src/v3/compiler/tests/integration/` or fixture dir (cementing test pinning set equality)

## §4. Cross-cutting constraints

Per [PM dispatch at #issuecomment-4413207527](https://github.com/gunb-ai/gunbc/issues/2366#issuecomment-4413207527):

- **No new hand-Rust tests** — `.dag` TestClaim form preferred. If hand-Rust ratchet required: explicit dissolution-trigger comment in `sg0_census_test.rs` per option-(c) discipline.
- **STOP-and-PING via PM inbox (#846)** if shape questions arise.
- **Substrate authority canonical**: `.dag` declarations are the authority; Rust constants are the mirror.

## §5. Receipt

When work lands:
- u128 in pilot Rust constant + structural cite to `.dag` authority (Option B), OR Rust constants deleted + pilot reads `.dag` (Option A)
- Cementing test pins `.dag`-set ⊆ Rust-set (Option B) OR pins pilot-output ⊆ `.dag`-declared-set (Option A)
- Comment at `dsl/extdeps/languages/rust/primitives.dag:29-37` updated to reflect actual state
- SG-0 census: any new test entries marked with explicit dissolution-trigger comment per option-(c) (cite this brief as authority)

## §6. Dispatch trigger

PM-authored brief; awaiting worker dispatch (Substrate Mgr standing authority OR direct dispatch on operator-authorized session). Drift remains live on main until fixed.

---

**End of brief.**
