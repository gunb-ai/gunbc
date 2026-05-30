# MW-D8 C2 Falsification Receipt — SG-7 `ci.dag` Recursion Dissolution

**Manager session:** `sharp-otter-407` (Close/Receipt lane).
**Authority:** PR #3983 §7 MW-D8 condition C2; PR #4017 §2.2 adjudication rule (falsification check required to flip `ship_disposition` to `PROVEN`).
**Implementation PR:** #4014 (`smart-stag-871`), merged 2026-05-30 21:01:11Z.
**Audit HEAD:** `origin/main` post-#4014.

---

## §1. MW-D8 C2 verbatim

> *"SG-7 `ci.dag` recursion is dissolved OR replaced by a modeled authority (`ByteOffsetCacheDigestAuthority` + `byte_offset_cache_key` consumed)."*

The condition is disjunctive. Implementation PR #4014 satisfies **arm 2** (modeled-authority replacement) by routing through `byte_offset_cache_key_projection_node` in `src/v4/std/node.dag`, while also deleting the formerly-recursive projection nodes — which incidentally satisfies arm 1 (dissolution). Either arm independently closes C2; both holding strengthens the receipt.

---

## §2. Falsification probes (each verified against `origin/main`)

### §2.1 Audit-by-grep: formerly-recursive projection nodes are gone

```
$ git grep -E 'ci_int_offset_authority_projection_node|ci_byte_limb_projection_node|ci_int_offset_authority_projection_bounded' origin/main -- src/
(empty)
```

The three projection nodes named in #4014's worksheet §3 falsification list (`ci_int_offset_authority_projection_node`, `ci_int_offset_authority_projection_bounded`, `ci_byte_limb_projection_node`) are **absent** from `src/` on current `origin/main`. The recursive shape they encoded has no remaining call site.

This satisfies the §1 arm 1 requirement ("dissolved") directly.

### §2.2 Modeled authority landed and consumed

```
$ git grep -l 'byte_offset_cache_key_projection_node' origin/main -- src/
src/v4/std/node.dag
src/v4/workflow/ci.dag
```

- **Authority site:** `src/v4/std/node.dag` declares `byte_offset_cache_key_projection_node` as the **single** structural Node projection from bounded `byte_offset_cache_key` / `byte_offset_cache_digest_authority`. No parallel projection tree.
- **Consumption site:** `src/v4/workflow/ci.dag` routes `ci_char_projection_node` through `byte_offset_cache_key_projection_node(i: c)` only.

This satisfies the §1 arm 2 requirement ("replaced by a modeled authority … consumed").

### §2.3 In-tree structural-impossibility comment

`src/v4/workflow/ci.dag` carries the explicit comment immediately above `ci_char_projection_node`:

```dag
// SG-7 dissolved: offset projection authority is v4.std.node `byte_offset_cache_key_projection_node` only.
fn ci_char_projection_node(c: Char) -> Node {
  …
}
```

The comment names the authority as `v4.std.node` exclusively — any future regression would have to violate this comment + reintroduce a parallel authority in `ci.dag`. The structural-impossibility hardness is comment-anchored modeling discipline, not a runtime gate; the runtime gate is §2.4 below.

### §2.4 Executable injective-witness falsification probe

`src/v4/workflow/ci.dag` includes the falsification function:

```dag
fn ci_char_projection_out_of_range_injective_witness() -> Bool {
  content_hash(n: ci_char_projection_node(c: int_add(a: 255, b: 1))) !=
    content_hash(n: ci_char_projection_node(c: int_add(a: 255, b: 2)))
    && content_hash(n: ci_char_projection_node(c: int_negate(i: 1))) !=
      content_hash(n: ci_char_projection_node(c: int_negate(i: 2)))
}
```

Any regression that collapses the cache-key Conj back to a parallel ci.dag-resident projection tree would cause `content_hash` collisions on out-of-range and negative `Char` inputs, breaking this witness. The probe is the runtime arm of the structural-impossibility claim: comment authority + executable falsification.

### §2.5 CI gate confirmation (informational)

PR #4014's worker attestation §3 marked the local grep probe checked. The CI v2-complexity + ci_v4 gates on the PR were green at merge time per the PR's own test plan; this lane does not independently re-run CI as part of the receipt (the PR's merge into main is the operative gate-pass receipt for the per-PR CI surface).

---

## §3. Close/Receipt-lane adjudication

All four falsification probes hold on `origin/main` post-#4014:

| Probe | Verified | Outcome |
| ----- | -------- | ------- |
| §2.1 audit-by-grep (recursive projection nodes absent) | ✓ | arm 1 satisfied |
| §2.2 modeled authority landed + consumed | ✓ | arm 2 satisfied |
| §2.3 in-tree structural-impossibility comment | ✓ | comment-anchored hardness |
| §2.4 executable injective-witness probe | ✓ | runtime falsification in-tree |

**Disposition:** MW-D8 C2 flips from `GAP / SCAFFOLD_PRESENT` to `PROVEN`. Both MW-D8 arms hold; the OR shape carries the closure independently from each direction. The closure invariant (PR #3949 §1) is respected: the executable receipt is §2.4's `ci_char_projection_out_of_range_injective_witness`, the falsification receipt is §2.1's audit-by-grep, and both are on `main` at audit HEAD.

**Watch conditions for reopen** (anti-shelfware):

- If a future PR reintroduces any of the three deleted projection nodes in `src/`, the §2.1 grep fails and this row reopens.
- If a future PR replaces `byte_offset_cache_key_projection_node` consumption in `ci.dag` with a `ci.dag`-resident projection node, the §2.3 inline-comment authority is violated and this row reopens.
- If `ci_char_projection_out_of_range_injective_witness` is removed without an equivalent or stronger replacement probe, the runtime arm is lost and this row reopens.

---

## §4. What this receipt is NOT

- **Not a re-implementation review.** PR #4014's substrate decisions (routing through `std.node`, deleting three projection nodes, intentional evaluator-input hash shift) were settled at #4014 merge; this lane does not re-litigate them.
- **Not a TASKS.md amendment.** No predicate or operational text is altered.
- **Not a Wave 1 close ceremony.** Wave 1 still has remaining MW-D8 conditions (C4); when all five PROVEN this lane will author the separate Wave 1 close-receipt artifact per PR #4012 §4.

## §5. Related artifacts

- PR #4014 — implementation; merge commit carries the substrate.
- PR #4017 (`docs/planning/v4-mw-d8-wave1-exit-ledger-2026-05-30.md`) — live ledger; this receipt is cited from the C2 row update committed in the same PR as this file.
- PR #3949 §1 — closure invariant honored: executable receipt + falsification receipt.
- PR #3983 §7 MW-D8 — operator-ratified C2 disjunctive condition this receipt closes.
- `docs/planning/v4-sg7-ci-offset-complexity-worksheet-2026-05-30.md` — SG-7 worksheet (#3977); the modeling-DFS authority PR #4014 implements.
