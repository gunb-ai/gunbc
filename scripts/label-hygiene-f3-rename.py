#!/usr/bin/env python3
"""F3 label-hygiene rename tool (tranche workers reuse; T1 landed manually).

Operator-approved naming 2026-06-12. Tranche 1 (#4784): v3 compiler tests/
sg* → self_gen* only. Later tranches use subsets of REPLACEMENTS per mgr plan.
Never touch #4741-blocked paths until that PR merges. Atoms (^dag_*) = F4 last.
target_model.dag SG catalog labels = separate signed tranche (distinct acronym).
"""

from __future__ import annotations

import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CODE_AREAS = ["src", "dsl", "tools", "scripts", ".github", "fixtures"]
SKIP_FILES = {
    "src/v4/std/compilers/target_model.dag",  # deferred v4 SG catalog family
}
SKIP_DOC_SUFFIX = ".md"

# Content replacements: longest / most specific first.
CONTENT_REPLACEMENTS: list[tuple[str, str]] = [
    # --- body_producer before  and wave* ---
    ("add_body", "add_body"),
    ("branch_body", "branch_body"),
    ("body_producer", "body_producer"),
    # --- model_core / runtime wave specials before global wave* ---
    ("model_core_bool", "model_core_bool"),
    ("bool_primitive_model_core", "bool_primitive_model_core"),
    ("bool_primitive_void", "bool_primitive_void"),
    ("v4_evaluator_runtime_core", "v4_evaluator_runtime_core"),
    ("effect_io_host_runtime", "effect_io_host_runtime"),
    # --- wave (delete vocab) — grammar_typed before grammar_control_flow before grammar_core ---
    ("grammar_typed", "grammar_typed"),
    ("ci_shadow", "ci_shadow"),
    ("ci_shadow", "ci_shadow"),
    ("grammar_control_flow", "grammar_control_flow"),
    ("grammar_core", "grammar_core"),
    # --- mvp2 before mvp1 ---
    ("RUNTIME_VALUE_", "RUNTIME_VALUE_"),
    ("runtime_value_", "runtime_value_"),
    # --- comprep (domain names) ---
    ("omni_emit_", "omni_emit_"),
    ("", ""),
    # --- mvp1 (drop prefix) ---
    ("_", "_"),
    ("", ""),
    # --- sg → self_gen (reverse numeric order) ---
    ("self_gen_census", "self_gen_census"),
    ("self_gen_non_test", "self_gen_non_test"),
    ("r3_self_gen", "r3_self_gen"),
    ("self_gen7", "self_gen7"),
    ("self_gen6", "self_gen6"),
    ("self_gen5", "self_gen5"),
    ("self_gen3", "self_gen3"),
    ("self_gen2c5", "self_gen2c5"),
    ("self_gen2c1", "self_gen2c1"),
    ("self_gen2b", "self_gen2b"),
    ("self_gen2", "self_gen2"),
    ("self_gen1b", "self_gen1b"),
    ("self_gen1", "self_gen1"),
    ("self_gen0", "self_gen0"),
    # --- rung → descriptive (ladder vocabulary out of ids) ---
    ("nat_semiring_full_law_emit_eval", "nat_semiring_full_law_emit_eval"),
    ("nat_semiring_roundtrip_emit_eval", "nat_semiring_roundtrip_emit_eval"),
    ("nat_semiring_deferred_zero", "nat_semiring_deferred_zero"),
    ("nat_semiring_full_law_roster", "nat_semiring_full_law_roster"),
    ("nat_semiring_post_emit_law", "nat_semiring_post_emit_law"),
    ("branch_dispatch_roundtrip_emit_eval", "branch_dispatch_roundtrip_emit_eval"),
    ("branch_dispatch_deferred_zero", "branch_dispatch_deferred_zero"),
    ("loop_linear_bound_roundtrip_emit_eval", "loop_linear_bound_roundtrip_emit_eval"),
    ("loop_linear_bound_deferred_zero", "loop_linear_bound_deferred_zero"),
    ("compiles_three_targets", "compiles_three_targets"),
    ("full_law_emit_eval_common", "full_law_emit_eval_common"),
    ("roundtrip_emit_eval_common", "roundtrip_emit_eval_common"),
    ("l1_go_compiler_slice", "l1_go_compiler_slice"),
    ("l1_python_runtime", "l1_python_runtime"),
    ("full_law_emit_eval_", "full_law_emit_eval_"),
    ("roundtrip_emit_eval_", "roundtrip_emit_eval_"),
    ("deferred_zero", "deferred_zero"),
    ("post_emit_law", "post_emit_law"),
    ("full_law_roster", "full_law_roster"),
    ("roundtrip_emit_eval", "roundtrip_emit_eval"),
    ("compiles_three_targets", "compiles_three_targets"),
    ("phase1_nat_semiring", "phase1_nat_semiring"),
    ("_acceptance_gate", "_acceptance_gate"),
    ("acceptance_gate", "acceptance_gate"),
    ("acceptance-gate", "acceptance-gate"),
    ("acceptance_gate", "acceptance_gate"),
    ("ACCEPTANCE_GATE", "ACCEPTANCE_GATE"),
    ("acceptance gate", "acceptance gate"),
    # script/env names
    ("V4_NAT_SEMIRING_ACCEPTANCE_GATE", "V4_NAT_SEMIRING_ACCEPTANCE_GATE"),
    ("nat-semiring-acceptance-gate", "nat-semiring-acceptance-gate"),
    # pr3 codename inside filenames already handled
    ("_typed_fn_", "_typed_fn_"),
]

