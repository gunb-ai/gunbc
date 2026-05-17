#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

verilog_head="$(mktemp)"
llvm_head="$(mktemp)"
ptx_head="$(mktemp)"

cat >"$verilog_head" <<'EOF'
// src/v4/extdeps/languages/verilog.dag
//
// Scope: Verilog HDL (IEEE 1364-2005) — modules, ports, wire/reg, assign, always, instances.
// Anchor: IEEE 1364-2005 (L-2 versioned authority); informal web pages are context only.
//
// Owns:
//   - Verilog grammar (module / port / wire / reg / assign / always /
//     instantiation) as declarative data (B2 — generic walker, not code)
//   - Concurrency expressed as effect-typed Bind composition (per IN-B;
//     NOT a new behavior — that would be a C1 STOP)
//   - Inhabitance: bit-vectors (wire/reg [N:0]) per std/algebra.dag
//   - Cost realization: per-construct cost on a Verilog target
//
// Consumes:
//   - std/node.dag, std/algebra.dag, std/primitive.dag
//   - extdeps/coordination.dag (concurrency via effect-typed Bind — IN-B)
//
// Status: T-4.9 PASS (IN-B probe: no 6th behavior; no core/pipeline change).
//   Scaffold `Consumes` lines are verbatim; stale citations + narrative: docs/v4-dag-rationale.md.
//
//
module v4.extdeps.languages.verilog


// Imports: `Symbol` via std/node.dag only — deferrals: docs/v4-dag-rationale.md.
import v4.std.node { Symbol }


// Relocated modeling notes + Practice-4 banner: docs/v4-dag-rationale.md.


EOF

cat >"$llvm_head" <<'EOF'
// src/v4/extdeps/languages/llvm_ir.dag
//
// Scope: LLVM IR (LangRef-pinned, e.g. LLVM 18) — modules, functions, blocks, SSA, instructions, types.
// Anchor: LLVM Language Reference Manual (L-2: spec, not a library build).
//
// Owns:
//   - LLVM IR grammar as declarative bidirectional data (B2-OMNI):
//     module / function / basic-block / instruction / SSA value / type
//   - SSA + dominance modeled structurally (NOT a 6th behavior — the
//     value graph is Node edges; phi is a Node, per IN-B discipline)
//   - Inhabitance: LLVM integer/float/vector/pointer types per std/algebra.dag
//   - Cost realization: per-instruction cost on an LLVM target
//
// Consumes:
//   - std/node.dag, std/algebra.dag, std/primitive.dag
//
// Status: T-4.12 PASS (B2-OMNI probe). Header `Consumes` verbatim; staleness + narrative: docs/v4-dag-rationale.md.
//
//
module v4.extdeps.languages.llvm_ir


// Imports: `Symbol` via std/node.dag only — algebra deferral: docs/v4-dag-rationale.md.
import v4.std.node { Symbol }


// Relocated modeling notes: docs/v4-dag-rationale.md.


EOF

cat >"$ptx_head" <<'EOF'
// src/v4/extdeps/languages/ptx.dag
//
// Scope: PTX ISA layer (DECISIONS.md L-3) — SIMT grid/block/thread, memory hierarchy, kernels, barriers.
// Anchor: NVIDIA PTX ISA 8.5 (HTML + PDF pinned in prior header text; full URLs in docs/v4-dag-rationale.md).
//
// Owns:
//   - PTX coordinate classifiers as closed structural carriers:
//     StateSpace (memory hierarchy — ParamScope / SharedScope enforced per ISA),
//     BarrierScope, ThreadAxis, ThreadCoordSource,
//     BitsWidth / IntegerWidth / FloatWidth, RegisterScalarKind
//   - PTX value shapes: RegisterScalar; Dim3 / ThreadCoord; ThreadHierarchyShape
//   - PTX cost-realization carrier: PtxCost
//   - IN-B mapping narrative (relocated): docs/v4-dag-rationale.md
//
// Owned ELSEWHERE (do not duplicate here):
//   - LanguageModel substrate (T-4 bundle)
//   - PTX scalar algebra instance-values (std/integer.dag / std/machine.dag / std/float.dag — T-3 A3)
//   - SIMT effect-typed Bind carrier (extdeps/coordination.dag — T-4.8)
//
// Consumes:
//   - nothing — closed enums + Conj records only; `Int` kernel-ambient.
//
// Status: T-4.14 PASS (no 6th behavior). Header contract + long rationale: docs/v4-dag-rationale.md.
//
//
module v4.extdeps.languages.ptx


// Relocated modeling notes: docs/v4-dag-rationale.md.


EOF

# verilog: drop lines 1-280, prepend new head, keep from line 281
{ cat "$verilog_head"; tail -n +281 src/v4/extdeps/languages/verilog.dag; } >src/v4/extdeps/languages/verilog.dag.tmp
mv src/v4/extdeps/languages/verilog.dag.tmp src/v4/extdeps/languages/verilog.dag

# llvm_ir: drop lines 1-169, keep from 170 (FloatKind section)
{ cat "$llvm_head"; tail -n +170 src/v4/extdeps/languages/llvm_ir.dag; } >src/v4/extdeps/languages/llvm_ir.dag.tmp
mv src/v4/extdeps/languages/llvm_ir.dag.tmp src/v4/extdeps/languages/llvm_ir.dag

# ptx: drop lines 1-238, keep from 239
{ cat "$ptx_head"; tail -n +239 src/v4/extdeps/languages/ptx.dag; } >src/v4/extdeps/languages/ptx.dag.tmp
mv src/v4/extdeps/languages/ptx.dag.tmp src/v4/extdeps/languages/ptx.dag

rm -f "$verilog_head" "$llvm_head" "$ptx_head"
