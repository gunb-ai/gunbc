### 2026-03-21 — `v2-compiler-convergence`

- Deleted `src/v2/04a_normalize.dag` and removed the extra
  reconcile→normalize→emit boundary. The stage introduced unused and
  lossy fact tables (`func_facts`, `enum_facts`, `field_facts`) that
  were not consumed by any emitter, and some entries were already
  degraded (shadowed bindings collapsed by name, match-arm context lost,
  placeholder function classifications). Emit now consumes the existing
  reconcile boundary directly again until an exact, authoritative
  emitter-facing index is needed.

