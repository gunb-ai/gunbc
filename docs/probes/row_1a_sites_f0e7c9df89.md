# Row 1a exact site list — `materialization_carriers` at `f0e7c9df89`

Requested by `silent-raven-853`. Measured on the tree they named, not transcribed from another
head: worktree at `f0e7c9df89` (`#8430`), `gunbc` + `cssl_assemble` built from it, emit →
`cssl_assemble` → `cargo build --release --lib --message-format=json`. Raw diagnostics for exactly
these rows: `docs/probes/row_1a_sites_f0e7c9df89.json` (14 objects: code, file, line, column,
message, primary-span label, span text, children).

Whole-module total at that head: **53 errors** —
`E0308 20 · E0277 16 · E0425 3 · E0422 2 · E0369 2 · E0282 2 · E0061 2 · E0599 3 · E0560 1 · unreachable_pattern 2`.
(That head does not carry PR #8460, so its `E0061 ×2` are still present and its `E0599` count is 3,
not 9. Do not compare it to my branch's 51 without that in mind.)

## The correction this measurement forced: 1a is 12, not 14

I put the two `CacheLookupResult<T>` E0599 rows in 1a. **Reading the specimens they asked for shows
that is wrong**, and it moves the scope boundary they are negotiating with `deep-swift-570`:

```
error[E0599] std_cache_interface.rs:564  the method `clone` exists for enum `CacheLookupResult<T>`,
                                         but its trait bounds were not satisfied
  children: trait bound `T: Clone` was not satisfied
            consider restricting the type parameter to satisfy the trait bound
  span text: match (*lookup.clone()).clone() {          <- a CALL SITE, not a declaration
```

`#[derive(Clone)]` on `CacheLookupResult<T>` already generates `impl<T: Clone> Clone for …`, which
is correct. What is missing is `T: Clone` on the **enclosing generic fn** — `realize_route<T>` at
`dag/std/cache_interface.dag:325` and its sibling at `:334`. So both rows are **fn-signature**
(1b, `emit_fn_def`), not derive-side. **1a = 12; those 2 belong to `deep-swift-570`.**

## 1a — 12 rows, two declarations, one mechanism

Both are generic declarations with a `Rc<im::Vector<T>>` field under a full derive set. `im::Vector<T>`
requires **`T: Clone`** to implement `Debug`, `PartialEq`, `Serialize` and `Deserialize`; the std
derive expansion adds only `T: Debug` / `T: PartialEq` / … and never `T: Clone`, so every derive on
the declaration is unsatisfiable at once. That is why one declaration yields five or six rows.

**`FreeMonoidUniqueState<T>`** — `src/v2/std/algebra.dag:46` → `v2_std_algebra.rs:42-47`

```rust
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FreeMonoidUniqueState<T> {
    pub seen: Rc<Vec<T>>,        // Vec is `im::Vector` under the crate's alias
    pub unique: bool,
    pub _phantom: std::marker::PhantomData<T>,
}
```

| code | site | obligation (from `children`) |
|---|---|---|
| E0277 | `v2_std_algebra.rs:45:5` | `im::Vector<T>` to implement `Debug` |
| E0369 | `v2_std_algebra.rs:45:5` | `==` on `Rc<im::Vector<T>>` (derived `PartialEq`) |
| E0277 | `v2_std_algebra.rs:43:35` | `im::Vector<T>` to implement `Serialize` |
| E0277 | `v2_std_algebra.rs:45:15` | `im::Vector<T>` to implement `Deserialize<'_>` |
| E0277 | `v2_std_algebra.rs:45:15` | `im::Vector<T>` to implement `Deserialize<'_>` |
| E0277 | `v2_std_algebra.rs:43:53` | `im::Vector<T>` to implement `Deserialize<'_>` |

**`ListTailResult<T>`** — `src/v2/std/algebra.dag:88` → `v2_std_algebra.rs:84-90`

```rust
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum ListTailResult<T> { TailFound { tail: Rc<Vec<T>> }, TailAbsent }
```

| code | site | obligation |
|---|---|---|
| E0277 | `v2_std_algebra.rs:88:9` | `im::Vector<T>` to implement `Debug` |
| E0369 | `v2_std_algebra.rs:88:9` | `==` on `&Rc<im::Vector<T>>` (derived `PartialEq`) |
| E0277 | `v2_std_algebra.rs:88:9` | `im::Vector<T>` to implement `Serialize` |
| E0277 | `v2_std_algebra.rs:88:15` | `im::Vector<T>` to implement `Deserialize<'_>` |
| E0277 | `v2_std_algebra.rs:88:15` | `im::Vector<T>` to implement `Deserialize<'_>` |
| E0277 | `v2_std_algebra.rs:84:53` | `im::Vector<T>` to implement `Deserialize<'_>` |

Note the columns: `43:35` / `43:53` and `84:53` land **inside the derive attribute** (on
`Serialize` / `Deserialize` respectively), `45:5` / `88:9` on the field, `45:15` / `88:15` on the
field's type. So the derive macro invocation and the offending field are both directly located in
the JSON — no reconstruction needed.

## What the repair has to decide, stated as the measurement sees it

The bound that is missing is `T: Clone`, and it is missing because the **field's carrier**
(`im::Vector<T>`) imposes it while the derive expansion only knows about the derived trait. So the
fix is a fact about *which carrier a field renders to*, not about the derive list: any generic
declaration with an `im::Vector` / `im::HashMap` / `im::OrdSet` field has it, and no declaration
without one does. A per-derive `#[serde(bound(…))]` reaches only the two serde rows and leaves
`Debug` and `PartialEq` (E0277 ×2 + E0369 ×2 here) failing.

The other three carriers do not appear in this module's 12 rows — every one is `im::Vector`. That is
this closure's population, not a claim about the corpus.
