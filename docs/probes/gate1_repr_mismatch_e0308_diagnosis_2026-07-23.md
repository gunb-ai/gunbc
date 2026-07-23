# Gate-1 repr-mismatch E0308 diagnosis — silent-fox-559, 2026-07-23

**Representative module:** `src/v2/compiler/06_translate.dag`  
**Probe:** `scripts/repr_mismatch_emitted_e0308_probe.sh` (EMITTED_namespace, head `608749d287`)  
**Classifier:** execution histogram on full emitted-crate `cargo build` log (E0433 is first-error but E0308 counted across all diagnostics)

## Uniformity across six deep modules

| Module | E0308 (all) | Total rustc errors | First error |
|---|---:|---:|---|
| 06_translate | 3176 | 3561 | E0433 UriScheme |
| 05_emit | 3177 | 3562 | E0433 UriScheme |
| emit_module | 3185 | 3570 | E0433 UriScheme |
| program_partition | 3207 | 3621 | E0433 UriScheme |
| emit_semantic_decl | 3275 | 3671 | E0433 UriScheme |
| emit_host | 3409 | 3978 | E0433 UriScheme |

Counts vary by closure size (±233) but share the same histogram shape — one shared closure-side fork, not six independent problems.

## Coarse bucket histogram (06_translate)

| Count | % | Bucket |
|---:|---:|---|
| 2185 | 67.9% | **TEXT: FreeMonoid↔String (text carrier)** |
| 310 | 9.6% | COLLECTION: Vector vs FreeMonoid |
| 234 | 7.3% | OPTIONAL: modeled Optional vs native Option |
| 107 | 3.3% | DIAG: Diagnostics type leak |
| 98 | 3.0% | WITNESS: Witness&lt;T&gt; param mismatch |
| 65 | 2.0% | RC_WRAP: Option&lt;T&gt; vs Rc&lt;Option&lt;T&gt;&gt; |
| 46 | 1.4% | OWNERSHIP: Node vs Rc&lt;Node&gt; |
| 175 | 5.4% | OTHER (≤1% each) |

**202 distinct fine-grained expected/found pairs; one pair dominates.**

## Dominant fine pair

```
2108×  expected `Rc<FreeMonoid<Rc<Nat>>>`  found `String`
  58×  expected `String`                    found `Rc<FreeMonoid<Rc<Nat>>>`
```

### Exemplar (v2_extdeps_languages_dag.rs — largest file, 1553 E0308)

```rust
static CACHED: String = {
    "fn add(x:Int, y:Int) -> Int { x + y }".to_string()
    // ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected Rc<FreeMonoid<Rc<Nat>>>, found String
};
```

### Root mechanism

- Substrate: `type String = FreeMonoid<Char>` and `type Char = Nat` (`src/v2/std/text.dag`).
- Symbol already grounds to native Rust `String` (`dag/extdeps/languages/rust/types.dag`).
- **Emit gap:** under `RustCorpusRepr::FaithfulFreeMonoid` (default for deep-module probes), the `String` type-alias decl path rendered `pub type String = Rc<FreeMonoid<Char>>` while literals, host interop (`v1_rt::concat`, `.to_string()`), and Symbol all use native `std::string::String`.
- `is_host_text_carrier_type` / `render_rust_text_carrier` already ground **use-site** renders; the **type-alias declaration** was gated on `corpus_repr_is_host` only — HostNative modules got `type String = std::string::String`, FaithfulFreeMonoid modules did not.

This is the numeric-tower precedent (#5428) applied to the text carrier: construction-side grounding so native form == modeled form.

## Verdict: **ONE TRACTABLE ROOT** (+ staged secondaries)

- **Primary (clears ~68%):** text-carrier grounding — `String` type alias must emit native `std::string::String` in all corpus reprs, not only HostNative.
- **Secondaries (do not block primary; stage after E0433 wall):** Vector/FreeMonoid collection repr (~10%), Optional/Option (~7%), Rc-wrap on Option fields (~2%), Node/Rc ownership (~1.4%). None exceed 10%; not systemic 3200-way fork.

## Phase-2 fix (this PR)

Extend Gate-1 text-carrier grounding to the type-alias emit path (`rust_string_grounded_type_alias_decl_line` — corpus-repr-agnostic). Discriminating tests: `faithful_string_*` witness suite.

**Seam walls respected:** no E0433 import-closure changes; no Lane-D dotted-name rendering changes.

## Post-fix receipt (Phase 2, same probe)

| Metric | Before | After | Δ |
|---|---:|---:|---:|
| E0308 (06_translate) | 3176 | 1099 | −2077 (−65%) |
| TEXT FreeMonoid↔String bucket | 2185 (67.9%) | 41 (3.7%) | −2144 |
| `v2_std_text` String alias | `Rc<FreeMonoid<Char>>` | `std::string::String` | grounded |

Remaining E0308 is secondary buckets (Vector/FreeMonoid ~25%, Optional/Option ~21%, RC_WRAP ~6%) — staged successors, not systemic fork.

## Successor lanes (pre-scoped ratchet — do not re-diagnose from scratch)

Each remaining bucket is the **same construction-grounding move family** as text (#5428: native form == modeled form at the emit seam), but each names its **own carrier authority** — not one mega-fix:

| Bucket | ~share | Grounding shape | Authority / seam |
|---:|---:|---|---|
| COLLECTION Vector↔FreeMonoid | 10% | Type-alias + collection-repr choice (`TargetCollectionRealization`; text carrier is the Char→String special case already in `06_translate`) | `project_*_collection_type_node` + `rust.dag` realization rows |
| OPTIONAL Optional↔Option | 7% | Type-alias + use-site: modeled `Optional<T>` → native `Option<T>` | Mirror `is_host_text_carrier_type` pattern for Optional |
| RC_WRAP `Option<T>` vs `Rc<Option<T>>` | 2% | **Wrap-decision** grounding (ownership facts), not type-alias alone | `wrap_decision_predicate` (#6776) — struct-field emission |
| OWNERSHIP Node vs `Rc<Node>` | 1% | Same wrap-decision family | `wrap_decision_predicate` |
| DIAG / WITNESS residue | ~6% | Typed diagnostics + Witness param alignment — separate axes, not carrier alias | Per-class rows after carrier ratchet |

**Ratchet order:** text (this PR) → collection → Optional → wrap-decision residue. E0433 import-closure (#7125) is a parallel wall, not an E0308 grounding.
