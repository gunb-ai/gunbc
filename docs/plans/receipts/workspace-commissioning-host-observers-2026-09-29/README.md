# Host-observation and controller source checks

Both runs used binary SHA256 `717e9b4159dc7bc722e94a235325e6760f5acd246e63d8bed14b828dac970243`, MemoryMax=6G, MemorySwapMax=0, MALLOC_ARENA_MAX=2. Both exited zero. The manifests identify byte-identical copied source; all listed source files were rechecked against the working tree after completion.

- `directive-controls`: six executable claims passed. Only its actual 496-module import closure was evaluated; copying the controller into the pool does not qualify that controller.
- `controller-composition`: explicit import of the installed controller and compilation of all 777 modules, then `workspace_commissioning_typecheck` returned ExitSuccess. Max RSS 5,216,184 KiB. No host effects were invoked.

These are source qualification, not privileged commissioning, concurrent acquisition, full serving startup, or live VM acceptance. The reviewed fleet commissioning plan/apply and its exclusion/recovery boundary remain unfinished. No commissioning installation is authorized by these receipts alone.

An earlier manual invocation omitted `--function workspace_commissioning_typecheck`; it completed typechecking but failed with NoSuchFunction main. The corrected composition run above is the successful execution receipt.

The subsequent sanitation run passed all ten claims under the same 6 GiB/no-swap settings, exit zero, max RSS 4,047,092 KiB. Its separate manifest covers late process appearance, unreadable TAP ownership, and an actual mountinfo-shaped old-attempt mount. These remain supplied-boundary controls, not a live sanitation receipt.

Full serve startup at `c2528d9ef3451d53dde3a0a47523e9d712e0016c` reached its own loopback listening announcement and accepted a TCP connection in 240.323 seconds under 12 GiB/no swap. Peak 12,579,467,264 bytes; OOM counters zero. The owned process was stopped after checking. This establishes integrated startup, not HTTP allocation behavior or live deployment. Only documentation/receipt files changed during this check.
