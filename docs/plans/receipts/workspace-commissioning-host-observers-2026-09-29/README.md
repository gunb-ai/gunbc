# Host-observation and controller source checks

Both runs used binary SHA256 `717e9b4159dc7bc722e94a235325e6760f5acd246e63d8bed14b828dac970243`, MemoryMax=6G, MemorySwapMax=0, MALLOC_ARENA_MAX=2. Both exited zero. The manifests identify byte-identical copied source; all listed source files were rechecked against the working tree after completion.

- `directive-controls`: six executable claims passed. Only its actual 496-module import closure was evaluated; copying the controller into the pool does not qualify that controller.
- `controller-composition`: explicit import of the installed controller and compilation of all 777 modules, then `workspace_commissioning_typecheck` returned ExitSuccess. Max RSS 5,216,184 KiB. No host effects were invoked.

These are source qualification, not privileged commissioning, concurrent acquisition, full serving startup, or live VM acceptance. The reviewed fleet commissioning plan/apply and its exclusion/recovery boundary remain unfinished. No commissioning installation is authorized by these receipts alone.

An earlier manual invocation omitted `--function workspace_commissioning_typecheck`; it completed typechecking but failed with NoSuchFunction main. The corrected composition run above is the successful execution receipt.
