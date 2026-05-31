# v4 rustc error catalog → class-fix table — 2026-05-31 (post-SG-1)

**Manager session:** `sharp-otter-407` (Close/Receipt lane).
**Authority:** PR #3938 §11.1 + §11.3 lane map; P3-B operator dispatch 2026-05-31 ("fresh measurement after systemic fix, not stale baseline reuse").
**Replaces (does NOT delete):** `docs/audit/v4-rustc-error-catalog-2026-05-29.md` as the **live** M1 ratchet meter. The 2026-05-29 catalog stays as the **pre-SG-1 baseline** for delta computation.
**Reference commit:** `origin/main` at **`78b9698ab`** — the SG-1 squash (#3956) is the tip at probe time, sitting on top of `17a9ac4c3` (post-#4075 burn-down, the prior W2.3 + W1.5 landings parent). v2-compiler binary and the M1 probe both measured against `78b9698ab` as `git rev-parse HEAD`.
**Probe:** `scripts/v4-m1-rust-emit-probe.sh` run by `sharp-otter-407` (2026-05-31 ~04:25Z) → `docs/audit/v4-rustc-error-catalog-2026-05-31.m1-probe-summary.txt` (committed in this PR).

---

## §1 Headline

| Metric | 2026-05-31 (post-SG-1) | 2026-05-29 (pre-SG-1) | Δ |
| ------ | ---------------------: | --------------------: | -:|
| v2 emit `compiled:` diagnostics | **0** | 24 | **−24 (SG-7 cleared)** |
| `.rs` files emitted | 335 | 295 | +40 (more substrate compiles to emit) |
| rustc `error[E####]` line count | **6991** | 7951 | **−960 (~12% reduction)** |
| Distinct emitted `.rs` files with errors | 298 / 335 | 262 / 294 | +36 |
| Top code (count) | **E0308 (2862)** | E0423 (2978) | **dominant code shifted SG-1 → SG-3-family** |
| E0423 (SG-1 signature failure mode) | **0** | 2978 | **−2978 (SG-1 partial dissolution; see §3)** |

**Plain-English summary.** SG-1 #3956 + SG-7 #4014 between them eliminated 3,002 errors (E0423 + v2 complexity diagnostics) — those classes are **closed at the dominant-failure-mode level**. Net rustc population dropped ~12%, less than the ~38% pure-E0423-removal would have predicted, because **~1,317 of the eliminated E0423s cascaded forward** into E0308 "expected `String`, found `Symbol`" — the SG-1 emitter started returning `Symbol` values but the generated function signatures still annotate `-> String`. The remainder of E0308 (~1,545 errors) and E0107 / E0282 (2,457 errors combined) constitute the new dominant classes.

This is exactly the "fresh measurement after systemic fix" the operator framework demands — the **dominant class shifted**, and the new classes need their own §10.0 dispatch routing.

---

## §2 Code histogram (post-SG-1, full src/v4 tree, 6991 errors)

| Code | Count | Share | Concept |
| ---- | ----:| -----:| ------- |
| `E0308` | 2862 | 41% | mismatched types (largely SG-1 follow-on + Rc/Box layering) |
| `E0107` | 1504 | 22% | missing generics (SG-2 family) |
| `E0282` | 953 | 14% | type annotations needed (SG-2 family) |
| `E0425` | 479 | 7% | cannot find type (SG-8 family + post-SG-1 type-lookups) |
| `E0277` | 330 | 5% | trait bound (SG-3 family) |
| `E0432` | 234 | 3% | unresolved import (SG-8) |
| `E0573` | 159 | 2% | expected type, found variant (SG-3) |
| `E0560` | 118 | 2% | struct field missing (SG-3) |
| `E0369` | 110 | 2% | binary op on Rc<T> (SG-3) |
| `E0433` | 83 | 1% | failed to resolve (SG-8) |
| `E0121` | 44 | 1% | placeholder `_` in item signature (SG-3) |
| `E0391` | 29 | <1% | cyclic dependency |
| `E0599` | 28 | <1% | no method found |
| other (12 codes) | 58 | <1% | long tail |
| **TOTAL** | **6991** | 100% | |

---

## §3 SG-1 partial dissolution — E0308 "expected `String`, found `Symbol`" cascade

SG-1 #3956 landed `TargetAtomRealization` with Rust Symbol/Bool/Char rows and a `translate` consumer. The probe shows **E0423 = 0** (was 2,978) — the structural "type alias used as constructor" failure mode is closed. However, the emitter now produces:

```rust
pub fn loop_bound_edge() -> String { Symbol("loop_bound_edge".to_string()) }
//                          ^^^^^^                                 // signature still String
//                                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^  // body returns Symbol
```

rustc reports this as E0308 ("expected `String`, found `Symbol`"). **1,317 of the 2,862 E0308 errors** (46%) carry exactly this `expected String, found Symbol` shape (verified by `grep -A 6 'error\[E0308\]' rustc.log | grep -oE "expected \\\`[^\\\`]+\\\`, found \\\`[^\\\`]+\\\`" | sort | uniq -c`).

**Missing modeled fact:** the realization-row output ship-type. SG-1's `TargetAtomRealization` row tells the emitter *how to construct* a Symbol value, but the **function-signature realization** for an atom-typed function body is not consistently re-projected from the same row. Net: SG-1 is a partial dissolution; the residue is its own dispatch sub-class.

**Disposition:** **SG-1-FOLLOWON** — same Modeling DFS authority (`proud-pike-680`); recommend **amend SG-1 worksheet** to add the function-signature realization step OR a small sibling worksheet. ~1,317 errors / 19% of population — single dispatchable receipt.

---

## §4 Two NEW dominant classes (operator P3-D elastic core)

Aside from SG-1-FOLLOWON, two newly-dominant classes surface from the type-mismatch tail.

### §4.1 SG-RC-LAYERING (NEW class)

| Mismatch shape | Count |
| -------------- | -----:|
| `expected \`Rc<Diagnostics>\`, found \`Diagnostics\`` | 277 |
| `expected \`Box<_>\`, found \`Rc<Node>\`` | 121 |
| `expected \`Node\`, found \`Rc<Node>\`` | 108 |
| `expected \`TestClaim\`, found \`Rc<TestClaim>\`` | 45 |
| `expected \`FreeMonoid<_>\`, found \`Rc<FreeMonoid<_>>\`` | 31 |
| `expected \`Rc<Diagnostics>\`, found \`Option<_>\`` | 29 |
| `expected \`Outcome<_>\`, found \`Rc<Outcome<_>>\`` | 20 |
| `expected \`Rc<Node>\`, found \`Box<Rc<Node>>\`` | 11 |
| `expected \`ModelCore\`, found \`Rc<ModelCore>\`` | 8 |
| `expected \`AlgebraInhabitanceDecl\`, found \`Rc<AlgebraInhabitanceDecl>\`` | 8 |
| other 1–7 count Rc/Box/raw bands | ~50 |
| **subtotal (Rc/Box layering at boundary)** | **~700** |

**Missing modeled fact:** at a function boundary (return type, argument site, struct-field site), when does a substrate value emit as `T` vs `Rc<T>` vs `Box<T>`? Today the emitter is inconsistent — some sites wrap, some don't, depending on context that the substrate doesn't constrain. The fact to model is **ownership-realization** per use-site, projected from a substrate row (analogous to SG-1's `TargetAtomRealization` for the atom case).

**Disposition:** **NEW §10.0 WORKSHEET REQUIRED** — `SG-RC-LAYERING`. Authority: Modeling DFS (`proud-pike-680`) primary; Target Realization secondary (Rust-side realization-row authoring). ~700 errors / 10% of population — single dispatchable receipt class.

### §4.2 SG-COLLECTION-PROJECTION (NEW class)

| Mismatch shape | Count |
| -------------- | -----:|
| `expected \`Vec<Rc<Edge>>\`, found \`FreeMonoid<_>\`` | 42 |
| `expected \`Vec<Rc<PrimitiveFactBundle>>\`, found \`FreeMonoid<_>\`` | 32 |
| `expected \`Vec<Rc<AlgebraInhabitanceDecl>>\`, found \`FreeMonoid<_>\`` | 22 |
| `expected \`Vec<Rc<FormalGrammarSymbol>>\`, found \`FreeMonoid<_>\`` | 14 |
| `expected \`Vec<Rc<Node>>\`, found \`FreeMonoid<_>\`` | 11 |
| `expected \`Vec<Rc<FormalProduction>>\`, found \`FreeMonoid<_>\`` | 10 |
| `expected \`Vec<Rc<AlgebraLawObligation>>\`, found \`FreeMonoid<_>\`` | 8 |
| other Vec-vs-FreeMonoid mismatches | ~30 |
| **subtotal** | **~170** |

**Missing modeled fact:** `FreeMonoid<T>` substrate is lowered to `Vec<Rc<T>>` in some contexts (consumer sites) but not others (constructor sites). The collection-realization projection is incomplete — the substrate row that should declare "FreeMonoid `T` projects to `Vec<Rc<T>>` at consumer boundary" is missing or partial.

**Disposition:** **EXTEND EXISTING SG-5/SG-6 worksheet OR new sub-worksheet** (`SG-COLLECTION-PROJECTION`). Authority: Modeling DFS primary (substrate-side); Target Realization secondary. ~170 errors / 2.4% of population. This class is small enough that proud-pike may decide it's a sub-row inside the existing SG-5 (Set-ord) or SG-6 (BoundedLattice) worksheet rather than a fresh §10.0 worksheet.

---

## §5 Class routing table (post-SG-1 elastic core)

Each remaining dominant class is mapped to either an **existing** worksheet/dispatch OR named as needing a **new** §10.0 worksheet. Number of classes mapped to a receipt-producing dispatch is the **P3-D elastic core count** the operator framework demands.

| Class | Pop. | Rustc codes | Dispatch routing | Worksheet status |
| ----- | ---:| ----------- | ---------------- | ---------------- |
| **SG-1-FOLLOWON** (return-type annotation for atom fns) | **1,317** | E0308 (subset) | Modeling DFS — **amend SG-1 worksheet** to add function-signature realization step | **AMEND existing** (proud-pike to ratify amendment vs sibling) |
| **SG-2** (generic-arity on modeled carriers) | **2,457** | E0107 + E0282 | Modeling DFS — existing worksheet (#3962 path per proud-pike) | **EXISTS — worker dispatch needed** |
| **SG-RC-LAYERING** (Rc/Box/raw at boundaries) | **~700** | E0308 (Rc-band subset) | Modeling DFS + Target Realization (Rust ownership-realization rows) | **NEW §10.0 WORKSHEET REQUIRED** |
| **SG-COLLECTION-PROJECTION** (FreeMonoid vs Vec) | **~170** | E0308 (FreeMonoid-band) | Modeling DFS + Target Realization | **EXTEND SG-5/SG-6** *or* new sub-worksheet (proud-pike's call) |
| **SG-8** (module graph + carrier re-exports) | **~796** | E0425 + E0432 + E0433 | Modeling DFS — existing routing | **EXISTS — worker dispatch needed** |
| **SG-3-CASCADE** (trait bounds + struct fields + binary ops, post-SG-1/2 cascade) | **~1,191** | E0277 + E0573 + E0560 + E0369 + E0121 | Modeling DFS — clears as cascade from SG-1-FOLLOWON / SG-2 land | **EXISTING — mop-up after primaries** |
| **Long tail** (cyclic deps, no method, etc.) | **~58** | E0391 / E0599 / + 12 other codes | Modeling DFS — case-by-case | **No worksheet — naturally bounded** |
| **SG-1** (Symbol-as-callable) | 0 | E0423 | CLOSED via #3956 | **CLOSED** |
| **SG-7** (v2 complexity recursion) | 0 | v2 diagnostic | CLOSED via #4014 (MW-D8 C2 PROVEN per PR #4050) | **CLOSED** |
| **SG-5** (Set non-Ord BTreeSet) | (unchanged — `compile_error!` stubs) | n/a | Modeling DFS — existing worksheet | **EXISTS** |
| **SG-6** (BoundedLattice partial instances) | (unchanged — `compile_error!` stubs) | n/a | Modeling DFS — existing worksheet | **EXISTS** |

**P3-D elastic core count (classes mapped to receipt-producing dispatch):**

- **CLOSED: 2** (SG-1, SG-7)
- **EXISTS — amendment / extension / worker-only:** 5 (SG-1-FOLLOWON amend, SG-2, SG-8, SG-3-CASCADE, SG-5/SG-6)
- **NEW WORKSHEET REQUIRED: 1** (SG-RC-LAYERING)
- **OPTIONAL NEW vs extend:** 1 (SG-COLLECTION-PROJECTION — proud-pike's call)
- **Bounded long tail (no worksheet):** 1 class (~58 errors across 12 codes)

**Total receipt-producing classes:** **8** active + 2 closed. The "elastic core" can dispatch worker briefs against all 8 today without authoring further classification work; the one mandatory new worksheet is SG-RC-LAYERING.

---

## §6 Cross-receipt notes (informational)

- **SG-1's partial dissolution is good news**, not a regression. Pre-SG-1, the cascade Behind E0423 was invisible; with E0423 = 0, the underlying SG-1-FOLLOWON shape (return-type-annotation mismatch) is now first-class diagnosable. The "12% reduction" headline understates the modeling progress: the dominant **failure mode** changed from "type alias used as callable" to "callable returns the right value but signature lies", which is one structural step closer to closure.
- **SG-RC-LAYERING is the highest-leverage NEW class**: at ~700 errors and a single missing modeled fact (per-use-site ownership realization), one §10.0 worksheet's PROVEN receipt will retire ~10% of the population.
- **MW-D8 ledger (PR #4017 + #4050)** stays unchanged by this catalog — MW-D8 is gate-condition tracking, not rustc-class tracking. C4 PROVEN per #4073 (per the burn-down log mentioned in #4077); C2 PROVEN per #4050.
- **PR #4060 P1 roster** also unchanged — the rustc-error classes are not enumerated in TASKS.md, so this catalog does not affect P1 row dispositions.

---

## §7 Repro

```bash
# 1. build v2-compiler (post-SG-1 main HEAD)
PATH=/opt/cargo/bin:$PATH CARGO_BUILD_JOBS=4 cargo build -p v2-compiler --release

# 2. run M1 probe
export V2_COMPILER=target/release/gunbc
export V4_M1_RUST_EMIT_OUT=/tmp/v4-rust-emit-p3b
export V4_M1_CARGO_CHECK_JOBS=4
bash scripts/v4-m1-rust-emit-probe.sh
#   → /tmp/v4-rust-emit-p3b.m1-probe-summary.txt (committed verbatim as docs/audit/v4-rustc-error-catalog-2026-05-31.m1-probe-summary.txt)
#   → /tmp/v4-rust-emit-p3b.rustc.log (full rustc output, 6991 error lines)
```

**E0308 sub-class verification** (used in §3 + §4):

```bash
grep -A 6 "error\[E0308\]" /tmp/v4-rust-emit-p3b.rustc.log \
  | grep -oE "expected \`[^\`]+\`, found \`[^\`]+\`" \
  | sort | uniq -c | sort -rn | head -20
```

---

## §8 What this catalog is NOT

- **Not a worker dispatch.** Worker briefs follow from proud-pike's §10.0 worksheet ratifications, not from this catalog. This catalog provides the classification + receipt counts; proud-pike-680 decides whether SG-RC-LAYERING gets its own worksheet vs an amendment, and whether SG-COLLECTION-PROJECTION extends SG-5/SG-6.
- **Not a P1 roster amendment.** Rustc error classes are not TASKS.md tasks; the P1 roster (#4060 + #4065) tracks T-* tasks, not SG-* error classes.
- **Not a MW-D8 ledger amendment.** Wave 1 exit conditions are independent of rustc population.
- **Not the SG-1 close receipt.** SG-1 #3956 already merged; the partial dissolution is an observation about the residual SG-1-FOLLOWON class, not a reopening of SG-1.

## §9 Related artifacts

- `docs/audit/v4-rustc-error-catalog-2026-05-29.md` — pre-SG-1 baseline (kept on main for delta computation).
- `docs/audit/v4-rustc-error-catalog-2026-05-31.m1-probe-summary.txt` — raw probe output (committed in this PR).
- PR #3956 — SG-1 TargetAtomRealization (the systemic fix this fresh measurement audits).
- PR #4050 — MW-D8 C2 falsification receipt (SG-7 closed).
- PR #4060 + #4065 — P1 roster + per-GAP routing (unaffected by this catalog).
- PR #3938 §10.0 — DFS Root-Cause Worksheet template (the shape SG-RC-LAYERING worksheet must follow).
- PR #3949 §1 — two-axis vocabulary (each remaining class is `ship_disposition: GAP` / `engineering_state` varying per receipt status).
