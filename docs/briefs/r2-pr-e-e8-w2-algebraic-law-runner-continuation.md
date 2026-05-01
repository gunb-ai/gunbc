# PR-E E8 W2 — `AlgebraicLaw` runner continuation (Identity / Distributivity gates)

**Status:** AUDIT / **docs-only** continuation after **`AlgebraicLawKind::Commutativity`** landed in the shared helper (`eval_algebraic_law_for_claim_program` in `src/v3/compiler/src/test_runner.rs`). **Does not** change `TestPredicate`, `AlgebraicLawKind`, or release-deferral / fixed-point runner locks.

**Parent dispatch:** [`r3-evaluator-dispatch.md`](r3-evaluator-dispatch.md) §E8 — Runner Extension Follow-Ons. **Bundle authority:** [`r2-pr-b-2-runner-extension-bundle.md`](r2-pr-b-2-runner-extension-bundle.md) §W2.

---

## 1. Live state on `main` (post–Commutativity wiring)

| `AlgebraicLawKind` | Substrate (`src/v3/std/verification.dag`) | `eval_algebraic_law_for_claim_program` | `TestRunner::eval_algebraic_law` |
| --- | --- | --- | --- |
| `Associativity` | Declared | Wired — bounded triple table via `int_associativity_holds_all_triples` + `ASSOCIATIVITY_WITNESS_TRIPLES` | Clean-compile gate + operational witness |
| `Commutativity` | Declared | Wired — bounded pair table via `int_commutativity_holds_all_pairs` + `COMMUTATIVITY_WITNESS_PAIRS` + shared `runner_structural_values_equal` | Same |
| `Identity` | Declared | **`UnsupportedLaw`** early return (no witness loop) | **`NotYetImplemented`** — explicit “no lens identity-element edge exposed yet” |
| `Distributivity` | **Not** a variant of `AlgebraicLawKind` | N/A — unknown labels route to `UnsupportedLaw` with “not in enum” semantics via `variant_fields` | **`NotYetImplemented`** text names P1 enum extension |

Operational checks remain **sample-table scaffolds**, not declared law facts from `dsl/std/algebra.dag` — same dissolution story as [`r2-evaluator-test-runner-authority-ratchet.md`](r2-evaluator-test-runner-authority-ratchet.md) §2.8 / `verification.dag` scaffold comment on `AlgebraicLaw`.

---

## 2. `Identity` — what remains (no safe runner-only step today)

**Target check (from W2 bundle):** for a named binary lens, pick the **declared** monoid identity for the lens’s **sequential** composition (`Lens<C>.sequential: Monoid<C>` in [`src/v3/std/lens.dag`](../../src/v3/std/lens.dag)) and assert `lens(identity, x) == x` and `lens(x, identity) == x` over a bounded witness table — **no** literals guessed in `test_runner.rs`.

**Substrate / compiler gap:** `Monoid<T>` already carries `identity: T` in [`dsl/std/algebra.dag`](../../dsl/std/algebra.dag). The lens carrier names `sequential: Monoid<C>`. There is **no** today-public Rust path in `lens_apply` (or adjacent lowering) that **reads** `sequential.identity` as a `FieldValue` / `Value` suitable for `apply_lens_declaration` arguments for an arbitrary compiled lens declaration ID (`rg` over `lens_apply.rs` finds no `identity` / `sequential` / `Monoid` extraction). Without that edge, a runner “Identity” arm would **re-introduce parallel authority** (magic identity literals), violating the W2 bundle and INVARIANTS modeling discipline.

**Next allowed shapes (pick one; do not stack ad-hoc literals):**

1. **Design + implementation (substrate/compiler adjacent):** add a **single** extraction or lowering witness that maps `program_dag` + `lens_decl_id` → identity carrier for `sequential` (typed, fail-closed when absent), then add `Identity` branch beside `Commutativity` using the **same** `runner_structural_values_equal` comparator and a **new** bounded witness table file in `lens_apply` (analogous to commutativity pairs).
2. **Docs-only / blocked:** keep `Identity` as `UnsupportedLaw` / `NotYetImplemented` until (1) lands — this document is the explicit gate record.

---

## 3. `Distributivity` — explicit routing (not W2, not `AlgebraicLaw` without P1)

`AlgebraicLawKind` is **`Associativity | Commutativity | Identity`** only ([`src/v3/std/verification.dag`](../../src/v3/std/verification.dag)). **Do not** overload another variant or smuggle distributivity through `LensOutputEquals` oracle hacks ([`r2-pr-b-2-runner-extension-bundle.md`](r2-pr-b-2-runner-extension-bundle.md) §W2).

Adding `Distributivity` is a **substrate sum extension** on `AlgebraicLawKind` → [`INVARIANTS.md`](../../INVARIANTS.md) §P1 substrate-fact-introduction (three-step procedure), same class of gate as [`docs/briefs/r3-v-l4-l7-direct-scaffold-notes.md`](r3-v-l4-l7-direct-scaffold-notes.md) §“do **not** add variants from this lane”. Runner work **follows** enum + law witness design; it does not lead.

---

## 4. Out of scope for this lane (Director locks)

- **Release-deferral / fixed-point** fixture constant sweeps (superseded **#1357 / #1385** thread) — **do not touch** `RELEASE_DEFERRAL_FIXTURE_PATH` or fixed-point pins under this dispatch.
- **E1** value-behavior / body-evaluator spine — separate branch/PR.
- **New `TestPredicate` variants** — forbidden without manager + P1 routing ([`r3-evaluator-dispatch.md`](r3-evaluator-dispatch.md) §E8 **STOP+PING**).

---

## 5. Verdict for immediate implementation PR

**No safe `test_runner`-only PR** for `Identity` or `Distributivity` under current gates without (§2.1) identity extraction or (§3) P1 enum work. **Safe next step:** land §2 design in a substrate/compiler-facing brief or implement (1) on a branch scoped to that extraction — **not** mixed with unrelated runner sweeps.
