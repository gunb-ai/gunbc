> Part of: [THESIS.md](../../THESIS.md) > [ROADMAP.md](../../ROADMAP.md) > **Tier 1 + Free consequences** (parallelism prerequisite)
> See also: [cx-design.md](cx-design.md), [ROADMAP.md](../../ROADMAP.md) (Track 2: LS-4)

# Ownership Analysis Design

Design and workboard for the ownership pipeline. Covers clone
elision, borrow propagation, and fold accumulator optimization.

---

## Thesis

The ownership pipeline (`ownership.dag`) computes correct facts about
value usage — fan-out, edge classification (Consumed/Read/Threaded/
Projected), fold accumulator eligibility. The emitter doesn't consume
all the facts it has. The result: 23,733 `.clone()` calls in stage0
(~0.479 clones/line), most unnecessary.

This is the same construct-discard-reconstruct pattern as CX:
- Inference computes structural relationships
- TypeBinding discards everything except the type
- Ownership reconstructs via string name matching and AST re-walking

## Shared infrastructure with CX

Both CX and ownership need the same fact about each binding: **what
is this value's structural relationship to the function's inputs?**

- CX needs it to prove: "this argument is a sub-value → descend → O(|tree|)"
- Ownership needs it to prove: "this value is used once → move, not clone"
  and "this is the fold accumulator → unwrap, not clone-fallback"

The shared work is Track 1 in the ROADMAP: SubValueRelation on
TypeBinding. Items S1-S8 in `cx-design.md` serve both consumers.
This doc covers only the ownership-specific work that builds on top.

---

## Current reconstruction heuristics

Three specific reconstruction points in `ownership.dag`:

**1. Fold detection by string name** (lines 213-214, 241-243)

```
if fname == "fold" {
  let init_arg = texpr.children |> filter(a => a.name == "init") |> first
```

Reconstruction: the compiler determines this is a fold call by
matching function/method name, then finds the init argument by
matching `a.name == "init"`. The fold structure (which arg is init,
which is body, which is collection) was known at inference time.

**2. Accumulator struct detection by terminal name** (lines 430-491)

```
let terminal = fold_terminal_expr(body: lambda_body(texpr: fold_lambda_node))
match terminal.expr_data {
  ExprRecordLit { parent_enum: _ } => terminal.name  // string name match
```

Reconstruction: walks the lambda body to find the terminal expression,
checks if it's a record literal with the same name as the accumulator
type. The type system already knows this.

**3. Field move collection by AST re-walk** (lines 438-454)

```
fn collect_acc_field_moves(...) -> List<String> {
  // re-walks body to find acc.field accesses
```

Reconstruction: re-walks the entire fold body looking for field
accesses on the accumulator variable. The type definition already
lists all fields; the ownership proof already has edge classification
for each use.

**What dissolves with provenance on bindings:**
- Fold detection: CallbackContract on fold method declares which
  param is init, which is body. No name matching.
- Accumulator detection: lambda param binding carries provenance
  (IteratedSubValue or fold-accumulator). No terminal inspection.
- Field move collection: type definition lists fields; ownership
  proof's edge classification says which are moved. No AST re-walk.

---

## Violation classes

Conceptual categories for design orientation. The current ratchet
does NOT measure these individually — it produces two coarse
aggregates via scope-blind string matching (see pipeline.rs).

| Class | What | Root cause | Measured by |
|-------|------|------------|-------------|
| V1: Last-use clone | Fan-out > 1, last use clones when it could move | Emitter doesn't track which use is last | `movable_but_cloned` (conflated with V2) |
| V2: TCO-gated move | Fan-out = 1 + owned, but TCO gate zeroes movable set | TCO runs before ownership | `movable_but_cloned` (conflated with V1) -- **0 in practice** (branch merge handles correctly) |
| V3: Fold fallback | Proof says eligible, emitter emits fallback anyway | Emitter doesn't trust proof | `try_unwrap_fallbacks` |
| V4: Read-as-clone | Read edge emitted as `.clone()` when `&x` suffices | No borrow model in LanguageSpec | Not yet measured |

## Three layers to clone elimination

| Layer | Size | Blocked on | Impact (est.) |
|-------|------|-----------|---------------|
| 1. Last-use elision | 1-2 PRs | Nothing | ~2,000-4,000 of 23,733 clones |
| 2. Post-TCO ownership | 1 PR | Nothing | ~0 clones (V2 resolved by branch merge; emitter identity elision done) |
| 3. Borrow propagation (LS-4) | 3-5 PRs | LanguageSpec design | ~15,000-18,000 clones |

---

## Workboard

### Layer 1: Last-use elision (blocked on stable binding identity)

Threading last-use facts through the emit boundary requires stable
binding identity. A name-keyed Map<String, Int> collapses distinct
bindings with the same authored name, violating the explicit boundary
contracts invariant. The ownership-internal span_start data is correct,
but cannot be exposed at the emit boundary until bindings have unique
identifiers.

**Blocked on:** Track 3 (stable binding identity via InternTable or
declaration span). Once bindings have unique identifiers, last-use
facts can flow through EmitGraphInfo without lossy name collisions.

| # | Item | Test | Cleanup target | Status |
|---|------|------|---------------|--------|
| O1 | Track use-site ordering in BindingUsage | span_start populated in walk_expr | — | Not started (needs identity-keyed table) |
| O2 | Emitter skips `.clone()` on last use of fan-out > 1 binding | Needs stable binding identity at emit boundary | — | Blocked on Track 3 |
| O3 | V1 ratchet at 0 for focused test programs | Test: `count_ownership_violations` V1 = 0 | — | Not started |