# File renames: basename or relative path suffix → new basename
FILE_RENAMES: list[tuple[str, str]] = [
    # sg test files
    ("self_gen_census_test.rs", "self_gen_census_test.rs"),
    ("self_gen1_tokenize_authority_test.rs", "self_gen1_tokenize_authority_test.rs"),
    ("self_gen2_parse_authority_test.rs", "self_gen2_parse_authority_test.rs"),
    ("self_gen2c1_parse_tables_authority_test.rs", "self_gen2c1_parse_tables_authority_test.rs"),
    ("self_gen2c5_soft_keyword_ident_test.rs", "self_gen2c5_soft_keyword_ident_test.rs"),
    ("self_gen3_lower_parse_surface_stack_test.rs", "self_gen3_lower_parse_surface_stack_test.rs"),
    ("self_gen3_surface_reflection_consumer_test.rs", "self_gen3_surface_reflection_consumer_test.rs"),
    ("self_gen6_hand_authored_census_test.rs", "self_gen6_hand_authored_census_test.rs"),
    ("self_gen7_prep_variant_payload_freshness_test.rs", "self_gen7_prep_variant_payload_freshness_test.rs"),
    ("r3_self_gen_non_test_zero.dag", "r3_self_gen_non_test_zero.dag"),
    ("r3_self_gen_non_test_zero_test.rs", "r3_self_gen_non_test_zero_test.rs"),
    # add-fn claim files
    ("cpp_add_translate.dag", "cpp_add_translate.dag"),
    ("go_add_translate.dag", "go_add_translate.dag"),
    ("python_add_translate.dag", "python_add_translate.dag"),
    ("rust_add_translate.dag", "rust_add_translate.dag"),
    ("typescript_add_translate.dag", "typescript_add_translate.dag"),
    ("dag_add_round_trip.dag", "dag_add_round_trip.dag"),
    ("typescript_typed_fn_typed_fn_translate.dag", "typescript_typed_fn_translate.dag"),
    ("typescript_record_task_translate.dag", "typescript_record_task_translate.dag"),
    ("go_grammar_claim.dag", "go_grammar_core_claim.dag"),
    ("kotlin_grammar_claim.dag", "kotlin_grammar_core_claim.dag"),
    ("python_grammar_claim.dag", "python_grammar_core_claim.dag"),
    ("dag_round_trip_mvp1.dag", "dag_round_trip_add.dag"),
    # comprep claim files
    ("eval_by_execution.dag", "eval_by_execution_add.dag"),
    ("branch_eval_by_execution.dag", "eval_by_execution_branch.dag"),
    ("add_body_producer.dag", "add_body_producer.dag"),
    ("add_body_emit_typescript.dag", "add_body_emit_typescript.dag"),
    ("value_expression_fold_typescript.dag", "value_expression_fold_typescript.dag"),
    ("branch_lazy_arm_eval_acceptance.dag", "branch_lazy_arm_eval_acceptance.dag"),
    ("omni_emit_ts_descriptor_node_run.dag", "ts_descriptor_node_run.dag"),
    ("add_body_subject_producer.dag", "add_body_subject_producer.dag"),
    ("branch_body_subject_producer.dag", "branch_body_subject_producer.dag"),
    # wave anchor / parse
    ("model_core_bool_anchor.dag", "model_core_bool_anchor.dag"),
    ("java_grammar_control_flow_grammar_structure.dag", "java_grammar_control_flow_structure.dag"),
    ("rust_grammar_control_flow_grammar_structure.dag", "rust_grammar_control_flow_structure.dag"),
    ("typescript_grammar_typed_type_alias_task.dag", "typescript_grammar_typed_type_alias_task.dag"),
    ("go_grammar_typed.dag", "go_grammar_typed.dag"),
    ("kotlin_grammar_typed.dag", "kotlin_grammar_typed.dag"),
    ("lean_grammar_typed.dag", "lean_grammar_typed.dag"),
    ("python_grammar_typed.dag", "python_grammar_typed.dag"),
    ("rust_grammar_typed.dag", "rust_grammar_typed.dag"),
    ("swift_grammar_typed.dag", "swift_grammar_typed.dag"),
    ("typescript_grammar_typed.dag", "typescript_grammar_typed.dag"),
    ("ci_shadow_roster.dag", "ci_shadow_roster.dag"),
    ("ci_shadow_receipt.rs", "ci_shadow_receipt.rs"),
    ("emit_ci_shadow_receipt.rs", "emit_ci_shadow_receipt.rs"),
    # sg manual claims
    ("self_gen1b_signature_realization_failclosed.dag", "self_gen1b_signature_realization_failclosed.dag"),
    ("self_gen2_mode2_non_grammar_emit.dag", "self_gen2_mode2_non_grammar_emit.dag"),
    ("self_gen2_type_expression_projection.dag", "self_gen2_type_expression_projection.dag"),
    ("self_gen2_typescript_type_expression_projection.dag", "self_gen2_typescript_type_expression_projection.dag"),
    ("self_gen5_set_non_ordable_falsification.dag", "self_gen5_set_non_ordable_falsification.dag"),
    ("sg_rc_layering.dag", "self_gen_rc_layering.dag"),
    ("sg_collection_projection.dag", "self_gen_collection_projection.dag"),
    # rung claim files
    ("compiles_three_targets.dag", "compiles_three_targets.dag"),
    ("roundtrip_emit_eval.dag", "roundtrip_emit_eval.dag"),
    ("full_law_roster.dag", "full_law_roster.dag"),
    ("post_emit_law.dag", "post_emit_law.dag"),
    ("deferred_zero.dag", "deferred_zero.dag"),
    ("l1_go_compiler_slice.dag", "l1_go_compiler_slice.dag"),
    ("l1_python_runtime.dag", "l1_python_runtime.dag"),
    ("roundtrip_emit_eval_common.dag", "roundtrip_emit_eval_common.dag"),
    ("full_law_emit_eval_common.dag", "full_law_emit_eval_common.dag"),
    ("nat_semiring_roundtrip_emit_eval.dag", "nat_semiring_roundtrip_emit_eval.dag"),
    ("nat_semiring_full_law_emit_eval.dag", "nat_semiring_full_law_emit_eval.dag"),
    ("nat_semiring_full_law_roster_eval.dag", "nat_semiring_full_law_roster_eval.dag"),
    ("nat_semiring_deferred_zero_eval.dag", "nat_semiring_deferred_zero_eval.dag"),
    ("branch_dispatch_roundtrip_emit_eval.dag", "branch_dispatch_roundtrip_emit_eval.dag"),
    ("branch_dispatch_deferred_zero_eval.dag", "branch_dispatch_deferred_zero_eval.dag"),
    ("loop_linear_bound_roundtrip_emit_eval.dag", "loop_linear_bound_roundtrip_emit_eval.dag"),
    ("loop_linear_bound_deferred_zero_eval.dag", "loop_linear_bound_deferred_zero_eval.dag"),
    # scripts
    ("v4-nat-semiring-acceptance-gate.sh", "v4-nat-semiring-acceptance-gate.sh"),
    ("self_gen0-pr-body-append", "self_gen0-pr-body-append"),
    # fixtures dir
    ("fixtures/v4-mvp1", "fixtures/v4-add-translate"),
]


