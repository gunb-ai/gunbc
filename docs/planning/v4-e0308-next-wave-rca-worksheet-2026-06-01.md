# v4 E0308 Next-Wave Stratified RCA Worksheet — 2026-06-01

> **Status:** DRAFT RCA WORKSHEET — routes implementation-ready subfamilies; does **not** authorize a broad E0308 fix lane.
> **Worker session:** `keen-owl-601` (`adhoc-219fc359-11b`).
> **Supersedes routing detail in:** `docs/planning/v4-e0308-stratified-rca-worksheet-2026-06-01.md` (wave-1 pair table at #4140 measurement); this document is the **next-wave** stratification at current `origin/main` HEAD.
> **Ratchet authority (unchanged meter):** PR #4140 / `docs/audit/v4-rustc-error-catalog-2026-06-01-post-jun1-cascade.md` — 7,724 total rustc lines, **2,953** `E0308`.
> **Fresh remeasure (this session):** `17a539cc57` — see §1; pair histogram is authoritative for **dispatch shape**, not for replacing the #4140 headline total until a clean v2-emit remeasure is ratcheted.

---

## §1 Measurement receipt

| Metric | #4140 (`483d82a78`) | Fresh probe (`17a539cc57`) | Notes |
| ------ | --------------------:| ---------------------------:| ----- |
| v2 emit diagnostics | **0** | **104** | All 104 are `complexity: same-argument recursion in source_atom_string_projection_from_node` (SG-1b / text-projection spine); emit still completes. |
| Files emitted | 351 | 358 | +7 |
| rustc `error[E####]` lines | 7,724 | **10,969** | +3,245 — **do not treat as ratchet until v2 emit is clean again** |
| `E0308` lines | 2,953 | **7,761** | +4,808 parsed pairs below explain shape |
| `E0107` | 1,654 | 56 | collapsed — many arity failures now surface as `E0308` under substrate `String` alias |
| `E0282` | 1,007 | 524 | same aliasing effect |

**Probe commands (this session):**

```bash
RUSTC_WRAPPER= CARGO_BUILD_JOBS=4 /opt/cargo/bin/cargo build -p v2-compiler --release

RUSTC_WRAPPER= V2_COMPILER=target/release/gunbc \
  V4_M1_RUST_EMIT_OUT=/tmp/v4-rust-emit-keen-owl-601 \
  V4_M1_CARGO_CHECK_JOBS=4 V4_M1_USE_CTRL_BUILD=0 \
  bash scripts/v4-m1-rust-emit-probe.sh
```

**Key structural discovery.** Emitted Rust defines `pub type String = Rc<FreeMonoid<Rc<Nat>>>;` in `v4_std_text.rs`. Wave-1 counted `String => Symbol` (1,344). At HEAD, the same failure often appears as `Rc<FreeMonoid<Rc<Nat>>> => Symbol` (2,073) because the alias is expanded in diagnostics. **Do not open a new “FreeMonoid/Nat” SG class** — this is **text-carrier realization** composed with **SG-1b atom value form**.

---

## §2 Pair histogram (parsed `expected \`...\`, found \`...\``)

**Parsed:** 7,738 of 7,761 `E0308` lines (23 lines use non-pair note shapes; hold in **W1-G** until remeasure).

| Pair | Count | Next-wave route |
| ---- | ----:| --------------- |
| `Rc<FreeMonoid<Rc<Nat>>>` => `Symbol` | 2,073 | **W1-A1** |
| `String` => `Symbol` | 1,359 | **W1-A1** (alias display of same row) |
| `Rc<FreeMonoid<Rc<Nat>>>` => `String` | 1,111 | **W1-A2** |
| `Rc<Diagnostics>` => `Diagnostics` | 294 | **W1-B** |
| `String` => `Rc<FreeMonoid<Rc<Nat>>>` | 235 | **W1-A2** (inverse site) |
| `Box<_>` => `Rc<Node>` | 135 | **W1-B** |
| `Node` => `Rc<Node>` | 114 | **W1-B** |
| `Box<Rc<Node>>` => `Rc<Node>` | 100 | **W1-B** |
| `Rc<Node>` => `Box<Rc<Node>>` | 90 | **W1-B** |
| `TestClaim` => `Rc<TestClaim>` | 70 | **W1-B** |
| `Rc<FreeMonoid<Rc<Edge>>>` <=> `Rc<Vec<Rc<Edge>>>` (both directions) | 107 | **W1-C** |
| `Vec<Rc<Edge>>` => `FreeMonoid<_>` | 40 | **W1-C** |
| `Vec<Rc<PrimitiveFactBundle>>` => `FreeMonoid<_>` | 32 | **W1-C** |
| `Rc<Diagnostics>` => `Option<_>` | 32 | **W1-D** |
| `Rc<HashMap<String, Rc<Node>>>` => `Rc<HashMap<Rc<FreeMonoid<...>>, ...>>` | 41 | **W1-E** |
| Remaining pairs (≤30 each) | 558 | **W1-G** |

### Route buckets (implementation fanout)

| Route | Count | % of parsed |
| ----- | ----:| ----------:|
| **W1-A** Text-carrier × atom/host value | **4,778** | 62% |
| **W1-B** SG-RC ownership / Box layer | **1,669** | 22% |
| **W1-C** SG-COLLECTION-PROJECTION | **462** | 6% |
| **W1-D** Diagnostic / Option result shape | **90** | 1% |
| **W1-E** Map key/value shell | **60** | 1% |
| **W1-G** Tail / unparsed | **581** | 8% |

Top file surfaces for **W1-A1** `… => Symbol`: `v4_extdeps_languages_{rust,swift,java,wasm,typescript,kotlin}.rs`, `v4_std_target_model.rs`, `v4_workflow_ci.rs` — all fn-boundary atom/tool-id returns, not nat-semiring runtime.

---

## §3 Implementation-ready subfamilies

Mechanical rule (unchanged): **no worker title may say “fix E0308.”** Each row below is a separate dispatch with its own single-authority fact and falsification probe.

### W1-A — Text-carrier × value-form coupling (P0, ~4.8k)

**Representative:**

```rust
// v4_std_text.rs
pub type String = Rc<FreeMonoid<Rc<Nat>>>;

// emitted extdeps (example)
pub fn pyright_tool_id() -> String { Symbol("pyright_tool_id".to_string()) }
//                              ^^^^^^ substrate text   ^^^^^^ SG-1 atom value
```

**Split (do not merge):**

| Slice | Count | Single-authority fact | Existing / new worksheet |
| ----- | ----:| --------------------- | ------------------------ |
| **W1-A1** Return/substrate `String`, value `Symbol` | ~3,434 | `TargetFunctionSignatureRealization` + `TargetAtomRealization` (SG-1b must track value form) | `v4-sg-1b-function-signature-realization-worksheet-2026-05-30.md` (**APPROVED** — implement) |
| **W1-A2** Return/substrate `String`, value host `std::string::String` / `.to_string()` | ~1,346 | **NEW:** `TargetTextCarrierRealization` (provisional name) — how `v4.std.text.String` (`FreeMonoid<Rc<Nat>>`) lowers to Rust `String` for **fixture literals and foreign host strings** at compile-time constant sites | **READY-FOR-WORKSHEET-AUTHOR** — Collection/Algebra or Target Realization manager; Arbiter must ratify before impl |

**Why not a broad patch:** Patching emit to wrap `Symbol` in `FreeMonoid` or to call `.to_string()` at each site creates a second authority beside SG-1b and text-carrier rows (INVARIANTS P2).

**Acceptance (A1):** SG-1b falsification probes — signature type and realized value form change together; no `String` return with `Symbol` body without a realization row.

**Acceptance (A2):** New falsification probe: fixture field typed `String` in `.dag` with a string literal compiles without `Rc<FreeMonoid<Rc<Nat>>>` vs `std::string::String` mismatch; row change only.

**Dispatch:** **A1** may proceed on approved SG-1b worksheet. **A2** blocked on new worksheet §8 — do not implement host-string shims in `06_translate`.

---

### W1-B — Per-use-site ownership layering (P0, ~1.7k)

**Representative:** `expected Rc<Diagnostics>, found Diagnostics`; `expected Node, found Rc<Node>`; `Box<Rc<Node>>` vs `Rc<Node>`.

| Slice | Count | Authority |
| ----- | ----:| --------- |
| Raw/Rc/Box at param, return, field | ~1,523 | `TargetUseSiteOwnershipRealization` / SG-RC-LAYERING |
| `TestClaim` double-wrap | 70 | Same + TestClaim boundary rows |

**Worksheet:** `v4-sg-rc-layering-worksheet-2026-05-31.md` (**APPROVED**). Partial land: #4153 Outcome slice — **do not** claim SG-RC closed.

**Acceptance:** F1–F6 falsification in `sg_rc_layering.dag`; type and value positions move together when `reference_layer` row changes.

**Dispatch:** Continue SG-RC implementation tranches by carrier (`Diagnostics`, `Node`, `TestClaim`, …) — not by error text.

---

### W1-C — Collection boundary projection (P1, ~462)

**Representative:** `expected Vec<Rc<Edge>>, found FreeMonoid<_>`; `Rc<FreeMonoid<…>>` vs `Rc<Vec<Rc<…>>>`.

**Authority:** extend `TargetCollectionRealization` per Arbiter #4170 adjudication (reject parallel `TargetCollectionBoundaryProjection` carrier unless extension proof fails).

**Worksheet:** `v4-sg-collection-projection-worksheet-2026-06-01.md` (#4151, **DRAFT** — needs §8 ratification before impl).

**Compose with SG-RC** for inner `Rc<T>` — forbidden: fold ownership into collection emit branches.

**Dispatch:** Ratify worksheet → implement → remeasure **before** opening W1-G tail that mentions `FreeMonoid`.

---

### W1-D — Diagnostic / Option result shape (P2 hold, ~90)

**Representative:** `Rc<Diagnostics>` vs `Option<_>`; `Diagnostics` vs `Option<_>`.

**Split:** Rc/raw → **W1-B**; Option vs `Diagnostics` → hold until post-**W1-A/W1-B** remeasure. If band persists, author small **diagnostic-constructor** worksheet (single authority for fail-closed diagnostic bundle vs `Option`).

**Dispatch:** **No implementation** in next wave.

---

### W1-E — Map key/value shell mismatch (P2, ~60)

**Representative:** `Rc<HashMap<String, Rc<Node>>>` vs `Rc<HashMap<Rc<FreeMonoid<…>>, …>>`.

**Authority:** likely SG-2 generic instantiation + collection key projection; may shrink after **W1-A** text-carrier fix.

**Dispatch:** Worksheet-only triage after P0 remeasure; do not add `HashMap` key shim tables in Rust.

---

### W1-G — Tail / unparsed (P3, ~581)

23 unparsed `E0308` note shapes + 558 low-count pairs (refinement `DecimalDigit`, lens `Coverage<…>`, duplicate `Rc<Rc<T>>`, etc.).

**Dispatch:** Classify only after P0 remeasure on **clean v2 emit**. Attach each survivor to an existing route; **forbid** new top-level SG class without Modeling DFS.

---

## §4 Next-wave fanout order (manager)

1. **Restore ratchet hygiene:** remeasure M1 only after v2 emit returns to **0 diagnostics** (resolve `source_atom_string_projection_from_node` recursion cycle — likely SG-1b/text projection modeling, not a Rust hotfix).
2. **W1-A1 (SG-1b):** highest leverage; same route as wave-1 but now 62% of parsed `E0308` when text alias is expanded.
3. **W1-A2 worksheet:** author + Arbiter §8 before any host-string / fixture literal impl.
4. **W1-B (SG-RC):** parallel tranches by carrier; Outcome slice (#4153) is not blanket closure.
5. **W1-C:** ratify SG-COLLECTION-PROJECTION worksheet (#4151) then implement.
6. **Hold W1-D / W1-E / W1-G** until post-P0 remeasure.

---

## §5 Non-goals

- Broad E0308 implementation or “drive count to zero” acceptance.
- Per-function `-> String` / `Symbol` patches in emitted Rust.
- Treating `Rc<FreeMonoid<Rc<Nat>>>` as a new domain type (it is the declared substrate text carrier).
- `HashMap` / `Vec` / `collect()` shims without collection-realization rows.
- Replacing #4140 headline totals with this probe while v2 emit reports 104 diagnostics.

---

## §6 Manager decision

**Next-wave P0 fanout = W1-A1 + W1-B**, with **W1-A2 worksheet** in parallel (author only). **W1-C** follows worksheet §8. Diagnostic (W1-D), map shell (W1-E), and tail (W1-G) are **remeasure-gated**.

Report to parent: fresh probe shows **E0308 shape shift** (text-carrier alias expansion) and **non-clean v2 emit**; stratification above is valid for dispatch; ratchet replacement requires clean emit + operator acceptance.
