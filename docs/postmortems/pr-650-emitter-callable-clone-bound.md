# Post-mortem: PR #650 — callable generic `Clone` bound vs emitter ownership

**Program:** Emitter Fidelity (callable generic ownership / bounds)  
**Status:** Receipt for the failed micro-fix; stop boundary for structural follow-up.

When shipped as its own PR, treat this as the **prerequisite / boundary receipt** (#650 analysis + regression pin), not a claim that the structural emitter-fidelity lane has closed.

---

## Exact attempted change

PR **#650** tried to remove the synthesized Rust trait bound **`+ Clone`** from **first-class callable parameters** emitted as `impl Fn(...) -> T + Clone`, compensating instead inside the Rust emitter (use-site cloning / move decisions) so emitted code would still type-check without advertising `Clone` on the parameter type.

The affected seam is **`emit_rust_param_type`** in `src/v2/05_emit_rust.dag`: callable-shaped parameters (`n.params` non-empty) render as `impl Fn(<args>) -> <ret> + Clone` rather than reflecting only the source `fn(...)->...` type.

---

## Why it was unsound (for this codebase)

1. **Source authority vs target realization** — Source programs declare `fn(A) -> B`; they do **not** declare a Rust `Clone` obligation on that callable. Synthesizing `+ Clone` is already a **target-side admission** (see THESIS.md “two groundings”). Removing the bound without replacing it with an **equivalent declared fact** elsewhere reintroduces “plausible Rust” that is not mechanically justified from one authority.

2. **Second move/clone authority** — The emitter already has a single **by-value vs `.clone()`** authority for ordinary bindings: **`emit_info.movable`** (from `build_movable_set` / ownership proof), documented at `emit_var_ref` in `05_emit_rust.dag`. The #650-style compensation tied callable **typing** to the same movable machinery **without** keeping the type signature consistent with emitted `.clone()` calls. That split **who decides cloning** between the type line (`impl Fn + Clone`) and the ownership map, i.e. parallel authority for overlapping semantics.

3. **Self-host / fixed-point instability** — Stage0 is emitted by the same pipeline. A partial change to callable param typing plus emitter-only fixes perturbs generated `v2_compiler_emit_rust.rs` / `v2_compiler_emit.rs` signatures (many `impl Fn(...) + Clone` closure parameters). The result did **not** converge to a sound, stable fixed point: the diff could not land as a coherent “pure fidelity” migration.

---

## Where ownership authority split

| Authority | Role today |
|-----------|------------|
| **`emit_info.movable`** | Sole gate for **plain variable references**: emit by value vs `SharingStrategy` `.clone()` (`emit_var_ref`). |
| **Synthesized `+ Clone` on `impl Fn` params** | Rust target contract for **reusing** the callable value across emitter patterns that **do** materialize `.clone()` (non-movable locals, stage0 closure params, etc.). The v2 Rust emitter lowers call sites to **plain** `f(arg)` text (not a spelled `Fn::call(...)` in generated Rust); two uses can therefore omit a visible `f.clone()` while still type-checking. Dropping the bound still desyncs type lines from clone sites where the emitter **does** spell `.clone()`. |

#650 effectively asked **`emit_info.movable`** (or ad hoc emitter rules) to **subsume** the second row **without** deleting the need for `Clone` at the type level in Rust. That is the authority split: **two loci** for “when is this callable value copied,” only one of which is grounded in declared source types.

---

## Stop boundary for the Emitter Fidelity lane

Structural work on this seam must obey **all** of:

1. **No second authority beside `emit_info.movable`** for **ordinary binding** move vs clone. Do not re-encode callable cloning decisions in a shadow path keyed only on emitter heuristics.

2. **No “remove `+ Clone`” without a declared substitute** — Either keep the current **explicit** `+ Clone` on `impl Fn` params as the admitted Rust lowering contract until **declared-bound modeling** exists, **or** introduce **one** upstream fact that makes the bound faithful (e.g. explicit source-level or `EmitGraphInfo`-level carrier) and migrate **types + uses** in the **same** change.

3. **No target-only strengthening** of user contracts (no hidden stricter bounds than the source model admits).

4. **Fixed-point first** — Any change must pass **regenerate-stage0** self-compile / fixed-point checks before review.

   **Enforcement (not added in this receipt PR):** the tree already has operator workflow `./scripts/regenerate-stage0.sh` and ignored CI-style gates **`ci_freshness`** / **`ci_fixed_point`** in `src/v2/tests/src/bootstrap.rs` (typically `cargo test -p v2-compiler-tests ci_ -- --ignored` on the lane that runs full v2 CI). A **structural** follow-up that edits `05_emit_rust.dag` emission should treat regen + those gates as merge evidence; tightening policy (e.g. non-ignored CI) is a separate process change.

---

## Honest outcome of this lane (wave 5)

The minimal faithful step **after** this post-mortem is **not** to delete `+ Clone` again. It is to **document and test** the seam: callable param position uses **`impl Fn(...) + Clone`** as the **Rust storage/reuse** contract; **`emit_info.movable`** remains the sole authority for **non-callable** locals. Removing the synthesized bound belongs to a later lane that either lands **declared-bound modeling** or a **single** new carrier that subsumes both typing and use emission.

### Signpost in `05_emit_rust.dag`

The comment block immediately above `emit_rust_param_type` should stay a **short seam signpost**. If more boundary prose is needed, add it **here** (this post-mortem) rather than growing meta “how to maintain this comment” footers in the `.dag` source — those read as defensive noise next to real emission logic.

### Regression test scope (`pipeline.rs`)

The hermetic test asserts the **`impl Fn(...) + Clone`** signature on `twice`, then checks `f(0)` / `f(1)` only in the suffix **after** that signature so incidental substring matches elsewhere in the emitted module file cannot satisfy the call-site part. It does **not** assert a `f.clone()` substring: `compile_dag` output for this fixture is plain **`f(0)`** / **`f(1)`** call syntax on the param, with no `.clone()` in the emitted source, while `+ Clone` on the type remains load-bearing elsewhere (stage0 / other emitter paths). A future tightening could pin an explicit clone site once a small fixture is found that **deterministically** materializes asymmetric `f.clone()` in emitted Rust.

This pin is **necessary but not sufficient** for a structural #650-style retry: it can pass while stage0 self-host fails, so **item 4** gates (`regenerate-stage0.sh`, `ci_freshness`, `ci_fixed_point`) remain the authoritative merge evidence for emission changes.

**Verify locally** (workspace package name is `v2-compiler-tests`, hyphenated — not the underscored Rust crate id used in test binary paths):

```bash
cargo test -p v2-compiler-tests rust_emit_callable_param_double_use_keeps_clone_bound_on_signature
```

**TESTING.md §4:** the test checks signature and two call-site substrings in one `#[test]`; that bundles two *surface* checks but one *behavioral* receipt for this seam. It is not precedent for unrelated multi-claim tests.