def iter_code_files() -> list[Path]:
    files: list[Path] = []
    for area in CODE_AREAS:
        base = ROOT / area
        if not base.exists():
            continue
        for path in base.rglob("*"):
            if not path.is_file():
                continue
            rel = path.relative_to(ROOT).as_posix()
            if rel in SKIP_FILES:
                continue
            if path.suffix == SKIP_DOC_SUFFIX:
                continue
            files.append(path)
    return files


def apply_content_replacements(text: str) -> str:
    for old, new in CONTENT_REPLACEMENTS:
        text = text.replace(old, new)
    return text


def rename_files() -> None:
    # Directory renames first (deepest paths)
    dir_renames = [(o, n) for o, n in FILE_RENAMES if "/" in o and not o.endswith((".dag", ".rs", ".sh", ".txt"))]
    for old_rel, new_rel in dir_renames:
        old_path = ROOT / old_rel
        new_path = ROOT / new_rel
        if old_path.exists() and not new_path.exists():
            old_path.rename(new_path)
            print(f"git mv dir: {old_rel} -> {new_rel}")

    # Collect all file paths that need renaming
    renames: list[tuple[Path, Path]] = []
    for path in iter_code_files():
        rel = path.relative_to(ROOT)
        name = path.name
        new_name = name
        for old_suffix, new_suffix in FILE_RENAMES:
            if "/" not in old_suffix and name == old_suffix:
                new_name = new_suffix
                break
            if old_suffix in rel.as_posix() and "/" in old_suffix:
                # partial path match handled below for fixtures
                pass
        if new_name != name:
            renames.append((path, path.with_name(new_name)))

    # fixtures/v4-mvp1 directory
    old_fixture = ROOT / "fixtures/v4-mvp1"
    new_fixture = ROOT / "fixtures/v4-add-translate"
    if old_fixture.exists() and not new_fixture.exists():
        subprocess.run(["git", "mv", str(old_fixture), str(new_fixture)], check=True, cwd=ROOT)

    for old_path, new_path in sorted(renames, key=lambda x: len(x[0].as_posix()), reverse=True):
        if old_path.exists() and not new_path.exists():
            subprocess.run(["git", "mv", str(old_path), str(new_path)], check=True, cwd=ROOT)
            print(f"git mv: {old_path.relative_to(ROOT)} -> {new_path.name}")


def rewrite_contents() -> int:
    changed = 0
    for path in iter_code_files():
        try:
            original = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue
        updated = apply_content_replacements(original)
        if updated != original:
            path.write_text(updated, encoding="utf-8")
            changed += 1
    return changed


def main() -> int:
    os.chdir(ROOT)
    print("=== F3 label-hygiene renames ===")
    print("Step 1: content replacements...")
    n = rewrite_contents()
    print(f"  updated {n} files")
    print("Step 2: file renames...")
    rename_files()
    print("Step 3: second content pass (post-rename path strings)...")
    n2 = rewrite_contents()
    print(f"  updated {n2} additional files")
    return 0


if __name__ == "__main__":
    sys.exit(main())
