# v4 SG-7 Worksheet — CI offset projection complexity (`ci.dag` T-22)

> **Status:** WORKSHEET APPROVED — Modeling DFS Manager §8 sign-off 2026-05-30 (proud-pike-680; PM option **(c)** — manager-authored, worker layer skipped due to dashboard messaging pattern #5).
> **Date:** 2026-05-30
> **Dispatch anchor:** `docs/audit/v4-rustc-error-catalog-2026-05-29.md` SG-7 row; blocks PR **#3974** (short-term CI relief, quiet-ant-650) and T-38-PR1 zero-diagnostic receipt.
> **Prerequisite for implementation:** None beyond this worksheet — does **not** block M1 rustc iteration meter (catalog §3.2).

---

## Mechanical dispatch rule

> **No SG-7 implementation worker may land until this worksheet is complete and Modeling DFS Manager–approved.**

Same discipline as PR #3938 §10.0. Acceptance is **T-22 complexity pass + cache-behavior preservation**, not error-count reduction on the M1 probe.

---

## §10.0-adapted worksheet

```text
SG class:               SG-7 (v2 complexity — same-argument recursion; NOT rustc E04xx)
Representative failure:  v2-diagnostic: complexity same-argument recursion (24×) at
                         `fn ci_int_offset_authority_projection_node(i: Int) -> Node {`
                         @ src/v4/workflow/ci.dag:776 (catalog row cites :721 — re-anchor on landing)
Immediate local patch:   Add `ci_int_offset_authority_projection_bounded` peel only (HEAD may have this) while
                         keeping parallel ci tree; or memoize — still forbidden vs std/node single authority.
Why forbidden:           Parallel offset authority vs `v4.std.node` `byte_offset_cache_digest_authority`
                         (limbs_remaining-bounded, already landed for T-22 cache claims); same-arg recursion
                         in ci.dag blocks honest T-38 / pipeline_rejections compile receipts.
DFS path:
  std/ authority (SINGLE):
    - src/v4/std/node.dag — ByteOffsetCacheDigestAuthority coproduct;
      byte_offset_cache_digest_authority / byte_offset_cache_digest_nonneg_bounded_authority
      (limbs_remaining counter — NOT same-arg recursion);
      byte_offset_cache_key / byte_offset_cache_key_fingerprint;
      canonical tags (byte_offset_neg, byte_offset_out_of_ceiling, …)
  v4 workflow (CONSUMER — dissolve parallel tree):
    - src/v4/workflow/ci.dag:704-818 — ci_byte_limb_projection_node,
      ci_int_offset_authority_projection_bounded (:724), ci_int_offset_authority_projection_node (:776),
      ci_char_projection_node, ci_string_projection_node
      (🟡 dissolve-on L704; bounded peel is interim — still parallel authority vs std/node)
  v4 test consumers:
    - src/v4/test/claim/manual/test_claim_cache_digest_sensitivity.dag — byte_offset_cache_key*
    - src/v4/test/claim/workflow/pipeline_rejections.dag — CiPipeline eval subjects → ci projections
    - src/v4/workflow/ci.dag:1299-1308 — ci_char_projection_out_of_range_injective_witness
  extdeps:               (none)
  compiler stage:         v2 complexity pass on generated workflow module (T-22), not 04_infer/05_emit
Deepest unsound boundary:
  ci.dag re-implements unbounded Int→Node offset expansion while std/node already owns bounded
  ByteOffsetCacheDigestAuthority keyed by content_hash semantics (IRT-4 / T-22).
Systemic fix:
  Delete or dissolve ci_int_offset_authority_projection_node + ci_byte_limb_projection_node offset
  encoding; introduce ONE Node projection wrapper in v4.std.node (or consume existing fingerprint
  projection) and route ci_char_projection_node / string fold through std authority only.
Non-goals:
  - SG-1..6 rustc substrate fixes; full retirement of all ci_*_projection_node helpers in one PR;
  - Changing byte_offset_cache_digest_max_limbs without Modeling DFS worksheet;
  - M1 probe error-count claims as primary metric.
Falsification probe:
  (1) v2 compile of workflow/ci.dag + pipeline_rejections.dag: zero "same-argument recursion" diagnostics.
  (2) ci_char_projection_out_of_range_injective_witness still true (content_hash distinguishability).
  (3) test_claim_cache_digest_sensitivity byte_offset_* rows unchanged (no silent cache aliasing).
  (4) Grep implementation PR: no `ci_int_offset_authority_projection_node` self-call remains.
Metric allowed only as secondary:
  T-38-PR1 host_scaffold_receipt progresses from blocked_m1_subset — after (1)-(4) pass.
```

---

## §1 Single-authority fact

| Field | Value |
| ----- | ----- |
| **Authority type** | `ByteOffsetCacheDigestAuthority` |
| **Canonical home** | `src/v4/std/node.dag` |
| **Public API** | `byte_offset_cache_digest_authority(i: Int)`, `byte_offset_cache_key(i: Int)`, `byte_offset_cache_key_fingerprint(key: ByteOffsetCacheKey)` |
| **CI obligation** | Project/consume — **do not** re-encode limb/quotient recursion in `ci.dag` |

### 1.1 Implementation shape (manager-approved; Compiler Spine lands)

```dag
// NEW in v4.std.node (names illustrative — one wrapper only):
fn byte_offset_cache_key_projection_node(i: Int) -> Node {
  // Project ByteOffsetCacheKey / authority tags structurally.
  // MUST NOT call ci_* helpers. MUST use bounded authority fns only.
}

// ci.dag — replace body of ci_char_projection_node out-of-range / offset arms:
//   target: byte_offset_cache_key_projection_node(i: c)   // not ci_int_offset_authority_projection_node
```

**Delete** after migration: `ci_int_offset_authority_projection_node`, `ci_byte_limb_projection_node` (succ/pred limb tree), and any dead canonical_tag_scalar_int_* edges used only by those fns.

**Preserve:** `ci_string_projection_node` list fold structure; swap per-char target to std projection.

---

## §2 Spot-fix register (forbidden)

| Pattern | Why forbidden |
| ------- | ------------- |
| `ci_int_offset_authority_projection_node(i: int_negate(i: i))` with same `i` | Same-arg recursion (current failure) |
| Memoization `Map<Symbol, Node>` keyed by stringified offset | Name-keyed authority (P2) |
| Unbounded `limb - 1` tree in ci.dag only | Duplicates std/node bounded authority |
| Lowering `byte_offset_cache_digest_max_limbs` in ci.dag | Changes T-22 cache contract without worksheet |
| Ignoring pipeline_rejections.dag compile | Hides T-38 blocker behind partial ci.dag fix |

---

## §3 Downstream worker brief (Compiler Spine + quiet-ant #3974)

```text
Land after this worksheet (pair with #3974 CI relief PR):

MUST:
  - Route ci char/string offset projections through v4.std.node byte_offset authority
  - Clear 24 v2 same-argument recursion diagnostics
  - Keep ci_char_projection_out_of_range_injective_ok passing
  - Keep test_claim_cache_digest_sensitivity byte_offset rows passing

MUST NOT:
  - Any §2 forbidden pattern
  - Claim M1 rustc residual improvement as acceptance (SG-7 is orthogonal per catalog §3.2)

Escalate to Modeling DFS:
  - If Node projection cannot preserve injective witness without new std/node variant
  - If pipeline cache digests require semantics beyond ByteOffsetCacheKey
```

---

## §4 Manager approval checklist — CLOSED 2026-05-30

- [x] Single-authority fact: `ByteOffsetCacheDigestAuthority` / `byte_offset_cache_key` in `v4.std.node`
- [x] ci.dag parallel tree marked for dissolution (L704-818)
- [x] Falsification probes (complexity + injective witness + cache sensitivity)
- [x] M1 vs T-38 scope boundary (catalog §3.2)
- [x] Implementation dispatch authorized → **Compiler Spine** (`smart-stag-871`) + **#3974** owner

---

## Related artifacts

- `docs/audit/v4-rustc-error-catalog-2026-05-29.md` — SG-7 row, §3.2 M1 vs T-38
- `docs/planning/v4-correctness-ladder-2026-05-30.md` §10.0 — worksheet discipline
- `src/v4/std/node.dag` — `byte_offset_cache_digest_*` / `byte_offset_cache_key` (L675+)
- `src/v4/workflow/ci.dag` — L704-818, L1299-1308
- `src/v4/test/claim/workflow/pipeline_rejections.dag` — compile consumer
