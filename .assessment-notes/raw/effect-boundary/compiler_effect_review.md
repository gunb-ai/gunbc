# Compiler/seed effect boundary

Every matching source file is listed. This separates durable compiler state from local emission/control/test effects; generated status alone is never an exclusion.

`src/v1/05_emit_rust.dag` — Compiler-authored Rust file-transport backend fragments (and stdout/list operations); retain at extdeps realization/emit boundary, not standalone durable domain store.

`src/v1/compiler_tests_rust.dag` — No production filesystem mutation established after bounded cfg(test) and source-body review: fixture construction, pure collection append/truncate, diagnostic/prose or generated expected-source strings. Retain controls/exclude false positives.

`src/v1/runtime_rust.dag` — Compiler-authored Rust file-transport backend fragments (and stdout/list operations); retain at extdeps realization/emit boundary, not standalone durable domain store.

`src/v1/stage0/src/behavioral_receipt_host.rs` — Driver workspace source writes and mirror mutation/restore for behavioral differential; retain compiler validation realization.

`src/v1/stage0/src/bin/bootstrap_witness.rs` — Bootstrap stage source/binary trees and control fixtures; CI timing append is explicit measurement output, not shared application state.

`src/v1/stage0/src/bin/carrier_realization_census.rs` — Explicit CLI census receipt path; retain verification output.

`src/v1/stage0/src/bin/claim_executor.rs` — No production filesystem mutation established after bounded cfg(test) and source-body review: fixture construction, pure collection append/truncate, diagnostic/prose or generated expected-source strings. Retain controls/exclude false positives.

`src/v1/stage0/src/bin/generic_identity_census.rs` — Explicit CLI census receipt path; retain verification output.

`src/v1/stage0/src/bin/interp_recorded_fixture_witness.rs` — Controls for fixture provider, mutate isolated fixture roots and cleanup; retain tests, distinct from recorded_fixture.rs production provider.

`src/v1/stage0/src/bin/parse_witness.rs` — No production filesystem mutation established after bounded cfg(test) and source-body review: fixture construction, pure collection append/truncate, diagnostic/prose or generated expected-source strings. Retain controls/exclude false positives.

`src/v1/stage0/src/cli_run.rs` — Local source overlay, compiler measurement/bootstrap invocation output, Unix socket unlink/chmod; stdout/socket writes and Vec append/truncate are not storage. Process workspace root/source-root/serve command provide binding.

`src/v1/stage0/src/cli_run/checker_dependency.rs` — Explicit checker input record output path, not inferred database root; retain compiler observation artifact interface.

`src/v1/stage0/src/cli_run/demand_acquire.rs` — No production filesystem mutation established after bounded cfg(test) and source-body review: fixture construction, pure collection append/truncate, diagnostic/prose or generated expected-source strings. Retain controls/exclude false positives.

`src/v1/stage0/src/cli_run/derived_row_roster.rs` — No production filesystem mutation established after bounded cfg(test) and source-body review: fixture construction, pure collection append/truncate, diagnostic/prose or generated expected-source strings. Retain controls/exclude false positives.

`src/v1/stage0/src/cli_run/emit_host.rs` — No production filesystem mutation established after bounded cfg(test) and source-body review: fixture construction, pure collection append/truncate, diagnostic/prose or generated expected-source strings. Retain controls/exclude false positives.

`src/v1/stage0/src/cli_run/emitted_crate_workspace_host.rs` — Emitted crate materialization and scoped root removal; explicit compiler artifact adapter.

`src/v1/stage0/src/cli_run/entry_resolve.rs` — No production filesystem mutation established after bounded cfg(test) and source-body review: fixture construction, pure collection append/truncate, diagnostic/prose or generated expected-source strings. Retain controls/exclude false positives.

`src/v1/stage0/src/cli_run/native_lane_runner.rs` — M4 census-scoped incomplete/completed run records at workspace/target/census-runs; all other crate/source/native control fixtures and pipe/stdout writes stay local. See durable_resources.

`src/v1/stage0/src/cli_run/native_product_cache.rs` — M3 native-product cache and append-only diagnostic receipt; see durable_resources, not excluded by Rust placement.

`src/v1/stage0/src/cli_run/provider_acquire.rs` — No production filesystem mutation established after bounded cfg(test) and source-body review: fixture construction, pure collection append/truncate, diagnostic/prose or generated expected-source strings. Retain controls/exclude false positives.

`src/v1/stage0/src/cli_run/reach_base_standings.rs` — No production filesystem mutation established after bounded cfg(test) and source-body review: fixture construction, pure collection append/truncate, diagnostic/prose or generated expected-source strings. Retain controls/exclude false positives.

`src/v1/stage0/src/cli_run/required_ci_measurement.rs` — Required CI measurement receipt output path; retain verification artifact interface.

