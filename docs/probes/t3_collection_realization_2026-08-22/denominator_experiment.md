# T3 step 0 — the denominator experiment (2026-08-22)

Run to validate the migration denominator before any migration. **Result: the alias flip is not a
population control, and the reason is itself a finding.** A control that does work was found and
run, and it joins to the census exactly on the half it can reach.

| | |
|---|---|
| ref | `259fe83addfc842a1361cb3ceda97941034c48b6` (this branch; docs-only above `753c7d1def0`) |
| instrument | `gunbc compile --source-root dag --source-root src/v2 --entry <e> --target rust`, built from this tree |
| verdict line | `N blocking error(s), M advisory diagnostic(s)` — the compile's own summary |
| entry | a probe module importing one symbol from **all 19** modules holding a `Set { … }` literal, authored only in a throwaway worktree and never committed |
| isolation | a detached `git worktree`; the session branch was never edited |

## What was measured

| arm | tree | blocking | advisory | verdict |
|---|---|---:|---:|---|
| A | control | **0** | 333 | — |
| B | `type Set<element> = PointwisePower<element>` → `type Set<element>` (opaque) | **0** | 333 | no refusal |
| C | `Set` renamed out of existence (`type SetRenamedControl<element>`) | **refuses**, exit 1 | — | `unresolved type 'Set'` at every type position |
| D | `PointwisePower.member` → `member_predicate` | **0** | 333 | no refusal *at the literals* |

Arm C is the discriminating positive control: the instrument reads `dag/std/types.dag`, and a
change that should refuse does refuse, loudly, with located sites. So arm B's zero is a real zero.

## Finding 1 — the alias flip cannot validate the denominator

Flipping the alias produced **no diagnostic anywhere** across a closure covering all 19 modules, and
the **entire** emitted difference between arms A and B is one line:

```
- pub type Set<Element> = Rc<crate::std_algebra::PointwisePower<Element>>;
+ #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
+ pub struct Set<Element>(pub std::marker::PhantomData<Element>);
```

Every `Set { member: … }` literal emitted byte-identically in both arms. So a temporary-branch alias
flip yields a refusal set of **zero regardless of the true population** — it would have "validated"
any roster, including a wrong one.

## Finding 2 — why: the literals are unchecked against their carrier

Arm D renames the carrier's only field. The literals still do not refuse; they still emit
`member:`; and the emitter silently *degrades* the callable wrapper it no longer recognises:

```
- member: { Rc::new(move |_| false) },      (arm A)
+ member: |_| false,                        (arm D)
```

So record-literal field agreement is not enforced on this path at all. No std-side edit can make
the **literal** population refuse, which means the literal half of the census is establishable only
by source enumeration — the roster — and not by any refusal experiment.

## Finding 3 — a control that does work, and an exact join

Arm D *does* reach the **call sites**. Every `.member(…)` call emits, into the Rust artifact:

```rust
compile_error!("method member is neither a resolved callable receiver field nor a registered v1_rt bridge function")
```

Counting that marker across the emitted tree: **0 in arm A → 20 in arm D**, and the per-file join
against the authored `.member(` census is exact:

| emitted file | `compile_error!` in arm D | `.member(` in the `.dag` module |
|---|---:|---:|
| `v2_compiler_resolve.rs` | 1 | 1 |
| `v2_compiler_parse.rs` | 5 | 5 |
| `v2_extdeps_languages_dag.rs` | 1 | 1 |
| `v2_extdeps_languages_typescript.rs` | 1 | 1 |
| `v2_std_grammar.rs` | 4 | 4 |
| `v2_test_manual_dissolution_subsumption_reverification.rs` | 8 | 8 |
| **total** | **20** | **20** |

Corpus-wide there are 28 `.member(` occurrences in 8 files; the 8 not seen here are the two modules
outside this entry's closure (`trait_derive_supplemental_generic_bound_contract_test` 5,
`lens_subsumption_family_eval_test` 3). Nothing is unaccounted for.

## Finding 4 — a deferred refusal, reportable as its own class

The arm-D refusal is real, located, and typed — and the compile that produced it printed
`0 blocking error(s)` and **exited 0**. The refusal was written *into the artifact* instead of
stopping the line, so the defect is discovered by `rustc` at a later stage rather than by the
compiler that knew it. That is the §5 shape: the line does not stop, and a consumer counting
blocking diagnostics sees a clean compile. It is named here rather than repaired, and it is
independent of T3 — T3 is why it was found, not what it is.

## What this changes for the sequence

- Step 0 as specified (alias flip → refusal set → join) **cannot be completed**, and no branch
  should be spent trying: findings 1 and 2 say the refusal set is empty by construction.
- The denominator is therefore **half validated by execution and half by census**: the 20 in-closure
  call sites join exactly; the literal roster (18 predicate / 12 enum / 6 empty / 1 update / 145
  map) stands on source enumeration, pre-registered before this run and unchanged by it.
- Steps 1–5 are unaffected in substance. What changes is that no step may use "it refuses" as its
  completion evidence for a literal-shaped change; only call-site-shaped changes have that lever.
