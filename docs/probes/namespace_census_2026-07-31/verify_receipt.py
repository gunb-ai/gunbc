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


def comma(value: int) -> str:
    return f"{value:,}"


def require_doc_claims(doc: str, summary: dict) -> None:
    inputs = summary["inputs"]
    compiler = summary["compiler_authoritative"]
    classification = compiler["classification"]
    scenarios = summary["regex_sensitivity_scenarios"]
    grouping = summary["human_or_inferred_grouping"]
    textual = summary["reproducible_textual_classification"]
    agnostic = scenarios["category_agnostic_regex_scenario"]
    strict = scenarios["category_strict_regex_scenario"]
    claims = {
        "corpus identity": inputs["corpus_commit"],
        "compiler PR": f"PR {inputs['compiler_pull_request']} commit",
        "compiler source identity": inputs["compiler_source_commit"],
        "compiler binary identity": (
            f"{comma(inputs['compiler_binary_bytes'])} bytes, SHA-256\n"
            f"  `{inputs['compiler_binary_sha256']}`"
        ),
        "raw log identity": inputs["raw_log_sha256"],
        "synthetic root identity": inputs["synthetic_root_sha256"],
        "plain compile population": (
            f"reached only {comma(inputs['plain_compile_reached_modules'])} of the\n"
            f"{comma(inputs['declared_corpus_modules'])} declared modules, leaving "
            f"{comma(inputs['plain_compile_uncompiled_modules'])} uncompiled"
        ),
        "synthetic compile population": (
            f"all {comma(inputs['declared_corpus_modules'])} corpus modules "
            f"(the compiler reports {comma(inputs['compiler_reported_modules_with_synthetic_root'])}"
        ),
        "compiler total": f"reported {comma(compiler['compiler_reported_hard_diagnostics'])} hard diagnostics",
        "unresolved-name total": f"{comma(classification['unresolved_name'])} unresolved-name",
        "synthetic ambiguity total": (
            f"{comma(classification['ambiguous_variant_synthetic_root_diagnostic'])} "
            "ambiguous-variant diagnostics"
        ),
        "no-field total": f"{comma(classification['no_field'])} no-field",
        "type-mismatch total": f"{comma(classification['type_mismatch'])} type-mismatch",
        "singleton total": f"{comma(classification['singleton'])} singleton diagnostic shapes",
        "diagnostic-section lines": (
            f"contains {comma(compiler['classification_sum'] + compiler['header_lines_excluded_from_diagnostics'])} "
            "lines in the diagnostic section"
        ),
        "agnostic scenario": (
            f"reports {agnostic['apparent_single_provider_share_percent']}% apparent single-provider\n"
            f"rows and {comma(agnostic['unique_apparent_single_provider_edges'])} unique apparent"
        ),
        "strict scenario": (
            f"reports {strict['apparent_single_provider_share_percent']}% and "
            f"{comma(strict['unique_apparent_single_provider_edges'])} respectively"
        ),
        "consumer mapping zeros": "both unmapped and\nduplicate mapping counts are asserted to be zero",
        "ambiguity decision total": (
            f"maps {comma(grouping['ambiguous_variant_occurrences'])}\nsynthetic-root diagnostics to "
            f"{comma(grouping['ambiguous_variant_decisions'])} decisions"
        ),
        "A grouping": (
            f"A_SELF ({grouping['decision_counts']['A_SELF']}\ndecisions/"
            f"{grouping['occurrence_counts']['A_SELF']} occurrences)"
        ),
        "B grouping": (
            f"B_PARALLEL_TOWER ({grouping['decision_counts']['B_PARALLEL_TOWER']}/"
            f"{grouping['occurrence_counts']['B_PARALLEL_TOWER']})"
        ),
        "C grouping": (
            f"C_TRUE_HOMONYM ({grouping['decision_counts']['C_TRUE_HOMONYM']}/"
            f"{grouping['occurrence_counts']['C_TRUE_HOMONYM']})"
        ),
    }
    for bucket, count in textual["counts"].items():
        claims[f"textual bucket {bucket}"] = f"{count:>3} {bucket}"
    claims["textual partition total"] = (
        f"{textual['synthetic_root_ambiguity_diagnostics']} synthetic-root ambiguity diagnostics"
    )
    for label, claim in claims.items():
        if claim not in doc:
            raise SystemExit(f"receipt prose {label} drift: missing {claim!r}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("summary", type=pathlib.Path)
    parser.add_argument("receipt_doc", type=pathlib.Path)
    parser.add_argument("compiler_binary", type=pathlib.Path)
    parser.add_argument("synthetic_root", type=pathlib.Path)
    parser.add_argument("parser_result", type=pathlib.Path)
    parser.add_argument("provider_result", type=pathlib.Path)
    parser.add_argument("grouping_result", type=pathlib.Path)
    parser.add_argument("textual_result", type=pathlib.Path)
    args = parser.parse_args()

    summary = load_summary(args.summary)
    require_doc_claims(args.receipt_doc.read_text(encoding="utf-8"), summary)
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
