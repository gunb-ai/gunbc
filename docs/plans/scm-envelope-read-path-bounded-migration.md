# The envelope read path: bounded, and the one-time rebuild to v7

Status: declared 2026-09-23. Authority: the production incident —
`belt_integrate_one_cli` over the `shell-typed-invocation` project envelope
(`attempt-state/projects/shell-typed-invocation.gunbc-repository.json`, a 104 MB
`gunbc-scm-repository-v6` monolith) exceeded ninety minutes at ~100% CPU and was
killed while reading; a rehearsal probe of the same read ran past two hours
before it too was killed.

## What was unbounded

The monolith decode admitted the object table entry by entry through the
per-object store verbs. `find_object` recomputes a content identity for every
already-held object on every question, and the insert copies the object spine
per append — so decoding an n-object table pays the square of n in interpreted
identity compares. At the 7,051-object project envelope that is ~25M identity
computations and the same again in spine copies; the read never completed. The
2026-09-21 intake measurement pinned the same shape (~50M compares holding one
belt tick inside a single seed commit for over an hour), and the JSON unescaper
additionally paid the square of each large source token (a 15 KB source cost
225 MB of accumulator copies) until the same day's span fix.

## The fix

One admission rule, two carriers. `gunbc.scm.object_store` now carries a
fold-shaped `StoreAdmission` accumulator (digest-keyed map, prepend-and-reverse-
once) beside `insert_admission`; the object-table decoder admits through the
accumulator, so the table decode is O(n log n) with the same idempotence and the
same collision arms. Object construction for sources, derived nodes, and
manifests is split into constructors behind the sole_constructor wall, and the
single-insert verbs are re-derived on them — one construction authority, one
admission rule. The sharded (v7) read keeps its own two linear passes (per-shard
decode, then `store_assemble_decoded`), now on the same accumulator.

## The declared migration

The production envelope is converted to the v7 sharded layout by the next
successful integrate over it — the integrate's save is already sharded, so the
conversion is a side effect of the first bounded run, not a separate tool, and
no monolith writer remains to recreate it. After that run:

- every load is the sharded read: the index plus ~110 shard reads at the current production
  scale (7,051 objects at a batch of 64), each decoded and admitted in linear passes;
- the v6 monolith reader stays in the tree for old documents and fixtures, but
  it is no longer on any production path, so its remaining costs are bounded by
  history, not by the read path this declaration governs.

Rollback: the conversion writes the v7 index and shard files beside the
monolith's path; the pre-conversion monolith is not deleted by the save (the
sharded writer writes `<base>.object.<digest>.json` siblings and replaces the
index document's contents), so the v6 bytes remain recoverable from backup
state until the envelope's next writes settle. The load path dispatches on the
`format` tag, so a restored v6 document reads exactly as today.
