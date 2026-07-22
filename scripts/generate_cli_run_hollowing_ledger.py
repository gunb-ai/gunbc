#!/usr/bin/env python3
"""Generate dag/gunbc/cli_run_hollowing_ledger.dag from src/v1/stage0/src/cli_run.rs.

Re-run after any cli_run.rs edit that adds/removes/renames functions so the ledger
stays complete (witness: cli_run_hollowing_ledger_witness_test.dag).
"""

from __future__ import annotations

import re
import sys
from collections import Counter
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
CLI_RUN = REPO / "src/v1/stage0/src/cli_run.rs"
OUT_DAG = REPO / "dag/gunbc/cli_run_hollowing_ledger.dag"

# Narrow file sections (start inclusive, end exclusive) → feature override for prod fns.
NAMED_SECTIONS: list[tuple[int, int, str]] = [
    (24801, 99_999, "SerializationOneGrammar"),
    (21958, 24801, "LensHostBridge"),
    (21539, 21958, "LensHostBridge"),
    (18191, 21539, "NamespaceResolutionSymbolIndex"),
    (2135, 2435, "RegenSelfHost"),
    (2435, 2900, "CompileCleanGate"),
]

# (regex, feature) — first match wins.
RULES: list[tuple[str, str]] = [
    (r"^phase_profile|FloorPhase|PhaseProfile|set_phase", "PhaseProfileInstrumentation"),
    (
        r"wire_|rest_transport|value_to_wire|serialize|deserialize|shell_argv|"
        r"dag_string_escape|dag_manifest|dag_embedded|canonical_json|sort_json|quoted_field_value",
        "SerializationOneGrammar",
    ),
    (
        r"typed_module|typed_cache|parse_cache|resolved_graph_cache|reconcile|content_key|"
        r"content_digest|subject_digest|cache_lookup|cross_process|shared_typecheck|shared_cache|"
        r"module_interface_key|typed_module_key|check_module_source_identity|check_index_module|"
        r"index_get_typed|index_insert_typed|shared_get_typed|index_retention|retention_snapshot|"
        r"enforce_typed_cache|try_reconcile_all_cache|note_interface_hash|note_source_hash|"
        r"finish_resolved_graph|with_typecheck_compute|typecheck_compute_count|shared_caches_",
        "TypedModuleCacheComputationIdentity",
    ),
    (
        r"affected_set|affected_by|touched_path|floor_skip|witness_exclusion|whole_tree_strict|"
        r"whole_tree_resolve|frontier_seed|skip_label|regen_floor|regen_path_affects|"
        r"regen_not_affected|regen_input|regen_collect|regen_build_module|regen_source_roots|"
        r"regen_workspace|compile_clean_scope|compile_clean_shard|compile_clean_touched|"
        r"compile_clean_departed|compile_clean_all_touched|documentation_only_floor|"
        r"selection_disposition|predict_skip|runtime_data_dependency|live_read|carrier_home|"
        r"carrier_closure|module_grain_affected|node_frontier|floor_row_precompute|floor_diff|"
        r"floor_witness_run|entry_file_touched|rerun_frontier|precompute_whole_tree|"
        r"published_mock|repo_paths_match_touched|entry_eligible_for_discovery|"
        r"entry_qualifies_for_skip|discovery_entry_fast_skip|call_floor_kernel_would_skip|"
        r"call_floor_row_would_skip|floor_git_diff|diff_file_matches|parse_unified_diff|"
        r"changed_new_lines|departed_paths|added_paths|path_excluded|floor_discovery_path|"
        r"witness_admission|refuse_reads_live_tree|refuse_unexecuted|collect_unexecuted|"
        r"live_tree_disposition|reads_live_tree|effect_reach|content_contains_host_sink|"
        r"source_has_path_like|source_data_path_literal|declared_source_ref|"
        r"parse_source_ref_storage|parse_named_source_ref|parse_declared_source_ref|"
        r"path_to_module_from_declaration|apply_effect_reach|data_item_declared|"
        r"declared_repo_paths|destructuring_bound_spans|entry_has_edited_test_fn|"
        r"matching_discovery_exclusion|path_matches_any_substring|path_matches_any_subpath|"
        r"witness_budget_policy|newline_index_for_span|span_file_matches|"
        r"source_line_has_path_like",
        "AffectedSetSelection",
    ),
    (
        r"resolve_|resolution|import_closure|import_resolution|module_declaration|"
        r"reference_edge|reference_resolution|symbol_index|module_graph|module_index|"
        r"module_path|module_binding|moduleless|qualified_name|binding_fork|"
        r"resolution_divergence|bare_identifier|dependency_resolution|extend_with_reference|"
        r"extend_with_bare|referenced_module|virtual_source|extract_import|extract_module|"
        r"build_module_index|index_source_root|load_sources|multi_entry|discovery_corpus|"
        r"resolve_entry|resolved_graph_from|load_compile_clean_entry|free_monoid_symbol|"
        r"lookup_resolved|local_binding|containment_walk|walk_target_alias|import_adjacency|"
        r"resolve_transitively|resolve_func_sigs|peel_|flat_parents|scoped_gate|"
        r"process_resolve_store|module_schedule|pool_head|pool_parse|pool_qualified|"
        r"pool_bare|pool_roots|parse_module_heads|parse_module_node|tree_bare_census|"
        r"collect_both_closure|extend_sources_to_both|entry_source_from_index|"
        r"canonical_shared_index|closure_subject|whole_tree_ancestry|make_eval_context|"
        r"format_error|defining_module|resolved_decl|resolved_initializer|"
        r"declared_type_name|path_to_source|workspace_relative|same_canonical_file|"
        r"source_tree_root|bare_ref_reachability|reference_accounting|ref_field_chain|"
        r"collect_node_refs|module_prefix_shared|longest_declared_module|"
        r"normalize_decl_body|decl_body_hash|extract_top_level_decls|rel_path_within_tree|"
        r"is_excluded_import_path|parse_module_node_tolerant|collect_module_decl_names|"
        r"module_defined_type_names|module_emit_repr_fingerprint|whole_corpus_semantic|"
        r"dag_source_roots|is_resolve_typecheck_blocking|ResolveTypecheckGate|"
        r"augment_closure_modules|rewrite_import",
        "NamespaceResolutionSymbolIndex",
    ),
    (
        r"discovery_row|discovery_group|discovery_summary|run_discovery|spawn_width|"
        r"memory_bounded|parallel_worker|worker_spawn|shard|schedule_batch|realization|"
        r"materialization|process_shared_index|populate_worker|width_cap|budget_completion|"
        r"witness_timing|run_claim|claim_outcome|floor_verbose|floor_ts|floor_stream|"
        r"floor_color|floor_gantt|install_output_policy|install_group_syntax|"
        r"resolve_floor_runner|memory_governor|adaptive_governor|claim_executor|run_walk|"
        r"entry_row_groups|merge_discovery_summaries|emit_floor_drain|floor_drain|"
        r"peak_rss|compute_percentiles|top_n_slowest|compute_histogram|realize_advisory|"
        r"emit_realize_advisory|wet_hermetic|is_governed_service|accumulate|attributed_total",
        "RealizationMaterializationScheduling",
    ),
    (r"^regen_", "RegenSelfHost"),
    (
        r"compile_clean|compile_dag_rust|floor_compile_clean|enable_floor_compile|"
        r"disable_floor_compile|produce_floor_compile|install_floor_compile|"
        r"consume_floor_compile|reset_floor_compile",
        "CompileCleanGate",
    ),
    (
        r"inert_lens|inert_carrier|non_fold_residue|fact_cardinality|languages_consumer|"
        r"layer_import|transport_script|extdeps_|doc_graph|complexity_linearity|"
        r"construction_justification|sidecar_placement|test_migration|medium_structure|"
        r"lens_string|census_corpus|fn_arrow_decl_substrate|census_heads|stripped_fn_body|"
        r"is_census_heads|wall_now_authority|construction_authority|unjustified_lens|"
        r"is_top_level_lens|build_floor_lens|discover_floor_corpus|check_floor_filename|"
        r"scan_test_decl|default_floor_lens",
        "LensHostBridge",
    ),
    (
        r"^handle_|^run_value|host_effect|converge|pre_push|handle_serve|handle_ci|"
        r"handle_run|classify_exit|serve_read|serve_write|dump_residual",
        "WorkflowHostEffect",
    ),
    (
        r"workspace_root|collect_dag_files|repo_relative|cli_path_arg|is_cargo_target|"
        r"manifest_stub|process_workspace|anchor_source|module_index_path_key|"
        r"insert_module_path|fixture_base|fixture_root|canonical|write",
        "HostPhysics",
    ),
    (
        r"witness_layer|ci_layer_roots|ci_floor|UNIFIED_CLAIM|BOOL_WITNESS|NODE_CORPUS|"
        r"lens_string_list|report_moduleless|string_list_from_value|normalize_repo_path|"
        r"owned_data|bool_witness|unified_claim|discover_owned_data|closure_group|"
        r"emit_owned_data|verify_bool_witness|illegal_other_init|value_is_test_claim|"
        r"value_is_node|call_test_claim|test_claim_selection|literal_string_from|"
        r"symbol_name_from|field_init|binding_name_from|extract_bool_witness|"
        r"is_resolved_bool|is_resolved_node|manifest_symbol_for|seed_kernel_intern|"
        r"top_level_decl_names|add_closure_to_group|variant_field|collect_node_values|"
        r"list_value_from_vec|decl_span_end_line|collect_sorted_decl_lines_for_file",
        "WitnessDiscoveryExecution",
    ),
    (
        r"build_module_path_index|for_each_parsed_module|collect_module_binding|"
        r"module_binding_manifest|source_path_for_module|module_path_collision|"
        r"emit_module_storage_binding|module_storage_binding|source_root_ingest|"
        r"source_root_ref|discover_source_root|emit_source_root|parse_source_root_entry|"
        r"emit_import_admission",
        "ModuleIdentityStorage",
    ),
    (r"^new$|^drop$|^flat$|^default$|^verdicts$|^record_field$|^push_pair$|^computes_with_store$", "TestReceiptLane"),
    (r"_for_test$|_len_for_test$", "TestReceiptLane"),
]

