---
name: FF-8 Regeneration Blockers
description: Root causes preventing stage0 regeneration; must fix in emitter (.dag) + dual-patch stage0 (.rs)
type: project
---

Self-compile works: 0 diagnostics, 40 files, 103MB, 30s.
Regenerated .rs files have ~1400 compile errors from 5 root causes.

## Root Causes (ordered by error count)

### RC-1: lazy_static + Rc incompatibility (~700 errors cascade)
`lazy_static` requires `Sync`. `Rc` is not `Sync`. All data defs that produce
`Rc<HashMap>` or `Rc<Vec>` as lazy_static fail. 52 blocks across 8 files.
**Fix:** Emit data defs as functions returning Rc, not lazy_static.
This is the highest-leverage fix — eliminates most mismatched type errors too
since callers reference the lazy_static with wrong type.

### RC-2: Rc::new() empty collection init (~133 errors)
Emitter generates `<Rc<HashMap<_,_>>>::new()` but `Rc::new()` takes 1 arg.
Should be `Rc::new(HashMap::new())`. Fix already in .dag source (cf1bb3ee)
but needs dual-patch in stage0 .rs to take effect on next regen.

### RC-3: std library modules unrenderable (~87 errors)
std_algebra.rs, std_syntax.rs contain generic algebraic types (`free_monoid<T>`,
`Unit`) that the Rust emitter can't render. std_types.rs has `compile_error!`.
**Fix:** Exclude from lib.rs, use minimal std_types.rs stub.

### RC-4: Rc<Vec> not IntoIterator (~50 errors)
Generated code does `for x in rc_vec.clone() {}` but `Rc<Vec<T>>` doesn't
implement `IntoIterator`. Needs `.iter().cloned()`.
**Status:** Emitter already generates `.iter().cloned()` for most patterns.
Remaining sites might be edge cases or from data def emission.

### RC-5: v2_rt.rs bare type signatures (~remainder)
Runtime functions accept `Vec<T>` / `HashMap<K,V>` but regen code passes
`Rc<Vec<T>>` / `Rc<HashMap<K,V>>`.
**Fix:** v2_rt.rs written with Rc signatures (done, saved but not committed).

## Bootstrap Loop
Each emitter fix needs dual-patching: .dag source + stage0 .rs. The .dag fix
improves future regens; the .rs hand-patch makes the current binary produce
correct output.

## Recommended Priority: RC-1 first (lazy_static → functions)
This is the single change that eliminates the most errors. The emitter already
has a function-based data def path for complex types. Extend it to all map data.
