# Changed-debt classification by observed file pair

The cancelled exact-head run 36813645117 spent its long gaps in edit classification,
not in witness-body evaluation. Job 110213900916 logged the three fleet-plan debt
classifications at 04:51:22, 05:09:53 and 05:28:07 UTC. The corresponding host call is
inside claim construction, before the nominal claim fold. Discovery had already completed.

Each call previously reread the comparison file and tokenized both complete sources.
The fleet carrier is about 2,700 lines. The new model entry reads the base once and
tokenizes each source once per file batch. Declaration selection, removal classification,
callee admission, labels and budget authority remain the existing model's decisions.
The host requires one result for each requested identity and refuses duplicate, foreign
or missing results. This is scoped to one prepared file observation, not a cross-run cache.
Per-file begin/complete timing now makes this phase visible in CI.

The witness population, ceilings, timeout and execution fold are unchanged. This repair
addresses repeated work before that fold; it does not establish that the subsequent
full required floor fits its job deadline. Exact-head CI is still required.

Local validation uses the existing compiler at /home/briansrls/gunbc/target/release/gunbc,
SHA256 b45f00e7ce95814a6c45394b58270fb1d2c9ca67acebabb1151366b6da9bc790.
Rust integration is separately checked with cargo check -p v1-compiler --bin claim_executor.
No host installation, commissioning, allocation, or VM operation is part of this change.

All 17 edit-policy controls pass in a fresh process under MemoryMax=6G and
MemorySwapMax=0 (controls.log). This includes mixed per-identity budgets, missing
declarations, and unreadable token streams. The Rust integration check passes
(cargo-check.log). These are local source controls, not a green required-floor receipt.
