#!/usr/bin/env python3
"""Append grounding-axis columns to continuation table rows (ephemeral)."""

from __future__ import annotations

import re
import sys
from pathlib import Path


def axis_for(path: str, typ: str) -> tuple[str, str, str]:
    if path.endswith("logic.dag") and typ == "Bool":
        return (
            "🟢-GROUNDED",
            "`data bool_boolean_algebra` supplies executable lattice+complement laws per `True|False` arm.",
            "—",
        )
    if path.endswith("typescript.dag"):
        return (
            "🟡-UNGROUNDED-WITH-PLAN",
            f"`{typ}` names ECMA surface lanes without TS→`v4.std` numeric morphism TestClaim can execute.",
            "`T-4` `typescript.dag` fact-bundle (`src/v4/TASKS.md` T-4) + `Int`/`Nat`/`Float` lowering witnesses.",
        )
    if path.endswith("verification.dag"):
        return (
            "🟡-UNGROUNDED-WITH-PLAN",
            f"`{typ}` classifies verifier claims without per-arm executable predicate over `Node`/`Outcome`.",
            "`verification.dag` consumer wiring each arm to `fn`/`data` used by `TestClaim` runner (R3 verification substrate).",
        )
    if path.endswith("testgen.dag"):
        return (
            "🟡-UNGROUNDED-WITH-PLAN",
            f"`{typ}` lacks emit morphism into `TestClaim` spine / expected outcomes.",
            "`T-22` testgen↔eval bind + `lens/testgen.dag` lowering table (`src/v4/TASKS.md`).",
        )
    if path.endswith("coordination.dag"):
        return (
            "🟡-UNGROUNDED-WITH-PLAN",
            f"`{typ}` not yet a law table on `WireContract` correlation/settlement carriers.",
            "`T-4.8` `coordination.dag` consumer + `DECISIONS.md` wire-contract rows.",
        )
    if path.endswith("rust.dag"):
        return (
            "🟡-UNGROUNDED-WITH-PLAN",
            f"`{typ}` arm lacks lowering morphism TestClaim can assert against kernel carriers.",
            "`T-4` rust slice: `RustScalar`/`RustIntWidth` + emit-site morphisms (`src/v4/TASKS.md` T-4).",
        )
    if path.endswith("go.dag"):
        return (
            "🟡-UNGROUNDED-WITH-PLAN",
            f"`{typ}` Go surface tag without Go→std numeric/chan morphism family.",
            "`T-4` `go.dag` fact-bundle + std channel/int carriers.",
        )
    if path.endswith("cpp.dag"):
        return (
            "🟡-UNGROUNDED-WITH-PLAN",
            f"`{typ}` lacks C++ ABI-tied morphism into std machine model.",
            "`T-4` cpp slice + `T-29` ABI substrate (`src/v4/TASKS.md`).",
        )
    if path.endswith("lean.dag"):
        return (
            "🟡-UNGROUNDED-WITH-PLAN",
            f"`{typ}` Lean surface enum without Lean→std proof/term morphism consumer.",
            "`T-4` Lean slice + `extdeps/languages/lean.dag` B-2 obligations (`DECISIONS.md` L-4).",
        )
    if path.endswith("llvm_ir.dag"):
        return (
            "🟡-UNGROUNDED-WITH-PLAN",
            f"`{typ}` IR label without LLVM→`machine_code.dag` semantic morphism.",
            "`T-4.12` `llvm_ir.dag` + `machine_code.dag` memory-order / opcode witness consumer.",
        )
    if path.endswith("machine_code.dag"):
        return (
            "🟡-UNGROUNDED-WITH-PLAN",
            f"`{typ}` ISA-level tag without ISA semantics morphism TestClaim can execute.",
            "`T-4.13` `machine_code.dag` disassembly/encoding witness harness.",
        )
    if path.endswith("ptx.dag"):
        return (
            "🟡-UNGROUNDED-WITH-PLAN",
            f"`{typ}` SIMT tag without PTX→std concurrency morphism.",
            "`T-4.14` `ptx.dag` consumer + SIMT law table (`src/v4/TASKS.md`).",
        )
    if path.endswith("verilog.dag"):
        return (
            "🟡-UNGROUNDED-WITH-PLAN",
            f"`{typ}` HDL tag without verilog→kernel simulation morphism.",
            "`T-4.9` `verilog.dag` B2-OMNI falsification consumer.",
        )
    if path.endswith("python.dag"):
        return (
            "🟡-UNGROUNDED-WITH-PLAN",
            f"`{typ}` Python surface tag without Python→std morphism.",
            "`T-4` `python.dag` fact-bundle.",
        )
    if "formats/spice.dag" in path:
        return (
            "🟡-UNGROUNDED-WITH-PLAN",
            f"`{typ}` deck/netlist sum without SPICE→executable analog/digital witness.",
            "`T-4.10` `spice.dag` + Shape-A emit consumer (`src/v4/TASKS.md`).",
        )
    if "formats/openapi.dag" in path:
        return (
            "🟡-UNGROUNDED-WITH-PLAN",
            f"`{typ}` OpenAPI carrier without OpenAPI→`network.dag`/`coordination.dag` bridge morphism.",
            "`T-4.6` OpenAPI consumer (`DECISIONS.md` T-4.6-P4 HTTP/status/header objects).",
        )
    if "formats/json_schema.dag" in path or "formats/json.dag" in path:
        return (
            "🟡-UNGROUNDED-WITH-PLAN",
            f"`{typ}` JSON/Schema sum without JSON-Schema→typed value morphism harness.",
            "`T-4.6` JSON/JSON-Schema slice (`src/v4/TASKS.md` T-4.6).",
        )
    if "formats/yaml.dag" in path or "formats/toml.dag" in path or "formats/csv.dag" in path:
        return (
            "🟡-UNGROUNDED-WITH-PLAN",
            f"`{typ}` format sum without parse/roundtrip TestClaim morphism.",
            "`T-4.6` corresponding `extdeps/formats/*` consumer.",
        )
    if path.endswith("lens/registry.dag"):
        return (
            "🟡-UNGROUNDED-WITH-PLAN",
            f"`{typ}` registry id without generated registry-row consumer (`INVARIANTS` §P2 staging).",
            "`ROADMAP` T-PB-B / `v4_lens_registry_dag_smoke_test` dissolution plan (`src/v4/TASKS.md`).",
        )
    if "lens/affected_set.dag" in path:
        return (
            "🟡-UNGROUNDED-WITH-PLAN",
            f"`{typ}` frontier shape without affected-set consumer morphism.",
            "`T-21` `lens/affected_set.dag` + CI job selection consumer (`src/v4/TASKS.md`).",
        )
    if path.endswith("algebra.dag"):
        return (
            "🟡-UNGROUNDED-WITH-PLAN",
            f"`{typ}` algebraic sum without law witness table TestClaim can enumerate.",
            "`T-2` algebra harness + law-index rows in `std/algebra.dag` consumers.",
        )
    if path.endswith("cardinality.dag"):
        return (
            "🟡-UNGROUNDED-WITH-PLAN",
            f"`{typ}` witness sum without descent-evidence consumer morphism.",
            "`T-3` cardinality/`Nat` spine + complexity lens consumer (`src/v4/TASKS.md`).",
        )
    if path.endswith("diagnostic.dag"):
        return (
            "🟡-UNGROUNDED-WITH-PLAN",
            f"`{typ}` diagnostic shape without verifier-visible outcome morphism.",
            "`T-3` `std/diagnostic.dag` + verifier harness binding (`verification.dag`).",
        )
    if path.endswith("float.dag"):
        return (
            "🟡-UNGROUNDED-WITH-PLAN",
            f"`{typ}` float sum without IEEE profile morphism TestClaim can assert.",
            "`T-3` `std/float.dag` + binary64 special-value lattice consumer.",
        )
    if path.endswith("network.dag"):
        return (
            "🟡-UNGROUNDED-WITH-PLAN",
            f"`{typ}` URI/HTTP slot sum without wire-level morphism into `coordination.dag`.",
            "`T-3` `network.dag` + `T-4.8` coordination consumer (`src/v4/TASKS.md`).",
        )
    if path.endswith("node.dag"):
        return (
            "🟡-UNGROUNDED-WITH-PLAN",
            f"`{typ}` node/connective sum without canonical `Node` builder TestClaim uses.",
            "`T-1` `std/node.dag` + parser/normalize consumers (`T-6`–`T-8`).",
        )
    if path.endswith("witness.dag"):
        return (
            "🟡-UNGROUNDED-WITH-PLAN",
            f"`{typ}` witness sum without A2/descent evidence consumer morphism.",
            "R3 descent-evidence / cementing harness rows (`docs/r3-program-plan.md` §1.8).",
        )
    if path.endswith("nat.dag"):
        return (
            "🟡-UNGROUNDED-WITH-PLAN",
            f"`{typ}` natural-number sum without Peano/witness consumer TestClaim can run.",
            "`T-3` `std/nat.dag` + mirror perf / cost basis consumers.",
        )
    if "compiler/01_tokenize.dag" in path:
        return (
            "🟡-UNGROUNDED-WITH-PLAN",
            f"`{typ}` lexer sum without tokenization morphism witness in harness.",
            "`T-6` `compiler/01_tokenize.dag` smoke + golden lexeme claims.",
        )
    if "compiler/03_normalize.dag" in path:
        return (
            "🟡-UNGROUNDED-WITH-PLAN",
            f"`{typ}` normalize result sum without normalize-pipeline TestClaim morphism.",
            "`T-8` normalize bundle consumer (`src/v4/TASKS.md`).",
        )
    if "compiler/03_resolve.dag" in path:
        return (
            "🟡-UNGROUNDED-WITH-PLAN",
            f"`{typ}` resolve result sum without scope/resolve TestClaim morphism.",
            "`T-8` resolve bundle consumer (`src/v4/TASKS.md`).",
        )
    if "compiler/02_parse.dag" in path:
        return (
            "🟡-UNGROUNDED-WITH-PLAN",
            f"`{typ}` grammar sum without parse-tree TestClaim morphism.",
            "`T-7` `compiler/02_parse.dag` consumer.",
        )
    if "extdeps/file_system.dag" in path:
        return (
            "🟡-UNGROUNDED-WITH-PLAN",
            f"`{typ}` path sum without host-path law morphism TestClaim can assert.",
            "`T-4.5` `file_system.dag` + `workflow/ci.dag` sandbox envelope (`T-24`).",
        )
    if "extdeps/process.dag" in path:
        return (
            "🟡-UNGROUNDED-WITH-PLAN",
            f"`{typ}` process sum without capture/termination OS morphism.",
            "`T-4.5` `process.dag` consumer.",
        )
    if "extdeps/cpp_abi.dag" in path:
        return (
            "🟡-UNGROUNDED-WITH-PLAN",
            f"`{typ}` ABI enum without data-model morphism tied to `T-29` carriers.",
            "`T-29` C++ ABI / data-model substrate (`src/v4/TASKS.md`).",
        )
    return (
        "🟡-UNGROUNDED-WITH-PLAN",
        f"`{typ}` in `{path}` lacks executable per-arm law decomposition TestClaim can construct today.",
        f"Owning schedule row for `{Path(path).name}` in `src/v4/TASKS.md` (critical path / side branch per graph).",
    )


