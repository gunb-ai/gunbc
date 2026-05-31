# v4-done Predicate ↔ `TASKS.md` Line-Anchor Mapping — 2026-05-30

**Manager session:** `sharp-otter-407` (Close/Receipt lane).
**Authority:** PR #3938 §11.3 (Close/Receipt owns ledger coherence) + PR #3948 (Self-host/Release v4-done predicate tracker, merged) — this doc is the mechanical line-anchor mapping the two artifacts agreed to land separately under Close/Receipt during the cross-link coordination 2026-05-30.
**Sibling owner:** `nimble-crane-490` (Self-host/Release) — owns predicate disposition + sub-tasks; this doc owns line-anchor coherence and `:819` Close-status numbering.
**Does NOT amend:** `src/v4/TASKS.md`. The six predicate bullets at `:806-817` are operational authority untouched by this doc. PR #3938 §8.D4 forbids predicate relaxation by manager pass; that bar is respected.

---

## §1. Predicate ↔ line-anchor table

| # | Predicate (verbatim short form) | TASKS.md anchor | #3948 tracker row | Close ledger (PR #3949) section |
| - | ------------------------------- | --------------- | ------------------ | ------------------------------- |
| P1 | Every other scheduled task complete (whole plan minus T-15) | `src/v4/TASKS.md:806-812` | `docs/planning/v4-done-predicate-tracker-2026-05-30.md:18` | `docs/audit/v4-close-ledger-2026-05-30.md` §7 (Cross-doc ledger coherence) |
| P2 | v4 compiles `src/v4/compiler/*.dag` end-to-end | `src/v4/TASKS.md:813` | `...tracker...:19` | `...ledger...` §3.1 (omni-emission entry) |
| P3 | v4 emits Rust source that compiles to a binary | `src/v4/TASKS.md:814` | `...tracker...:20` | `...ledger...` §3.1 (omni-emission) |
| P4 | Binary on `src/v4/compiler/*.dag` produces bit-identical output | `src/v4/TASKS.md:815` | `...tracker...:21` | `...ledger...` §4.2 (self-host fixed point) |
| P5 | TestClaim suite passes | `src/v4/TASKS.md:816` | `...tracker...:22` | `...ledger...` §3.3 (tests-as-data) |
| P6 | Hand-authored Rust not editable authority (reproduction-proven) | `src/v4/TASKS.md:817` | `...tracker...:23` | `...ledger...` §2.1 (Pure Bootstrap) + §4.2 |

**P1 anchor is a range, not a single line**, because the "every other scheduled task" predicate carries an in-line caveat that explicitly forbids enumeration: *"This is deliberately not an enumeration: an explicit list goes stale … The close gate is the whole plan minus T-15, resolved against the plan as it stands at close time — never a hardcoded list or count that can omit in-scope work."* (`:806-812`) Any future ledger consumer must read the full range to honor the no-enumeration discipline.

---

## §2. `:819` Close-status numbering discrepancies (recommendation to Self-host/Release lane)

The Close-status paragraph at `src/v4/TASKS.md:819` predates this two-axis-vocabulary / six-predicate reconciliation work. Two observed issues:

### §2.1 P6 absent from the status line

The bullets at `:806-817` enumerate **six** predicates. The Close-status paragraph at `:819` reports disposition for "1–2 CLOSABLE" / "3 PARTIAL" / "4 PARTIAL" / "5 PARTIAL" — no row for **predicate 6** (hand-Rust not editable authority, reproduction-proven). This is the most consequential predicate per PR #3938 §8.D4 PM-recommendation Option A. Silent absence is read by downstream consumers (including this ledger lane in earlier drafts) as if the six-predicate count were five.

### §2.2 "P5" name collision

The same Close-status paragraph uses "P5" in two distinct senses:
- *predicate 5* — "TestClaim suite passes" (`:816`)
- *the P5 bridge* — the legacy `scripts/v4-bootstrap-resolve-posture-gate.sh` + `ci.yml:249` bridge whose removal is gated by INVARIANTS A3/P5 (a separate concept; "P5" here is the INVARIANTS clause label, not a v4-done predicate)

