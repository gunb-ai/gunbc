# MW-D8 Wave 1 Exit-Condition Running Ledger — 2026-05-30

**Manager session:** `sharp-otter-407` (Close/Receipt lane).
**Authority:** PR #3983 §7 MW-D8 — Wave 1 exit requires **all five** conditions met. PM dispatched 2026-05-30 ~19:40Z.
**Vocabulary:** two-axis `ship_disposition` × `engineering_state` per PR #3949 §1 (Close/Receipt manager-pass receipt).
**Adjudication rule:** Close/Receipt lane adjudicates whether each cited evidence PR **actually closes** the named condition. A worksheet, scaffold, or substrate landing is NOT sufficient to flip a condition to `PROVEN` — see PR #3949 §1 closure invariant.
**Closes when:** all five rows reach `ship_disposition: PROVEN`. At that point this lane authors a separate Wave 1 receipt-of-complete artifact.

---

## §1. Conditions table (live)

Each row's `Condition` column quotes MW-D8 **verbatim** from PR #3983 `docs/planning/v4-merge-wave-and-next-waves-2026-05-30.md:186-196`. Where MW-D8 spells out an `OR`-arm (C2, C3, C5), the lane explicitly identifies which arm is satisfied — the row text is not rewritten to a narrower paraphrase.

