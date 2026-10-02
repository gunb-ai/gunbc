# Shared workspace mutation exclusion: local source evidence

The recorded dependency snapshot contains 788 byte-identical source modules. All hashes were checked against this working tree after the run. The compiler resolved a 784-file composition; four commissioning readback claims passed under MemoryMax=6G and MemorySwapMax=0. Reported maximum RSS was 5,273,932 KiB.

Compiler: `/home/briansrls/scratch-wt/allocation-vertical-resume/target/release/gunbc`, SHA-256 `717e9b4159dc7bc722e94a235325e6760f5acd246e63d8bed14b828dac970243`. This existing compiler is not an exact-head compiler build. These are source composition and pure-control results, not an integrated CI verdict.

No host mutation, competing-process race, crash recovery, fleet apply, initial readiness, reservation or guest launch was performed. Shared guard recovery and the enclosing fleet apply remain incomplete. A separate nested-compiler census run was stopped before results as it approached the 6 GiB bound; it is not recorded as a pass. Direct compiler probes replace that nested validation grain.

The separate direct-compiler runner completed all three controls: unauthorized fenced release refused with the exact call-admission diagnostic, unauthorized credential cleanup refused with the exact diagnostic, and the permitted public route compiled and returned ExitSuccess. Each ran under its own 6 GiB/no-swap scope. Reproduce with `python3 tools/tests/workspace_commissioning_access_controls.py --binary <compiler> --output target/<fresh-directory>`. The positive control executes no host mutation. Earlier exploratory positive probes used a Bool entry and were refused by the CLI; the recorded successful probe uses ProcessExit.