FEATURE_V2_AUTHORITY: dict[str, str] = {
    "RealizationMaterializationScheduling": "std.realization_schedule / gunbc.ci_floor_plan / gunbc.floor_materialization",
    "NamespaceResolutionSymbolIndex": "v2.lens.module_graph / v2.compiler.resolve / namespace-resolution-design",
    "AffectedSetSelection": "v2.lens.affected_set / v2.workflow.affected_set_floor_runner / tools.dag_compile_clean_scope",
    "TypedModuleCacheComputationIdentity": "std.cache_interface / std.materialize / extdeps.realization.resolved_graph",
    "SerializationOneGrammar": "dag/extdeps/languages / wire transport rows / emission_ingestion_inverse",
    "WorkflowHostEffect": "gunbc.host_effect / shell-intent-emit-realization-design",
    "WitnessDiscoveryExecution": "gunbc.ci_layer_roots / gunbc.ci_spec / v2.std.verification",
    "CompileCleanGate": "tools.dag_compile_clean_scope / dag_compile_clean_transport",
    "RegenSelfHost": "regen_stage0 / self_host/frontier / module-identity-storage-binding",
    "LensHostBridge": "v2.lens.* projections / hand_lens_host_bridge_scaffold_watchdog",
    "ModuleIdentityStorage": "v2.compiler.source_authority / module-identity-storage-binding-design",
    "HostPhysics": "terminal bootstrap kernel (physics-bound until host-effect realize)",
    "PhaseProfileInstrumentation": "realization-measurement-loop Phase 0 / gunbc.ci_render",
    "TestReceiptLane": "dissolves with its subject feature v2 witness",
    "Unclassified": "ESCALATE - assign before Chunk F dissolution",
}


