# Executor settlement and approved integrated qualification — 2026-09-30

The operator approved a temporary 12 GiB/no-swap scope for integrated qualification. Production limits, runner services and srv1 were not changed.

## Source change

The existing commissioning finalizer now records accepted strong readback and canonical credential absence in the protected executor journal before releasing its exact mutation generation. After release it records `ExecutorReleaseObserved`. A lost publication response remains an explicit recovery obligation; the finalizer cannot report completion without reading back the matching transition. The protected journal publisher admits only the controller invocation binder and the finalizer's settlement function.

Supply admission additionally requires `ExecutorGenerationCommitted` for the exact commissioning plan. A free mutation key and a committed controller journal alone cannot expose the slot during the fleet-publication gap.

These changes do not implement the still-missing production helper, commissioning fleet scope or complete shared-generation crash recovery. No installed executor, readiness publication, reservation or VM is claimed. HOLD remains.

## Evidence

Seven readback/recovery controls passed on the final dependency snapshot under 6 GiB/no swap. The closure includes the real finalizer, protected journal adapter, controller and supply admission. These checks typecheck actual effectful adapters but do not exercise privileged persistence or live commissioning.

The first full fleet run under the approved 12 GiB scope completed with source resolution failure, not OOM. It reported maximum RSS 11,445,956 KiB. Its root diagnostic was unresolved `KvmObservedScreen` in `gunbc.host_standup_assimilation_deduction`, followed by effect-summary errors from callers. The module lacked an import for the existing type in `gunbc.os_install_deduction`; this checkpoint adds that import. A fresh full-suite dependency snapshot was created for the rerun.

Compiler: existing allocation-vertical-resume release binary, SHA-256 `717e9b4159dc7bc722e94a235325e6760f5acd246e63d8bed14b828dac970243`. Local snapshot evidence is not exact-head rebuilt CI or wet acceptance.

The next full run resolved the KVM type and exposed four further missing references in that module: the pre/post-install lease tables, the installation stall budget and the lease-subsumption function. Their existing owners are now imported explicitly. The same import omissions remain in fetched `origin/main@8e212f9af5db42cec0f3470d99e9db0cc36b7d99`; this is a dependency repair, not a parallel policy change. A 135-module isolated witness closure also refused because the older host-standup code relies on additional ambient names from the full tree; its failure is preserved rather than counted as qualification. The full suite is the deciding check.

Final full-fleet result: **87 PASS, zero FAIL, exit 0**, over 1,910 unchanged modules, under the approved 12 GiB/no-swap scope. Reported maximum RSS: 11,578,228 KiB. The scope was `fleet-generation-imports-12g-20260930.scope`. This establishes the repaired fleet witness closure locally; it is not the complete required CI floor, a rebuilt exact-head compiler, commissioning, or VM acceptance.
