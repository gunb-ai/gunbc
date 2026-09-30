# Readiness-store substrate qualification

At source ba0ca2fefd6a6e2ec80883b3b923f84c1cfe039e, the existing controller
installer provisions the readiness directory through its root-owned directory
operation and requires an ownership readback. The store and installer share one
path authority. This creates no readiness records and establishes neither
sanitation nor permission to allocate.

The full serve entry reaches readiness under the existing 12 GiB/no-swap budget
in 260.456 seconds, with peak memory 12,573,483,008 bytes and zero OOM events.
The attached receipt binds the source and binary. The temporary loopback server
was stopped by the qualification helper; this was not a deployment.

Live installer execution and initial-slot commissioning remain outstanding.
