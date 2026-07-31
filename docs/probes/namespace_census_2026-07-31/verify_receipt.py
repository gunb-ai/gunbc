#!/usr/bin/env python3
"""Verify every derived receipt against the single summary authority."""

import argparse
import hashlib
import json
import pathlib

from receipt_common import load_summary


def load(path: pathlib.Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def require_equal(label: str, actual, expected) -> None:
    if actual != expected:
        raise SystemExit(f"{label} drift:\nexpected {expected!r}\nactual   {actual!r}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("summary", type=pathlib.Path)
    parser.add_argument("compiler_binary", type=pathlib.Path)
    parser.add_argument("synthetic_root", type=pathlib.Path)
    parser.add_argument("parser_result", type=pathlib.Path)
    parser.add_argument("provider_result", type=pathlib.Path)
    parser.add_argument("grouping_result", type=pathlib.Path)
    parser.add_argument("textual_result", type=pathlib.Path)
    args = parser.parse_args()

    summary = load_summary(args.summary)
    compiler = args.compiler_binary.read_bytes()
    require_equal("compiler binary bytes", len(compiler), summary["inputs"]["compiler_binary_bytes"])
    require_equal("compiler binary SHA-256", hashlib.sha256(compiler).hexdigest(),
                  summary["inputs"]["compiler_binary_sha256"])
    require_equal("synthetic-root SHA-256", hashlib.sha256(args.synthetic_root.read_bytes()).hexdigest(),
                  summary["inputs"]["synthetic_root_sha256"])

    parsed = load(args.parser_result)
    compiler_expected = summary["compiler_authoritative"]
    require_equal("diagnostic classification", parsed["classification"], compiler_expected["classification"])
    require_equal("classification sum", parsed["classification_sum"], compiler_expected["classification_sum"])
    require_equal("compiler diagnostic total", parsed["compiler_reported_hard_diagnostics"],
                  compiler_expected["compiler_reported_hard_diagnostics"])
    require_equal("diagnostic header count", parsed["header_lines"],
                  compiler_expected["header_lines_excluded_from_diagnostics"])
    require_equal("raw log SHA-256", parsed["raw_log_sha256"], summary["inputs"]["raw_log_sha256"])

    provider = load(args.provider_result)
    provider_expected = summary["regex_sensitivity_scenarios"]
    for name in ("category_strict_regex_scenario", "category_agnostic_regex_scenario"):
        require_equal(name, provider[name], provider_expected[name])

    grouping = load(args.grouping_result)
    grouping_expected = summary["human_or_inferred_grouping"]
    require_equal("ambiguity decision counts", grouping["decision_counts"],
                  grouping_expected["decision_counts"])
    require_equal("ambiguity occurrence counts", grouping["occurrence_counts"],
                  grouping_expected["occurrence_counts"])
    require_equal("ambiguity decision total", len(grouping["decisions"]),
                  grouping_expected["ambiguous_variant_decisions"])
    require_equal("ambiguity occurrence total", sum(grouping["occurrence_counts"].values()),
                  grouping_expected["ambiguous_variant_occurrences"])

    textual = load(args.textual_result)
    textual_expected = summary["reproducible_textual_classification"]
    require_equal("textual partition", textual["counts"], textual_expected["counts"])
    require_equal("textual semantic-visibility note", textual["semantic_visibility_note"],
                  textual_expected["semantic_visibility_note"])
    require_equal("textual input total", textual["synthetic_root_ambiguity_diagnostics"],
                  textual_expected["synthetic_root_ambiguity_diagnostics"])
    require_equal("textual/compiler input total", textual["synthetic_root_ambiguity_diagnostics"],
                  compiler_expected["classification"]["ambiguous_variant_synthetic_root_diagnostic"])
    print("namespace census receipt verified")


if __name__ == "__main__":
    main()
