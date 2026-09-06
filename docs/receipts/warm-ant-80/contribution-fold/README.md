# Unvalidated contribution-fold draft

This side ref preserves the unpushed follow-up to PR #10600 at a0efb6f57f. It is a recovery packet, not a production collector, gate, or claim of completed provenance coverage. The two .dag files on this ref contain the latest paired-algebra draft; draft.patch records their delta from that PR head.

The scripts preserve the exact source payloads used by the probes. They create a clean detached worktree at ddc79f41024453fe4a643a11f6711603fe968f9f and assert both revision and cleanliness before building claim_batch. They then install the embedded source payloads and report their digests before invoking the controls. Run the scripts inside a BuildBuddy guest sized with CTRL_BUILD_RUNNER_EXEC_PROPERTIES=EstimatedMemory=24GB; their kernel-binding preconditions require the memory cgroup capability measured by this session. They bind memory.max=21474836480 and memory.swap.max=0. These scripts are not intended for a local session build.

Retained producing invocations:

- Earlier selective draft: https://app.buildbuddy.io/invocation/ff540871-3c57-43bd-bebe-ae1d0bdadb54
- Latest paired-algebra draft: https://app.buildbuddy.io/invocation/747cf3e9-6435-46b4-ba15-8556ef463944
- Prior PR-head model observation: https://app.buildbuddy.io/invocation/2c9f7880-5775-40bb-9bda-b159456b5826

The latest paired-algebra probe refused while resolving origin.decl_name against Empty at emission_provenance_test.dag:179:40. No witness executed. The earlier selective draft refused at the corresponding expression on line 176; the line movement is not evidence of a second defect. The identical executor digest in the paired-algebra and prior PR-head observations rules out executable-byte drift between those two probes; their source populations differ.

Latest paired probe digests:

- claim_batch: 07a950c35a1e665791d2a69afc1ac357a76fcfab04bbfc4ce776d392e082c94c
- dag/std/emission_provenance.dag: c1e21a0ae6c647d9f87766c4c8b378717cb64037cfb4e8b76a2dd8da1c2c78f4
- dag/test/claim/emission_provenance_test.dag: b64c03bc9348cfee04aad67895c9d7c35e707bac162960404919eff022dd0637

Keep the pairing control as regression evidence for delegation to the shared layer readers. Await L4's stabilized repaired seed, then rerun with the source payload unchanged and separately record the new executor identity. No caller workaround, independent compiler repair, regeneration, or placement is authorized by this packet.
