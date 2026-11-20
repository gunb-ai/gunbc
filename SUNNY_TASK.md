# sunny-otter-114 — node adhoc-aec83303-fc5

Compiler for witness runs: /tmp/sunny-build/target/release/gunbc (built from 398136026b3; my sha256 1d31ff0b32f4f7f301ef03a2d4c8d92af9c3b21596b59c30e37fcbee57bac1e7). ONE --claim-run at a time; check `ps aux | grep -- --claim-run` first. Validate DAG edits cheaply with `gunbc compile` not claim-run.

## Coordination (done)
- keen-swift-260 (gate op): I land first; I do NOT reshape work_outcome_json/provide_result_document; RemoteRunClaims.functions wire shape unchanged; only shared file is remote_build_client.dag (different functions).
- zesty-pike-761 (hosted-CI reader): told field names.
- quiet-carp-163: owner-only workdir fix + --host pin both in my scope; will ping + tidy-pike-449 when both land.

## Receipt-body field names (agreed)
- claims: [{name, verdict}], verdict in {"pass","fail","unobserved"}
- claims_truncated: Bool (typed truncation marker, true when bounded)
- serve_binary: {revision, sha256}  (sha256 = recorded digest only, never re-hashed live; null if not recorded)

## Build list
- [ ] Rust claim_batch: --verdicts-out <path> writes {"claims":[{name,verdict}]} from the SAME ClaimOutcome rows that print PASS/FAIL (single authority). verdict helper maps ClaimOutcome->pass/fail/unobserved.
- [ ] work_provider_local RunClaims argv: add --verdicts-out <path>; declare it as an output so it flows into WorkSucceeded.outputs / provider document.
- [ ] remote_build_serve: at line ~242/244 wrap provide_result_document with a serve-local receipt-body builder that adds claims (from the verdicts output, bounded + claims_truncated) and serve_binary {revision from converger-pinned checkout HEAD, sha256 from recorded digest if present}.
- [ ] converger (serving_setup/converge): record the serve binary digest next to the binary (InstallServeBinary records expected_sha256 -> write <serve_binary>.sha256) so serve can read the recorded digest (not re-hash).
- [ ] remote_build_client: (a) optional functions arg -> RemoteRunClaims.functions, typed refusal for unknown; (b) optional --host arg on remote_test_cli + remote_compile_cli pinning roster walk to one rostered host, typed refusal if not in remote_build_serving_hosts, receipt names it; (c) client ensures owner-only working dir (mux dir target/fleet-ssh-mux + anchor) instead of assuming target/ exists; (d) parse claims+serve_binary from body, print PASS/FAIL lines on stdout.
- [ ] witnesses (add to existing test/claim/compute/remote_build_*witness.dag; run each ONCE): red control per new field (claim lines, claims_truncated, serve_binary revision/sha256, host pin, workdir) + one inhabitance claim on real serve route.
- [ ] commit on my branch + ctrl; push; open PR.
- [ ] Rollout srv1/srv3/srv4 via fleet-converge plan->apply, one at a time, cancel queued; message sleek-koi-601 + tidy-pike-449 BEFORE dispatching srv1 group. Then one live remote claim run; paste receipt + stdout (PASS/FAIL lines + receipt names revision).

## STATE at ~02:10Z (pushed to origin/session/sunny-otter-114; fmt-clean)
DONE+committed:
- d3cd48d580a claim_batch --verdicts-out: single-authority verdict doc (header + '<name>\t<verdict>' lines; verdicts lowercase pass/fail/unobserved). cargo check passed.
- 3f2c71613f6 remote_build_client working-dir ensure (job root target/ owner-only via shell.Mkdir.NewOwnerOnly before context). NOT yet locally compile-validated (client-closure compile >7min, near-whole-corpus).
AGREED: manager option A + 3 conditions (typed {name,verdict} records on ProvideResult parsed at compute_provide; serve renders never parses; missing/unparseable=typed refusal not empty; Optional, present only on RunClaims ending). keen: no collision, owns no WorkSucceeded/ProvideResult site, don't reorder WorkOperation arms; I own all those sites. zesty: keep verdicts lowercase-stable. quiet-carp: client file is mine; wants exact --host spelling when it lands.
TODO (large, NOT started): work_request typed ClaimVerdict + ProvideResult Optional verdicts field (ProvideFresh/ProvideAttached sites 2307/2308/2427 + provide_result_with_host_record + accessor patterns; RunClaims argv --verdicts-out <target_dir>/claim-verdicts.json + work_declared_outputs RunClaims=[DeclaredOutput TargetRoot claim-verdicts.json]; parse staged bytes at compute_provide). serve receipt body builder (claims+serve_binary, bounded+claims_truncated). serve_binary identity: converger records digest next to serve binary; serve reads revision (pinned checkout HEAD)+recorded sha256. client functions-arg (delegate unknown-fn refusal to serve claim_batch) + --host pin. witnesses (add to existing test/claim/compute/remote_build_*witness; ONE --claim-run at a time, check ps first). CI compiles model; validate locally is slow.
CONSTRAINTS: DAG compile of this module ~7min near-whole-corpus; witnesses ~14GB serialized; srv1 group CONTENDED (01:20Z ctrl sweep killed a remote-build cargo job on srv1).
