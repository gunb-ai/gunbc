# v4 UPSERT stray scan receipt — 2026-05-30

**Node:** `adhoc-74470445-f54`

**Parent workstream:** `adhoc-342cfea2-653` workstream 2 kickoff.

**Scope:** `src/v4/compiler`, `src/v4/std`, and `src/v4/lens`.

**Purpose:** record the bounded scan requested for the v4 compiler/std/lens
surface against the UPSERT canon from
`docs/audit/upsert-pattern-compiler-stray-2026-05-29.md`.

This is a scan receipt, **not** a mark ledger. It does not classify inline
`🟢`/`🟡`/`🔴` marks, does not assign dissolve-on status, and does not replace
the owning carrier comments, `src/v4/TASKS.md`, or PR review. Per
`docs/modeling-discipline.md`'s no-ledger rule, any future correction belongs
at the inline mark or owning task row, not in this file.

**UPSERT canon:** verify-first, satisfy dependencies recursively,
create-if-missing, cache-outcome.

---

## §0. Reproducibility

Run from repo root:

```bash
rg -n "Upsert|upsert|UPSERT|ensure|create|write|overwrite|cache|memo|idempotent|EffectClassification|signature-deferred|deferred" \
  src/v4/compiler src/v4/std src/v4/lens -S
```

Result in this PR workspace: **258** hits.

Deferral cross-check:

```bash
rg -n "feature:T22-EVAL-CACHE-HASHES|EffectClassification|signature-deferred|T-13-effect-signature-deferred|T-13-idempotency|config-patch-record-projection|UpsertEffect|idempotent-operation" \
  src/v4/compiler src/v4/std src/v4/lens docs/audit/v4-deferral-audit-2026-05-29.md -S
```

Result in this PR workspace: **40** hits.

Literal marker check:

```bash
rg -n "STRAY-FROM-UPSERT" src/v4/compiler src/v4/std src/v4/lens -S
```

Result in this PR workspace: **0** hits.

---

## §1. Inspection Anchors

These anchors were inspected during the scan because they carry either
UPSERT vocabulary, idempotency/effect semantics, cache receipt semantics, or
yellow-tagged deferred substrate bridges. They are listed only to make the
scan reproducible; the source files and their inline marks remain the
authority.

| Anchor | Reason inspected |
| --- | --- |
| `src/v4/std/effects.dag` | Declares `UpsertEffect` under the idempotent effect partition. |
| `src/v4/lens/testgen.dag` | Schedules idempotent-operation TestClaims, including the upsert sample. |
| `src/v4/lens/effect.dag` | Carries the signature-deferred `EffectClassification` bridge. |
| `src/v4/lens/idempotency.dag` | Carries the algebra-witness-required idempotency verdict surface. |
| `src/v4/compiler/05_eval.dag` | Carries TestClaim cache key/receipt and `T22-EVAL-CACHE-HASHES` bridge marks. |
| `src/v4/compiler/00_compile.dag` | Carries the local compile lens adapter pending the T-23 lens application migration. |
| `src/v4/std/dependency.dag` | Carries dependency usage classification pending resolve-authored ground facts. |
| `src/v4/std/patch.dag` | Carries `FieldPatch<T>` / `ConfigPatchRecord` config overlay vocabulary. |

---

## §2. Receipt

No untagged literal `STRAY-FROM-UPSERT` marker exists in the scoped source
tree at this snapshot. The scan found UPSERT-relevant surfaces, but their
status is already carried by inline source comments and the existing
`docs/audit/v4-deferral-audit-2026-05-29.md` snapshot. This PR therefore
does not add a second classification table.

Detailed judgment for the inspected anchors belongs in PR review discussion.
If a reviewer or owner finds an incorrect inline mark, the fix is to edit the
owning `.dag` mark or task row directly.