`src/v1/stage0/src/cli_run/rostered_row_join.rs` — No production filesystem mutation established after bounded cfg(test) and source-body review: fixture construction, pure collection append/truncate, diagnostic/prose or generated expected-source strings. Retain controls/exclude false positives.

`src/v1/stage0/src/cli_run/user_work_scope.rs` — Pure String path truncation in observed_cgroup_bound_in; test-only fixture writes remain isolated controls, no production filesystem write at this hit.

`src/v1/stage0/src/codex_app_server_stdio_session.rs` — C2 durable command ID pre-submission write; direct stdio session adapter shares supervisor identity resource. Pipe handles are not filesystem persistence.

`src/v1/stage0/src/compiler_tests.rs` — No production filesystem mutation established after bounded cfg(test) and source-body review: fixture construction, pure collection append/truncate, diagnostic/prose or generated expected-source strings. Retain controls/exclude false positives.

`src/v1/stage0/src/cssl_seed_linked_closure_assembly.rs` — No production filesystem mutation established after bounded cfg(test) and source-body review: fixture construction, pure collection append/truncate, diagnostic/prose or generated expected-source strings. Retain controls/exclude false positives.

`src/v1/stage0/src/emitted_closure_compile_host.rs` — Exclusive temporary probe root, mutation/restore controls and not-selected identity receipt; retain verification/compiler evidence adapter.

`src/v1/stage0/src/evaluation_budget_consequence_falsifier_host.rs` — HTTP socket write plus controlled worktree/source perturbation/restore; retain verification effect.

`src/v1/stage0/src/gunbc_file_transport_generated.rs` — Filesystem backend projection from extdeps.filesystem.rust_realization: staged fsync + hard-link create-only publication and cleanup; retain backend, not domain adopter.

`src/v1/stage0/src/main.rs` — Compiler output file emission to explicit output_dir; stdout return channel is not storage; retain command output API.

`src/v1/stage0/src/memory_governor.rs` — Kernel cgroup interface /sys/fs/cgroup: mkdir/remove/write memory.max/cgroup.procs. Retain ephemeral process-control effect; not durable application data.

`src/v1/stage0/src/namespace_baseline.rs` — Revision source closure archive/extraction and cleanup under scoped temporary checkout roots; retain compiler input realization.

`src/v1/stage0/src/partition_crate_boundary_host.rs` — Materializes emitted crate files for boundary validation; retain compiler verification artifact.

`src/v1/stage0/src/process_group.rs` — No production filesystem mutation established after bounded cfg(test) and source-body review: fixture construction, pure collection append/truncate, diagnostic/prose or generated expected-source strings. Retain controls/exclude false positives.

`src/v1/stage0/src/recorded_fixture.rs` — M4 recorded fixtures from claim_batch --record --fixture-store path; persistent operation/input/response/freshness store. See durable_resources.

`src/v1/stage0/src/required_regen_host.rs` — M3 formatter cache + M4 convergence rollback journal, backups and manifest; emitted/normalized sources and installation remain local. See durable_resources.

`src/v1/stage0/src/target_invocation_host.rs` — Creates source fixtures for census/cost controls in selected scratch root; retain verification fixture effects.

`src/v1/stage0/src/v1_compiler_emit_rust.rs` — Compiler-authored Rust file-transport backend fragments (and stdout/list operations); retain at extdeps realization/emit boundary, not standalone durable domain store.

`src/v1/stage0/src/v1_compiler_infer.rs` — No production filesystem mutation established after bounded cfg(test) and source-body review: fixture construction, pure collection append/truncate, diagnostic/prose or generated expected-source strings. Retain controls/exclude false positives.

`src/v1/stage0/src/v1_compiler_infer_method.rs` — No production filesystem mutation established after bounded cfg(test) and source-body review: fixture construction, pure collection append/truncate, diagnostic/prose or generated expected-source strings. Retain controls/exclude false positives.

`src/v1/stage0/src/v1_compiler_runtime_rust.rs` — Compiler-authored Rust file-transport backend fragments (and stdout/list operations); retain at extdeps realization/emit boundary, not standalone durable domain store.

`src/v1/stage0/src/v1_interpreter.rs` — M3 native compile workspace cache/publication lock/ready marker; extdeps file transport backend retains Write/Delete; shell detached log files retain supplied output adapter; Vec append/truncate excluded. See durable_resources.

`src/v1/stage0/src/v1_rt.rs` — Filesystem backend projection of same extdeps authority plus pure list.append false positives; retain backend.

`src/v2/extdeps/languages/rust_fixtures.dag` — No production filesystem mutation established after bounded cfg(test) and source-body review: fixture construction, pure collection append/truncate, diagnostic/prose or generated expected-source strings. Retain controls/exclude false positives.
