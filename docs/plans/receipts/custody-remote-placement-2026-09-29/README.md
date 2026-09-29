# Remote custody runner placement

Run 36605269073 completed fabric-state credential custody on srv1 through the
modeled administrator SSH transport. Run 36607031635 built its binary but its
custody job remains queued on `[self-hosted,linux,arm64,srv2]`. Read-only local
observation found srv2's five runner instances masked and no Runner.Listener.
No runner was started or unmasked.

The custody implementation already resolves the explicit host input and performs
remote delivery through the fleet SSH authority. Requiring execution on that host
creates an unnecessary bootstrap dependency. The shared job now selects the
existing fleet runner labels only for host_credential_custody_converge; other
shared modes retain their host label. The credential target, WIF and SSH prelude,
row admission, ownership/readback and concurrency are unchanged.

Local witness execution over 1,842 unchanged closure modules was OOM-killed under
6 GiB/no swap before producing a verdict. This is not a pass. CI qualification and
canonical workflow regeneration remain required. No ceiling was increased and
this scheduling repair has not been deployed.

The same run also exceeded 6 GiB with MALLOC_ARENA_MAX=2. Removing the unused
ledger-row-coherence import from ci_spec reduced the copied witness closure from
1,842 to 1,265 unchanged modules, but both that reduced witness and the canonical
workflow emitter still exceeded 6 GiB. No local pass or emitted workflow is claimed.
CI must qualify the source and generate its workflow; no ceiling was increased.