def section_feature(lineno: int) -> str | None:
    for start, end, feat in NAMED_SECTIONS:
        if start <= lineno < end:
            return feat
    return None


def categorize(name: str, lineno: int, is_test: bool) -> str:
    if is_test:
        for pat, cat in RULES:
            if re.search(pat, name):
                return cat
        return "TestReceiptLane"
    sec = section_feature(lineno)
    if sec:
        return sec
    for pat, cat in RULES:
        if re.search(pat, name):
            return cat
    return "Unclassified"


def extract_functions(lines: list[str]) -> list[dict]:
    test_names: set[str] = set()
    for i, line in enumerate(lines):
        if "#[test]" in line:
            for j in range(i + 1, min(i + 8, len(lines))):
                m = re.match(r"^\s*fn\s+(\w+)", lines[j])
                if m:
                    test_names.add(m.group(1))
                    break

    # Ledger scope: column-0 fns + every #[test] fn (incl. nested in test mods).
    rows: list[dict] = []
    seen: set[tuple[int, str]] = set()

    for i, line in enumerate(lines):
        lineno = i + 1
        col0 = re.match(r"^(pub\s+)?(async\s+)?fn\s+(\w+)", line)
        nested = re.match(r"^\s+(pub\s+)?(async\s+)?fn\s+(\w+)", line)
        m = col0 or nested
        if not m:
            continue
        name = m.group(3)
        is_col0 = bool(col0)
        is_test = name in test_names
        if not is_col0 and not is_test:
            continue
        key = (lineno, name)
        if key in seen:
            continue
        seen.add(key)
        rows.append(
            {
                "line": lineno,
                "name": name,
                "pub": bool(m.group(1)),
                "test": is_test,
                "feature": categorize(name, lineno, is_test),
            }
        )
    return rows