### Layer 2: Post-TCO ownership (resolved)

V2 as originally conceived ("TCO gate zeroes movable set") does not
manifest in practice. The branch-aware merge in ownership analysis
already computes correct fan-out for TCO functions:

- Parameters only threaded through self-calls (not used in body)
  already have fan-out=1 and ARE movable. No V2 violation.
- Parameters used in body AND self-call genuinely have fan-out>=2.
  The clone is needed because Rust's loop semantics require the
  value for both the body use and the next-iteration use.

**Done:** TCO identity pass-through elision in the emitter (PR TBD).
Self-calls like `f(tokens: tokens, state: new_state)` now skip the
`tokens = tokens` reassignment, reducing generated code size. This
does not change clone count but removes dead code from TCO loops.

| # | Item | Test | Cleanup target | Status |
|---|------|------|---------------|--------|
| O4 | Skip identity pass-through in TCO reassignment | Test: TCO function with pass-through param has no `param = param` reassignment | Redundant `__tco_N = param; param = __tco_N;` lines | Done |
| O5 | V2 ratchet at 0 for focused test programs | Test: `count_ownership_violations` V2 = 0 | — | N/A (V2 = 0 by construction) |

### Layer 3: Borrow propagation (needs LS-4 design)

| # | Item | Test | Cleanup target | Status |
|---|------|------|---------------|--------|
| O6 | Add borrow syntax to SharingStrategy in LanguageSpec | Test: Rust SharingStrategy has `borrow_syntax: "&{T}"` | — | Not started (design needed) |
| O7 | Classify function params as read-only from ownership proof | Test: function where all param edges are Read/Projected → param marked read-only | — | Not started |
| O8 | Emit read-only params as `&Rc<T>` in function signatures | Test: read-only param compiles as `&Rc<T>`, call site passes `&x` | Per-function `.clone()` at call sites | Not started |
| O9 | Cascade: all call sites match new signatures | Test: stage0 regen compiles with borrow signatures | — (atomic with regen) | Not started |
| O10 | V4 ratchet at 0 for focused test programs | Test: `count_ownership_violations` V4 = 0 | — | Not started |

### Shared infra items (from cx-design.md)

| # | Item | Ownership benefit |
|---|------|-------------------|
| S1 | SubValueRelation on TypeBinding | Fold accumulator known from binding, not name matching |
| S6 | Lambda params with element expected | Fold callback param carries IteratedSubValue |
| S7 | Lambda params with Callable expected | HOF callback params carry structural provenance |
| S8 | BoundedLattice on SubValueRelation | `merge_branch_usages` lattice meet derived, not hand-coded |

### TDD strategy

**Layer 1+2 tests exist** (PR #373): `count_ownership_violations`
already counts V1-V4 violations from the pipeline result. The test
programs have known ownership properties. TDD: write expected
violation counts, then make the layers pass.

**Layer 3 needs a design phase** before tests. The LanguageSpec
borrow model must be designed — how does each target language express
borrows? Once designed, tests follow the same pattern: expected
signature shape in emitted code, then make it pass.

**Comparison harness (for shared infra):** When S1-S7 land, compare
ownership's fold detection with and without provenance. The old
path (name matching) should agree with the new path (read binding
provenance). When they agree, delete the name matching.

### Cleanup catalog

**Dissolves after S1 + S6 + S7 (provenance on bindings):**

| Code | File | Lines | Deletes after |
|------|------|-------|--------------|
| `fname == "fold"` / `mname == "fold"` name matching | ownership.dag:213, 241 | ~10 | S7 (callee contracts) |
| `a.name == "init"` arg name matching | ownership.dag:214 | ~5 | S7 |
| `fold_terminal_expr` body re-walk | ownership.dag:408-420 | ~15 | S6 (lambda provenance) |
| `fold_body_constructs_acc_struct` name matching | ownership.dag:426-435 | ~10 | S6 |
| `collect_acc_field_moves` AST re-walk | ownership.dag:438-454 | ~20 | S6 |
| `terminal.name == acc_type_name` string match | ownership.dag:430 | ~5 | S6 |
| **Subtotal** | | **~65** | |

**Dissolves after S8 (lattice consolidation):**

| Code | File | Lines | Deletes after |
|------|------|-------|--------------|
| `merge_branch_usages` hand-coded lattice meet | ownership.dag:156-177 | ~25 | S8 |

**Dissolves after O8-O9 (borrow propagation):**

| Code | File | Lines | Deletes after |
|------|------|-------|--------------|
| Hardcoded `.clone()` at Read edge sites | 05_emit_rust.dag (scattered) | ~50+ | O9 |
| `is_clone_needed` heuristics in emitter | 05_emit_rust.dag | ~30 | O9 |

**Total estimated dissolution: ~170+ lines** (less than CX because
ownership.dag is smaller, but the clone reduction in emitted code is
the bigger win — ~15,000-18,000 fewer `.clone()` calls in stage0).

---

## Relationship to other docs

- **cx-design.md** — shared infrastructure (S1-S8), provenance model,
  construct-discard-reconstruct diagnosis
- **ROADMAP.md Track 2** — LS-4 ownership layers, clone census data
- **INVARIANTS.md** — "Facts Flow Forward" guarantees fan-out is
  syntactic, which is why ownership analysis works at all
