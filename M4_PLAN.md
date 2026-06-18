M4 PLAN — eval_service_call hermetic-realization fold (PLAN-FIRST, no code touched)

Verified current state (read, not assumed):
- eval_service_call lives at v1_interpreter.rs:3532 (not 3360; line drift, same fn). Its hermetic
  branch (3565-3581) decides replay DIRECTLY from ExecutionMode + fixture_store:
  Hermetic + Some(store) -> store.lookup(key, inputs_hash, inputs_json) -> value_from_fixture_json (fail-closed on miss/stale).
  Hermetic + None -> eval_mock_response(op_node) -> evaluates the first inline mock_* property as a record literal (v1_interpreter.rs:4528).
  It NEVER reads the published mock corpus. This is the model-not-in-runtime parallel ledger.
- The published corpus (PublishedMockCase rows: operation_key + case_id, structural keys ONLY, payloads in RecordedFixture) exists ONLY in the v2 tree: v2/extdeps/filesystem/mock_corpus.dag and v2/extdeps/test/mock_corpus.dag. mock_totality_lens (v2/lens/mock_totality.dag) reads it at compile time; its RED-on-omission witnesses execute. The v1 runtime loads dsl/ modules, NOT src/v2 — so today the runtime cannot even SEE this corpus.
- Blast radius of the inline path: 50 mock_response props across 16 dsl/extdeps files (git 9, github/pulls 7, gcp 5, shell 4, ...). Only 2 layers (filesystem, test/http_pilot) publish a corpus, and only in v2. So eval_mock_response is heavily load-bearing for the OTHER 14 layers.
- No HermeticRealization decision model exists yet anywhere (grep empty).

THE FOLD, framed as §2 Realization (content-addressed pure-spec -> host-effect, one kernel N handlers):
- pure-spec  = the published mock corpus, List<PublishedMockCase> (WHICH operation/case is hermetically realizable). Closed, declared, structural keys only.
- host-effect / named irreducible kernel = the RecordedFixtureStore lookup keyed by (operation_key, content_hash(inputs)). This is the SINGLE payload authority and the only genuine I/O — it stays.
- decision  = "is this call hermetically realizable?" == "is operation_key a published member of the corpus?" A PURE fold over the model — the SAME read the M2 lens performs.

So eval_service_call's hermetic branch folds to: read-the-model (membership, pure) THEN realize-via-store (kernel, fail-closed on miss). Everything between dissolves.

ANSWERS TO THE 5 QUESTIONS

(1) How does eval_service_call come to READ the published corpus as the SINGLE procedure the M2 lens ALSO reads?
The corpus is declared data of type PublishedMockCase. The interpreter already holds all typed modules (InterpContext.modules) and already evaluates data items to Values (data_cache). So the runtime CAN resolve any data ...published_mock_corpus: List<PublishedMockCase> in its loaded module set into a membership set keyed by operation_key, and gate the hermetic decision on membership. ONE corpus, read by both the compile-time lens and the runtime.
CENTRAL MODELING DECISION (escalate-worthy, needs sign-off): the corpus today is v2-only, but the v1 runtime loads dsl/. "One authority, not two" forbids minting a second dsl-side corpus. The right move is to make the published corpus live where BOTH consumers reach it: alongside the operation-set authority in dsl/extdeps/<layer>/ (the service decl's home), with the v2 mirror dissolving (its header already promises "dissolve when v2 resolves dsl std.hermetic_replay directly"). The M2 lens then re-points at the dsl corpus. Location is a discriminator, not gospel (§3: a fact's home is its layer). I want sign-off on THIS (migrate corpus dsl-ward + dissolve v2 mirror) vs. the alternative (teach the v1 runtime to also load the v2 corpus modules), because it touches the M2 lens/tests and is the linchpin of "one authority."

