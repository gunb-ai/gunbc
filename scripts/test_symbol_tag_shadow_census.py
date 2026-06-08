#!/usr/bin/env python3
"""Self-test for scripts/symbol_tag_shadow_census.py."""

from __future__ import annotations

import importlib.util
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/symbol_tag_shadow_census.py"


def load_census_module():
    spec = importlib.util.spec_from_file_location("symbol_tag_shadow_census", SCRIPT)
    if spec is None or spec.loader is None:
        raise SystemExit("failed to load symbol_tag_shadow_census.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main() -> None:
    census = load_census_module()
    text = """
module v4.test.symbol_tag_shadow_census

type BridgeProbe
  = ProbeAlpha { x: Int }
  | ProbeBeta { y: Int }

data probe_alpha_tag: Symbol = probe_alpha_tag
data probe_beta_tag: Symbol = probe_beta_tag

fn real_discriminant(v: BridgeProbe) -> Symbol {
  match v {
    ProbeAlpha { x: _ } => probe_alpha_tag
    ProbeBeta { y: _ } => probe_beta_tag
  }
}

fn payload_projection(v: BridgeProbe) -> Symbol {
  match v {
    ProbeAlpha { x: c } => c
    ProbeBeta { y: c } => c
  }
}

fn field_label_shadow_discriminant(v: BridgeProbe) -> Symbol {
  match v {
    ProbeAlpha { probe_alpha_tag: _ } => probe_alpha_tag
    ProbeBeta { probe_beta_tag: _ } => probe_beta_tag
  }
}

fn dotted_projection(v: BridgeProbe, projection: Projection) -> Symbol {
  match v {
    ProbeAlpha { x: _ } => projection.alpha.surface
    ProbeBeta { y: _ } => projection.beta.surface
  }
}
"""
    tags, bridges, shadow_syms, pin_tests = census.analyze("fixture.dag", text)
    if tags != {"probe_alpha_tag", "probe_beta_tag"}:
        raise SystemExit(f"unexpected tags: {tags}")
    expected_bridges = [
        ("real_discriminant", 2, {"probe_alpha_tag", "probe_beta_tag"}),
        ("field_label_shadow_discriminant", 2, {"probe_alpha_tag", "probe_beta_tag"}),
    ]
    if bridges != expected_bridges:
        raise SystemExit(f"unexpected bridges: {bridges}")
    if shadow_syms != {"probe_alpha_tag", "probe_beta_tag"}:
        raise SystemExit(f"unexpected shadow symbols: {shadow_syms}")
    if pin_tests != 0:
        raise SystemExit(f"unexpected pin test count: {pin_tests}")

    print("OK: symbol_tag_shadow_census self-test passed.")


if __name__ == "__main__":
    main()
