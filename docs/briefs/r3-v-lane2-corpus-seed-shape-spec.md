# R3 Lane 2 — corpus seed shape spec (BoundDeclaration expectation + import alignment)

**Status:** PROPOSAL — research-only. **Does not claim [PR #1449](https://github.com/gunb-ai/gunbc/pull/1449) is merged.** After #1449 lands, re-verify §1 against `src/v3/std/substrate.dag` at `origin/main` HEAD.

**Canonical upstream:** Lane 1→Lane 2 import contract ([#1421](https://github.com/gunb-ai/gunbc/pull/1421) / [#1443](https://github.com/gunb-ai/gunbc/pull/1443) / [#1444](https://github.com/gunb-ai/gunbc/pull/1444) on `main`) — [`r3-v-lane1-lane2-corpus-identity-import-spec.md`](r3-v-lane1-lane2-corpus-identity-import-spec.md). L5 harness/extension narrative ([#1412](https://github.com/gunb-ai/gunbc/pull/1412) on `main`) — [`r3-v-l5-corpus-extension-spec.md`](r3-v-l5-corpus-extension-spec.md). Worker standby ordering — [`r3-v-l5-corpus-worker.md`](r3-v-l5-corpus-worker.md).

**Non-goals:** no substrate edits in this brief; no new `TestPredicate` variants; no fixture authoring; no parser / `Int(lo..hi)` surface (#1449 boundaries).

---

## 1. Expected `BoundDeclaration` carrier (#1449 — verify on merge)

Per **#1449** summary and branch `session/crisp-ibex-569-bound-declaration`, `src/v3/std/substrate.dag` is expected to gain a **closed sum** adjacent to `Interval<D>`:

```text
type BoundDeclaration
  = StaticBound(Interval<Int>)
  | PlatformDependent
```

**Semantics (substrate comments on PR):** `StaticBound` uses the shared `Interval<Int>` parent for static value-domain bounds; `PlatformDependent` is a distinct kind whose interval resolves after the target platform is known. **Slice A is carrier-only:** no parser/lowerer syntax, no Grounding projection reader, no per-target inhabitance population in the same PR.

**Honesty:** `git grep -n '^type BoundDeclaration' src/v3/std/substrate.dag` on **`origin/main` before #1449** returns nothing — that is expected. Post-merge, this section must match landed spelling exactly.

---

## 2. Mapping Mechanism (a) — program identity stays paired strings

[`r3-v-lane1-lane2-corpus-identity-import-spec.md`](r3-v-lane1-lane2-corpus-identity-import-spec.md) §Mechanism **(a)** authorizes a **shared `.dag` corpus module** whose **single editable transaction per row** defines **both** strings fed to `compile_to_dag(source, file_name)`, then projects into sibling **`TestClaim.source` / `TestClaim.file_name`** ([`verification.dag`](../../src/v3/std/verification.dag)). Cross-lane pairing uses **`TestClaim.name` → `TestClaimValue.claim_name`** ([`test_runner.rs`](../../src/v3/compiler/src/test_runner.rs); live consumer sketch in [`r3_verification_l4_l7_l5_skeleton_test.rs`](../../src/v3/compiler/tests/integration/r3_verification_l4_l7_l5_skeleton_test.rs)).

**Alignment with #1449:** `BoundDeclaration` answers **coercion-fold bind-shaped bounds** over **`Interval<Int>`**, not corpus program text. It **does not** replace Mechanism **(a)**’s paired program authority or merge **`source` + `file_name` + row key** into one nominal at #1449.

**Optional later composition (§P1 only):** If strict rows need **machine-checked static bounds** alongside program text (see [`r3-v-l5-corpus-extension-spec.md`](r3-v-l5-corpus-extension-spec.md) §1.1 numeric policy / overflow-freedom posture), a Director-ratified substrate record could **compose** paired program identity **with** bound metadata carried as `BoundDeclaration` **after** coercion-fold consumers exist — not a fixture workaround ([INVARIANTS §P1](../../INVARIANTS.md#p1-modeling-faithfulness)).

---

## 3. Pre-authored seed shape — `add_then_branch` row (illustrative)

**Intent:** preserve today’s slice‑1 skeleton semantics ([`r3_verification_l5_corpus.dag`](../../src/v3/compiler/tests/fixtures/r3_verification_l5_corpus.dag) claim `r3_verification_l5_cross_target_skeleton`) while moving toward **one shared authority module** consumed by **both** Lane 1 (`r3_verification_l4_emit_eval_match.dag` family) and Lane 2 fixtures.

### 3.1 Shared module sketch (Mechanism (a))

Illustrative path + identifiers (rename to match fixture policy when implementing):

- Module file: `src/v3/compiler/tests/fixtures/corpus/r3_certification_corpus.dag`
- One **paired binding** per logical row (exact surface syntax is worker-owned; semantics are **atomic edit** of both strings):

```text
// Illustrative constants — must be authored in one transaction.
// data r3_cert_add_then_branch_source: String = "<verbatim program text bytes>"
// data r3_cert_add_then_branch_file: String = "add_then_branch_seed.v3"
```

Each lane’s **`TestClaim`** references these declarations (import/projection) so Lane 1 vs Lane 2 **never** independently edits duplicated string blobs — [**§P2**](../../INVARIANTS.md#p2-boundary-discipline) posture from the import contract.

**Ground-truth bytes today:** sidecar [`add_then_branch_seed.v3`](../../src/v3/compiler/tests/fixtures/r3_l5_corpus/add_then_branch_seed.v3) must remain the byte authority referenced by both lanes until generator mechanism **(b)** replaces it.

### 3.2 Optional `BoundDeclaration` adjunct (post–#1449 + §P1 consumers)

Orthogonal to §3.1, a row-level witness might someday declare a **static integer interval** for literals/intermediates using **`StaticBound(...)`** once lowering attaches bounds to the declarations coercion-fold cares about. Sketch only — **`BoundedInterval { lower, width }`** is the **bounded** arm of substrate **`Interval<Int>`** (see §1 and `src/v3/std/substrate.dag`: the sibling arm is **`Unbounded`**). This is the same carrier **`StaticBound(Interval<Int>)`** names, not a parallel type.

```text
// PROPOSAL — requires landed consumers beyond #1449 carrier slice.
// data r3_cert_add_then_branch_int_domain: BoundDeclaration =
//   StaticBound(BoundedInterval { lower: ..., width: ... })
```

This supports **numeric policy** articulation for Tier‑2 strict rows; it does **not** change how **`TestClaim.source` / `file_name`** are populated.

---

## 4. CI ratchet + harness defaults

**Import-resolution ratchet** (from [`r3-v-lane1-lane2-corpus-identity-import-spec.md`](r3-v-lane1-lane2-corpus-identity-import-spec.md) §Mechanism **(a)**):

1. Compile Lane 1 verification DAG + Lane 2 verification DAG using **`cached_compile`** under **`OnceLock`** amortization where appropriate ([**TESTING.md**](../../TESTING.md)).
2. **Join:** locate structural **`TestClaim`** declarations whose substrate **`name`** field matches across lanes ([`verification.dag`](../../src/v3/std/verification.dag)). **`TestClaimValue.claim_name`** is **`name`** projected through **`TestClaimValue::from_declaration`** — there is **no** separate `claim_name` field on the `.dag` `TestClaim` record.
3. `TestClaimValue::from_declaration` each matched declaration; assert:

```rust
assert_eq!(l4.source, l5.source);
assert_eq!(l4.file_name, l5.file_name);
```

Match **`ClaimResult`** **by variant shape** only (`Pass` / `Fail` / `NotYetImplemented(_)`) — no diagnostic substring pinning ([**TESTING.md** — Don’t assert on implementation details](../../TESTING.md#dont-assert-on-implementation-details); [**DB‑1**](../../INVARIANTS.md#db-1) / [**C‑5**](../../INVARIANTS.md#c-5) hooks via import contract).

**DB‑3 / DB‑20 posture:** compile-time dimension proofs stay on the `AnalysisDimension`-style carriers documented in [`design-dimension-abstraction.md`](../design-dimension-abstraction.md); workflow parallelism remains ordinary lens data under DB‑20 — do not route harness exits through parallel dimension slots when authoring L5 receipts.

---

## 5. Live-path verification receipt

Extend whenever hyperlinks change:

```bash
git fetch origin
for p in \
  INVARIANTS.md \
  TESTING.md \
  docs/design-dimension-abstraction.md \
  docs/briefs/r3-v-lane1-lane2-corpus-identity-import-spec.md \
  docs/briefs/r3-v-l5-corpus-extension-spec.md \
  docs/briefs/r3-v-l5-corpus-worker.md \
  src/v3/compiler/tests/fixtures/r3_l5_corpus/add_then_branch_seed.v3 \
  src/v3/compiler/tests/fixtures/r3_verification_l5_corpus.dag \
  src/v3/compiler/tests/integration/r3_verification_l4_l7_l5_skeleton_test.rs \
  src/v3/compiler/src/test_runner.rs \
  src/v3/std/substrate.dag \
  src/v3/std/verification.dag
do git cat-file -e "origin/main:$p" || exit 1; done
```

After this brief merges, add **`docs/briefs/r3-v-lane2-corpus-seed-shape-spec.md`** to the loop above (same discipline as sibling briefs — receipt lists every hyperlinked path).

**Post‑#1449 smoke:** `rg -n '^type BoundDeclaration' src/v3/std/substrate.dag` must show the **two-variant** sum above; if Substrate adjusts naming/payload during review, update §1 before relying on this brief for implementation dispatch.

---

## 6. Re-engagement triggers

1. **#1449 merges:** verify §1–§3 against landed `substrate.dag`; refresh §5 smoke if needed.
2. **Fixture/module PR:** implement §3.1 under Mechanism **(a)** or fall through to Mechanism **(b)** generator discipline per import contract — still **no** steady-state `include_str!` duplication ([#1394](https://github.com/gunb-ai/gunbc/pull/1394) posture cited upstream).

**Manager inbox:** [#1276](https://github.com/gunb-ai/gunbc/issues/1276).