(2) What dissolves, what is the named irreducible kernel?
Dissolves: the Hermetic + None -> eval_mock_response inline-stub branch (the forked second payload authority), once a layer publishes a corpus AND has fixtures. The ExecutionMode.is_hermetic() check itself STAYS (it is the legitimate mode selector, not a parallel decider) — what dissolves is the "+ fixture_store-or-fabricate" fork beneath it, which collapses to: published -> realize via store (fail-closed on miss); NOT published -> loud fail-closed refusal (§5). eval_mock_response and the 50 inline mock_* props are the migration target.
Named irreducible kernel: RecordedFixtureStore.lookup (the content-hash-keyed host read of a recorded payload). It is the actual effect and cannot be folded away.

(3) The §5 discriminator (runtime decision DISAGREES with published model -> RED, not a truth-table):
A real consumer green-by-execution: run a hermetic service call for an operation that IS in the published corpus and HAS a fixture -> the call realizes successfully (GREEN). Use Filesystem.Read on the filesystem layer.
The discriminating input that goes RED on disagreement: REMOVE that operation's row from the published corpus (or call an operation absent from it). The runtime MUST now fail-closed loudly — even if a stale fixture OR an inline mock_* prop still exists for it. The teeth: a runtime that kept fabricating/replaying for an unpublished op would let runtime-decision and published-model disagree silently; the witness flips RED exactly when membership and the runtime decision diverge. (Mirror of the M2 lens's RED-on-omission, but on the RUNTIME side — proving the runtime reads the same model.) This is an execution witness over eval_service_call, not a Bool table.

(4) Load-bearing / escalate-worthy + smallest first slice:
Load-bearing: v1_interpreter.rs is the runtime seed (DESIGN.md substrate). eval_service_call + eval_mock_response carry a high bar. This plan-first sign-off IS the gate; I will not touch it until signed off.
Escalate-worthy modeling decision: the corpus-location single-authority choice in (1).
Smallest first slice (M4.0): scope to the filesystem layer ONLY (it already publishes a corpus). (a) land the corpus in dsl/extdeps/filesystem/ as the single authority + re-point the M2 lens; (b) teach eval_service_call to resolve that corpus to an operation_key membership set and gate the hermetic decision for filesystem ops on it (published -> store realize; unpublished -> fail-closed); (c) the RUNTIME §5 discriminator witness from (3). Inline mock_* props for the OTHER 14 un-corpused layers stay as an explicitly-marked staged fallback (dissolution trigger named), because dissolving all 50 at once is out of scope and unsafe. eval_mock_response fully dissolves only once every layer publishes a corpus — a later slice, tracked, not M4.0.

(5) Risks + interaction with the v1/v2 store fork:
- RISK A (biggest): the inline path is load-bearing for 14 layers. Mitigation: stage by layer; never delete eval_mock_response in M4.0; gate only corpus-bearing layers; keep a named dissolution trigger on the fallback.
- RISK B: making "unpublished -> fail-closed" live could break existing hermetic tests that run WITHOUT a fixture store and rely on inline mocks. Mitigation: the membership gate only HARDENS ops that are in a published corpus; ops with no corpus keep today's behavior in M4.0.
- RISK C: corpus migration dsl-ward breaks v2 lens imports/tests. Mitigation: same-PR re-point + dissolve v2 mirror; covered by the existing mock_totality witnesses going green against the new location.
- v1/v2 store fork (v1 RecordedFixtureStore on-disk op-keyed vs v2 EffectIoFixtureStore in-mem): DEFER to M5. M4's decision is "read the model for MEMBERSHIP"; it is independent of WHICH store realizes the payload. M4.0 keeps the existing v1 RecordedFixtureStore as the realization handler unchanged. Unifying the two stores is a separate realization-handler concern (the kernel has N handlers; M4 does not pick between them).

REQUESTED SIGN-OFF DECISIONS:
- D1: corpus single-authority location — migrate to dsl/extdeps/<layer>/ + dissolve v2 mirror + re-point M2 lens (my recommendation), vs. teach v1 runtime to load v2 corpus.
- D2: M4.0 scope = filesystem layer only, inline path retained for the other 14 (recommended) vs. wider.
- D3: confirm store-fork deferral to M5.
I will not touch v1_interpreter.rs or any substrate file until these are signed off.
