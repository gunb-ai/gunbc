# CI end-to-end accounting

**Goal:** remove repeated preparation while preserving every applicable execution control. This is analysis of [run 37501346261](https://github.com/gunb-ai/gunbc/actions/runs/37501346261), 2026-10-06, not current CI policy; DESIGN owns the required route.

**Method:** build/restore the seed once; scope Rust/prose checks by effective inputs; reuse each native entry's product with complete closure, declaration-universe, driver, target, dependency and toolchain identity; reduce cold reconcile/emit cost. Keep controls against reused products. Instrument mutation experiments run when their producer/wiring/cache/toolchain changes. Fixture compilation is test work, fixture-union emission is blocking coverage, slow projection typechecks are cache misses, and the two native drivers cannot share one emitted closure. Assertion-only fixture changes need not invalidate compilation.

**Validation:** compare whole-run runner time and latency from job/step timestamps and phase logs; use `required-floor-claim-cost` for marginal claim CPU, excluding separately reported shared fills. Overlapping queue/child durations are not additive. Count PR and merge-group work together. The historical ≤29 runner-minute budget was a target, not a forecast; cache reuse and cold-path improvements were necessary beyond easy sharing cuts. The original account retains its budget breakdown and measured baseline.

[Prior design and dated receipts](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/ci-end-to-end-accounting.md).