def patch_line(line: str) -> str | None:
    if not line.startswith("| `src/v4/"):
        return None
    if "DEFERRED (post-§Checkpoint" not in line:
        return None
    if "🟢-GROUNDED" in line or "🟡-UNGROUNDED-WITH-PLAN" in line and line.count("|") > 10:
        # Heuristic: already widened (>=10 pipes for 10 cols)
        parts = [p.strip() for p in line.rstrip("\n").split("|")]
        if len(parts) >= 12:
            return None
    parts = [p.strip() for p in line.rstrip("\n").split("|")]
    if len(parts) < 9:
        return None
    # leading empty
    file_cell = parts[1].strip("`")
    typ_cell = parts[2].strip("`")
    g, why, wait = axis_for(file_cell, typ_cell)
    core = "|".join(parts[1:8])
    return f"|{core}| {g} | {why} | {wait} |\n"


def main() -> None:
    path = Path("docs/audit/coproduct-anemia-inventory.md")
    text = path.read_text()
    out_lines: list[str] = []
    changed = 0
    for line in text.splitlines(keepends=True):
        nl = patch_line(line) if line.endswith("\n") else patch_line(line + "\n")
        if nl is None:
            out_lines.append(line)
        else:
            out_lines.append(nl)
            changed += 1
    if changed == 0:
        print("no continuation rows patched", file=sys.stderr)
        sys.exit(1)
    path.write_text("".join(out_lines))
    print(f"patched {changed} rows")


if __name__ == "__main__":
    main()
