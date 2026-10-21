# Post-fix versioning rows (inclusion-gate receipt for clever-carp-254)

Run: driver captured from instrument prep at head cfef56c261a3 (branch
session/quiet-carp-503-census-hermeticity, PR #14710 — substrate roster in
src/v2/compiler/source_root_eval.dag). Driver sha256 in capture log:
the run recorded GOT at size 28970672, V1 rc=0.
Invocation: <driver> census-scoped //extdeps/git/versioning:all dag src/v2
(post-fix head tree; population = named subject + declared substrate roster).
File: post-fix-versioning-rows.tsv — raw stdout of the run: leading JSON lines
(census_scope / census_residual records), then tab rows (binding, binding_partial,
binding_unbound, binding_subject, binding_partial_module, binding_unbound_module...).
Gate note (quiet-carp-503's read, YOUR gate to run per your rule): vs base-versioning.tsv
(68 rows), 64 keys (kind,module,name) included as-is modulo the +1 line shift from the wave
edit at ~line 48; chars x2 now binding_partial declared std.host_text.chars 79:32/87:32 (shifted)
— admitted, matching sunny-bee's EVIDENCE.txt join-admission lines; none x2 no longer binding_unbound,
resolved via std.optional.Optional.Absent generated rows at 80:15/88:15; aggregates balance exactly:
binding_unbound_module 16->12 (-4), binding_partial_module 49->53 (+4) — 4 rows moved unbound->partial,
zero dropped. That is the fix's signature on this module: ambient bindings added, module rows preserved.