| # | Condition (MW-D8 verbatim) | `ship_disposition` | `engineering_state` | Evidence | Last update | Revisit-by |
| - | -------------------------- | ------------------ | ------------------- | -------- | ----------- | ---------- |
| C1 | "Step 4 R1 produces an actual leaf-model verdict (`rust.dag` R1 → `rustc` → `Verdict<R1>`)." | `PROVEN` | n/a (closure reached) | #3972 merged 2026-05-30 18:24Z | 2026-05-30 19:55Z | n/a — closed |
| C2 | "SG-7 `ci.dag` recursion is dissolved OR replaced by a modeled authority (`ByteOffsetCacheDigestAuthority` + `byte_offset_cache_key` consumed)." | `PROVEN` (both arms hold) | n/a (closure reached) | #3977 worksheet + #4014 impl merged 2026-05-30 21:01:11Z; falsification receipt `docs/audit/v4-mw-d8-c2-falsification-receipt-2026-05-30.md` (this update) verifies both arms on `origin/main` | 2026-05-30 22:55Z | n/a — closed |
| C3 | "`Upsert<T>` is either landed as usable substrate primitive OR explicitly blocked with a Modeling DFS worksheet naming the parser/substrate gap." | `PROVEN` (via OR-arm: explicit block with worksheet) | `SUBSTRATE_PRESENT` (skeleton landed; not full usable substrate per #3981 self-assessment) | #3981 + #3989 merged + `docs/planning/v4-upsert-t-substrate-worksheet-2026-05-30.md` (Modeling DFS worksheet naming SG-2b parser/substrate gap) | 2026-05-30 19:55Z | 2026-05-31 12:00Z (revisit if SG-2b dissolution path changes) |
| C4 | "`ci_selection_receipt_shadow` exists and can be generated for at least one PR/change fixture (shadow mode, not active gating yet)." | `GAP` | `NO_ARTIFACT_FOUND` | none yet — `smart-stag-871` next-up post-#4014 (C2 prereq now closed) | 2026-05-30 22:55Z | 2026-06-01 12:00Z (24h now that C2 prereq is closed) |
| C5 | "R2a/R2b/R3-external/R3-internal claim authoring has ready-to-run OR explicitly-blocked status (each claim authored or its blocker named)." | `PROVEN` (mixed-arm: 3 ready-to-run + 1 explicitly-blocked-and-named) | n/a (closure reached) | #4000 merged 2026-05-30 19:55Z (`sharp-swift-715` via `quick-tern-735`) | 2026-05-30 20:25Z | n/a — closed |

**Headline:** **4 of 5** conditions `PROVEN` (C1; C2 via both arms post-#4014 + falsification receipt; C3 via OR-arm; C5 via mixed-arm). 1 remaining (C4). C4 is unblocked structurally now that C2 is closed.

---

## §2. Per-condition adjudication notes

### §2.1 C1 — Step 4 R1 leaf-model verdict (PROVEN)

**MW-D8 verbatim:** "Step 4 R1 produces an actual leaf-model verdict (`rust.dag` R1 → `rustc` → `Verdict<R1>`)."

**Evidence:** PR #3972 merged 2026-05-30 18:24Z produces the first R1 leaf-model verdict via the `rust.dag` R1 → `rustc` → `Verdict<R1>` chain. The MW-D8 phrasing "**actual** leaf-model verdict" rules out structural-only presence — the merged PR carries the executed verdict surface. `ship_disposition: PROVEN`. **Watch condition:** if a follow-on audit finds the verdict is structurally-only-present-but-not-executed, this row reopens.

### §2.2 C2 — SG-7 recursion dissolved AND replaced by modeled authority (PROVEN via both arms)

**MW-D8 verbatim:** "SG-7 `ci.dag` recursion is dissolved OR replaced by a modeled authority (`ByteOffsetCacheDigestAuthority` + `byte_offset_cache_key` consumed)."

**Both arms hold post-#4014.** The falsification receipt at `docs/audit/v4-mw-d8-c2-falsification-receipt-2026-05-30.md` (landed in this same PR) verifies the four probes on `origin/main`:

- §2.1 audit-by-grep on `src/` confirms `ci_int_offset_authority_projection_node`, `ci_byte_limb_projection_node`, and `ci_int_offset_authority_projection_bounded` are absent (arm 1 — dissolved).
- §2.2 confirms `byte_offset_cache_key_projection_node` lands in `src/v4/std/node.dag` as the modeled authority and is consumed at the `ci.dag` projection site (arm 2 — modeled authority + consumed).
- §2.3 confirms in-tree structural-impossibility comment authority on `ci_char_projection_node`.
- §2.4 confirms executable injective-witness probe `ci_char_projection_out_of_range_injective_witness` in `ci.dag` — runtime falsification gate.

Lane adjudication: closure invariant (PR #3949 §1) honored — executable receipt is §2.4's witness probe; falsification receipt is §2.1's audit-by-grep. Both on `main`. Row flips to `PROVEN`. Both arms holding strengthens the receipt: a future regression has to defeat both the dissolution audit and the modeled-authority routing to reopen.

### §2.3 C3 — `Upsert<T>` either landed-as-usable OR explicit-block-with-worksheet (PROVEN via OR-arm)

**MW-D8 verbatim:** "`Upsert<T>` is either landed as usable substrate primitive OR explicitly blocked with a Modeling DFS worksheet naming the parser/substrate gap."

**Which arm is satisfied:** the **OR-arm** (explicit block with Modeling DFS worksheet). PR #3981's own summary explicitly states it is **"not full usable substrate"** — `ResolveExpr<T>` brands `T` while still Node-backed; phase carriers and pattern bodies deferred. So the **first arm** (landed as usable substrate primitive) is **not** satisfied. The PR cites `docs/planning/v4-upsert-t-substrate-worksheet-2026-05-30.md` (§1.1) and routes the parser gap to SG-2 parser-staging dissolution per `_internal/ROADMAP_OPS.md`. That worksheet **is** the explicit block naming the parser/substrate gap MW-D8 requires for the OR-arm.

**Adjudication:** `ship_disposition: PROVEN` via OR-arm; `engineering_state: SUBSTRATE_PRESENT` (parseable skeleton on main, not full usable substrate). PR #3989 extends with `CiUpsertStep<T>` + `UpsertInputRef` substrate; it inherits the same "skeleton not full usable" status and the same OR-arm satisfaction.

**Closure invariant (PR #3949 §1) is respected:** the closure receipt for the OR-arm is the worksheet itself — it answers the exact MW-D8 probe ("if Upsert isn't usable yet, where's the substrate gap named?") and is checkable against the parser-staging dissolution path. This is NOT a substrate-only landing being elevated past the closure invariant; it is the OR-arm of an explicitly-disjunctive condition.

**Watch conditions for reopen:**
- If the SG-2b parser-staging dissolution path is abandoned without an alternative MW-D8-named-block landing, this row reopens to `GAP / NO_ARTIFACT_FOUND` for the OR-arm receipt.
- If `Upsert<T>` later lands as full usable substrate primitive (first-arm satisfaction), this row stays `PROVEN` but the §2.3 note updates to cite the first-arm receipt as the now-stronger evidence.
- Activation debt on the skeleton (no consumer wired) triggers the anti-shelfware policy under PR #3949 §4 separately; it does NOT reopen this MW-D8 row, because MW-D8 C3 is explicitly disjunctive and the OR-arm receipt remains intact.

### §2.4 C4 — `ci_selection_receipt_shadow` generatable for at least one fixture (GAP / NO_ARTIFACT_FOUND)

**MW-D8 verbatim:** "`ci_selection_receipt_shadow` exists and can be generated for at least one PR/change fixture (shadow mode, not active gating yet)."

**Evidence:** none currently. `smart-stag-871` queued to author post-SG-7 closure (depends on C2 flipping). Lane adjudication: row stays `GAP / NO_ARTIFACT_FOUND` until a generator artifact exists. When a PR lands claiming generatability, the lane adjudicates whether the generator actually runs against at least one PR/change fixture (engineering_state: `EXECUTION_NOT_WIRED` if claim-only; `PROVEN` only if a fixture-fired generation receipt accompanies it). The MW-D8 "shadow mode, not active gating yet" caveat means active gating is **not** required for `PROVEN` — generation against one fixture is the bar.

**Dependency note:** C4's revisit-by is set 72h out because C2 is its prerequisite. If C2 slips, C4's window extends pari-passu via this lane's per-PR revisit-by update — not a separate operator extension.

### §2.5 C5 — R2a/R2b/R3-external/R3-internal ready-to-run OR explicitly-blocked (PROVEN via mixed-arm)

**MW-D8 verbatim:** "R2a/R2b/R3-external/R3-internal claim authoring has ready-to-run OR explicitly-blocked status (each claim authored or its blocker named)."

**Evidence:** PR #4000 merged 2026-05-30 19:55Z. The four MW-D8 elements are each individually satisfied:

| Element | Arm | Receipt |
| ------- | --- | ------- |
| R2a (i32 algebra inhabitance — rustc accepts ordered ring ops) | **ready-to-run** | `src/v4/test/claim/language_model/rust_r2a.dag` + runner `scripts/v4-leaf-model-rust-r2a-verify.sh` + rustc test `src/v3/compiler/tests/boundary/v4_leaf_model_rust_r2_r3_external_rustc_test.rs` |
| R2b (i32 overflow runtime behavior — debug panic vs release wrap, ×4 mode rows) | **ready-to-run** | `src/v4/test/claim/language_model/rust_r2b.dag` + runner `scripts/v4-leaf-model-rust-r2b-verify.sh` |
| R3-external (Symbol projection to Rust) | **ready-to-run** | `src/v4/test/claim/language_model/rust_r3_external.dag` + runner `scripts/v4-leaf-model-rust-r3-external-verify.sh` |
| R3-internal (Symbol emit coupling) | **explicitly-blocked-and-named** | `claim_rust_leaf_r3_internal_symbol_coupling` authored at `src/v4/test/claim/language_model/rust.dag`; named blocker per inline comment: *"R3-internal — emit coupling receipt; exercisable post-SG-1 (§4)."* |

**Lane adjudication:** all four elements individually satisfy at least one MW-D8 arm; the row is PROVEN under the MW-D8 disjunction. Three ready-to-run + one explicitly-blocked-and-named is exactly the mixed-arm shape MW-D8 anticipates ("each claim authored **or** its blocker named"). `engineering_state: n/a` because every per-element receipt is on main; the row has no engineering remainder.

**Closure invariant (PR #3949 §1) respected:** every PROVEN per-element claim is an executable receipt with falsification (e.g., `falsification_case` rows on each `LeafModelClaim`) or, for R3-internal, an explicit named-blocker receipt. The MW-D8 OR-arm carries closure-equivalent weight to ready-to-run for this condition, by operator design.

**Watch conditions for reopen:**
- If SG-1 dissolution unwinds before R3-internal is exercisable, the named blocker still exists but the path forward becomes unclear; the lane re-adjudicates whether the named blocker still qualifies as MW-D8 "blocker named".
- If a per-element runner breaks against future main (regression), the affected element's ready-to-run claim no longer holds; lane reopens C5 at that point.

---

## §3. Update protocol

This lane updates the ledger on these triggers:

1. **In-flight PR lands** — re-adjudicate the affected row, update `ship_disposition` / `engineering_state` / `Last update` / `Revisit-by`.
2. **In-flight PR slips past its ETA** — row stays unchanged but `Last update` advances; if the slip crosses a `Revisit-by` deadline, this lane authors a single dashboard message to the owning manager naming the deadline crossing (not a separate PR).
3. **Closure invariant violation surfaced** — e.g., a row was marked `PROVEN` but a follow-on audit shows the receipt was scaffold-only; this lane reopens the row and amends the adjudication note.

Each update is a single docs commit on this branch (or a follow-on small PR if this PR is already merged). No code, no substrate, no `.dag`.

---

## §4. Wave 1 close ceremony

When all five rows reach `ship_disposition: PROVEN`:

1. This lane authors `docs/audit/v4-wave1-close-receipt-2026-MM-DD.md` — a separate artifact citing each row's evidence PR + the final adjudication.
2. Wave 1 close receipt grade per PR #3949 §2: `RECEIPT_CLOSED` (since by then each row's executable receipt exists and falsification was checked at adjudication time).
3. PR #3983 §7 MW-D8 is then formally met; PM owns the Wave 2 dispatch decision (PR #3983 §5 territory, out of scope here).

---

## §5. What this ledger is NOT

- **Not a dispatch instrument.** This lane does not author or re-route the in-flight worker PRs. C2's `smart-stag-871` impl, C4's queued shadow generator, C5's `sharp-swift-715` authoring all remain owned by their respective sibling managers.
- **Not Wave 2 planning.** PR #3983 §5 owns Wave 2 sequencing; this ledger closes at Wave 1 exit.
- **Not a TASKS.md amendment.** No predicate or operational-authority text is altered.

## §6. Related artifacts

- PR #3983 (`docs/planning/v4-merge-wave-and-next-waves-2026-05-30.md`) §7 MW-D8 — dispatch authority.
- PR #3949 (`docs/planning/v4-close-receipt-manager-pass-2026-05-30.md`) §1 — two-axis vocabulary; §2 — close grades; §4 — anti-shelfware deadlines.
- PR #3973 (`docs/planning/v4-done-predicate-tasks-mapping-2026-05-30.md`) — line-anchor mapping; cross-link for downstream consumers wanting to trace MW-D8 conditions into TASKS.md predicates.
- PR #3972, PR #3977, PR #3981, PR #3989, PR #4000 — Wave 1 worker PRs cited in §1.
