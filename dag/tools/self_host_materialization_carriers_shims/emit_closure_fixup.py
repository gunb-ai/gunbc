#!/usr/bin/env python3
"""Post-emit patches for materialization_carriers self-emit closure."""
from __future__ import annotations

from pathlib import Path
import sys

EXTDEPS_EXTRA = (
    "use crate::extdeps_cache_types::{CatalogKeyDerivationFacts, "
    "CacheInterfaceCatalogFacts, CacheInterfaceCatalogPlacement, "
    "CacheInterfaceCatalogIoSemantics};\n"
    "use crate::std_cache_interface::CacheEvidence;\n"
)


def insert_after_use_block(text: str, insert: str) -> str:
    if insert.strip() in text:
        return text
    lines = text.splitlines(keepends=True)
    pos = 0
    for i, line in enumerate(lines):
        if line.startswith("use ") or line.startswith("pub use "):
            pos = i + 1
    lines.insert(pos, insert)
    return "".join(lines)


def patch_rs(path: Path) -> None:
    text = path.read_text()
    if path.name in {"lib.rs", "main.rs", "v1_rt.rs", "v2_std_text.rs"}:
        return
    if path.name in {
        "extdeps_realization_parse_table_memo.rs",
        "extdeps_realization_compile_stage_memo.rs",
    } and "CatalogKeyDerivationFacts" in text:
        if "use crate::extdeps_cache_types" not in text:
            text = insert_after_use_block(text, EXTDEPS_EXTRA)
            path.write_text(text)


def patch_lib(src: Path) -> None:
    lib = src / "lib.rs"
    text = lib.read_text()
    if "pub mod v2_std_text;" not in text:
        text = text.replace(
            "pub mod v2_std_staging;",
            "pub mod v2_std_staging;\npub mod v2_std_text;",
        )
    lib.write_text(text)


def main() -> int:
    src = Path(sys.argv[1])
    for path in sorted(src.glob("*.rs")):
        patch_rs(path)
    patch_lib(src)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
