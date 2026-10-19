# Pre-registered predictions — call-arm discriminator (probe v18) and the TM repair

Written 2026-10-19 ~04:40Z, BEFORE the v18 rows or any post-repair closure log were read.
Purpose: the parent asked me to "Predict the per-closure delta beforehand, and show via the
comparator that no new refusal appears anywhere, by identity", and my last location claim came
from a static read that two instruments falsified. So both predictions are fixed here first.

## Measurement basis (measured, not assumed)
- Gate at the pre-repair head 8d8d8aac0e1 (and re-measured at 3797d05457d, identical):
  self-host **77** hard diagnostics / 17 identities; v2-native-cli **67** / 18; main **0/0** both
  closures with the positive emitted marker.
- Family split in the gate, by bound head: bound-head `Map` = **40 rows / 4 identities** (self-host),
  **23 rows / 2 identities** (v2-native-cli).
- Fold probe, arms withdrawn, 236 rows, 0 caps: Map-family **46**, of which producer 1 = **39**
  (bound node dag/std/algebra.dag:73644-73647) and producer 2 = **7** (<synthetic>:0-0).

## PREDICTION 1 — discriminator outcome (probe v18, tags TM/BC/RT/unset)
Because `type_match_for_direct_call_target` returns `lookup_type_by_name(env, func_name)` unless the
target is a DeclaredCallableIdentityResolved, and a plain global `fn` name IS registered in the type
env, I predict the Map family is tagged **TM** (type_match Present → `binding.resolved`) on the
overwhelming majority of rows, and specifically on the 39 producer-1 rows. BC should appear only on
rows whose callee is absent from the type env; RT should be rare or zero, since it requires
`gnode.params |> count == 0` while `empty_map` declares two parameters.
Falsifiable and cheap: if BC dominates, my first read was right and the inert repair failed for a
different reason; if `unset` dominates, the probe does not cover this path and I report rows, not a
branch.

## PREDICTION 2 — per-closure delta for the TM repair
Scope, per the parent's caution: instantiate **only when the callee declares type parameters**
(`fn_type_param_names(item: gnode)` non-empty), so every non-generic direct call keeps its present
`binding.resolved` node byte-for-byte. Predicted effect:
- self-host: **77 → between 37 and 44**. 37 is the ceiling case where all 40 Map-headed rows clear;
  44 is where producer 2's minted rows survive (46-row probe split says producer 2 is 7 of 46, i.e.
  ~6-7 of the 40 gate rows). Anything OUTSIDE [37, 44] is a surprise I must explain, not absorb.
- v2-native-cli: **67 → between 44 and 48**, same reasoning on 23 Map-headed rows.
- main: must remain **0/0** with the emitted marker, or the control is void and I fix the harness.
- **NO NEW REFUSAL ANYWHERE, by identity** (the parent's requirement, stated as a testable
  predicate): the post-repair identity set must be a SUBSET of the pre-repair identity set, in BOTH
  closures. Formally: `set(post_identities) - set(pre_identities) == {}` for self-host and for
  v2-native-cli. Any identity appearing post and not pre is a NEW refusal produced by my change --
  an instantiation that failed somewhere else -- and is a FAIL, not a finding to explain away.
  Identities that DISAPPEAR are expected (those are the rows the repair cleared) and are listed by
  identity in the PR body.
- Per-module: the two closures each emit exactly ONE module (source_root_eval.dag, compile_cli.dag);
  any module that newly appears in the citing-location list is listed and dispositioned, which is
  the disposition duty from msg_79410475.
- Suite: 30 passed / 0 failed must hold, and specifically the two accept controls and the refusal
  twins must still pass; the accept control for the generic-declaration-own-form fold must not
  change verdict, because it does not go through a call arm.
- Byte-identity check for the non-generic population: the identities that are NOT Map-headed must be
  unchanged in both count and identity between pre and post (they are the same rows, same counts).
  If any of them moves, the "only when the callee declares type parameters" gate is not doing its
  job and the blast radius is wider than the parent's caution allows.

## PREDICTION 3 — producer 2's 7 rows (restated, unchanged)
They REMAIN: minted at a different site (04_infer.dag:9430 → bare_map_node), and neither skip arm
sees through its single `Present { Resolved }` hop. If they clear anyway, the honest reading is
that they were never an independent producer and I over-split by shape. My earlier "hit" on this
prediction is recorded as UNINFORMATIVE, because the repair that left them in place was inert.

## What I will NOT do
No fold predicate written to absorb remaining rows. No widening of the fold's unsettled tests to
clear producer 2 -- its boundary is the minter, and if it needs a change that is a separate,
separately-justified commit. If the delta lands outside the predicted band, I stop and report
rather than adjusting the acceptance.
