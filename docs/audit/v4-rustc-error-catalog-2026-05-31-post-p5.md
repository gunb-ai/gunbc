# v4 rustc error catalog → delta (post-#4115 / P5 bridge replacement) — 2026-05-31

**Manager session:** `sharp-otter-407` (Close/Receipt lane).
**Authority:** PR #3938 §11.1 + §11.3 lane map; PM dispatch 2026-05-31 ("Fresh M1 probe post-#4115 — substrate landings have made the #4086 residual stale").
**Live ratchet meter:** this catalog (replaces `docs/audit/v4-rustc-error-catalog-2026-05-31.md` from PR #4086 as the live meter; #4086 kept on main as the post-SG-1 baseline for delta).
**Reference commit:** `origin/main` at **`d015b76dd`** — verified via `git rev-parse origin/main` immediately before the probe ran. v2-compiler rebuilt against this tree.
**Probe:** `scripts/v4-m1-rust-emit-probe.sh` run by `sharp-otter-407` (2026-05-31 ~15:30Z) → `docs/audit/v4-rustc-error-catalog-2026-05-31-post-p5.m1-probe-summary.txt` (committed in this PR).
**Substrate landings since #4086 (`78b9698a`):** #4099 SG-1b worksheet, #4100 SG-RC worksheet, #4102 R3-internal, #4105 + #4107 substrate (compute_fabric + cache_interface), #4115 P5 bridge replacement.

---

## §1 Headline

| Metric | 2026-05-31 post-#4115 (this catalog) | 2026-05-31 post-SG-1 (PR #4086) | 2026-05-29 pre-SG-1 (baseline) | Δ vs #4086 | Δ vs baseline |
| ------ | -----------------------------------:| -------------------------------:| ------------------------------:| ----------:| -------------:|
| v2 emit diagnostics | **0** | 0 | 24 | 0 | −24 |
| `.rs` files emitted | 345 | 335 | 295 | +10 | +50 |
| rustc `error[E####]` lines | **7,175** | 6,991 | 7,951 | **+184 (+2.6%)** | **−776 (−9.8%)** |
| Distinct emitted `.rs` files with errors | 305 | 298 | 262 | +7 | +43 |
| Top code (count) | `E0308` (2,905) | `E0308` (2,862) | `E0423` (2,978) | E0308 still dominant | dominant code shifted SG-1 → SG-3-family |
| `E0423` (SG-1 closure check) | **0** | 0 | 2,978 | 0 (SG-1 stays closed) | −2,978 |

**Plain-English summary.** Six substrate PRs landed between the #4086 baseline and this probe (#4099, #4100, #4102, #4105, #4107, #4115). Substrate expansion grew the emit surface from 335 → 345 `.rs` files; rustc population grew **+184 errors** (+2.6%). That is the expected "more substrate → proportionally more emit surface to fix" pattern. SG-1 stays closed (E0423 = 0). SG-7 stays closed (v2 emit diagnostics = 0). **No new SG-class signatures emerged** — all 184 new errors land into existing classes (mostly SG-2 generic-arity from the new compute_fabric / cache_interface carriers, plus a small SG-RC-LAYERING tick on `TestClaim`).

The P5 bridge replacement (#4115) does not affect the M1 rustc path directly (M1 measures emit-then-rustc; P5 bridge work touches CI control-plane). The receipt that #4115 does not regress v2 emit is: **v2 emit diagnostics = 0** in this probe, identical to #4086.

---

## §2 Code histogram (post-#4115, 7,175 errors)

| Code | This probe | Prior (#4086) | Δ | Concept |
| ---- | ---------:| -------------:| -:| ------- |
| `E0308` | 2,905 | 2,862 | **+43** | mismatched types (SG-1-FOLLOWON + SG-RC-LAYERING) |
| `E0107` | 1,629 | 1,504 | **+125** | missing generics (SG-2 family) — biggest single delta |
| `E0282` | 957 | 953 | +4 | type annotations needed (SG-2 family) |
| `E0425` | 485 | 479 | +6 | cannot find type (SG-8) |
| `E0277` | 330 | 330 | 0 | trait bound (SG-3) |
| `E0432` | 238 | 234 | +4 | unresolved import (SG-8) |
| `E0573` | 159 | 159 | 0 | expected type, found variant (SG-3) |
| `E0560` | 122 | 118 | +4 | struct field missing (SG-3) |
| `E0369` | 110 | 110 | 0 | binary op on Rc<T> (SG-3) |
| `E0433` | 81 | 83 | −2 | failed to resolve (SG-8) |
| `E0121` | 44 | 44 | 0 | placeholder `_` in signature (SG-3) |
| `E0391` | 29 | 29 | 0 | cyclic dependency (long tail) |
| `E0599` | 28 | 28 | 0 | no method found (long tail) |
| other (12 codes) | 58 | 58 | 0 | long tail unchanged |
| **TOTAL** | **7,175** | **6,991** | **+184** | |

**Net delta interpretation.** The +184 line population growth is concentrated in **E0107 (+125)** — new generic carriers landed by #4105 + #4107 (`compute_fabric`, `cache_interface`) require type-argument projection at their emit sites, and SG-2 worksheet dispatch hasn't reached those carriers yet. The remaining +59 spreads across E0308 / E0282 / E0425 / E0432 / E0560 with no class jumping in shape. E0277 / E0573 / E0369 / E0121 / E0391 / E0599 are pinned (zero Δ) — those are the genuinely-stable mop-up classes the #4086 catalog called out.

---

## §3 SG-1-FOLLOWON delta (`expected String, found Symbol`)

| Probe | "expected String, found Symbol" E0308 count | E0308 total | Share of E0308 |
| ----- | -------------------------------------------:| -----------:| --------------:|
| #4086 (post-SG-1) | 1,317 | 2,862 | 46.0% |
| **This (post-#4115)** | **1,330** | **2,905** | **45.8%** |
| Δ | **+13** | +43 | −0.2pp |

Same conceptual shape as #4086 §3 — atom-typed function bodies returning a `Symbol` value where the signature still annotates `-> String`. The +13 reflects a handful of new atom-typed functions in the recent substrate landings (no shape change). SG-1b worksheet (#4099, per `proud-pike-680` routing receipt msg_6db2dc9e) is the operative dispatch surface; once SG-1b implementation receipts land via TR lane (`keen-heron-687` per #3938 §11.3), this row collapses.

---

## §4 SG-RC-LAYERING delta (Rc / Box / raw boundary)

| Mismatch shape | This probe | Prior (#4086) | Δ |
| -------------- | ---------:| -------------:| -:|
| `expected \`Rc<Diagnostics>\`, found \`Diagnostics\`` | 279 | 277 | +2 |
| `expected \`Box<_>\`, found \`Rc<Node>\`` | 122 | 121 | +1 |
| `expected \`Node\`, found \`Rc<Node>\`` | 108 | 108 | 0 |
| `expected \`TestClaim\`, found \`Rc<TestClaim>\`` | **69** | 45 | **+24** |
| `expected \`FreeMonoid<_>\`, found \`Rc<FreeMonoid<_>>\`` | 31 | 31 | 0 |
| `expected \`Rc<Diagnostics>\`, found \`Option<_>\`` | 29 | 29 | 0 |
| `expected \`Outcome<_>\`, found \`Rc<Outcome<_>>\`` | 20 | 20 | 0 |
| `expected \`Rc<Node>\`, found \`Box<Rc<Node>>\`` | 11 | 11 | 0 |
| other 1–10 count Rc/Box/raw bands | ~110 | ~57 | ~+53 (mostly new substrate boundaries) |
| **subtotal** | **~780** | **~700** | **+80** |

The `TestClaim` row's +24 reflects the substrate landings touching `TestClaim` consumer surfaces. Otherwise the shape is pinned — SG-RC-LAYERING worksheet (#4100, per `proud-pike-680` routing) is the operative dispatch surface; this delta does not change the worksheet's authority claim.

---

## §5 SG-COLLECTION-PROJECTION delta (`FreeMonoid<T>` vs `Vec<Rc<T>>`)

| Mismatch shape | This probe | Prior (#4086) | Δ |
| -------------- | ---------:| -------------:| -:|
| `expected \`Vec<Rc<Edge>>\`, found \`FreeMonoid<_>\`` | 42 | 42 | 0 |
| `expected \`Vec<Rc<PrimitiveFactBundle>>\`, found \`FreeMonoid<_>\`` | 32 | 32 | 0 |
| `expected \`Vec<Rc<AlgebraInhabitanceDecl>>\`, found \`FreeMonoid<_>\`` | 22 | 22 | 0 |
| `expected \`Vec<Rc<FormalGrammarSymbol>>\`, found \`FreeMonoid<_>\`` | 14 | 14 | 0 |
| `expected \`Vec<Rc<Node>>\`, found \`FreeMonoid<_>\`` | 11 | 11 | 0 |
| `expected \`Vec<Rc<FormalProduction>>\`, found \`FreeMonoid<_>\`` | 10 | 10 | 0 |
| `expected \`Vec<Rc<AlgebraLawObligation>>\`, found \`FreeMonoid<_>\`` | 8 | 8 | 0 |
| other Vec-vs-FreeMonoid bands | ~30 | ~30 | 0 |
| **subtotal** | **~170** | **~170** | **0** |

**Completely pinned.** The new substrate landings (#4105 + #4107) did not introduce new FreeMonoid-vs-Vec mismatches because their carriers do not collection-project. This class can stay routed to the existing SG-5/SG-6 worksheet amendment per `proud-pike-680`'s 2026-05-31 04:51Z routing receipt (msg_6db2dc9e), with no urgency change.

---

## §6 Class routing table (post-#4115 — no new classes required)

Reuses the §5 elastic-core table from PR #4086. **No new §10.0 worksheets needed**; the existing routing absorbs the +184 delta entirely.

| Class | This probe | Prior (#4086) | Δ | Worksheet status (unchanged from #4086 + `proud-pike-680` routing) |
| ----- | ---------:| -------------:| -:| ------------------------------------------------------------------ |
| **SG-1** (E0423) | 0 | 0 | 0 | **CLOSED** via #3956 (stays closed across all substrate landings) |
| **SG-7** (v2 complexity) | 0 | 0 | 0 | **CLOSED** via #4014 (PR #4050) |
| **SG-1-FOLLOWON** (expected-String-found-Symbol) | 1,330 | 1,317 | +13 | **SG-1b WORKSHEET LANDED** (#4099 per proud-pike); TR lane worker dispatch is the next-step receipt |
| **SG-2** (E0107 + E0282) | 2,586 | 2,457 | **+129** | **EXISTS — worker dispatch needed** (worksheet path #3962 per proud-pike); the +129 is concentrated on new substrate carriers (compute_fabric / cache_interface), confirming SG-2's modeled-fact still holds |
| **SG-RC-LAYERING** (Rc/Box/raw at boundaries) | ~780 | ~700 | +80 | **WORKSHEET LANDED** (#4100 per proud-pike) |
| **SG-COLLECTION-PROJECTION** (FreeMonoid vs Vec) | ~170 | ~170 | 0 | **EXTEND SG-5/SG-6** (routing per proud-pike); no urgency change |
| **SG-8** (E0425 + E0432 + E0433) | 804 | 796 | +8 | **EXISTS — worker dispatch needed** |
| **SG-3-CASCADE** (E0277 + E0573 + E0560 + E0369 + E0121 + **unclassified-E0308 mop-up**) | **~1,390** | **~1,436** (corrected vs #4086's stated 1,191; recomputed under this bucket's true definition) | **−46** (SG-RC-LAYERING absorbed +80 of what was previously E0308 mop-up; net cascade-mop-up shrinks even though named SG-3 codes are flat) | **EXISTING — mop-up after primaries** |
| **Long tail** (E0391 29 + E0599 28 + other 12 codes 58) | **115** | 115 (same shape, was mis-stated as 58 in §6 mid-edit) | 0 | Naturally bounded, no worksheet |

**P3-D elastic core count (unchanged from #4086):** **8 active receipt-producing classes** + 2 closed. The +184 delta does not change the number of dispatchable classes.

**Arithmetic reconciliation (response to openai-pro review of `a2d0c71d`):**

| Bucket | Count | Code coverage |
| ------ | -----:| ------------- |
| SG-1-FOLLOWON | 1,330 | E0308 subset (`expected String, found Symbol`) |
| SG-2 | 2,586 | E0107 (1,629) + E0282 (957) |
| SG-RC-LAYERING | ~780 | E0308 subset (Rc/Box/raw boundary mismatches) |
| SG-COLLECTION-PROJECTION | ~170 | E0308 subset (FreeMonoid vs Vec) |
| SG-8 | 804 | E0425 (485) + E0432 (238) + E0433 (81) |
| SG-3-CASCADE | ~1,390 | E0277 (330) + E0573 (159) + E0560 (122) + E0369 (110) + E0121 (44) + remaining-E0308 (~625) |
| Long tail | 115 | E0391 (29) + E0599 (28) + 12 other codes (58) |
| **Sum** | **~7,175** | reconciles to the §1 headline within ±1 rounding band |

*(Spot-correction of two arithmetic slips: (a) #4086's SG-3-CASCADE row's `~1,191` figure conflated the SG-3 grand-total `1,221` with the E0277+E0573+E0560+E0369+E0121 sum `761` — the live live figure for this row is the E0277..E0121 sum **plus the unclassified-E0308 mop-up**, since by definition the cascade soaks up E0308 not picked up by SG-1-FOLLOWON / SG-RC-LAYERING / SG-COLLECTION-PROJECTION. (b) The mid-draft §6 long-tail row listed `58` (the "other 12 codes" sub-bucket only) instead of the full long-tail `115`. Both fixed here.)*

**Probe-summary footnote.** The raw probe summary's top-25 histogram sums to 7,174 vs §1's 7,175 — a single error in a code outside the top-25 (the probe script displays only the top-25 codes; the rustc log carries the full population). The catalog uses the full population (7,175) consistently.

---

## §7 What the substrate landings DID and DID NOT change

**Substrate landings retained the closure invariants of the prior catalog:**

- **SG-1 stays closed.** E0423 = 0 across all five new substrate PRs.
- **SG-7 stays closed.** v2 emit diagnostics = 0 — confirms #4115 P5 bridge replacement work did not regress v2 emit (the operative receipt for the SG-7 / resolve-posture-bridge dissolution scenario).
- **#4099 SG-1b worksheet** is the dispatch surface for the +13 SG-1-FOLLOWON delta; no new worksheet authored.
- **#4100 SG-RC-LAYERING worksheet** is the dispatch surface for the +80 SG-RC delta; no new worksheet authored.
- **#4102 R3-internal substrate** contributed a handful of new emit sites; all errors land in existing classes.

**Substrate landings did NOT introduce:**

- Any new dominant class signature (the post-SG-1 elastic core remains 8 classes).
- Any per-class arithmetic flip (proportional growth only, no class jumping rank order).
- Any P5-bridge regression (v2 emit diagnostics stay at 0).

---

## §8 What this catalog is NOT

- **Not a worker dispatch.** Worker briefs against SG-1b / SG-RC-LAYERING / SG-2 / SG-8 / SG-COLLECTION-PROJECTION are `proud-pike-680`'s routing per §6; this catalog provides the receipt-producing residual count, not the dispatch.
- **Not a SG-1 / SG-7 reopening.** Both stay closed at the rustc-population check.
- **Not a P5 bridge receipt.** PR #4097 (P2-B M2 probe) is the P5/resolve-posture-bridge safety-net receipt; the relationship to this catalog is only that **v2 emit diagnostics = 0** here, identical to that probe's bootstrap-path receipt.
- **Not a P1 / MW-D8 amendment.** Both ledgers track different surfaces; no row in either flips on the basis of this delta.

## §9 Related artifacts

- `docs/audit/v4-rustc-error-catalog-2026-05-31.md` (PR #4086) — post-SG-1 baseline for the §1 / §2 / §3 / §4 / §5 deltas.
- `docs/audit/v4-rustc-error-catalog-2026-05-31-post-p5.m1-probe-summary.txt` (this PR) — raw probe output verbatim.
- PR #3956 — SG-1 systemic fix (closure-check across all post-SG-1 catalogs).
- PR #4099 / #4100 — SG-1b / SG-RC-LAYERING worksheets (proud-pike routing).
- PR #4102 — R3-internal substrate.
- PR #4105 / #4107 — compute_fabric + cache_interface substrate (the +125 E0107 delta source).
- PR #4115 — P5 bridge replacement (the receipt that v2 emit stays clean here is its implicit non-regression check).
- PR #4097 (open) — P2-B M2 probe (P5/resolve-posture-bridge bootstrap-path safety-net receipt).
- `proud-pike-680` routing receipt msg_6db2dc9e — the canonical routing source for SG-1b / SG-RC-LAYERING / SG-COLLECTION-PROJECTION dispatch shape.