def dag_symbol(name: str) -> str:
    s = re.sub(r"[^a-zA-Z0-9_]", "_", name)
    s = re.sub(r"_+", "_", s).strip("_")
    if not s or not s[0].isalpha():
        s = f"sym_{s}"
    return s


def render_dag(rows: list[dict], source_loc: int) -> str:
    counts = Counter(r["feature"] for r in rows)
    prod = sum(1 for r in rows if not r["test"])
    test = sum(1 for r in rows if r["test"])
    uncl = counts.get("Unclassified", 0)

    law = (
        "cli_run.rs hollowing ledger quick-moth-273: every module-scope fn and test-fn "
        f"mapped to a v2 dissolution feature. Receipt {len(rows)} rows "
        f"({prod} production, {test} test), {uncl} unclassified. "
        "Regenerate via generate_cli_run_hollowing_ledger script. "
        "Dissolve-on seed-shrink Chunk F."
    )

    lines = [
        "module gunbc.cli_run_hollowing_ledger",
        "",
        "import std.types { Int, List, String }",
        "import v2.std.algebra { length }",
        "import v2.std.node { Symbol }",
        "",
        "type CliRunHollowFeature",
        "  = RealizationMaterializationScheduling",
        "  | NamespaceResolutionSymbolIndex",
        "  | AffectedSetSelection",
        "  | TypedModuleCacheComputationIdentity",
        "  | SerializationOneGrammar",
        "  | WorkflowHostEffect",
        "  | WitnessDiscoveryExecution",
        "  | CompileCleanGate",
        "  | RegenSelfHost",
        "  | LensHostBridge",
        "  | ModuleIdentityStorage",
        "  | HostPhysics",
        "  | PhaseProfileInstrumentation",
        "  | TestReceiptLane",
        "  | Unclassified",
        "",
        "type CliRunHollowDisposition",
        "  = HollowTarget { feature: CliRunHollowFeature, v2_authority: String }",
        "  | PinnedHostPhysics { reason: String }",
        "  | TestReceiptRow { subject_feature: CliRunHollowFeature }",
        "",
        "type CliRunHollowLedgerRow {",
        "  id: Symbol,",
        "  fn_name: String,",
        "  source_line: Int,",
        "  disposition: CliRunHollowDisposition",
        "}",
        "",
        f"data cli_run_hollowing_row_count_baseline: Int = {len(rows)}",
        f"data cli_run_hollowing_production_row_baseline: Int = {prod}",
        f"data cli_run_hollowing_test_row_baseline: Int = {test}",
        f"data cli_run_hollowing_unclassified_baseline: Int = {uncl}",
        "",
        f'data cli_run_hollowing_ledger_law: String = "{law}"',
        "",
    ]

    def feature_to_ctor(feature: str) -> str:
        return feature

    def disposition_for(row: dict) -> str:
        feat = row["feature"]
        if row["test"]:
            subj = feat if feat != "TestReceiptLane" else "Unclassified"
            return f"TestReceiptRow {{ subject_feature: {feature_to_ctor(subj)} }}"
        if feat == "HostPhysics":
            return 'PinnedHostPhysics { reason: "terminal bootstrap physics until host_effect realize" }'
        auth = FEATURE_V2_AUTHORITY.get(feat, "UNASSIGNED")
        return (
            f'HollowTarget {{ feature: {feature_to_ctor(feat)}, '
            f'v2_authority: "{auth}" }}'
        )

    if rows:
        lines.append("data cli_run_hollowing_ledger_rows: List<CliRunHollowLedgerRow> = [")
        for row in rows:
            sym = dag_symbol(f"line_{row['line']}_{row['name']}")
            disp = disposition_for(row)
            lines.append(
                f"  CliRunHollowLedgerRow {{"
                f"\n    id: ^{sym},"
                f"\n    fn_name: \"{row['name']}\","
                f"\n    source_line: {row['line']},"
                f"\n    disposition: {disp}"
                f"\n  }},"
            )
        lines.append("]")
    else:
        lines.append("data cli_run_hollowing_ledger_rows: List<CliRunHollowLedgerRow> = []")
    lines.append("")

    lines.extend(
        [
            "fn cli_run_hollowing_ledger_row_count() -> Int {",
            "  cli_run_hollowing_ledger_rows |> length",
            "}",
            "",
            "fn cli_run_hollowing_ledger_enumeration_holds() -> Bool {",
            "  cli_run_hollowing_ledger_row_count() == cli_run_hollowing_row_count_baseline",
            "}",
            "",
            "fn cli_run_hollowing_no_unclassified_holds() -> Bool {",
            "  cli_run_hollowing_unclassified_baseline == 0",
            "}",
            "",
        ]
    )
    summary_parts = [f"{k}={v}" for k, v in sorted(counts.items())]
    lines.extend(
        [
            "fn cli_run_hollowing_feature_counts_note() -> String {",
            f'  "{", ".join(summary_parts)}"',
            "}",
            "",
            f"// Generated from {CLI_RUN.relative_to(REPO)} ({source_loc} LOC). Regenerate via scripts/generate_cli_run_hollowing_ledger.py",
        ]
    )
    return "\n".join(lines) + "\n"


def main() -> int:
    if not CLI_RUN.is_file():
        print(f"missing {CLI_RUN}", file=sys.stderr)
        return 1
    text = CLI_RUN.read_text()
    lines = text.splitlines()
    rows = extract_functions(lines)
    rows.sort(key=lambda r: r["line"])
    dag = render_dag(rows, len(lines))
    OUT_DAG.write_text(dag)
    counts = Counter(r["feature"] for r in rows)
    print(f"Wrote {OUT_DAG} — {len(rows)} rows")
    for feat, n in sorted(counts.items(), key=lambda x: (-x[1], x[0])):
        print(f"  {feat}: {n}")
    if counts.get("Unclassified", 0):
        print("WARNING: unclassified rows remain — assign before merge", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