Both appear in the same paragraph without disambiguation. The collision is the kind of parallel-authority-of-the-same-symbol failure mode PR #3938 §10.0 flags as a worksheet trigger.

### §2.3 Recommended rewrite shape (not applied here)

The Close/Receipt lane recommends, but does not apply, the following shape for the next Self-host/Release status refresh (Self-host/Release owns the status content; Close/Receipt is the coherence consumer):

```
**Close-status (YYYY-MM-DD, `main@<sha>`):**
- P1 (whole plan minus T-15) — <disposition>
- P2 (v4 compiles corpus end-to-end) — <disposition>
- P3 (v4 emits Rust → binary) — <disposition>
- P4 (bit-identical self-output) — <disposition>
- P5 (TestClaim suite passes) — <disposition>
- P6 (hand-Rust not editable authority, reproduction-proven) — <disposition>

INVARIANTS A3/P5 bridge removal status: <separate line item>
```

Six explicit rows × predicate-numbering matches the bullets above × INVARIANTS-P5 bridge is broken out as a distinct status line so the name collision is impossible.

**Why a recommendation and not an edit:** the current `:819` text includes specific PR references and dispatch states (#3791, #3787, #3752, #3794, #3796, #3803) and a `main@678bb8bbd` anchor that are Self-host/Release-lane authoritative content. This lane does not own status reconnaissance; rewriting the prose would either duplicate stale content or require fresh status data outside this lane's authority. Self-host/Release rewrites on next status refresh; this doc's mapping table is the structural surface that rewrite should land against.

---

## §3. Cross-link confirmation with PR #3948

The Close/Receipt ledger (`docs/audit/v4-close-ledger-2026-05-30.md`) and the Self-host/Release predicate tracker (`docs/planning/v4-done-predicate-tracker-2026-05-30.md`) agree on **5 of 6** predicate → owner_manager pairings:

- P2/P3 (emit) — Target Realization + Compiler Spine ✓
- P4 (fixpoint) — Self-host/Release + Compiler Spine ✓
- P5 (TestClaim) — Runtime/TestClaim + Compiler Spine ✓
- P1 (whole plan) — Close/Receipt + ("all lanes" in #3948; ledger §7 secondary = Self-host/Release) — coherent (#3948 enumerates "all lanes"; ledger picks the most-acute single secondary)

The one mild divergence (P6 secondary = Modeling DFS in ledger §2.1 vs Close/Receipt in #3948) was raised with `nimble-crane-490`; their response confirms both slices are defensible (Modeling DFS for the substrate/hand-Rust-shape side; Close/Receipt for the reproduction-ledger side). The next ledger refresh may fold Close/Receipt in as an explicit dual-secondary at §2.1; this is optional and non-blocking.

---

## §4. What this doc is NOT

- **Not a TASKS.md amendment.** The six predicates at `:806-817` are unchanged. PR #3938 §8.D4 bars predicate relaxation by manager pass.
- **Not a status refresh.** Current disposition of each predicate lives in `docs/planning/v4-done-predicate-tracker-2026-05-30.md` (Self-host/Release lane). This doc adds line-anchor coherence and recommends a `:819` rewrite shape; it does not restate per-predicate state.
- **Not a worker dispatch.** No SG class, no fixture, no rung gate is touched here.

## §5. Related artifacts

- `src/v4/TASKS.md:805-817` — v4-done definition (operational authority).
- `src/v4/TASKS.md:819` — Close-status paragraph (Self-host/Release authoritative; recommended rewrite shape in §2.3 above).
- `docs/planning/v4-done-predicate-tracker-2026-05-30.md` — Self-host/Release predicate tracker (PR #3948, merged).
- `docs/planning/v4-close-receipt-manager-pass-2026-05-30.md` — Close/Receipt manager-pass receipt (PR #3949, merged): vocabulary, close grades, anti-shelfware policy.
- `docs/audit/v4-close-ledger-2026-05-30.md` — 346-probe close ledger (PR #3949, merged).
- `docs/planning/v4-correctness-ladder-2026-05-30.md` — PR #3938 (merged): lane architecture, §10.0 vocabulary origin, §11.3 lane-to-section map.
