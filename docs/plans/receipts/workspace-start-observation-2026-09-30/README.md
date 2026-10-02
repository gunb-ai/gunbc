# Named start observation controls

Four start-observation controls passed over a 129-module byte-identical dependency snapshot, maximum RSS 695,868 KiB. Seven existing systemd property-boundary controls passed over 27 modules, maximum RSS 119,620 KiB. Both ran under MemoryMax=6G and MemorySwapMax=0. The source hashes were rechecked after completion.

Compiler: existing allocation-vertical-resume release binary, SHA-256 `717e9b4159dc7bc722e94a235325e6760f5acd246e63d8bed14b828dac970243`. These are focused source controls, not exact-head integrated CI.

The new observer requires named fields and distinguishes known-empty Job from absent Job. It refuses pending work, nonzero PID, unloaded units, wrong units, duplicate and missing fields as quiescence. It does not create start permission or prove that a previous submitting process has stopped.

The actual observer was not executed against srv1-13; no unit or runner service was changed. No start intent, commissioning journal, readiness, reservation or guest was created. Guarded fleet execution and its start/restart lifetime remain unresolved, as recorded in the executor decision document.
