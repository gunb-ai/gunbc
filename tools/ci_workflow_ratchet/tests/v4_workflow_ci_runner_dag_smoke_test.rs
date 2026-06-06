//! **Layer:** integration
//!
//! PR-A relocation — host ratchet (ci.yml binding, mirror parity, grep assertions).
//! T-21/T-24 Wave-0: `src/v4/workflow/ci.dag` (`v4.workflow.ci`) exposes affected-set-driven
//! `TestClaim` roster selection over `RerunNodeSet` (kernel `filter`/`any`/`contains`;
//! evaluation-node projection in `verification.dag`). Receipt
//! claims live in `src/v4/test/claim/workflow/affected_set_ci_runner.dag`.
//!
//! **TESTING.md:** M1(2.7) tokenize/parse gate; full `compile_to_dag` import merge
//! deferred until cross-module v4 load lands (same posture as peer v4 smoke tests).
//!
//! **ROADMAP:** `ROADMAP.md` § **Nine lanes** row **T-PB-B** / `pb_rust_tests_outside_residual_zero`;
//! **TASKS.md** T-21 + T-24; bankruptcy B0/B1 Tier-0 binding smoke: `docs/design-ci-bankruptcy-rebuild.md` §4.1
//! Wave 1 §11.7.1 floor: `docs/planning/ci-required-surface-cut-2026-06-01.md` (`v4_workflow_ci_wave1_*`).
//! Affected-set component receipt promotion: `affected` is now a live fail-closed receipt; Wave 3
//! node-frontier TestClaim selection remains shadow until the whole-program Dag CI input lands.
//! Wave 3 §11.7.2 shadow receipt Phase 1: same doc (`v4_workflow_ci_wave3_*`; P5(b) receipt table).
//! (P5 same-path expansion — `_internal/INVARIANTS_OPS.md` → this file, PR #4101 / #4174 / #4214).
//! T-38 PR2 same-path assertion expansion: explicit P5 deferral to T-PB-B test sub-ratchet;
//! ROADMAP.md § "Milestone shape" row 4 ("Self-host fixed point") tracks hand-maintained file count -> 0.
//! T-38B same-path assertion expansion: explicit P5 deferral to ROADMAP.md § "Nine lanes"
//! row **T-PB-B** / `pb_rust_tests_outside_residual_zero`; this adds no new hand-Rust
//! test path and stays within the existing SG-0 census entry for this v4 CI smoke harness.
//! Dissolve-on: generated `.dag` TestClaim execution covers
//! `manual_testclaim_subject_roster_family_receipt` and the CI verdict-surface projection
//! without this hand-Rust parse/string ratchet.
//!
//! **INVARIANTS P5 — checkable receipt for this PR:** feature `affected-component-live-receipt`;
//! consumers `v4_workflow_ci_wave1_*` and `v4_workflow_ci_wave3_node_selection_still_shadow_*`.
//! Dissolve-on: A15 Shape-B/T-24 emitted `ci.yml` plus `.dag` TestClaim execution covers the
//! live component receipt and Wave 3 deferral without this hand-Rust parse harness.
//!
//! **INVARIANTS P5 — checkable receipt for PR #4251 (`infra_isolation` required-path):** feature
//! `infra-isolation-required-gate`; consumers `v4_workflow_ci_bankruptcy_tier0_discipline_off_required_ci_path`,
//! `v4_workflow_ci_wave1_*`, `v4_workflow_ci_wave3_node_selection_still_shadow_*`. SAME-PATH edit:
//! updates the existing `ci`-aggregator-needs assertions to the `[affected, ci_floor, infra_isolation]`
//! triple — no new test fn or authority surface (the modeled de-priv guard's coverage is the
//! byte-for-byte carrier structural-match, not a hand-Rust binding test; ROADMAP row **T-PB-B** /
//! `pb_rust_tests_outside_residual_zero`). Dissolve-on: same A15 Shape-B/T-24 lane as above.
//!
//! **INVARIANTS P5 — checkable receipt for THIS PR (Wave 3 §11.7.2 Phase-2 host emit wired):**
//! feature `wave3-host-emit-class-c-wired`; consumers
//! `v4_workflow_ci_wave3_host_emit_wired_class_c_in_ci_yml`,
//! `v4_workflow_ci_wave3_node_selection_still_shadow_*`. SAME-PATH edit: flips the prior
//! `*_live_emit_deferred_*` assertion to assert the host shadow-emit step IS now wired (Class C,
//! `continue-on-error: true`) — superseding the "live emit deferred" posture — while the modeled
//! live entry `ci_selection_receipt_shadow_from_git_diff` stays deferred to `node://adhoc-331899f9-19a`
//! (node-frontier claim/testgen selection remains shadow). No new hand-Rust test path or authority
//! surface (SG-0 delta 0 — path already in `EXPECTED_HAND_AUTHORED_TEST`). ROADMAP row **T-PB-B** /
//! `pb_rust_tests_outside_residual_zero`. Dissolve-on: same A15 Shape-B/T-24 lane as above.
//!
//! **INVARIANTS P5 — checkable receipt for PR #4323 (F.11b source-authority receipt consumption):**
//! feature `f11b-source-authority-ci-receipt`; consumer
//! `v4_workflow_ci_source_authority_receipt_consumes_h72_claims`. SAME-PATH SG-0 expansion:
//! stays inside this existing v4 CI smoke harness (+0 new hand-Rust test paths) and explicitly
//! defers to ROADMAP.md § **Nine lanes** row **T-PB-B** / `pb_rust_tests_outside_residual_zero`.
//! Dissolve-on: generated `.dag` TestClaim execution covers
//! `claim_source_authority_contract_compiles` and `claim_bmin_canonical_dag_source_parse_print_law`
//! through `SourceAuthorityReceiptEvalCommand` without this hand-Rust parse/string ratchet.
//!
//! **INVARIANTS P5 — checkable receipt for F.11a (`Upsert<T>` Node projection substrate):**
//! feature `f11a-ci-upsert-node-projection`; consumer
//! `v4_workflow_ci_upsert_node_projection_substrate`. SAME-PATH SG-0 expansion in this harness;
//! defers to **ROADMAP.md** `### Nine lanes` row **T-PB-B** /
//! `pb_rust_tests_outside_residual_zero` (ROADMAP.md:59-63; Public Operational Lanes summary
//! ROADMAP.md:43). Consumer test asserts those strings are present in-tree (checkable receipt).
//! Dissolve-on: `.dag` TestClaim execution proves `content_hash(ci_upsert_step_projection_node)`
//! sensitivity and `v4.std.patterns.Upsert<T>` field alignment without this hand-Rust ratchet.
//!
//! **INVARIANTS P5 — checkable receipt for node://adhoc-87c3a213-099 (F.11c receipt persistence + lookup):**
//! feature `f11c-ci-selection-receipt-persistence`; consumer
//! `v4_workflow_ci_selection_receipt_persistence_lookup_modeled`. SAME-PATH SG-0 expansion:
//! stays inside this existing v4 CI smoke harness (+0 new hand-Rust test paths) and defers to
//! ROADMAP.md § **Nine lanes** row **T-PB-B** / `pb_rust_tests_outside_residual_zero`.
//! Dissolve-on: runtime TestClaimRun/cache receipt storage persists and looks up
//! `CiSelectionReceipt` values from canonical receipt projection hashes without this string ratchet.
//!
//! **Dissolution:** remove when `.dag` TestClaim execution covers these claims without
//! this hand-Rust parse harness (A15 Shape-B emitted `ci.yml` retires `v4_workflow_ci_bankruptcy_tier0_*`).

use ci_workflow_ratchet::support::parse_for_test;
use ci_workflow_ratchet::support::tokenize_for_test;
use v3_compiler::parse_surface::{SurfaceExpr, SurfaceItem, SurfaceLiteral, SurfaceRecordField};

const CI_DAG: &str = include_str!("../../../src/v4/workflow/ci.dag");
const CI_DAG_PATH: &str = "src/v4/workflow/ci.dag";
const CI_YML: &str = include_str!("../../../.github/workflows/ci.yml");
const CI_YML_PATH: &str = ".github/workflows/ci.yml";
const CI_WORKFLOW_DAG: &str = include_str!("../../../dsl/gunbc/ci_github_actions_workflow.dag");
const CI_WORKFLOW_DAG_PATH: &str = "dsl/gunbc/ci_github_actions_workflow.dag";
const SHARED_CLOSURE_WORKSHEET: &str =
    include_str!("../../../docs/planning/v4-ci-rust-dag-shared-closure-worksheet-2026-06-01.md");
const SHARED_CLOSURE_WORKSHEET_PATH: &str =
    "docs/planning/v4-ci-rust-dag-shared-closure-worksheet-2026-06-01.md";
const M1_RUST_EMIT_PROBE_SCRIPT: &str =
    include_str!("../../../.github/ci-floor/v4-m1-rust-emit-probe.sh");
const TESTCLAIM_CORPUS_EVAL_SCRIPT: &str =
    include_str!("../../../scripts/v4-testclaim-corpus-eval.sh");
const TESTCLAIM_CORPUS_EVAL_SCRIPT_PATH: &str = "scripts/v4-testclaim-corpus-eval.sh";
const T15_SELF_HOST_STEP_NAME: &str = "T-15 self-host fixed-point harness (stage1==stage2)";
const CLAIM_DAG: &str =
    include_str!("../../../src/v4/test/claim/workflow/affected_set_ci_runner.dag");
const CLAIM_PATH: &str = "src/v4/test/claim/workflow/affected_set_ci_runner.dag";
const WAVE3_ROSTER_DAG: &str =
    include_str!("../../../src/v4/test/claim/workflow/wave3_shadow_roster.dag");
const WAVE3_ROSTER_PATH: &str = "src/v4/test/claim/workflow/wave3_shadow_roster.dag";
const TESTCLAIM_CORPUS_RUNNER_DAG: &str =
    include_str!("../../../src/v4/test/claim/workflow/testclaim_corpus_runner.dag");
const TESTCLAIM_CORPUS_RUNNER_PATH: &str = "src/v4/test/claim/workflow/testclaim_corpus_runner.dag";
const MANUAL_CORPUS_EVAL_EXPECTED_DAG: &str =
    include_str!("../../../src/v4/test/claim/workflow/manual_corpus_eval_expected.dag");
const MANUAL_CORPUS_EVAL_EXPECTED_PATH: &str =
    "src/v4/test/claim/workflow/manual_corpus_eval_expected.dag";
const MANUAL_CORPUS_ROSTER_DAG: &str =
    include_str!("../../../src/v4/test/claim/manual/manual_corpus_roster.dag");
const MANUAL_CORPUS_ROSTER_PATH: &str = "src/v4/test/claim/manual/manual_corpus_roster.dag";
// A.1.5a (RR-A §4 R7): in-process `TestClaimRun` equivalence claim — harness path
// (corpus-runner machinery) must produce the same runs as direct `run_test_claim`
// on a fixed corpus slice. SG-0 delta 0 — same-path expansion in this v4 CI smoke
// harness; `pb_rust_tests_outside_residual_zero`. Dissolve-on: `.dag` TestClaim
// execution exercises `inprocess_equivalence` without this hand-Rust parse ratchet.
const INPROCESS_EQUIVALENCE_DAG: &str =
    include_str!("../../../src/v4/test/claim/workflow/inprocess_equivalence.dag");
const INPROCESS_EQUIVALENCE_PATH: &str = "src/v4/test/claim/workflow/inprocess_equivalence.dag";
// F.12a recursive-flex THIN: a T-38 inspection receipt over `v4.workflow.ci` +
// `v4.workflow.bootstrap`, scoped to the A.1.5a slice (#4313) ONLY — it imports/
// inspects both workflow authorities (no edits) and conjoins (1) inprocess
// equivalence holds, (2a, fail-closed) every A.1.5a slice subject is in the live
// corpus subject roster `manual_corpus_node_subject_rows` that ci.dag's
// `TestClaimCorpusEvalCommand` evaluates, (2b, ci.dag structural consistency only)
// ci.dag's claim-id frontier has one id per rostered subject (cardinality — NOT
// per-id membership; the `TestClaim`->`Symbol` projection is intentionally absent in
// F.12a), and (3) the bootstrap fixed-point hash-pin projection holds. NOT the full
// self-host loop (F.12b). SG-0 delta 0 — same-path expansion in this v4 CI smoke
// harness; `pb_rust_tests_outside_residual_zero`.
const RECURSIVE_FLEX_INSPECTION_DAG: &str =
    include_str!("../../../src/v4/test/claim/workflow/recursive_flex_inspection.dag");
const RECURSIVE_FLEX_INSPECTION_PATH: &str =
    "src/v4/test/claim/workflow/recursive_flex_inspection.dag";
const CI_AFFECTED_COMPONENTS_LIB: &str =
    include_str!("../../../tools/ci_affected_components/src/lib.rs");
const ROADMAP: &str = include_str!("../../../ROADMAP.md");
const ROADMAP_PATH: &str = "ROADMAP.md";

const CI_CHANGED_PATH_AFFECTS_FNS: &[&str] = &[
    "ci_changed_path_affects_v2",
    "ci_changed_path_affects_v3",
    "ci_changed_path_affects_v4",
    "ci_changed_path_affects_testclaim_corpus",
    "ci_changed_path_affects_workflow_policy",
    "ci_changed_path_affects_release_distribution",
];

struct CiAffectedFixture {
    path: &'static str,
    v2: bool,
    v3: bool,
    v4: bool,
    testclaim_corpus: bool,
    workflow_policy: bool,
    release_distribution: bool,
}

/// Behavioral fixtures aligned with `src/v4/test/claim/workflow/ci_component_affected.dag`.
const CI_AFFECTED_BEHAVIORAL_FIXTURES: &[CiAffectedFixture] = &[
    CiAffectedFixture {
        path: "src/v2/stage0/Cargo.toml",
        v2: true,
        v3: false,
        v4: false,
        testclaim_corpus: false,
        workflow_policy: false,
        release_distribution: false,
    },
    CiAffectedFixture {
        path: "dsl/gunbc/ci.dag",
        v2: false,
        v3: true,
        v4: false,
        testclaim_corpus: false,
        workflow_policy: false,
        release_distribution: false,
    },
    CiAffectedFixture {
        path: "dsl/std/node.dag",
        v2: false,
        v3: true,
        v4: true,
        testclaim_corpus: false,
        workflow_policy: false,
        release_distribution: false,
    },
    CiAffectedFixture {
        path: ".github/ci-floor/v4-m1-rust-emit-probe.sh",
        v2: false,
        v3: false,
        v4: true,
        testclaim_corpus: false,
        workflow_policy: true,
        release_distribution: false,
    },
    CiAffectedFixture {
        path: "src/v4/workflow/ci.dag",
        v2: false,
        v3: false,
        v4: true,
        testclaim_corpus: false,
        workflow_policy: true,
        release_distribution: false,
    },
    CiAffectedFixture {
        path: "src/v4/workflow/release.dag",
        v2: false,
        v3: false,
        v4: false,
        testclaim_corpus: false,
        workflow_policy: false,
        release_distribution: true,
    },
    CiAffectedFixture {
        path: "src/v4/test/claim/lens_affected_set/irt1_leaf_claim_suite.dag",
        v2: false,
        v3: false,
        v4: false,
        testclaim_corpus: true,
        workflow_policy: false,
        release_distribution: false,
    },
    CiAffectedFixture {
        path: "src/v4/test/claim/workflow/affected_set_ci_runner.dag",
        v2: false,
        v3: false,
        v4: false,
        testclaim_corpus: true,
        workflow_policy: false,
        release_distribution: false,
    },
    CiAffectedFixture {
        path: "scripts/v4-testclaim-corpus-eval.sh",
        v2: false,
        v3: false,
        v4: false,
        testclaim_corpus: true,
        workflow_policy: false,
        release_distribution: false,
    },
    CiAffectedFixture {
        path: "src/v4/test/claim/manual/mvp1_rust_add_translate.dag",
        v2: false,
        v3: false,
        v4: true,
        testclaim_corpus: true,
        workflow_policy: false,
        release_distribution: false,
    },
    CiAffectedFixture {
        path: "src/v4/test/coercion_fold_int_rust_fixture.dag",
        v2: false,
        v3: false,
        v4: true,
        testclaim_corpus: false,
        workflow_policy: false,
        release_distribution: false,
    },
    CiAffectedFixture {
        path: "install.sh",
        v2: false,
        v3: false,
        v4: false,
        testclaim_corpus: false,
        workflow_policy: false,
        release_distribution: true,
    },
    CiAffectedFixture {
        path: "install/release-target-triples.sh",
        v2: false,
        v3: false,
        v4: false,
        testclaim_corpus: false,
        workflow_policy: false,
        release_distribution: true,
    },
    CiAffectedFixture {
        path: "src/v4/install/install.dag",
        v2: false,
        v3: false,
        v4: false,
        testclaim_corpus: false,
        workflow_policy: false,
        release_distribution: true,
    },
    CiAffectedFixture {
        path: "Cargo.lock",
        v2: true,
        v3: true,
        v4: true,
        testclaim_corpus: false,
        workflow_policy: false,
        release_distribution: false,
    },
    CiAffectedFixture {
        path: "Cargo.toml",
        v2: true,
        v3: true,
        v4: true,
        testclaim_corpus: false,
        workflow_policy: false,
        release_distribution: false,
    },
    CiAffectedFixture {
        path: "docs/README.md",
        v2: false,
        v3: false,
        v4: false,
        testclaim_corpus: false,
        workflow_policy: false,
        release_distribution: false,
    },
];

fn fn_body_end(rest: &str) -> usize {
    ["\npub fn ", "\nfn ", "\n#[cfg(test)]"]
        .iter()
        .filter_map(|prefix| rest.find(prefix))
        .min()
        .unwrap_or(rest.len())
}

fn extract_fn_body<'a>(source: &'a str, fn_name: &str) -> &'a str {
    let marker = format!("fn {fn_name}");
    let start = source
        .find(&marker)
        .unwrap_or_else(|| panic!("missing fn `{fn_name}`"));
    let after = start + marker.len();
    let rest = &source[after..];
    &source[start..after + fn_body_end(rest)]
}

fn literals_after_marker(line: &str, marker: &str) -> std::collections::BTreeSet<String> {
    let mut out = std::collections::BTreeSet::new();
    let mut search = line;
    while let Some(idx) = search.find(marker) {
        let rest = &search[idx + marker.len()..];
        if let Some(end) = rest.find('"') {
            out.insert(rest[..end].to_string());
            search = &rest[end + 1..];
        } else {
            break;
        }
    }
    out
}

fn dag_prefix_literals_in_fn(body: &str) -> std::collections::BTreeSet<String> {
    let mut out = std::collections::BTreeSet::new();
    for line in body.lines() {
        out.extend(literals_after_marker(line, "prefix: \""));
    }
    out
}

fn rust_prefix_literals_in_fn(body: &str) -> std::collections::BTreeSet<String> {
    let mut out = std::collections::BTreeSet::new();
    for line in body.lines() {
        out.extend(literals_after_marker(line, ".starts_with(\""));
    }
    out
}

fn dag_exact_path_literals_in_fn(body: &str) -> std::collections::BTreeSet<String> {
    let mut out = std::collections::BTreeSet::new();
    for line in body.lines() {
        out.extend(literals_after_marker(line, "changed == \""));
    }
    out
}

fn rust_exact_path_literals_in_fn(body: &str) -> std::collections::BTreeSet<String> {
    let mut out = std::collections::BTreeSet::new();
    for line in body.lines() {
        out.extend(literals_after_marker(line, "path == \""));
    }
    out
}

fn assert_ci_dag_rust_bucket_parity(fn_name: &str) {
    let dag_body = extract_fn_body(CI_DAG, fn_name);
    let rust_body = extract_fn_body(CI_AFFECTED_COMPONENTS_LIB, fn_name);
    assert_eq!(
        dag_prefix_literals_in_fn(dag_body),
        rust_prefix_literals_in_fn(rust_body),
        "{fn_name}: prefix literal sets must match between {CI_DAG_PATH} and ci_affected_components lib"
    );
    assert_eq!(
        dag_exact_path_literals_in_fn(dag_body),
        rust_exact_path_literals_in_fn(rust_body),
        "{fn_name}: exact-path literal sets must match between {CI_DAG_PATH} and ci_affected_components lib"
    );
}

fn assert_ci_dag_rust_mirror_release_distribution_only_parity() {
    use ci_affected_components::{
        ci_component_affected_from_changed_paths, ci_release_distribution_only_from_changed_paths,
    };
    assert!(ci_release_distribution_only_from_changed_paths([
        "install.sh",
        "src/v4/install/install.dag",
    ]));
    assert!(!ci_release_distribution_only_from_changed_paths([
        "install.sh",
        ".github/ci-floor/v4-m1-rust-emit-probe.sh",
    ]));
    assert!(!ci_release_distribution_only_from_changed_paths([
        "install.sh",
        "src/v4/test/claim/workflow/affected_set_ci_runner.dag",
    ]));
    let mixed = ci_component_affected_from_changed_paths([
        "install.sh",
        ".github/ci-floor/v4-m1-rust-emit-probe.sh",
    ]);
    assert!(mixed.release_distribution);
    assert!(mixed.v4);
    assert!(!mixed.release_distribution_only);
}

fn assert_ci_dag_rust_mirror_behavioral_parity() {
    use ci_affected_components::ci_component_affected_from_changed_paths;
    for fixture in CI_AFFECTED_BEHAVIORAL_FIXTURES {
        let flags = ci_component_affected_from_changed_paths([fixture.path]);
        assert_eq!(flags.v2, fixture.v2, "path `{}`: v2 flag", fixture.path);
        assert_eq!(flags.v3, fixture.v3, "path `{}`: v3 flag", fixture.path);
        assert_eq!(flags.v4, fixture.v4, "path `{}`: v4 flag", fixture.path);
        assert_eq!(
            flags.testclaim_corpus, fixture.testclaim_corpus,
            "path `{}`: testclaim_corpus flag",
            fixture.path
        );
        assert_eq!(
            flags.workflow_policy, fixture.workflow_policy,
            "path `{}`: workflow_policy flag",
            fixture.path
        );
        assert_eq!(
            flags.release_distribution, fixture.release_distribution,
            "path `{}`: release_distribution flag",
            fixture.path
        );
    }
}

fn assert_ci_dag_rust_mirror_full_parity() {
    for fn_name in CI_CHANGED_PATH_AFFECTS_FNS {
        assert_ci_dag_rust_bucket_parity(fn_name);
    }
    assert_ci_dag_rust_mirror_release_distribution_only_parity();
    assert_ci_dag_rust_mirror_behavioral_parity();
}

fn parse_module(source: &str, path: &str) -> v3_compiler::parse_surface::SurfaceModule {
    let tokens =
        tokenize_for_test(source, path).unwrap_or_else(|e| panic!("{path}: tokenize: {e:?}"));
    parse_for_test(&tokens, path).unwrap_or_else(|e| panic!("{path}: parse: {e:?}"))
}

fn module_paths(module: &v3_compiler::parse_surface::SurfaceModule) -> Vec<Vec<&str>> {
    module
        .items
        .iter()
        .filter_map(|item| match item {
            SurfaceItem::Module { path, .. } => {
                Some(path.iter().map(String::as_str).collect::<Vec<_>>())
            }
            _ => None,
        })
        .collect()
}

fn import_includes_name(
    module: &v3_compiler::parse_surface::SurfaceModule,
    path: &[&str],
    name: &str,
) -> bool {
    module.items.iter().any(|item| {
        let SurfaceItem::Import {
            path: item_path,
            names,
            ..
        } = item
        else {
            return false;
        };
        item_path.len() == path.len()
            && item_path
                .iter()
                .zip(path.iter())
                .all(|(a, &b)| a.as_str() == b)
            && names.iter().any(|n| n == name)
    })
}

fn surface_declares_fn(module: &v3_compiler::parse_surface::SurfaceModule, name: &str) -> bool {
    module.items.iter().any(|item| match item {
        SurfaceItem::Fn {
            name: item_name, ..
        }
        | SurfaceItem::FnExternalBody {
            name: item_name, ..
        } => item_name == name,
        _ => false,
    })
}

fn data_body<'a>(
    module: &'a v3_compiler::parse_surface::SurfaceModule,
    name: &str,
) -> &'a SurfaceExpr {
    module
        .items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::Data {
                name: item_name,
                body: Some(body),
                ..
            } if item_name == name => Some(body),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing data body `{name}`"))
}

fn record_body_field<'a>(body: &'a SurfaceExpr, field_name: &str) -> &'a SurfaceExpr {
    let fields = match body {
        SurfaceExpr::Record { fields, .. } | SurfaceExpr::VariantRecord { fields, .. } => fields,
        other => panic!("expected record body, got {other:?}"),
    };
    record_field_from_fields(fields, field_name)
}

fn expr_string(expr: &SurfaceExpr) -> &str {
    match expr {
        SurfaceExpr::Literal {
            value: SurfaceLiteral::String(value),
            ..
        } => value,
        other => panic!("expected string literal expr, got {other:?}"),
    }
}

fn expr_int(expr: &SurfaceExpr) -> i64 {
    match expr {
        SurfaceExpr::Literal {
            value: SurfaceLiteral::Int(value),
            ..
        } => value
            .parse()
            .unwrap_or_else(|_| panic!("expected int literal expr, got non-integer `{value}`")),
        other => panic!("expected int literal expr, got {other:?}"),
    }
}

fn expr_bool(expr: &SurfaceExpr) -> bool {
    match expr {
        SurfaceExpr::Literal {
            value: SurfaceLiteral::Bool(value),
            ..
        } => *value,
        SurfaceExpr::Var { name, .. } if name == "true" => true,
        SurfaceExpr::Var { name, .. } if name == "false" => false,
        other => panic!("expected bool literal expr, got {other:?}"),
    }
}

fn record_field_from_fields<'a>(
    fields: &'a [SurfaceRecordField],
    field_name: &str,
) -> &'a SurfaceExpr {
    fields
        .iter()
        .find(|field| field.name == field_name)
        .map(|SurfaceRecordField { value, .. }| value)
        .unwrap_or_else(|| panic!("record body missing `{field_name}` field"))
}

fn strict_env_binding(expr: &SurfaceExpr) -> Option<(&str, &str)> {
    match expr {
        SurfaceExpr::Var { name, .. } if name == "NoStrictEnvBinding" => None,
        SurfaceExpr::VariantRecord { target, fields, .. } if target == "StrictEnvBinding" => {
            Some((
                expr_string(record_field_from_fields(fields, "var")),
                expr_string(record_field_from_fields(fields, "value")),
            ))
        }
        other => panic!("expected strict env binding expr, got {other:?}"),
    }
}

fn host_script_shell_path(expr: &SurfaceExpr) -> Option<&str> {
    match expr {
        SurfaceExpr::Var { name, .. } if name == "NoShellScript" => None,
        SurfaceExpr::VariantRecord { target, fields, .. } if target == "ShellScript" => {
            Some(expr_string(record_field_from_fields(fields, "path")))
        }
        other => panic!("expected host script expr, got {other:?}"),
    }
}

/// True when `job_id` is a deleted bankruptcy legacy *workflow job* block (not `affected` outputs).
fn ci_yml_has_deleted_legacy_top_level_job(workflow_yml: &str, job_id: &str) -> bool {
    // Legacy jobs used `if:` before `needs:` / `runs-on:` (see main pre-bankruptcy ci.yml).
    workflow_yml.contains(&format!("\n  {job_id}:\n    if:"))
        || workflow_yml.contains(&format!("\n  {job_id}:\n    needs:"))
        || workflow_yml.contains(&format!("\n  {job_id}:\n    runs-on:"))
}

fn workflow_step_block<'a>(workflow_yml: &'a str, step_name: &str) -> &'a str {
    let marker = format!("    - name: {step_name}");
    let start = workflow_yml
        .find(&marker)
        .unwrap_or_else(|| panic!("{CI_YML_PATH}: missing workflow step `{step_name}`"));
    let rest = &workflow_yml[start..];
    let end = rest.find("\n    - name: ").unwrap_or(rest.len());
    &rest[..end]
}

fn workflow_dag_job_block<'a>(workflow_dag: &'a str, job_id: &str) -> &'a str {
    let marker = format!("id: \"{job_id}\"");
    let id_start = workflow_dag
        .find(&marker)
        .unwrap_or_else(|| panic!("{CI_WORKFLOW_DAG_PATH}: missing `{job_id}` job"));
    let block_start = workflow_dag[..id_start]
        .rfind('{')
        .unwrap_or_else(|| panic!("{CI_WORKFLOW_DAG_PATH}: malformed `{job_id}` job"));
    let mut depth = 0usize;
    for (offset, ch) in workflow_dag[block_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth = depth
                    .checked_sub(1)
                    .unwrap_or_else(|| panic!("{CI_WORKFLOW_DAG_PATH}: malformed job braces"));
                if depth == 0 {
                    let end = block_start + offset + ch.len_utf8();
                    return &workflow_dag[block_start..end];
                }
            }
            _ => {}
        }
    }
    panic!("{CI_WORKFLOW_DAG_PATH}: unterminated `{job_id}` job")
}

/// True for a generated-workflow job field line (6-space indent), not nested step rows (10+ spaces).
fn workflow_dag_line_is_job_level_field(line: &str) -> bool {
    line.strip_prefix("      ")
        .is_some_and(|rest| !rest.starts_with(' '))
}

/// Fail-closed: job-level `continue_on_error` must be present and exactly `false` (step-level must not satisfy).
fn workflow_dag_job_level_continue_on_error_is_false(job_block: &str) -> bool {
    let mut saw_exactly_false = false;
    for line in job_block.lines().filter(|line| {
        workflow_dag_line_is_job_level_field(line)
            && line.trim_start().starts_with("continue_on_error:")
    }) {
        let value = line
            .trim_start()
            .strip_prefix("continue_on_error:")
            .unwrap_or_default()
            .trim()
            .trim_end_matches(',');
        if value == "false" {
            saw_exactly_false = true;
        } else {
            return false;
        }
    }
    saw_exactly_false
}

fn is_ci_yml_job_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}

/// True for a workflow top-level job header line (`  job_id:`), not nested `    key:` rows.
fn ci_yml_line_is_top_level_job_header(line: &str) -> bool {
    let rest = match line.strip_prefix("  ") {
        Some(r) if !r.starts_with(' ') => r,
        _ => return false,
    };
    let Some(colon) = rest.find(':') else {
        return false;
    };
    let name = &rest[..colon];
    !name.is_empty() && name.chars().all(is_ci_yml_job_name_char)
}

/// Byte index in `rest` (suffix after `  {job_id}:`) of the next sibling top-level job, if any.
fn ci_yml_next_sibling_job_index(rest: &str) -> Option<usize> {
    let mut offset = 0;
    while offset < rest.len() {
        let newline_rel = rest[offset..].find('\n')?;
        let line_start = offset + newline_rel + 1;
        if line_start >= rest.len() {
            break;
        }
        let line = rest[line_start..].split('\n').next().unwrap_or("");
        if ci_yml_line_is_top_level_job_header(line) {
            return Some(line_start - 1);
        }
        offset = line_start;
    }
    None
}

/// Slice one top-level job block from `.github/workflows/ci.yml`.
fn ci_yml_job_block<'a>(workflow_yml: &'a str, job_id: &str) -> &'a str {
    let marker = format!("  {job_id}:");
    let start = workflow_yml
        .find(&marker)
        .unwrap_or_else(|| panic!("{CI_YML_PATH}: missing job `{job_id}`"));
    let rest = &workflow_yml[start + marker.len()..];
    let end = ci_yml_next_sibling_job_index(rest)
        .map(|i| start + marker.len() + i)
        .unwrap_or(workflow_yml.len());
    &workflow_yml[start..end]
}

/// True for a workflow job field line (`    key:`), not step list rows (`    - name:`) or nested keys.
fn ci_yml_line_is_job_level_field(line: &str) -> bool {
    line.strip_prefix("    ")
        .is_some_and(|rest| !rest.starts_with(' ') && !rest.starts_with('-'))
}

/// Live receipt: sliced job must not declare job-level `continue-on-error` anywhere (keys are order-insensitive).
fn ci_yml_job_level_omits_continue_on_error(job_block: &str) -> bool {
    !job_block.lines().any(|line| {
        ci_yml_line_is_job_level_field(line) && line.trim_start().starts_with("continue-on-error:")
    })
}

fn surface_declares_test_claim_data(
    module: &v3_compiler::parse_surface::SurfaceModule,
    name: &str,
) -> bool {
    use v3_compiler::parse_surface::SurfaceType;
    module.items.iter().any(|item| match item {
        SurfaceItem::Data {
            name: item_name,
            ty: SurfaceType::Named { name: ty_name, .. },
            ..
        } => item_name == name && ty_name == "TestClaim",
        _ => false,
    })
}

#[test]
fn v4_workflow_ci_dag_tokenizes_and_parses() {
    let _module = parse_module(CI_DAG, CI_DAG_PATH);
}

#[test]
fn v4_workflow_ci_test_claim_selection_entrypoints() {
    let module = parse_module(CI_DAG, CI_DAG_PATH);
    assert_eq!(
        module_paths(&module),
        vec![vec!["v4", "workflow", "ci"]],
        "{CI_DAG_PATH}: module authority path"
    );
    for name in [
        "ci_select_from_rerun_nodes",
        "ci_select_from_affected_set",
        "ci_select_ci_jobs_from_affected_set",
        "ci_component_affected_from_git_diff",
        "test_claim_in_rerun_frontier",
        "ci_all_gate_run_policies_resolve",
    ] {
        assert!(
            surface_declares_fn(&module, name),
            "{CI_DAG_PATH}: must declare {name}"
        );
    }
    assert!(
        import_includes_name(
            &module,
            &["v4", "lens", "affected_set"],
            "affected_set_rerun_nodes"
        ),
        "{CI_DAG_PATH}: must import affected_set_rerun_nodes from T-21 lens"
    );
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "verification"],
            "test_claim_evaluation_touches_rerun_frontier"
        ),
        "{CI_DAG_PATH}: must import subtree-aware frontier membership from verification"
    );
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "verification"],
            "test_claim_ci_selection_fail_closed"
        ),
        "{CI_DAG_PATH}: must import fail-closed diagnostic selection guard from verification"
    );
    assert!(
        import_includes_name(&module, &["v4", "std", "algebra"], "filter"),
        "{CI_DAG_PATH}: must import filter from std.algebra"
    );
    assert!(
        import_includes_name(&module, &["v4", "std", "algebra"], "any"),
        "{CI_DAG_PATH}: must import any from std.algebra for path-bucket existence"
    );
    assert!(
        CI_DAG.contains("any(xs: git_diff.changed_paths, predicate: ci_changed_path_affects_v2)"),
        "{CI_DAG_PATH}: component affected-set must use std.algebra any"
    );
    assert!(
        !CI_DAG.contains("CiGitDiffReadOutcome"),
        "{CI_DAG_PATH}: git diff read must use canonical Outcome<GitDiffNameOnly>, not a parallel coproduct"
    );
    assert!(
        CI_DAG.contains(
            "ci_component_affected_from_git_diff_read(outcome: Outcome<GitDiffNameOnly>)"
        ),
        "{CI_DAG_PATH}: git diff read boundary must project through std.diagnostic Outcome"
    );
    assert!(
        !CI_DAG.contains("ci_m1_probe_cargo_fanout_slots_from_fleet"),
        "{CI_DAG_PATH}: M1 cargo jobs must not use reverse-engineered fleet fanout derivation"
    );
    assert!(
        !CI_DAG.contains("data m1_probe_cargo_check_jobs_ceiling"),
        "{CI_DAG_PATH}: M1 gate is emit-receipt only — no cargo-check parallelism constant"
    );
    assert!(
        CI_DAG.contains("RerunNodeSetFailClosed { evidence: _ } => roster"),
        "{CI_DAG_PATH}: fail-closed must return full roster on RerunNodeSetFailClosed (AI-16/R1-7)"
    );
    assert!(
        CI_DAG.contains("test_claim_ci_selection_fail_closed(c: claim)"),
        "{CI_DAG_PATH}: DiagnosticClaim rows must bypass narrow filter via fail-closed guard"
    );
}

#[test]
fn v4_workflow_ci_workflow_policy_prefixes_align_rust_mirror() {
    assert_ci_dag_rust_bucket_parity("ci_changed_path_affects_workflow_policy");
}

#[test]
fn v4_workflow_ci_component_bucket_prefixes_align_rust_mirror() {
    for fn_name in [
        "ci_changed_path_affects_v2",
        "ci_changed_path_affects_v3",
        "ci_changed_path_affects_v4",
        "ci_changed_path_affects_testclaim_corpus",
    ] {
        assert_ci_dag_rust_bucket_parity(fn_name);
    }
}

#[test]
fn v4_workflow_ci_release_distribution_exact_paths_align_rust_mirror() {
    assert_ci_dag_rust_bucket_parity("ci_changed_path_affects_release_distribution");
}

#[test]
fn v4_workflow_ci_m1_rust_emit_probe_modeled_and_bound_to_ci_yml() {
    // ci.dag ↔ Rust mirror parity (P2 consumer — set equality + behavioral fixtures; CI `--exact` step).
    assert_ci_dag_rust_mirror_full_parity();

    let module = parse_module(CI_DAG, CI_DAG_PATH);
    assert!(
        module.items.iter().any(|item| matches!(
            item,
            SurfaceItem::TypeSum { name, .. } if name == "CiCommand"
        )),
        "{CI_DAG_PATH}: CiCommand must exist"
    );
    assert!(
        CI_DAG.contains("| M1RustEmitProbeCommand"),
        "{CI_DAG_PATH}: M1 probe must be a CiCommand arm"
    );
    assert!(
        CI_DAG.contains("feature:project-github-actions-landed")
            && CI_DAG.contains("consumer:v4.workflow.ci m1_ci_live_workflow_signal")
            && CI_DAG.contains(
                "bind ROADMAP.md T-PB-B + src/v4/test/claim/workflow/runner_pool_m1_probe.dag"
            ),
        "{CI_DAG_PATH}: M1 live-workflow bridge must carry checkable P5 dissolution tags"
    );
    assert!(
        surface_declares_fn(&module, "ci_command_authority_ok"),
        "{CI_DAG_PATH}: command authority must cover M1RustEmitProbeCommand"
    );
    assert!(
        CI_DAG.contains("M1RustEmitProbeCommand => true"),
        "{CI_DAG_PATH}: M1 probe command must pass authority check"
    );
    assert!(
        CI_DAG.contains("id: m1_rust_emit_probe_execution")
            && CI_DAG.contains("id: m1_rust_emit_probe_signal"),
        "{CI_DAG_PATH}: ci_pipeline must declare M1 job and gate"
    );
    let live_signal = data_body(&module, "m1_ci_live_workflow_signal");
    let live_step = record_body_field(live_signal, "step");
    let step_name = expr_string(record_body_field(live_step, "step_name"));
    let script_path = host_script_shell_path(record_body_field(live_step, "host_script"))
        .expect("{CI_DAG_PATH}: M1 probe must model ShellScript host transport");
    let non_blocking = expr_bool(record_body_field(live_step, "non_blocking"));
    let timeout_minutes = expr_int(record_body_field(live_step, "timeout_minutes"));
    let (strict_env_var, strict_env_value) =
        strict_env_binding(record_body_field(live_step, "strict_env_binding"))
            .unwrap_or_else(|| panic!("{CI_DAG_PATH}: M1 probe must model a strict env binding"));
    let rust_emit_probe_policy = record_body_field(live_signal, "rust_emit_probe_policy");
    let emit_preconditions_block_required_path = expr_bool(record_body_field(
        rust_emit_probe_policy,
        "emit_preconditions_block_required_path",
    ));
    let rustc_residuals_block_required_path = expr_bool(record_body_field(
        rust_emit_probe_policy,
        "rustc_residuals_block_required_path",
    ));
    assert!(
        !CI_YML.contains("M1 v4 workflow CI model/YAML binding smoke")
            && !CI_YML.contains(
                "v4_workflow_ci_runner_dag_smoke_test::v4_workflow_ci_m1_rust_emit_probe_modeled_and_bound_to_ci_yml"
            )
            && !CI_YML.contains("v4_workflow_ci_runner_dag_smoke_test::v4_workflow_ci_bankruptcy_tier0_")
            && !CI_YML.contains("v4_workflow_ci_runner_dag_smoke_test::v4_workflow_ci_wave1_"),
        "{CI_YML_PATH}: CI model/YAML binding smoke is retired from the every-PR ci_floor"
    );
    let m1_step = workflow_step_block(CI_YML, step_name);
    assert!(
        !m1_step.contains("needs.affected.outputs"),
        "{CI_YML_PATH}: Wave 1 §11.7.1 — `{step_name}` runs unconditionally on the safety floor (no component `if:`)"
    );
    assert!(
        m1_step.contains(&format!("{strict_env_var}: \"{strict_env_value}\"")),
        "{CI_YML_PATH}: `{step_name}` strictness env must come from {CI_DAG_PATH}:m1_ci_live_workflow_signal"
    );
    assert_eq!(
        strict_env_value == "1",
        rustc_residuals_block_required_path,
        "{CI_DAG_PATH}: strictness env and rustc residual required-path policy must agree"
    );
    assert!(
        emit_preconditions_block_required_path,
        "{CI_DAG_PATH}: required M1 probe must fail closed on missing compiler, v2 emit failure, and missing/zero-diagnostic compile receipt"
    );
    assert!(
        CI_DAG.contains(
            "data m1_rust_emit_probe_shared_dag_out_env: String = \"V4_M1_DAG_EMIT_OUT\""
        ) && CI_DAG.contains(
            "data m1_rust_emit_probe_shared_dag_log_env: String = \"V4_M1_DAG_EMIT_LOG\""
        ) && CI_DAG.contains(
            "data m1_rust_dag_emit_parity_receipt_test: String = \"cargo test -p v2-compiler-tests --release pipeline::dag_emit_from_resolved_matches_compile_sources_for_v4_slice -- --exact --quiet\""
        ),
        "{CI_DAG_PATH}: shared M1/bootstrap closure env names must be modeled"
    );
    assert!(
        m1_step.contains("V4_M1_DAG_EMIT_OUT:") && m1_step.contains("V4_M1_DAG_EMIT_LOG:"),
        "{CI_YML_PATH}: `{step_name}` must emit the DAG artifact from the shared rust+dag closure"
    );
    assert!(
        CI_DAG.contains(
            "M1RustEmitProbeCommand =>\n      ci_job_component_mask_row(\n        v2: false,\n        v3: false,\n        v4: true,\n        testclaim_corpus: true,\n        workflow_policy: true,\n        release_distribution: true\n      )"
        ),
        "{CI_DAG_PATH}: M1RustEmitProbeCommand mask must match ci.yml step if (I8 / T-22 upstream)"
    );
    if non_blocking {
        assert!(
            m1_step.contains("continue-on-error: true"),
            "{CI_YML_PATH}: `{step_name}` must set continue-on-error when modeled non_blocking is true"
        );
    } else {
        assert!(
            !m1_step.contains("continue-on-error: true"),
            "{CI_YML_PATH}: `{step_name}` must not set continue-on-error when modeled non_blocking is false"
        );
    }
    assert!(
        m1_step.contains(&format!("timeout-minutes: {timeout_minutes}")),
        "{CI_YML_PATH}: `{step_name}` must set timeout-minutes from modeled timeout_minutes"
    );
    assert!(
        m1_step.contains(&format!("run: bash {script_path}")),
        "{CI_YML_PATH}: `{step_name}` must invoke the modeled probe script"
    );
    assert!(
        CI_DAG.contains("type SelfHostedRunnerPool"),
        "{CI_DAG_PATH}: must model self-hosted runner pools (T-24 addendum)"
    );
    assert!(
        CI_DAG.contains("arch: ci_runner_arch_arm64"),
        "{CI_DAG_PATH}: fleet arch must be a typed Symbol field on SelfHostedRunnerPool"
    );
    assert!(
            !CI_DAG.contains("RunnerArch"),
            "{CI_DAG_PATH}: fleet arch uses Symbol field — no RunnerArch coproduct in Lens-CI entry compile"
        );
    assert!(
        CI_DAG.contains("data ci_srv1_pool: SelfHostedRunnerPool")
            && CI_DAG.contains("runner_count: 20")
            && CI_DAG.contains("jobserver_token_cap: 25")
            && CI_DAG.contains("data ci_srv2_pool: SelfHostedRunnerPool")
            && CI_DAG.contains("runner_count: 30")
            && CI_DAG.contains("jobserver_token_cap: 36"),
        "{CI_DAG_PATH}: srv1/srv2 pool rows must match operator spec"
    );
    assert!(
        !m1_step.contains("V4_M1_CARGO_CHECK_JOBS_CEILING")
            && !m1_step.contains("V4_M1_CARGO_CHECK_JOBS:")
            && !m1_step.contains("V4_M1_RUSTC"),
        "{CI_YML_PATH}: M1 step must not project non-gating cargo-check telemetry env"
    );
    assert!(
        M1_RUST_EMIT_PROBE_SCRIPT.contains("^compiled: [0-9]+ files emitted, [0-9]+ diagnostics$")
            && M1_RUST_EMIT_PROBE_SCRIPT.contains("0 diagnostics")
            && M1_RUST_EMIT_PROBE_SCRIPT.contains("at least one emitted file")
            && M1_RUST_EMIT_PROBE_SCRIPT.contains("M1 shared rust+dag probe requires non-empty DAG artifact"),
        ".github/ci-floor/v4-m1-rust-emit-probe.sh: gate must fail closed on compile receipt (0 diagnostics, N≥1)"
    );
    assert!(
        !M1_RUST_EMIT_PROBE_SCRIPT.contains("cargo check"),
        ".github/ci-floor/v4-m1-rust-emit-probe.sh: M1 gate is v2 emit + receipt only — no cargo check"
    );
    let parity_step = workflow_step_block(CI_YML, "v2 DAG emit parity receipt (shared closure)");
    assert!(
        parity_step.contains(
            "cargo test -p v2-compiler-tests --release pipeline::dag_emit_from_resolved_matches_compile_sources_for_v4_slice -- --exact --quiet"
        ),
        "{CI_YML_PATH}: bootstrap reuse must be preceded by the v2 DAG emit parity receipt"
    );
    assert!(
        !CI_YML.contains("v2 -> v4 bootstrap compile (fail-closed full)")
            && !CI_YML.contains("V4_BOOTSTRAP_REUSE_LOG:")
            && !CI_YML.contains("bash .github/ci-floor/v4-bootstrap-viability.sh"),
        "{CI_YML_PATH}: redundant bootstrap reuse step must be folded into the M1 rust+dag probe"
    );
}

#[test]
fn v4_ci_shared_closure_worksheet_is_ratified_against_live_authorities() {
    assert!(
        SHARED_CLOSURE_WORKSHEET.contains("**Status:** RATIFIED")
            && SHARED_CLOSURE_WORKSHEET.contains("node://adhoc-197c65c6-8cd"),
        "{SHARED_CLOSURE_WORKSHEET_PATH}: worksheet must be ratified by the active arbiter node"
    );
    assert!(
        SHARED_CLOSURE_WORKSHEET.contains("ROADMAP.md` row T-PB-B")
            && SHARED_CLOSURE_WORKSHEET.contains("pb_rust_tests_outside_residual_zero")
            && SHARED_CLOSURE_WORKSHEET.contains("src/v4/test/claim/workflow/{ci_component_affected,affected_set_ci_runner,runner_pool_m1_probe}.dag"),
        "{SHARED_CLOSURE_WORKSHEET_PATH}: P5 receipt must bind to checkable in-tree authorities"
    );
    assert!(
        !SHARED_CLOSURE_WORKSHEET.contains("src/v4/TASKS.md"),
        "{SHARED_CLOSURE_WORKSHEET_PATH}: ratification must not cite missing src/v4/TASKS.md"
    );
    assert!(
        SHARED_CLOSURE_WORKSHEET.contains("## §4 Lane H Lens Dispositions")
            && SHARED_CLOSURE_WORKSHEET.contains("TestgenSlotSelection")
            && SHARED_CLOSURE_WORKSHEET.contains("Generator<TestgenConcept>")
            && SHARED_CLOSURE_WORKSHEET.contains("target-arrow-domain-param-list-carrier"),
        "{SHARED_CLOSURE_WORKSHEET_PATH}: Lane H/testgen §8 dispositions must be explicit"
    );
}

// P5(b) receipt: `ROADMAP.md` § **Nine lanes** **T-PB-B** / `pb_rust_tests_outside_residual_zero`;
// same-path expansion in this file (SG-0 delta 0 — path already in `EXPECTED_HAND_AUTHORED_TEST`;
// #4091 §1.2 four-compile collapse modeled UpstreamUpsert pin for m1 / lens-CI / phase1 rung gate).

/// #4091 §1.2 four-compile collapse: the modeled ci_pipeline jobs that re-ran a full src/v4
/// 332-source closure compile must consume `v2_compile_src_v4`'s resolved closure via
/// UpstreamUpsert (front-end shared once; each job re-emits its own target) instead of each
/// independently re-running parse/lower/infer. Pins the artifact-consumption edge for m1,
/// lens-CI, and the phase1 rung gate. (bootstrap-dag is split out: its dissolution target is
/// `ModelMissingSubstrate { what: v4_bootstrap_dag_emit_ci_job }` — substrate, follow-up.)
#[test]
fn v4_workflow_ci_four_compile_collapse_jobs_consume_v2_compile_src_v4_closure() {
    // Slice the `data <symbol>...` definition up to the next top-level `\ndata ` so the
    // upstream-edge assertion is scoped to each job's own inputs block.
    let block_for = |symbol: &str| -> &str {
        let start = CI_DAG
            .find(&format!("data {symbol}"))
            .unwrap_or_else(|| panic!("{CI_DAG_PATH}: missing `data {symbol}`"));
        let rest = &CI_DAG[start..];
        let end = rest[1..]
            .find("\ndata ")
            .map(|i| i + 1)
            .unwrap_or(rest.len());
        &rest[..end]
    };
    let upstream_edge = "ci_upsert_upstream_job_input(job: v2_compile_src_v4)";
    // Each formerly-recompiling job declares the upstream artifact edge on its CiUpsertStep
    // inputs, mirroring the v4_t15 / testclaim_corpus_eval pattern at ci.dag:807.
    assert!(
        block_for("ci_upsert_m1_rust_emit_probe_execution_inputs").contains(upstream_edge),
        "{CI_DAG_PATH}: M1 rust emit probe must consume v2_compile_src_v4's resolved closure via UpstreamUpsert (#4091 §1.2 four-compile collapse)"
    );
    assert!(
        block_for("ci_upsert_phase1_nat_semiring_rung_gate_execution_inputs")
            .contains(upstream_edge),
        "{CI_DAG_PATH}: phase1 rung gate must consume v2_compile_src_v4's resolved closure via UpstreamUpsert (#4091 §1.2 four-compile collapse)"
    );
    // lens-CI inputs are computed (list_snoc_item over per-lens refs); the upstream edge is
    // appended in the data initializer.
    assert!(
        block_for("ci_upsert_lens_ci_registry_execution_inputs").contains(upstream_edge),
        "{CI_DAG_PATH}: lens-CI registry execution must consume v2_compile_src_v4's resolved closure via UpstreamUpsert (#4091 §1.2 four-compile collapse)"
    );
    // The three jobs already declare v2_compile_src_v4 in `needs` (ordering); the collapse turns
    // those ordering deps into real artifact-consumption edges (above). Spot-check needs stay.
    assert!(
        CI_DAG.contains("id: m1_rust_emit_probe_execution,")
            && CI_DAG.contains("id: lens_ci_registry_execution,")
            && CI_DAG.contains("id: phase1_nat_semiring_rung_gate_execution,"),
        "{CI_DAG_PATH}: collapse must not remove the three modeled closure-compile jobs"
    );
}

#[test]
fn v4_workflow_ci_testclaim_corpus_eval_modeled_and_bound_to_ci_yml() {
    let _module = parse_module(CI_DAG, CI_DAG_PATH);
    const CORPUS_EVAL_STEP: &str = "T-22 TestClaim corpus eval (tracked-expectation drift gate)";
    assert!(
        CI_DAG.contains("data testclaim_corpus_eval_ci_live_workflow_signal"),
        "{CI_DAG_PATH}: corpus eval must model the live-workflow signal ledger row"
    );
    assert!(
        CI_DAG.contains("ci_upsert_testclaim_corpus_eval_upstream_inputs")
            && CI_DAG.contains("ci_upsert_upstream_job_input(job: m1_rust_emit_probe_execution)"),
        "{CI_DAG_PATH}: testclaim corpus eval execution must consume M1 rust emit via UpstreamUpsert (#4091 §1.2)"
    );
    assert!(
        CI_DAG.contains("needs: [v2_compile_src_v4, m1_rust_emit_probe_execution]"),
        "{CI_DAG_PATH}: testclaim corpus eval must declare M1 in needs for selector needs-closure (I8)"
    );
    assert!(
        CI_YML.contains(&format!("- name: {CORPUS_EVAL_STEP}")),
        "{CI_YML_PATH}: tracked-expectation corpus eval must be on the required path"
    );
    assert!(
        CI_YML.contains("bash scripts/v4-testclaim-corpus-eval.sh"),
        "{CI_YML_PATH}: corpus eval host transport must invoke scripts/v4-testclaim-corpus-eval.sh (outside §11.7.5 ci-floor ratchet)"
    );
    assert!(
        CI_YML.contains("ci_corpus_eval:"),
        "{CI_YML_PATH}: must declare the `ci_corpus_eval` job"
    );
    assert!(
        !CI_YML.contains("v4-testclaim-corpus-gate.sh"),
        "{CI_YML_PATH}: legacy shell bridge script must be absent from workflow"
    );
}

#[test]
fn v4_workflow_ci_testclaim_corpus_eval_ratchet_is_asymmetric_and_modeled() {
    let _module = parse_module(
        MANUAL_CORPUS_EVAL_EXPECTED_DAG,
        MANUAL_CORPUS_EVAL_EXPECTED_PATH,
    );
    assert!(
        MANUAL_CORPUS_EVAL_EXPECTED_DAG.contains("type CorpusEvalPinStatus")
            && MANUAL_CORPUS_EVAL_EXPECTED_DAG.contains("= PinnedRed")
            && MANUAL_CORPUS_EVAL_EXPECTED_DAG.contains("| MustPass"),
        "{MANUAL_CORPUS_EVAL_EXPECTED_PATH}: corpus eval pins must model provisional-red vs ratcheted must-pass state"
    );
    assert!(
        MANUAL_CORPUS_EVAL_EXPECTED_DAG.contains("type CorpusEvalPinEvaluation")
            && MANUAL_CORPUS_EVAL_EXPECTED_DAG
                .contains("| CorpusEvalPinRatchetForward { dissolution_target: Symbol }")
            && MANUAL_CORPUS_EVAL_EXPECTED_DAG.contains("| CorpusEvalPinRegression"),
        "{MANUAL_CORPUS_EVAL_EXPECTED_PATH}: gate must classify ratchet-forward separately from regression"
    );
    assert!(
        MANUAL_CORPUS_EVAL_EXPECTED_DAG.contains("fn corpus_eval_pin_status_well_formed")
            && MANUAL_CORPUS_EVAL_EXPECTED_DAG.contains("MustPass =>")
            && MANUAL_CORPUS_EVAL_EXPECTED_DAG.contains("corpus_eval_verdict_pass_reason")
            && MANUAL_CORPUS_EVAL_EXPECTED_DAG
                .contains("PinnedRed => match corpus_eval_observation_is_pass")
            && MANUAL_CORPUS_EVAL_EXPECTED_DAG.contains("true => false")
            && MANUAL_CORPUS_EVAL_EXPECTED_DAG.contains("false => true"),
        "{MANUAL_CORPUS_EVAL_EXPECTED_PATH}: MustPass pins must be locked to ExecutedPass/pass-reason and PinnedRed pins must stay non-pass"
    );
    assert_eq!(
        MANUAL_CORPUS_EVAL_EXPECTED_DAG
            .matches("status: PinnedRed")
            .count(),
        5,
        "{MANUAL_CORPUS_EVAL_EXPECTED_PATH}: current 0/5 corpus baseline must keep all five rows provisional-red"
    );
    assert!(
        !MANUAL_CORPUS_EVAL_EXPECTED_DAG.contains("status: MustPass"),
        "{MANUAL_CORPUS_EVAL_EXPECTED_PATH}: no row is green yet; first green must land as an explicit pin-table ratchet"
    );
    assert!(
        MANUAL_CORPUS_EVAL_EXPECTED_DAG
            .contains("CorpusEvalPinRatchetForward { dissolution_target: pin.dissolution_target }")
            && MANUAL_CORPUS_EVAL_EXPECTED_DAG
                .contains("CorpusEvalPinRatchetForward { dissolution_target: _ } => false")
            && MANUAL_CORPUS_EVAL_EXPECTED_DAG.contains("CorpusEvalPinRegression => false"),
        "{MANUAL_CORPUS_EVAL_EXPECTED_PATH}: a PinnedRed pass must flow through modeled ratchet-forward state and fail until the pin is locked; regression also fails hard"
    );
    assert!(
        MANUAL_CORPUS_EVAL_EXPECTED_DAG.contains("fn corpus_eval_executed_row_ratchet_forward(")
            && MANUAL_CORPUS_EVAL_EXPECTED_DAG.contains("Fail { actual: Rejected")
            && MANUAL_CORPUS_EVAL_EXPECTED_DAG.contains("Deferred { diagnostic: d"),
        "{MANUAL_CORPUS_EVAL_EXPECTED_PATH}: executed-row ratchet prompts must inspect the actual TestClaimRun, not assume every gate failure is a pass"
    );
    assert!(
        MANUAL_CORPUS_EVAL_EXPECTED_DAG
            .contains("fn witness_corpus_eval_row_parallelism_transport_pass_ratchet_forward")
            && MANUAL_CORPUS_EVAL_EXPECTED_DAG
                .contains("fn witness_corpus_eval_row_effect_transport_pass_ratchet_forward"),
        "{MANUAL_CORPUS_EVAL_EXPECTED_PATH}: CDV HostRejected rows need modeled transport-pass ratchet witnesses for the first green"
    );
    assert!(
        CI_YML.contains("bash scripts/v4-testclaim-corpus-eval.sh"),
        "{CI_YML_PATH}: corpus eval transport must stay in the existing script; no ci.yml/carrier cascade for this ratchet"
    );
    for needle in [
        "claim_run_required()",
        "host_rejected_or_transport_pass_required()",
        "flip its pin PinnedRed->MustPass",
        "TestClaimRun { verdict: Pass",
        "witness_corpus_eval_row_parallelism_transport_pass_ratchet_forward",
        "witness_corpus_eval_row_effect_transport_pass_ratchet_forward",
        "--claim-run",
    ] {
        assert!(
            TESTCLAIM_CORPUS_EVAL_SCRIPT.contains(needle),
            "{TESTCLAIM_CORPUS_EVAL_SCRIPT_PATH}: thin transport must surface modeled ratchet outcome `{needle}`"
        );
    }
    assert!(
        TESTCLAIM_CORPUS_EVAL_SCRIPT.contains(
            "out=\"$(\"$bin\" run --source-root src/v4 --entry \"$row_entry\" --function \"$row_fn\" 2>&1)\""
        ),
        "{TESTCLAIM_CORPUS_EVAL_SCRIPT_PATH}: TestClaimRun row transport must not use --claim-run; row functions return TestClaimRun, not Bool"
    );
}

#[test]
fn v4_workflow_ci_bootstrap_gate_skip_policy_is_modeled() {
    let module = parse_module(CI_DAG, CI_DAG_PATH);
    assert!(
        module.items.iter().any(|item| matches!(
            item,
            SurfaceItem::TypeSum { name, .. } if name == "CiGateRunPolicy"
        )),
        "{CI_DAG_PATH}: must model live gate run policy"
    );
    assert!(
        CI_DAG.contains("| RequiresJobAttempt { job: Symbol }"),
        "{CI_DAG_PATH}: gate run policy must carry the required attempted job"
    );
    assert!(
        CI_DAG.contains("run_policy: RequiresJobAttempt { job: v2_compile_src_v4 }"),
        "{CI_DAG_PATH}: structural v2 compile gate must require the bootstrap compile attempt"
    );
    assert!(
        CI_DAG.contains("ci_all_gate_run_policies_resolve(gates: p.gates, jobs: p.jobs)"),
        "{CI_DAG_PATH}: ci_pipeline_well_formed must reject dangling gate run-policy jobs"
    );
    assert!(
        !CI_YML.contains("bash .github/ci-floor/v4-bootstrap-viability.sh")
            && M1_RUST_EMIT_PROBE_SCRIPT
                .contains("M1 shared rust+dag probe requires non-empty DAG artifact"),
        "{CI_YML_PATH}: bootstrap artifact viability is folded into the M1 rust+dag probe"
    );
    assert!(
        CI_DAG.contains("workflow_local_bootstrap_dag_upstream")
            && CI_DAG.contains("v4_bootstrap_dag_emit_ci_job"),
        "{CI_DAG_PATH}: testclaim corpus eval must document workflow-local bootstrap via ci_always_run_carveouts (P5 §1.2)"
    );
    assert!(
        !CI_YML.contains("T-22 TestClaim corpus structural bridge"),
        "{CI_YML_PATH}: retired bridge step name must not appear anywhere in workflow YAML"
    );
}

/// Wave 2 — §11.7.5 temporary shell exception table (first executable slice). Authority: #4137
/// §11.7.5. Models the Class-A shell-owned floor steps allowed to stay REQUIRED pending their
/// `CiUpsertStep` runtime-authority migration (bootstrap viability + M1 rust emit), and the
/// no-new-shell ratchet (§11.7.1 #5): every REQUIRED ci-floor shell invocation in ci.yml must
/// carry a row here. Dissolve-on: ci.yml emits these steps FROM their `CiUpsertStep` (T-24),
/// at which point the rows delete and this slice retires.
///
/// **P5 receipt (T-PB-B same-path SG-0 expansion):** this is a +0-new-path addition to the
/// existing `v4_workflow_ci_runner_dag_smoke_test` harness (already carried by one SG-0
/// hand-authored-test census entry), NOT a new hand-Rust test path/authority surface — same
/// posture as the T-38/T-38B expansions documented in this file's header. Explicit P5 deferral:
/// ROADMAP.md § "Nine lanes" row **T-PB-B** / `pb_rust_tests_outside_residual_zero`. Dissolution:
/// generated `.dag` `TestClaim` execution covers the §11.7.5 shell-exception table + no-new-shell
/// ratchet (and A15 Shape-B emitted `ci.yml`), retiring this hand-Rust parse/string harness.
///
/// **Required-CI wiring:** this ratchet used to run under the every-PR CI model/YAML binding smoke.
/// That smoke was retired from `ci_floor` because it compiled the v3 integration binary on every PR.
/// The test remains the modeled/local receipt for edits touching `ci.dag` or `.github/workflows/ci.yml`.
#[test]
fn v4_workflow_ci_wave1_class_a_shell_exception_table_first_slice() {
    let module = parse_module(CI_DAG, CI_DAG_PATH);

    // Type substrate: the exception-row record + the §11.7.1 safety-floor enum it cites.
    assert!(
        module.items.iter().any(|item| matches!(
            item,
            SurfaceItem::TypeRecord { name, .. } if name == "CiShellExceptionRow"
        )),
        "{CI_DAG_PATH}: §11.7.5 must model CiShellExceptionRow"
    );
    assert!(
        module.items.iter().any(|item| matches!(
            item,
            SurfaceItem::TypeSum { name, .. } if name == "CiSafetyFloorItem"
        )),
        "{CI_DAG_PATH}: §11.7.5 must model the §11.7.1 safety-floor item enum"
    );

    // Isolate the table body so per-row assertions can't accidentally match elsewhere.
    let table = CI_DAG
        .split("data ci_class_a_shell_exceptions:")
        .nth(1)
        .unwrap_or_else(|| panic!("{CI_DAG_PATH}: missing data ci_class_a_shell_exceptions"))
        .split("\n]")
        .next()
        .expect("ci_class_a_shell_exceptions table must terminate with `]`");

    // Exactly one Class-A shell-owned floor step remains: M1 rust+dag emit. Corpus eval uses
    // scripts/v4-testclaim-corpus-eval.sh (outside the §11.7.5 ci-floor ratchet).
    assert_eq!(
        table.matches("CiShellExceptionRow {").count(),
        1,
        "{CI_DAG_PATH}: §11.7.5 carries exactly one Class-A shell exception (M1)"
    );
    for needle in [
        "job: m1_rust_emit_probe_execution",
        ".github/ci-floor/v4-m1-rust-emit-probe.sh",
        "protects_floor: OneRustEmitProbe",
    ] {
        assert!(
            table.contains(needle),
            "{CI_DAG_PATH}: shell exception table must carry `{needle}`"
        );
    }

    // P2 single-authority: `replacement_upsert` binds the canonical `CiUpsertStepSymbol` data
    // declared elsewhere in this module — NOT a parallel `Symbol` alias that could drift or name
    // a non-existent upsert. Assert both the table reference and the authority declaration exist.
    for upsert in ["ci_upsert_m1_rust_emit_probe_execution"] {
        assert!(
            table.contains(&format!("replacement_upsert: {upsert}")),
            "{CI_DAG_PATH}: shell exception must point `replacement_upsert` at the canonical \
             upsert authority `{upsert}` (P2 single-authority — no parallel `_step` Symbol alias)"
        );
        assert!(
            CI_DAG.contains(&format!("data {upsert}: CiUpsertStepSymbol")),
            "{CI_DAG_PATH}: `replacement_upsert` authority `{upsert}` must be a declared \
             `CiUpsertStepSymbol` (proves the named upsert exists)"
        );
    }

    // §11.7.5 cond 4 — every row names a structural dissolution path (no calendar/owner carrier).
    assert_eq!(
        table
            .matches("dissolution_target: ModelMissingSubstrate")
            .count(),
        1,
        "{CI_DAG_PATH}: each shell exception must name a structural dissolution_target"
    );

    // no-new-shell ratchet (§11.7.1 #5) — bidirectional bijection between the REQUIRED
    // `.github/ci-floor/*.sh` invocations on the Class-A floor jobs (`ci_floor`, `ci_floor_emit`)
    // and the table's rows. Transport wrappers under `.github/ci-floor/` that run only in
    // non-floor jobs (e.g. `with-sccache-retry.sh` in `affected`) are out of §11.7.5 scope.
    //
    // Build the live invocation set FIRST, comment-safe, and reuse it for BOTH the row-liveness
    // (cond 1) and the bijection — never raw `CI_YML.contains(...)`, which a comment could satisfy.
    // Scan EVERY non-comment line (not just lines beginning `run: bash …`) so a block-scalar
    // `run: |` body such as `        bash .github/ci-floor/foo.sh` cannot evade the gate by its
    // YAML spelling, while a commented-out `# bash .github/ci-floor/foo.sh` is NOT counted as live
    // (openai-pro #4284: skip comment-only lines + strip inline ` # …` trailers).
    let ci_floor_live_yml: String = [
        CI_YML
            .split("  ci_floor:")
            .nth(1)
            .and_then(|rest| rest.split("\n  ci_floor_emit:").next()),
        CI_YML
            .split("  ci_floor_emit:")
            .nth(1)
            .and_then(|rest| rest.split("\n  ci:").next()),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join("\n");
    let mut ci_floor_scripts: Vec<String> = Vec::new();
    for raw in ci_floor_live_yml.lines() {
        let line = raw.trim_start();
        if line.starts_with('#') {
            continue;
        }
        // Drop any inline YAML comment (a ` #` trailer) before looking for an invocation.
        let line = line.split_once(" #").map_or(line, |(code, _)| code);
        // Key on the `.github/ci-floor/` PATH, not on a `bash ` prefix — a required floor step can
        // be spelled `sh …`, `./…`, `source …`, etc. Filtering on `bash ` first would let those
        // non-`bash` spellings escape the live set (openai-pro #4284). Any non-comment reference to
        // a ci-floor path is treated as a live invocation and must canonicalize to a modeled `.sh`.
        if let Some(idx) = line.find(".github/ci-floor/") {
            // Canonicalize the path token: read from the path start until whitespace OR a shell /
            // YAML separator/quote (`"' ; & | ) ``), so quoted/punctuated spellings like
            // `".github/ci-floor/x.sh"` or `.github/ci-floor/x.sh; echo` canonicalize to
            // the bare `.sh` path rather than being silently dropped.
            let token: String = line[idx..]
                .chars()
                .take_while(|&c| {
                    !c.is_whitespace() && !matches!(c, '"' | '\'' | ';' | '&' | '|' | ')' | '`')
                })
                .collect();
            // Fail CLOSED on any ci-floor reference we cannot canonicalize to a `*.sh` script:
            // a non-comment line referencing `.github/ci-floor/` that does not yield a `.sh` path
            // is treated as an unrecognized live shell the ratchet must not silently ignore
            // (openai-pro #4284 — a new required shell in an unhandled spelling must trip the gate).
            assert!(
                token.ends_with(".sh"),
                "{CI_YML_PATH}: ci-floor invocation `{}` did not canonicalize to a `*.sh` script — \
                 the no-new-shell ratchet fails CLOSED on unrecognized live shell spellings; \
                 normalize the invocation or extend the canonicalizer",
                line.trim()
            );
            ci_floor_scripts.push(token);
        }
    }
    ci_floor_scripts.sort();
    ci_floor_scripts.dedup();
    assert!(
        !ci_floor_scripts.is_empty(),
        "{CI_YML_PATH}: expected at least one required ci-floor shell invocation to ratchet"
    );

    // §11.7.5 cond 1 — each modeled shell_owner is a live REQUIRED ci.yml floor invocation,
    // checked against the SAME comment-safe parsed live set (not raw `CI_YML.contains`, which a
    // comment could satisfy). No phantom exception for a script that does not actually run on PRs.
    for script in [".github/ci-floor/v4-m1-rust-emit-probe.sh"] {
        assert!(
            ci_floor_scripts.iter().any(|s| s == script),
            "{CI_YML_PATH}: shell exception owner `{script}` must be a live (non-comment) floor invocation"
        );
    }

    // Forward — every required ci-floor shell must have an exception row, matched at the
    // `shell_owner` FIELD (`shell_owner: "<path>"`), NOT bare path containment over the slice.
    // A path can also appear in a row comment (e.g. the bootstrap row documents what the script
    // does), so containment would pass even with a stale/wrong `shell_owner` field — i.e. fail
    // OPEN. Pinning to the field spelling keeps it fail-CLOSED (openai-pro #4284).
    for script in &ci_floor_scripts {
        assert!(
            table.contains(&format!("shell_owner: \"{script}\"")),
            "{CI_YML_PATH}: required ci-floor shell `{script}` has no §11.7.5 \
             `CiShellExceptionRow.shell_owner` field (no-new-shell ratchet — the path appearing \
             only in a comment does not count; add/repair the `shell_owner` field or model it)"
        );
    }
    // Bijection — distinct required scripts must equal the row count, so a NEW ci-floor shell (or
    // a duplicate/extra row) breaks the 1:1 correspondence and fails the gate rather than passing
    // open on mere string containment.
    assert_eq!(
        ci_floor_scripts.len(),
        table.matches("CiShellExceptionRow {").count(),
        "{CI_DAG_PATH}: §11.7.5 table must be 1:1 with the required ci-floor invocations \
         in {CI_YML_PATH} (count {} scripts vs rows) — a new required shell or an extra/stale \
         row breaks the no-new-shell bijection",
        ci_floor_scripts.len()
    );

    // RESIDUAL (tracked, openai-pro #4284): this gate keys on the script PATH, not the modeled
    // `(job, step)` `CiStepId`, because ci.yml's step→`CiStepId` mapping is not yet available in a
    // string-checkable form here. A *second* required step reusing an already-listed ci-floor
    // script would not yet be distinguished by its own `step_id`. Dissolves on the same trigger as
    // this harness's overall retirement: the generated `.dag` CI step model lets the ratchet
    // compare live `(job, step, script)` against `step_id + shell_owner` directly. Until then the
    // bijection above (path-keyed, block-form-safe, count-exact) is the enforced contract.
}

#[test]
fn v4_workflow_affected_set_ci_runner_claim_dag_tokenizes_and_parses() {
    let _module = parse_module(CLAIM_DAG, CLAIM_PATH);
}

#[test]
fn v4_workflow_ci_bankruptcy_tier0_types_and_command_arms_modeled() {
    let module = parse_module(CI_DAG, CI_DAG_PATH);
    for name in ["CiBuildProfile", "CiSchedulePolicy"] {
        assert!(
            module.items.iter().any(|item| match item {
                SurfaceItem::TypeSum {
                    name: item_name, ..
                } => item_name == name,
                _ => false,
            }),
            "{CI_DAG_PATH}: must model bankruptcy Tier-0 enum `{name}`"
        );
    }
    for arm in [
        "| V2BootstrapCompileCommand",
        "| V3DeterminismCommand",
        "| V3SelfHostFixedPointCommand",
        "| V4T15SelfHostFixedPointCommand",
    ] {
        assert!(
            CI_DAG.contains(arm),
            "{CI_DAG_PATH}: CiCommand must include bankruptcy Tier-0 arm `{arm}`"
        );
    }
    assert!(
        CI_DAG.contains("fn ci_select_ci_jobs_from_affected_set("),
        "{CI_DAG_PATH}: must declare ci_select_ci_jobs_from_affected_set (I1 / A2)"
    );
    assert!(
        CI_DAG.contains("fn ci_select_ci_jobs_from_affected_set(\n  pipeline: CiPipeline,"),
        "{CI_DAG_PATH}: job selector must take well-formed CiPipeline (not bare job list)"
    );
    for step_id in [
        "v3_determinism_execution",
        "v3_self_host_fixed_point_execution",
        "v4_t15_self_host_fixed_point_execution",
    ] {
        assert!(
            CI_DAG.contains(step_id),
            "{CI_DAG_PATH}: ci_pipeline must include bankruptcy Tier-0 step `{step_id}`"
        );
    }
}

#[test]
fn v4_workflow_ci_bankruptcy_tier0_schedule_policy_carries_disposition() {
    assert!(
        CI_DAG.contains("feature:ci-bankruptcy-schedule-policy")
            && CI_DAG.contains("fn ci_job_scheduled_by_policy("),
        "{CI_DAG_PATH}: schedule-policy Bool dispatch must carry Practice-10 disposition"
    );
    assert!(
        CI_DAG.contains("feature:ci-bankruptcy-component-fail-closed")
            && CI_DAG.contains("fn ci_component_affected_is_fail_closed("),
        "{CI_DAG_PATH}: component fail-closed Bool classifier must carry Practice-10 disposition"
    );
    assert!(
        CI_DAG.contains("feature:ci-bankruptcy-tier0-gha-step-if")
            && CI_DAG.contains("ci_v3_self_host_fixed_point_ci_live_workflow_binding")
            && CI_DAG.contains("ci_v4_bootstrap_gate_result_skip_guard_if"),
        "{CI_DAG_PATH}: Tier-0 GHA if bridge carriers must carry Practice-10 disposition"
    );
}

#[test]
fn v4_workflow_ci_bankruptcy_tier0_legacy_top_level_jobs_deleted() {
    for legacy_job in ["v2", "v3", "v4", "self_host_ratchet"] {
        assert!(
            !ci_yml_has_deleted_legacy_top_level_job(CI_YML, legacy_job),
            "{CI_YML_PATH}: bankruptcy B0 must delete legacy top-level job `{legacy_job}`"
        );
    }
    assert!(
        !CI_YML.contains("v3 determinism (Tier-0 I3)")
            && !CI_YML.contains("v3 self-host fixed point (Tier-0 I4)"),
        "{CI_YML_PATH}: Wave 1 §11.7.3 — v3 integration permanently deleted from required CI"
    );
}

#[test]
fn v4_workflow_ci_bankruptcy_tier0_ci_integration_job_if_includes_release_distribution() {
    assert!(
        !CI_YML.contains("  ci_integration:"),
        "{CI_YML_PATH}: Wave 1 — `ci_integration` job dissolved into `ci_floor`"
    );
}

#[test]
fn v4_workflow_ci_bankruptcy_tier0_v3_bucket_includes_workspace_deps() {
    let v3_body = extract_fn_body(CI_DAG, "ci_changed_path_affects_v3");
    for path in ["Cargo.toml", "Cargo.lock"] {
        assert!(
            v3_body.contains(&format!("changed == \"{path}\"")),
            "{CI_DAG_PATH}: ci_changed_path_affects_v3 must include `{path}` (V3DeterminismCommand Upsert inputs at ci_upsert_v3_determinism_execution_inputs)"
        );
    }
    assert!(
        CI_DAG.contains("segment == \"dsl/**\""),
        "{CI_DAG_PATH}: ci_glob_segment_matches_changed_path must support dsl/** Upsert segment (P2 parity with ci_changed_path_affects_v3 dsl/ prefix)"
    );
    let i3_inputs = CI_DAG
        .split("data ci_upsert_v3_determinism_execution_inputs:")
        .nth(1)
        .and_then(|rest| {
            rest.split("fn ci_upsert_v3_determinism_execution_mk")
                .next()
        })
        .unwrap_or_else(|| {
            panic!("{CI_DAG_PATH}: missing data ci_upsert_v3_determinism_execution_inputs")
        });
    for segment in ["src/v3/**", "dsl/**", "Cargo.toml", "Cargo.lock"] {
        assert!(
            i3_inputs.contains(&format!("segment: \"{segment}\"")),
            "{CI_DAG_PATH}: ci_upsert_v3_determinism_execution_inputs must include `{segment}` (P2 parity with affected.v3)"
        );
    }
    assert_ci_dag_rust_bucket_parity("ci_changed_path_affects_v3");
}

#[test]
fn v4_workflow_ci_bankruptcy_tier0_discipline_off_required_ci_path() {
    assert!(
        CI_YML.contains("needs: [affected, ci_floor, ci_floor_emit, infra_isolation, ci_corpus_eval]"),
        "{CI_YML_PATH}: branch-protection `ci` aggregator must need live `affected` receipt, `ci_floor`, `ci_corpus_eval`, and the `infra_isolation` de-priv guard"
    );
    assert!(
        CI_YML.contains("needs.affected.result"),
        "{CI_YML_PATH}: `affected` must be checked by the fail-closed aggregator"
    );
    assert!(
        !CI_YML.contains("needs.discipline.result") && !CI_YML.contains("  discipline:"),
        "{CI_YML_PATH}: discipline job deleted per §11.7.3"
    );
    let ci_floor_block = CI_YML
        .split("  ci_floor:")
        .nth(1)
        .and_then(|rest| rest.split("\n  ci:").next())
        .unwrap_or("");
    assert!(
        !ci_floor_block.contains("needs: [affected]"),
        "{CI_YML_PATH}: `ci_floor` must not depend on `affected`"
    );
}

#[test]
fn v4_workflow_ci_bankruptcy_tier0_i4_if_matches_live_workflow_binding() {
    let module = parse_module(CI_DAG, CI_DAG_PATH);
    let i4_binding = data_body(
        &module,
        "ci_v3_self_host_fixed_point_ci_live_workflow_binding",
    );
    let i4_step_name = expr_string(record_body_field(i4_binding, "step_name"));
    assert!(
        !CI_YML.contains(&format!("- name: {i4_step_name}")),
        "{CI_YML_PATH}: Wave 1 — I4 v3 self-host step not on required path (binding remains in ci.dag)"
    );
}

#[test]
fn v4_workflow_ci_bankruptcy_tier0_corpus_eval_uses_one_canonical_ci_job_row() {
    assert!(
        CI_DAG.contains("data ci_job_testclaim_corpus_eval_execution_row: CiJob = ci_job_testclaim_corpus_eval_execution_mk()")
            && CI_DAG.contains("create: ci_job_projection_node(j: ci_job_testclaim_corpus_eval_execution_row)")
            && CI_DAG.contains("ci_job_testclaim_corpus_eval_execution_row,"),
        "{CI_DAG_PATH}: corpus eval must use one canonical CiJob row for pipeline + Upsert create (P2)"
    );
}

#[test]
fn v4_workflow_ci_bankruptcy_tier0_t15_schedule_matches_ci_yml() {
    assert!(
        !CI_YML.contains(T15_SELF_HOST_STEP_NAME),
        "{CI_YML_PATH}: Wave 1 — T-15 harness demoted from required path"
    );
    assert!(
        CI_DAG.contains(
            "V4T15SelfHostFixedPointCommand =>\n      ci_job_component_mask_row(\n        v2: false,\n        v3: false,\n        v4: true,\n        testclaim_corpus: false,\n        workflow_policy: true,\n        release_distribution: false\n      )"
        ),
        "{CI_DAG_PATH}: V4T15SelfHostFixedPointCommand mask remains modeled"
    );
    assert!(
        CI_DAG.contains("fn ci_v4_t15_scheduled(affected: CiComponentAffected, schedule: CiSchedulePolicy) -> Bool"),
        "{CI_DAG_PATH}: T-15 must model main-push schedule authority"
    );
}

#[test]
fn v4_workflow_ci_bankruptcy_tier0_ci_v4_job_if_matches_generated_workflow() {
    assert!(
        !CI_YML.contains("  ci_v4:"),
        "{CI_YML_PATH}: Wave 1 — `ci_v4` job dissolved into `ci_floor`"
    );
}

#[test]
fn v4_workflow_ci_bankruptcy_tier0_upsert_slice_registers_tier0_jobs() {
    assert!(
        CI_DAG.contains("data ci_upsert_steps_bankruptcy_tier0_slice_step_ids:")
            && CI_DAG.contains("ci_upsert_v2_bootstrap_smoke_execution")
            && CI_DAG.contains("ci_upsert_v3_determinism_execution")
            && CI_DAG.contains("ci_upsert_v3_self_host_fixed_point_execution")
            && CI_DAG.contains("ci_upsert_v4_t15_self_host_fixed_point_execution"),
        "{CI_DAG_PATH}: bankruptcy Tier-0 jobs must register in ci_upsert_steps + full-in-scope slice"
    );
    assert!(
        CI_DAG.contains("ci_job_v2_bootstrap_smoke_execution_row")
            && CI_DAG.contains("ci_job_v3_determinism_execution_row")
            && CI_DAG.contains("ci_job_v3_self_host_fixed_point_execution_row")
            && CI_DAG.contains("ci_job_v4_t15_self_host_fixed_point_execution_row"),
        "{CI_DAG_PATH}: Tier-0 CiJob rows must be canonical (pipeline + Upsert create)"
    );
    let v2_i2_inputs = CI_DAG
        .split("data ci_upsert_v2_bootstrap_smoke_execution_inputs:")
        .nth(1)
        .and_then(|rest| {
            rest.split("fn ci_upsert_v2_bootstrap_smoke_execution_mk")
                .next()
        })
        .unwrap_or_else(|| {
            panic!("{CI_DAG_PATH}: missing data ci_upsert_v2_bootstrap_smoke_execution_inputs")
        });
    for segment in ["src/v2/**", "Cargo.toml", "Cargo.lock"] {
        assert!(
            v2_i2_inputs.contains(&format!("segment: \"{segment}\"")),
            "{CI_DAG_PATH}: ci_upsert_v2_bootstrap_smoke_execution_inputs must include `{segment}` (P2 parity with ci_changed_path_affects_v2 / I2 selection)"
        );
    }
    assert!(
        !v2_i2_inputs.contains("segment: \"scripts/**\""),
        "{CI_DAG_PATH}: I2 Upsert must not declare scripts/** — script paths route via v4/workflow_policy buckets, not ci_changed_path_affects_v2 (design-ci-bankruptcy-rebuild.md I2 frontier v2/compiler/**)"
    );
}

#[test]
fn v4_workflow_ci_bankruptcy_tier0_v2_build_step_includes_main_push() {
    let v2_build = workflow_step_block(CI_YML, "Build v2 compiler (v4 floor)");
    assert!(
        !v2_build.contains("needs.affected.outputs"),
        "{CI_YML_PATH}: Wave 1 — v2 build runs unconditionally in `ci_floor`"
    );
    let v2_bin_cache = workflow_step_block(CI_YML, "Cache gunbc binary (v4 floor)");
    assert!(
        !v2_bin_cache.contains("needs.affected.outputs"),
        "{CI_YML_PATH}: Wave 1 — gunbc cache runs unconditionally in `ci_floor`"
    );
}

#[test]
fn v4_workflow_ci_bankruptcy_tier0_lens_ci_mask_matches_ci_yml() {
    assert!(
        CI_DAG.contains(
            "LensCiCommand { required_lenses: _ } =>\n      ci_job_component_mask_row(\n        v2: false,\n        v3: false,\n        v4: true,\n        testclaim_corpus: false,\n        workflow_policy: true,\n        release_distribution: false\n      )"
        ),
        "{CI_DAG_PATH}: LensCiCommand mask must match ci.yml step if (v4|workflow_policy; no v3 — I1 parity)"
    );
    let module = parse_module(CI_DAG, CI_DAG_PATH);
    let live_signal = data_body(&module, "lens_ci_live_workflow_signal");
    let smoke_step_name = expr_string(record_body_field(live_signal, "smoke_step_name"));
    let semantic_step_name = expr_string(record_body_field(live_signal, "semantic_step_name"));
    assert!(
        !CI_YML.contains(&format!("- name: {smoke_step_name}")),
        "{CI_YML_PATH}: Wave 1 — `{smoke_step_name}` demoted from required path"
    );
    assert!(
        !CI_YML.contains(&format!("- name: {semantic_step_name}")),
        "{CI_YML_PATH}: Wave 1 — `{semantic_step_name}` demoted from required path"
    );
}

#[test]
fn v4_workflow_ci_wave1_safety_floor_ci_yml_shape() {
    assert!(
        CI_YML.contains("Wave 1 §11.7.1 safety floor"),
        "{CI_YML_PATH}: must declare Wave 1 §11.7.1 safety floor"
    );
    assert!(
        CI_YML.contains("  ci_floor:"),
        "{CI_YML_PATH}: must define `ci_floor` job"
    );
    assert!(
        !CI_YML.contains("  ci_integration:") && !CI_YML.contains("  ci_v4:"),
        "{CI_YML_PATH}: legacy parallel lanes dissolved"
    );
    assert!(
        CI_YML.contains("needs: [affected, ci_floor, ci_floor_emit, infra_isolation, ci_corpus_eval]"),
        "{CI_YML_PATH}: `ci` aggregator must depend on live affected receipt, `ci_floor`, `ci_corpus_eval`, and the `infra_isolation` de-priv guard"
    );
    assert!(
        CI_YML.contains("  affected:") && !CI_YML.contains("  affected:\n    if: github.event.pull_request.draft != true\n    continue-on-error: true"),
        "{CI_YML_PATH}: component `affected` receipt must be live, not continue-on-error shadow"
    );
    for forbidden in [
        "check-pr-sg0-net-shrink-discipline.sh",
        "determinism_test",
        "self_host_fixed_point",
        "v4-mvp1-e2e-gate.sh",
        "v4-phase1-nat-semiring-rung-gate.sh",
    ] {
        assert!(
            !CI_YML.contains(forbidden),
            "{CI_YML_PATH}: Wave 1 cut — must not invoke `{forbidden}` on required path"
        );
    }
}

#[test]
fn v4_workflow_ci_wave1_generated_workflow_dag_matches_ci_yml_shape() {
    assert!(
        CI_WORKFLOW_DAG.contains("id: \"ci_floor\""),
        "{CI_WORKFLOW_DAG_PATH}: regen artifact must model `ci_floor`"
    );
    assert!(
        CI_WORKFLOW_DAG.contains("id: \"ci_floor_emit\""),
        "{CI_WORKFLOW_DAG_PATH}: regen artifact must model the parallel `ci_floor_emit` job (M1 emit probe split out of `ci_floor`)"
    );
    assert!(
        CI_WORKFLOW_DAG.contains("id: \"ci\"")
            && CI_WORKFLOW_DAG
                .contains("needs: [\"affected\", \"ci_floor\", \"ci_floor_emit\", \"infra_isolation\", \"ci_corpus_eval\"]"),
        "{CI_WORKFLOW_DAG_PATH}: `ci` job must need live `affected` receipt, `ci_floor`, the parallel `ci_floor_emit` lane, and the `infra_isolation` de-priv guard"
    );
    let affected_dag = workflow_dag_job_block(CI_WORKFLOW_DAG, "affected");
    assert!(
        workflow_dag_job_level_continue_on_error_is_false(affected_dag),
        "{CI_WORKFLOW_DAG_PATH}: component `affected` receipt must be live (job-level continue_on_error: false)"
    );
    let affected_yml = ci_yml_job_block(CI_YML, "affected");
    assert!(
        ci_yml_job_level_omits_continue_on_error(affected_yml),
        "{CI_YML_PATH}: component `affected` receipt must be live (no job-level continue-on-error anywhere in job block)"
    );
    assert!(
        !CI_WORKFLOW_DAG.contains("id: \"ci_integration\"")
            && !CI_WORKFLOW_DAG.contains("id: \"ci_v4\""),
        "{CI_WORKFLOW_DAG_PATH}: legacy parallel lanes dissolved in regen artifact"
    );
}

// P5(b) receipt: `docs/planning/ci-required-surface-cut-2026-06-01.md` § P5(b) `v4_workflow_ci_wave3_*`
// (SG-0 delta 0; ROADMAP T-PB-B; live emit deferred `node://adhoc-331899f9-19a`).

#[test]
fn v4_workflow_ci_wave3_roster_dag_tokenizes_and_parses() {
    let _module = parse_module(WAVE3_ROSTER_DAG, WAVE3_ROSTER_PATH);
}

#[test]
fn v4_workflow_ci_wave3_ci_dag_extension_and_fixture_receipt() {
    let module = parse_module(CI_DAG, CI_DAG_PATH);
    assert!(
        !CI_DAG.contains("CiWave3ShadowReceipt"),
        "{CI_DAG_PATH}: Wave 3 must extend CiSelectionReceipt — no parallel receipt type"
    );
    assert!(
        import_includes_name(
            &module,
            &["v4", "compiler", "eval"],
            "test_claim_claim_hash_digest"
        ),
        "{CI_DAG_PATH}: claim_projection_hash must import IRT-4 `test_claim_claim_hash_digest` from v4.compiler.eval"
    );
    assert!(
        import_includes_name(&module, &["v4", "lens", "testgen"], "Generator")
            && import_includes_name(&module, &["v4", "lens", "testgen"], "TestgenConcept")
            && import_includes_name(&module, &["v4", "lens", "testgen"], "TestgenRunReceipt")
            && import_includes_name(&module, &["v4", "lens", "testgen"], "testgen_scheduled_generators_outcome")
            && import_includes_name(&module, &["v4", "lens", "testgen"], "testgen_scheduled_generators_roster_holds"),
        "{CI_DAG_PATH}: testgen shadow CI must import Generator, Outcome roster authority, and roster_holds from v4.lens.testgen"
    );
    for sym in [
        "type CiActiveFloorSkipEvidence",
        "type CiSelectionMode",
        "type CiSelectionReceiptProvenance",
        "type CiClaimSelectionReason",
        "type CiTestClaimSelection",
        "type TestgenSlotSelection",
        "generator: Generator<TestgenConcept>",
        "Active { skip_evidence: CiActiveFloorSkipEvidence }",
        "fn ci_floor_held",
        "fn ci_wave3_shadow_testclaims_selected",
        "claim_projection_hash: test_claim_claim_hash_digest(c: claim)",
        "data ci_wave3_shadow_fixture_fail_closed_receipt",
        "data ci_wave3_shadow_fixture_receipt_ok",
        "type SelectedTestgenReceipt",
        "testgen_scheduled_generators_outcome",
        "fn ci_wave3_shadow_testgen_selection_rows_outcome",
        "fn ci_testgen_selected_receipt_outcome",
        "fn ci_testgen_selected_receipt_holds",
        "fn ci_wave3_shadow_testgen_run_receipt_holds",
        "data ci_wave3_shadow_testgen_run_receipt",
        "data ci_wave3_shadow_selected_testgen_receipt",
        "data witness_ci_wave3_shadow_testgen_receipts_ok",
        "fn ci_wave3_shadow_fixture_testgen_not_fail_closed_holds",
    ] {
        assert!(
            CI_DAG.contains(sym),
            "{CI_DAG_PATH}: Wave 3 Phase 1 must declare `{sym}`"
        );
    }
    let shadow_marker = "fn ci_selection_receipt_shadow(";
    let shadow_start = CI_DAG
        .find(shadow_marker)
        .unwrap_or_else(|| panic!("{CI_DAG_PATH}: missing `{shadow_marker}`"));
    let shadow_rest = &CI_DAG[shadow_start + shadow_marker.len()..];
    let shadow_end = shadow_rest
        .find("\nfn ")
        .map(|idx| shadow_start + shadow_marker.len() + idx)
        .unwrap_or(CI_DAG.len());
    let shadow_body = &CI_DAG[shadow_start..shadow_end];
    assert!(
        !shadow_body.contains("provenance: CiSelectionReceiptProvenance")
            && shadow_body.contains("provenance: FixtureReceipt")
            && shadow_body.contains("testclaim_decisions: Empty"),
        "{CI_DAG_PATH}: step-only ci_selection_receipt_shadow must hardcode FixtureReceipt (no caller provenance; empty claim/testgen until live entry)"
    );
    let heartbeat_body = extract_fn_body(CI_DAG, "ci_selection_receipt_shadow_heartbeat");
    assert!(
        heartbeat_body.contains("provenance: FixtureReceipt"),
        "{CI_DAG_PATH}: shadow heartbeat must tag FixtureReceipt (synthetic path, not live PR git_diff)"
    );
    assert!(
        !heartbeat_body.contains("provenance: LivePrGitDiff"),
        "{CI_DAG_PATH}: shadow heartbeat must not mis-label synthetic receipt as LivePrGitDiff"
    );
    let fixture_binding = CI_DAG
        .split("data ci_wave3_shadow_fixture_fail_closed_receipt:")
        .nth(1)
        .and_then(|rest| rest.split("\ndata ").next())
        .unwrap_or("");
    let fixture_mk_body = extract_fn_body(CI_DAG, "ci_wave3_shadow_fixture_fail_closed_receipt_mk");
    assert!(
        fixture_mk_body.contains("provenance: FixtureReceipt"),
        "{CI_DAG_PATH}: Wave 3 fixture receipt must construct with FixtureReceipt provenance"
    );
    assert!(
        !fixture_mk_body.contains("provenance: LivePrGitDiff"),
        "{CI_DAG_PATH}: Wave 3 fixture receipt must not use LivePrGitDiff"
    );
    assert!(
        fixture_binding.contains("ci_wave3_shadow_testgen_selection_rows_outcome(")
            && fixture_binding.contains("Outcome<CiSelectionReceipt>")
            && !fixture_mk_body.contains("testgen_slots: Empty"),
        "{CI_DAG_PATH}: F.2-P2 fixture must bind testgen_slots via selection_rows Outcome (not Empty truncation)"
    );
    assert!(
        CI_DAG.contains("data ci_wave3_shadow_testgen_run_receipt: Outcome<TestgenRunReceipt>")
            && CI_DAG.contains("data ci_wave3_shadow_selected_testgen_receipt: Outcome<SelectedTestgenReceipt>"),
        "{CI_DAG_PATH}: shadow testgen receipt data must stay Outcome-typed (no Rejected→Empty fixture collapse)"
    );
    let testgen_reason_body = extract_fn_body(CI_DAG, "ci_wave3_shadow_testgen_selection_reason");
    assert!(
        testgen_reason_body.contains("ci_affected_set_is_fail_closed")
            && testgen_reason_body.contains("TestgenSlotMapped")
            && !testgen_reason_body.contains("FailClosedSuperset"),
        "{CI_DAG_PATH}: testgen shadow selection must be profile-gated NOT fail-closed (no FailClosedSuperset on testgen rows)"
    );
    assert!(
        !CI_DAG.contains("floor_skip:"),
        "{CI_DAG_PATH}: arbiter ruling — do not persist floor_skip; derive via ci_floor_held"
    );
    assert!(
        CI_DAG.contains("reason: CiClaimSelectionReason"),
        "{CI_DAG_PATH}: claim rows must use closed CiClaimSelectionReason coproduct"
    );
    assert!(
        CI_DAG.contains("AffectedIntersectionNonempty")
            && CI_DAG.contains("AffectedIntersectionEmpty")
            && !CI_DAG.contains("| AffectedFrontierNonempty"),
        "{CI_DAG_PATH}: claim reasons must use per-claim ∩ frontier semantics (not global-only AffectedFrontierNonempty)"
    );
    let reason_body = extract_fn_body(CI_DAG, "ci_wave3_shadow_testclaim_selection_reason");
    assert!(
        reason_body.contains("test_claim_in_rerun_frontier")
            && reason_body.contains("AffectedIntersectionEmpty"),
        "{CI_DAG_PATH}: unselected rows must distinguish global frontier empty vs claim outside nonempty frontier"
    );
    assert!(
        !CI_DAG.contains("fn ci_test_claim_shadow_projection_node"),
        "{CI_DAG_PATH}: claim_projection_hash must use IRT-4 test_claim_claim_hash_digest, not input-only projection"
    );
    assert!(
        CI_DAG.contains("🟡 coproduct dissolution — feature:wave3-shadow-selection-receipt"),
        "{CI_DAG_PATH}: Wave 3 receipt carriers require dissolution disposition marks"
    );
    assert!(
        !CI_DAG.contains("generator_slot: Symbol") && !CI_DAG.contains("concept_category: Symbol"),
        "{CI_DAG_PATH}: TestgenSlotSelection must not flatten `Generator<TestgenConcept>` into Symbol labels"
    );
    for field in [
        "generator: Generator<TestgenConcept>",
        "emits_claim_anchor: ClaimAnchorKey",
        "selected: Bool",
    ] {
        assert!(
            CI_DAG.contains(field),
            "{CI_DAG_PATH}: TestgenSlotSelection must carry `{field}` per worksheet §2.1 + v4.lens.testgen authority"
        );
    }
    assert!(
        CI_DAG.contains("provenance: FixtureReceipt"),
        "{CI_DAG_PATH}: W1.5 + Wave 3 fixture receipts must tag FixtureReceipt (not LivePrGitDiff)"
    );
    assert!(
        !CI_DAG.contains("provenance: LivePrGitDiff"),
        "{CI_DAG_PATH}: Phase 1 — `LivePrGitDiff` is coproduct-only until `ci_selection_receipt_shadow_from_git_diff` lands; forbidden at construction sites"
    );
    assert!(
        CI_DAG.contains("ci_wave3_shadow_claim_roster()"),
        "{CI_DAG_PATH}: selection must consume wave3_shadow_roster authority"
    );
    let roster_module = parse_module(WAVE3_ROSTER_DAG, WAVE3_ROSTER_PATH);
    for name in [
        "ci_wave3_shadow_manual_claim_roster",
        "ci_wave3_shadow_generated_claim_roster",
        "ci_wave3_shadow_claim_roster",
    ] {
        assert!(
            surface_declares_fn(&roster_module, name),
            "{WAVE3_ROSTER_PATH}: must declare `{name}`"
        );
    }
}

#[test]
fn v4_workflow_ci_wave3_host_emit_wired_class_c_in_ci_yml() {
    // Phase 2: the host shadow emit step IS now wired into ci.yml (component_affected_comparison
    // is live-populated), as a non-blocking Class C step. The modeled live entry
    // `ci_selection_receipt_shadow_from_git_diff` stays deferred until the bootstrap eval
    // (`node://adhoc-331899f9-19a`) — the receipt's claim/testgen partitions remain queued.
    assert!(
        CI_YML.contains("emit-ci-wave3-shadow-receipt"),
        "{CI_YML_PATH}: Phase 2 — host shadow emit step must be wired into ci.yml"
    );
    let emit_block = CI_YML
        .split("- name: Emit Wave 3 shadow selection receipt")
        .nth(1)
        .and_then(|rest| rest.split("\n    - name:").next())
        .unwrap_or("");
    assert!(
        emit_block.contains("continue-on-error: true"),
        "{CI_YML_PATH}: Wave 3 host shadow emit must be Class C (continue-on-error: true)"
    );
    assert!(
        !CI_YML.contains("ci_selection_receipt_shadow_from_git_diff"),
        "{CI_YML_PATH}: modeled live entry (ci_selection_receipt_shadow_from_git_diff) not wired until eval harness lands (adhoc-331899f9-19a)"
    );
}

#[test]
fn v4_workflow_ci_wave3_node_selection_still_shadow_while_component_receipt_live() {
    assert!(
        CI_YML.contains(
            "needs: [affected, ci_floor, ci_floor_emit, infra_isolation, ci_corpus_eval]"
        ) && CI_YML.contains("needs.affected.result"),
        "{CI_YML_PATH}: component affected-set receipt must be live"
    );
    let ci_floor_block = CI_YML
        .split("  ci_floor:")
        .nth(1)
        .and_then(|rest| rest.split("\n  ci:").next())
        .unwrap_or("");
    assert!(
        !ci_floor_block.contains("needs: [affected]"),
        "{CI_YML_PATH}: `ci_floor` must not need `affected`"
    );
    // Phase 2: the host emit step is wired (component receipt live), but the node-frontier
    // modeled live entry stays deferred — claim/testgen selection remains shadow until eval.
    assert!(
        !CI_YML.contains("ci_selection_receipt_shadow_from_git_diff"),
        "{CI_YML_PATH}: Wave 3 node-frontier modeled live entry (ci_selection_receipt_shadow_from_git_diff) remains deferred until adhoc-331899f9-19a"
    );
    assert!(
        CI_YML.contains("emit-ci-wave3-shadow-receipt"),
        "{CI_YML_PATH}: Phase 2 host emit (Class C) is wired; component_affected_comparison live, node-frontier claim/testgen still queued"
    );
}

#[test]
fn v4_workflow_ci_wave3_fixture_receipt_documents_live_ci_deferral() {
    assert!(
        CI_DAG.contains("data ci_wave3_shadow_fixture_fail_closed_receipt")
            && CI_DAG.contains("FixtureReceipt")
            && CI_DAG.contains("adhoc-331899f9-19a"),
        "{CI_DAG_PATH}: must model fixture receipt and live-CI deferral"
    );
}

#[test]
fn v4_workflow_ci_t38_dissolution_step_modeled_and_wired() {
    let module = parse_module(CI_DAG, CI_DAG_PATH);
    let corpus_runner_module =
        parse_module(TESTCLAIM_CORPUS_RUNNER_DAG, TESTCLAIM_CORPUS_RUNNER_PATH);
    let manual_roster_module = parse_module(MANUAL_CORPUS_ROSTER_DAG, MANUAL_CORPUS_ROSTER_PATH);
    assert!(
        import_includes_name(
            &corpus_runner_module,
            &["v4", "test", "claim", "manual", "manual_corpus_roster"],
            "manual_corpus_node_subject_rows"
        ),
        "{TESTCLAIM_CORPUS_RUNNER_PATH}: runner must import the manual subject-roster authority"
    );
    assert!(
        surface_declares_fn(&corpus_runner_module, "run_node_runtime_value_subjects")
            && surface_declares_fn(
                &corpus_runner_module,
                "manual_testclaim_subject_roster_family_receipt"
            ),
        "{TESTCLAIM_CORPUS_RUNNER_PATH}: runner must declare the subject-to-run helper and family receipt"
    );
    assert!(
        matches!(
            data_body(&manual_roster_module, "manual_corpus_node_subject_rows"),
            SurfaceExpr::List { .. }
        ),
        "{MANUAL_CORPUS_ROSTER_PATH}: manual_corpus_node_subject_rows must remain a concrete subject roster"
    );
    assert!(
        CI_DAG.contains("| TestClaimCorpusEvalCommand"),
        "{CI_DAG_PATH}: T-38 dissolution step must declare TestClaimCorpusEvalCommand arm in CiCommand"
    );
    assert!(
        CI_DAG.contains("feature:t38-testclaim-corpus-eval"),
        "{CI_DAG_PATH}: TestClaimCorpusEvalCommand must carry t38-testclaim-corpus-eval dissolution tag"
    );
    assert!(
        CI_DAG.contains("ci_select_from_affected_set narrows roster to content_hash frontier"),
        "{CI_DAG_PATH}: TestClaimCorpusEvalCommand dissolution comment must name ci_select_from_affected_set \
         as the IRT-1 narrowing authority (checks the new dissolution comment, not the pre-existing helper)"
    );
    assert!(
        CI_DAG.contains("manual_corpus_eval_expected_pins` -> `witness_corpus_eval_tracked_expectation_closed")
            && CI_DAG.contains("manual_testclaim_subject_roster_family_receipt"),
        "{CI_DAG_PATH}: TestClaimCorpusEvalCommand dissolution comment must bind tracked-expectation pins and family receipt"
    );
    assert!(
        CI_DAG.contains("fn_name == ci_testclaim_corpus_selection_fn"),
        "{CI_DAG_PATH}: ci_command_authority_ok must enforce selection_fn == ci_testclaim_corpus_selection_fn (not unconditional true)"
    );
    for needle in [
        "type TestClaimCorpusDeclarationAuthority",
        "type TestClaimCorpusVerdictSurfaceAuthority",
        "module_path: ci_testclaim_corpus_module_manual_roster_path",
        "module_path: ci_testclaim_corpus_module_runner_path",
        "ci_testclaim_corpus_module_manual_roster_path: Symbol = v4_test_claim_manual_manual_corpus_roster",
        "ci_testclaim_corpus_module_runner_path: Symbol = v4_test_claim_workflow_testclaim_corpus_runner",
        "ci_testclaim_corpus_module_eval_path: Symbol = v4_test_claim_workflow_manual_corpus_eval",
        "declaration_name: ci_testclaim_corpus_decl_manual_corpus_node_subject_rows_name",
        "declaration_name: ci_testclaim_corpus_decl_run_manual_testclaim_corpus_eval_name",
        "declaration_name: ci_testclaim_corpus_decl_corpus_report_tally_name",
        "declaration_name: ci_testclaim_corpus_decl_witness_tracked_expectation_closed_name",
        "module_path: ci_testclaim_corpus_module_eval_expected_path",
        "ci_testclaim_corpus_decl_manual_corpus_eval_expected_pins_name: Symbol = manual_corpus_eval_expected_pins",
        "ci_testclaim_corpus_decl_witness_tracked_expectation_closed_name: Symbol = witness_corpus_eval_tracked_expectation_closed",
        "ci_testclaim_corpus_decl_manual_corpus_node_subject_rows_name: Symbol = manual_corpus_node_subject_rows",
        "ci_testclaim_corpus_decl_run_manual_testclaim_corpus_eval_name: Symbol = run_manual_testclaim_corpus_eval",
        "ci_testclaim_corpus_decl_corpus_report_tally_name: Symbol = corpus_report_tally",
        "scripts/v4-testclaim-corpus-eval.sh",
        "ci_upsert_file_set_input(segment: \"scripts/v4-testclaim-corpus-eval.sh\")",
        "ci_testclaim_corpus_module_eval_path: Symbol = v4_test_claim_workflow_manual_corpus_eval",
        "ci_testclaim_corpus_module_eval_expected_path: Symbol = v4_test_claim_workflow_manual_corpus_eval_expected",
        "ci_testclaim_corpus_decl_manual_testclaim_subject_roster_family_receipt_name: Symbol = manual_testclaim_subject_roster_family_receipt",
        "fn ci_testclaim_corpus_eval_command() -> CiCommand",
        "verdict_surface: ci_testclaim_corpus_verdict_surface_authority()",
        "surface == ci_testclaim_corpus_verdict_surface_authority()",
        "ci_projection_command_verdict_surface_edge",
        "ci_projection_corpus_surface_run_roster_edge",
        "ci_projection_corpus_surface_eval_report_edge",
        "ci_projection_corpus_surface_verdict_tally_edge",
        "ci_projection_corpus_surface_gate_witness_edge",
        "ci_projection_corpus_surface_family_receipt_edge",
        "ci_projection_declaration_module_edge",
        "ci_projection_declaration_name_edge",
        "ci_declaration_authority_projection_node",
        "ci_atom(sym: a.module_path)",
        "ci_atom(sym: a.declaration_name)",
        "feature:t38-testclaim-corpus-roster-claim-ref-frontier",
        "claim_lens_effect_depends_on_runtime_verdict",
        "generated roster/item-registry reflection derives these TestClaimRef inputs directly from",
        "Forbidden: adding/removing manual corpus rows without the matching claim id here.",
        "ci_upsert_file_set_input(segment: \"src/v4/test/claim/workflow/manual_corpus_eval.dag\")",
        "segment == \"src/v4/test/claim/workflow/manual_corpus_eval.dag\"",
    ] {
        assert!(
            CI_DAG.contains(needle),
            "{CI_DAG_PATH}: TestClaimCorpusEvalCommand must carry modeled corpus verdict authority `{needle}`"
        );
    }
    for needle in [
        "type TestClaimSubjectRosterUnsupportedRow",
        "type TestClaimSubjectRosterFamilyReceipt",
        "subjects: List<TestClaimEvalSubject<Node>>",
        "runs: List<TestClaimRun<Node, RuntimeValue>>",
        "unsupported_rows: List<TestClaimSubjectRosterUnsupportedRow>",
        "data testclaim_subject_roster_unsupported_rows: List<TestClaimSubjectRosterUnsupportedRow>",
        "testclaim_subject_roster_family_ci_pipeline",
        "testclaim_subject_roster_family_non_runtime_value",
        "testclaim_subject_roster_unsupported_until_runner_projection_lands",
        "fn run_node_runtime_value_subjects(",
        "map(subjects, fn(subject) { run_test_claim(subject: subject) })",
        "fn manual_testclaim_subject_roster_family_receipt() -> TestClaimSubjectRosterFamilyReceipt",
        "subject_family: testclaim_subject_roster_family_node_runtime_value",
        "run_family: testclaim_subject_roster_run_test_claim",
        "subjects: manual_corpus_node_subject_rows",
        "report: CorpusEvalReport",
        "data witness_manual_testclaim_subject_roster_family_receipt: Bool",
    ] {
        assert!(
            TESTCLAIM_CORPUS_RUNNER_DAG.contains(needle),
            "{TESTCLAIM_CORPUS_RUNNER_PATH}: T-38B subject-roster family contract must carry `{needle}`"
        );
    }
    assert!(
        CI_DAG.contains("data ci_testclaim_corpus_selection_fn: Symbol = ci_select_from_affected_set"),
        "{CI_DAG_PATH}: ci_testclaim_corpus_selection_fn must be declared as ci_select_from_affected_set (IRT-1 P2 single authority)"
    );
    assert!(
        CI_DAG.contains("testclaim_corpus_eval_execution"),
        "{CI_DAG_PATH}: ci_pipeline must include testclaim_corpus_eval_execution job"
    );
    assert!(
        CI_DAG.contains("testclaim_corpus_eval_signal"),
        "{CI_DAG_PATH}: ci_pipeline must include testclaim_corpus_eval_signal gate"
    );
    assert!(
        CI_DAG.contains("command: ci_testclaim_corpus_eval_command()"),
        "{CI_DAG_PATH}: testclaim_corpus_eval_execution job must bind the canonical corpus-eval command"
    );
    assert!(
        CI_DAG.contains("payload_type: ci_command_projection_node(")
            && CI_DAG.contains("c: ci_testclaim_corpus_eval_command()"),
        "{CI_DAG_PATH}: testclaim corpus CiUpsertStep payload_type must use command projection (content_hash authority, not static tag)"
    );
    assert!(
        !CI_DAG.contains("ci_cache_cmd_testclaim_corpus_eval_tag"),
        "{CI_DAG_PATH}: static testclaim cache tag dissolved — cache via ci_upsert_step_cache_digest / content_hash projection"
    );
    assert!(
        module.items.iter().any(|item| matches!(
            item,
            SurfaceItem::TypeSum { name, .. } if name == "CiCommand"
        )),
        "{CI_DAG_PATH}: CiCommand sum type must exist"
    );
}

/// A.1.5a (RR-A §4 R7): the in-process `TestClaimRun` equivalence claim must (a) tokenize/parse,
/// (b) build both paths from the *same* fixed corpus slice — the harness path via the corpus-runner
/// machinery (`run_node_runtime_value_subjects` + `corpus_entry_from_node_runtime_value_run`) and
/// the in-process path via direct `run_test_claim` — and (c) gate the equivalence on the
/// `witness_inprocess_equivalence` Bool. The file is a tracked deferral: it declares NO
/// `data run_*: TestClaimRun = run_test_claim(...)` row (RR-A §6 forbids that authoring-time
/// co-authority); runtime execution + roster wiring is the A.1 harness lane.
/// SG-0 delta 0 (same-path expansion; `pb_rust_tests_outside_residual_zero`).
#[test]
fn v4_workflow_ci_source_authority_receipt_consumes_h72_claims() {
    let module = parse_module(CI_DAG, CI_DAG_PATH);
    assert!(
        import_includes_name(
            &module,
            &[
                "v4",
                "test",
                "claim",
                "round_trip",
                "source_authority_contract"
            ],
            "claim_source_authority_contract_compiles"
        ) && import_includes_name(
            &module,
            &[
                "v4",
                "test",
                "claim",
                "round_trip",
                "source_authority_contract"
            ],
            "claim_bmin_canonical_dag_source_parse_print_law"
        ),
        "{CI_DAG_PATH}: F.11b must import H.7.2 source-authority claim declarations directly"
    );
    for needle in [
        "| SourceAuthorityReceiptEvalCommand { claims: List<Symbol> }",
        "data ci_source_authority_receipt_claim_ids: List<Symbol> = [",
        "claim_source_authority_contract_compiles",
        "claim_bmin_canonical_dag_source_parse_print_law",
        "fn ci_source_authority_receipt_eval_command() -> CiCommand",
        "SourceAuthorityReceiptEvalCommand { claims: ci_source_authority_receipt_claim_ids }",
        "id: source_authority_receipt_eval_execution",
        "command: ci_source_authority_receipt_eval_command()",
        "id: source_authority_receipt_eval_signal",
        "job: source_authority_receipt_eval_execution",
        "claims == ci_source_authority_receipt_claim_ids",
        "ci_cache_cmd_source_authority_receipt_eval_tag",
        "ci_projection_command_claim_ids_edge",
        "ci_symbol_list_projection_node(xs: claims)",
        "ci_upsert_file_set_input(segment: \"src/v4/compiler/source_authority.dag\")",
        "ci_upsert_file_set_input(segment: \"src/v4/test/claim/round_trip/source_authority_contract.dag\")",
        "segment == \"src/v4/compiler/source_authority.dag\"",
        "path == \"src/v4/compiler/source_authority.dag\"",
        "segment == \"src/v4/test/claim/round_trip/source_authority_contract.dag\"",
        "path == \"src/v4/test/claim/round_trip/source_authority_contract.dag\"",
        "ci_upsert_upstream_job_input(job: v2_compile_src_v4)",
        "ci_upsert_source_authority_receipt_eval_claim_ref_inputs",
        "ci_upsert_test_claim_ref_input(claim_id: claim_id)",
        "step: ci_upsert_source_authority_receipt_eval_execution",
        "step: ci_upsert_source_authority_receipt_eval_signal",
        "JobStep {\n    job: source_authority_receipt_eval_execution,\n    step: source_authority_receipt_eval_execution",
        "GateStep {\n    job: source_authority_receipt_eval_execution,\n    gate: source_authority_receipt_eval_signal",
        "SourceAuthorityReceiptEvalCommand { claims: _ } =>\n      ci_job_component_mask_row(",
    ] {
        assert!(
            CI_DAG.contains(needle),
            "{CI_DAG_PATH}: F.11b source-authority CI receipt consumption must carry `{needle}`"
        );
    }
    assert!(
        !CI_DAG.contains("dag-artifact.json") && !CI_DAG.contains("--target dag"),
        "{CI_DAG_PATH}: F.11b CI receipt consumption must not use JSON IR as source authority"
    );
}

#[test]
fn v4_workflow_ci_upsert_node_projection_substrate() {
    assert!(
        ROADMAP.contains("### Nine lanes")
            && ROADMAP.contains("| **T-PB-B** | `pb_rust_tests_outside_residual_zero`")
            && ROADMAP.contains("T-PB-B / `pb_rust_tests_outside_residual_zero`"),
        "{ROADMAP_PATH}: F.11a P5 deferral must bind to checkable T-PB-B authority (Nine lanes + Public Operational Lanes)"
    );
    let module = parse_module(CI_DAG, CI_DAG_PATH);
    assert!(
        import_includes_name(&module, &["v4", "std", "patterns"], "Upsert"),
        "{CI_DAG_PATH}: F.11a must import `Upsert` from `v4.std.patterns` (P2 single-authority)"
    );
    for needle in [
        "fn ci_upsert_projection_edges<T>(",
        "fn ci_upsert_projection_node<T>(upsert: Upsert<T>) -> Node",
        "fn ci_upsert_cache_digest<T>(upsert: Upsert<T>) -> Hash",
        "fn ci_upsert_step_projection_node<T>(step: CiUpsertStep<T>) -> Node",
        "fn ci_upsert_input_ref_projection_node(input_ref: UpsertInputRef) -> Node",
        "fn ci_upsert_step_cache_digest<T>(step: CiUpsertStep<T>) -> Hash",
        "content_hash(n: ci_upsert_step_projection_node(step: step))",
        "ci_upsert_projection_edges(",
        "ci_projection_upsert_verify_edge",
        "ci_projection_upsert_create_edge",
        "ci_projection_upsert_resolve_edge",
        "ci_upsert_cache_digest_create_sensitivity_witness_holds",
        "Structural consumer of `v4.std.patterns.Upsert<T>`",
    ] {
        assert!(
            CI_DAG.contains(needle),
            "{CI_DAG_PATH}: F.11a Upsert<T> Node projection substrate must carry `{needle}`"
        );
    }
}

// PR-A substitute/extract: relocated as-is from frozen integration (zero coverage gap).
// .dag fold-witness upgrades for these checks are ctrl#1467-style tracked follow-up.

#[test]
fn v4_workflow_affected_set_ci_runner_claim_wiring() {
    let module = parse_module(CLAIM_DAG, CLAIM_PATH);
    assert_eq!(
        module_paths(&module),
        vec![vec![
            "v4",
            "test",
            "claim",
            "workflow",
            "affected_set_ci_runner"
        ]],
        "{CLAIM_PATH}: module authority path"
    );
    for name in ["ci_select_from_rerun_nodes", "ci_select_from_affected_set"] {
        assert!(
            import_includes_name(&module, &["v4", "workflow", "ci"], name),
            "{CLAIM_PATH}: must import {name} from canonical workflow/ci authority"
        );
    }
    for claim in [
        "ci_runner_narrow_selection_claim",
        "ci_runner_fail_closed_superset_claim",
        "ci_runner_shape_collision_claim",
        "ci_runner_inner_frontier_claim",
    ] {
        assert!(
            surface_declares_test_claim_data(&module, claim),
            "{CLAIM_PATH}: must declare structural receipt `{claim}`"
        );
    }
    assert!(
        surface_declares_fn(&module, "ci_runner_narrow_selection_holds")
            && surface_declares_fn(&module, "ci_runner_fail_closed_superset_holds")
            && surface_declares_fn(&module, "ci_runner_shape_collision_holds")
            && surface_declares_fn(&module, "ci_runner_inner_frontier_holds"),
        "{CLAIM_PATH}: receipt predicates must exercise ci selection entrypoints"
    );
}
#[test]
fn v4_workflow_ci_bankruptcy_tier0_release_distribution_mask_axes_modeled() {
    assert!(
        CI_DAG.contains("release_distribution && affected.release_distribution"),
        "{CI_DAG_PATH}: ci_component_mask_intersects must include release_distribution axis"
    );
    assert!(
        CI_DAG.contains("testclaim_corpus && affected.testclaim_corpus"),
        "{CI_DAG_PATH}: ci_component_mask_intersects must include testclaim_corpus axis (I8 / IRT-1)"
    );
    let fail_closed_body = extract_fn_body(CI_DAG, "ci_component_affected_is_fail_closed");
    for axis in [
        "affected.v2",
        "affected.v3",
        "affected.v4",
        "affected.testclaim_corpus",
        "affected.workflow_policy",
        "affected.release_distribution",
    ] {
        assert!(
            fail_closed_body.contains(axis),
            "{CI_DAG_PATH}: ci_component_affected_is_fail_closed must require axis `{axis}`"
        );
    }
    assert!(
        CI_DAG.contains(
            "BootstrapStageCompile { produces: _ } =>\n      ci_job_component_mask_row(\n        v2: false,\n        v3: false,\n        v4: true"
        ),
        "{CI_DAG_PATH}: BootstrapStageCompile mask must not select on v2-only (I2 is ci_integration; gunbc build is ci_v4 v4-axis)"
    );
    assert!(
        CI_DAG.contains("fn ci_job_component_mask_row(")
            && CI_DAG.contains("release_distribution_only: false"),
        "{CI_DAG_PATH}: job masks must populate release_distribution_only (P2 carrier completeness)"
    );
}

#[test]
fn v4_workflow_ci_bankruptcy_tier0_needs_closure_is_bounded_and_skips_unresolved() {
    assert!(
        CI_DAG.contains("fn ci_select_ci_jobs_needs_closure_pass("),
        "{CI_DAG_PATH}: needs closure must be bounded (P4) — not unbounded recursion on unresolved needs"
    );
    assert!(
        CI_DAG.contains(
            "ci_symbol_resolves(s: n, jobs: jobs) && ci_symbol_not_in_ids(s: n, ids: selected_ids)"
        ),
        "{CI_DAG_PATH}: needs closure must ignore unresolved need symbols"
    );
}

#[test]
fn v4_workflow_ci_a15a_inprocess_equivalence_claim_modeled_and_wired() {
    let module = parse_module(INPROCESS_EQUIVALENCE_DAG, INPROCESS_EQUIVALENCE_PATH);
    assert_eq!(
        module_paths(&module),
        vec![vec![
            "v4",
            "test",
            "claim",
            "workflow",
            "inprocess_equivalence"
        ]],
        "{INPROCESS_EQUIVALENCE_PATH}: module authority path"
    );
    // The fixed slice and both equivalence paths are sourced from a single subject authority.
    assert!(
        import_includes_name(
            &module,
            &["v4", "test", "claim", "manual", "eval_runtime_mvp"],
            "subject_eval_mvp2_test_claim_route"
        ),
        "{INPROCESS_EQUIVALENCE_PATH}: fixed slice must come from the canonical eval_mvp2 subject authority"
    );
    for name in &[
        "run_node_runtime_value_subjects",
        "corpus_entry_from_node_runtime_value_run",
        "corpus_entries_from_node_runtime_value_runs",
    ] {
        assert!(
            import_includes_name(
                &module,
                &["v4", "test", "claim", "workflow", "testclaim_corpus_runner"],
                name
            ),
            "{INPROCESS_EQUIVALENCE_PATH}: harness path must reuse corpus-runner machinery `{name}` (no parallel runner)"
        );
    }
    for fn_name in &[
        "inprocess_equivalence_harness_entries",
        "inprocess_equivalence_inprocess_entries",
        "inprocess_equivalence_holds",
    ] {
        assert!(
            surface_declares_fn(&module, fn_name),
            "{INPROCESS_EQUIVALENCE_PATH}: must declare {fn_name}"
        );
    }
    for needle in &[
        // Harness path is the corpus-runner machinery applied to the fixed slice.
        "run_node_runtime_value_subjects(subjects: inprocess_equivalence_slice)",
        // In-process path is direct run_test_claim on the same subject.
        "run_test_claim(subject: subject_eval_mvp2_test_claim_route)",
        // Equivalence witness compares the two paths' rows.
        "inprocess_equivalence_harness_entries() == inprocess_equivalence_inprocess_entries()",
        "data witness_inprocess_equivalence: Bool = inprocess_equivalence_holds()",
    ] {
        assert!(
            INPROCESS_EQUIVALENCE_DAG.contains(needle),
            "{INPROCESS_EQUIVALENCE_PATH}: A.1.5a equivalence contract must carry `{needle}`"
        );
    }
    // Tracked deferral (RR-A §6): the equivalence spec must NOT declare an authoring-time
    // `data run_*: TestClaimRun = run_test_claim(...)` co-authority row. Runtime execution and
    // roster/claim-id wiring belong to the A.1 harness lane, named in the file's DEFERRAL note.
    // Structural check (substring would false-match the prose in the DEFERRAL note).
    let declares_test_claim_run_data = module.items.iter().any(|item| {
        use v3_compiler::parse_surface::SurfaceType;
        matches!(
            item,
            SurfaceItem::Data {
                ty: SurfaceType::Named { name, .. },
                ..
            } if name == "TestClaimRun"
        )
    });
    assert!(
        !declares_test_claim_run_data,
        "{INPROCESS_EQUIVALENCE_PATH}: must not declare a `data : TestClaimRun` harness-row receipt (RR-A §6 authoring-time co-authority)"
    );
    assert!(
        !surface_declares_fn(&module, "inprocess_equivalence_pass_node"),
        "{INPROCESS_EQUIVALENCE_PATH}: equivalence spec must not carry an EqualsClaim receipt scaffold"
    );
    assert!(
        INPROCESS_EQUIVALENCE_DAG.contains("DEFERRAL")
            && INPROCESS_EQUIVALENCE_DAG.contains("A.1 harness lane"),
        "{INPROCESS_EQUIVALENCE_PATH}: must carry the explicit harness-execution deferral note"
    );
}

/// F.12a recursive-flex THIN (inspection receipt): the receipt must (a) tokenize/parse,
/// (b) prove the FAIL-CLOSED load-bearing fact — every A.1.5a slice subject is in the live
/// corpus subject roster `manual_corpus_node_subject_rows` that ci.dag's TestClaimCorpusEval
/// job evaluates (well-typed `TestClaimEvalSubject` membership, single authority via the same
/// `subject_eval_mvp2_test_claim_route` binding) — and (c) conjoin it with the A.1.5a
/// `inprocess_equivalence_holds` law, a ci.dag frontier↔roster CARDINALITY consistency check
/// (NOT per-id membership; the `TestClaim`->`Symbol` projection is intentionally absent in
/// F.12a, so the receipt advertises only what it proves — P2/P3), and the bootstrap
/// fixed-point hash-pin witness, into a single `witness_recursive_flex_inspection` Bool.
/// It is import/inspect only: it must NOT declare a `data : TestClaimRun` co-authority row
/// (RR-A §6) and is the THIN slice — the full self-host loop is F.12b, named as deferred.
/// SG-0 delta 0 (same-path expansion of this census-listed v4 CI smoke harness — see #4313
/// A.1.5a, same shape; `pb_rust_tests_outside_residual_zero`; ROADMAP T-PB-B).

#[test]
fn v4_workflow_ci_f12a_recursive_flex_inspection_receipt_modeled_and_wired() {
    let module = parse_module(
        RECURSIVE_FLEX_INSPECTION_DAG,
        RECURSIVE_FLEX_INSPECTION_PATH,
    );
    assert_eq!(
        module_paths(&module),
        vec![vec![
            "v4",
            "test",
            "claim",
            "workflow",
            "recursive_flex_inspection"
        ]],
        "{RECURSIVE_FLEX_INSPECTION_PATH}: module authority path"
    );
    // (1) A.1.5a equivalence law — consumed from the #4313 module, no re-derivation.
    assert!(
        import_includes_name(
            &module,
            &["v4", "test", "claim", "workflow", "inprocess_equivalence"],
            "inprocess_equivalence_holds"
        ),
        "{RECURSIVE_FLEX_INSPECTION_PATH}: must consume A.1.5a `inprocess_equivalence_holds` (no re-derivation)"
    );
    // Single authority (P2 / facts-flow-forward): the rostering check consumes the SAME
    // `inprocess_equivalence_slice` the A.1.5a law proves over — not a re-minted id — so the
    // equivalence law and the rostering check cannot drift onto different subjects.
    assert!(
        import_includes_name(
            &module,
            &["v4", "test", "claim", "workflow", "inprocess_equivalence"],
            "inprocess_equivalence_slice"
        ),
        "{RECURSIVE_FLEX_INSPECTION_PATH}: rostering must key off the A.1.5a `inprocess_equivalence_slice` authority, not a re-minted id"
    );
    // (2a) Well-typed subject membership over the live corpus subject roster ci.dag evaluates
    // — `TestClaimEvalSubject<Node>` both sides, not a `TestClaim`-vs-`Symbol` cross-type compare.
    assert!(
        import_includes_name(
            &module,
            &["v4", "test", "claim", "manual", "manual_corpus_roster"],
            "manual_corpus_node_subject_rows"
        ),
        "{RECURSIVE_FLEX_INSPECTION_PATH}: must roster the slice against `manual_corpus_node_subject_rows` (well-typed subject membership)"
    );
    // (2b) Inspection over ci.dag — the live corpus claim-id frontier authority (cardinality cover).
    assert!(
        import_includes_name(
            &module,
            &["v4", "workflow", "ci"],
            "ci_testclaim_corpus_eval_claim_ids"
        ),
        "{RECURSIVE_FLEX_INSPECTION_PATH}: must inspect ci.dag `ci_testclaim_corpus_eval_claim_ids` frontier"
    );
    // (3) Inspection over bootstrap.dag — the fixed-point hash-pin projection witness.
    assert!(
        import_includes_name(
            &module,
            &["v4", "workflow", "bootstrap"],
            "bootstrap_plan_accepted_hash_pins_projectable_witness"
        ),
        "{RECURSIVE_FLEX_INSPECTION_PATH}: must inspect bootstrap.dag fixed-point hash-pin witness"
    );
    for fn_name in &[
        "recursive_flex_slice_rostered",
        "recursive_flex_ci_frontier_cardinality_matches_roster",
        "recursive_flex_inspection_holds",
    ] {
        assert!(
            surface_declares_fn(&module, fn_name),
            "{RECURSIVE_FLEX_INSPECTION_PATH}: must declare {fn_name}"
        );
    }
    for needle in &[
        // Rostering keys off the A.1.5a slice authority via for_all, probing the subject itself
        // (well-typed `TestClaimEvalSubject` membership — no `TestClaim`-vs-`Symbol` compare).
        "xs: inprocess_equivalence_slice",
        "item: subject",
        "xs: manual_corpus_node_subject_rows",
        // ci.dag touch: frontier↔roster CARDINALITY consistency only (well-typed Int equality);
        // NOT per-id membership — that fail-closed fact is the roster check above.
        "length(xs: ci_testclaim_corpus_eval_claim_ids) == length(xs: manual_corpus_node_subject_rows)",
        // The receipt conjoins the facts.
        "inprocess_equivalence_holds()",
        "&& recursive_flex_slice_rostered()",
        "&& recursive_flex_ci_frontier_cardinality_matches_roster()",
        "&& bootstrap_plan_accepted_hash_pins_projectable_witness",
        "data witness_recursive_flex_inspection: Bool = recursive_flex_inspection_holds()",
    ] {
        assert!(
            RECURSIVE_FLEX_INSPECTION_DAG.contains(needle),
            "{RECURSIVE_FLEX_INSPECTION_PATH}: F.12a inspection receipt must carry `{needle}`"
        );
    }
    // Import/inspect only — the receipt must NOT declare a `data : TestClaimRun` co-authority
    // row (RR-A §6 authoring-time co-authority forbidden); runtime execution is F.12b.
    let declares_test_claim_run_data = module.items.iter().any(|item| {
        use v3_compiler::parse_surface::SurfaceType;
        matches!(
            item,
            SurfaceItem::Data {
                ty: SurfaceType::Named { name, .. },
                ..
            } if name == "TestClaimRun"
        )
    });
    assert!(
        !declares_test_claim_run_data,
        "{RECURSIVE_FLEX_INSPECTION_PATH}: must not declare a `data : TestClaimRun` row (RR-A §6 co-authority); runtime loop is F.12b"
    );
    assert!(
        RECURSIVE_FLEX_INSPECTION_DAG.contains("F.12b"),
        "{RECURSIVE_FLEX_INSPECTION_PATH}: must name the deferred full self-host loop (F.12b) as out of scope"
    );
}

#[test]
fn v4_workflow_ci_selection_receipt_persistence_lookup_modeled() {
    let module = parse_module(CI_DAG, CI_DAG_PATH);
    for name in [
        "ci_selection_receipt_persist",
        "ci_selection_receipt_lookup",
        "ci_selection_receipt_storage_key",
        "ci_selection_receipt_shadow_fixture_persistence_lookup_holds",
    ] {
        assert!(
            surface_declares_fn(&module, name),
            "{CI_DAG_PATH}: F.11c must declare `{name}`"
        );
    }
    for (path, name) in [
        (&["v4", "std", "change"][..], "ArtifactChanged"),
        (&["v4", "std", "change"][..], "DependencyDependent"),
        (&["v4", "std", "change"][..], "DependencySource"),
        (&["v4", "std", "change"][..], "NodeAdded"),
        (&["v4", "std", "change"][..], "NodeChanged"),
        (&["v4", "std", "change"][..], "NodeRemoved"),
        (&["v4", "std", "change"][..], "ProjectionChanged"),
        (&["v4", "std", "dependency"][..], "BarrierBefore"),
        (&["v4", "std", "dependency"][..], "BindsTo"),
        (&["v4", "std", "dependency"][..], "BootstrapDependsOn"),
        (&["v4", "std", "dependency"][..], "Contains"),
        (&["v4", "std", "dependency"][..], "DataDependsOn"),
        (&["v4", "std", "dependency"][..], "EffectDependsOn"),
        (&["v4", "std", "dependency"][..], "GeneratedFrom"),
        (&["v4", "std", "dependency"][..], "ModelDependsOn"),
        (&["v4", "std", "dependency"][..], "ModuleDependsOn"),
        (&["v4", "std", "dependency"][..], "PlacementDependsOn"),
        (&["v4", "std", "dependency"][..], "ProjectionDependsOn"),
        (&["v4", "std", "dependency"][..], "PromotedBy"),
        (&["v4", "std", "dependency"][..], "ResourceDependsOn"),
        (&["v4", "std", "dependency"][..], "TypeDependsOn"),
        (&["v4", "std", "dependency"][..], "VerifiedBy"),
    ] {
        assert!(
            import_includes_name(&module, path, name),
            "{CI_DAG_PATH}: F.11c digest matches `{name}`, so the constructor must be explicitly imported"
        );
    }
    for needle in [
        "type CiSelectionReceiptStoreRow",
        "receipt: CiSelectionReceipt",
        "type CiSelectionReceiptLookup",
        "= CiSelectionReceiptFound { receipt: CiSelectionReceipt }",
        "| CiSelectionReceiptMissing { key: Hash }",
        "data ci_selection_receipt_shadow_fixture_storage_key: Hash",
        "ci_selection_receipt_storage_key(receipt: ci_selection_receipt_shadow_fixture_receipt)",
        "data ci_selection_receipt_shadow_fixture_store: List<CiSelectionReceiptStoreRow>",
        "ci_selection_receipt_persist(",
        "ci_change_set_digest(changes: receipt.pr)",
        "ci_affected_set_digest(affected: receipt.affected)",
        "ci_component_affected_digest(component: receipt.component_affected_comparison)",
        "ci_digest_empty_change_set_tag",
        "ci_digest_empty_affected_dependency_list_tag",
        "ci_digest_empty_diagnostics_list_tag",
        "ci_digest_empty_affected_exclusion_list_tag",
        "ci_digest_empty_dependency_kind_list_tag",
        "ci_digest_empty_step_selection_list_tag",
        "ci_digest_empty_testclaim_selection_list_tag",
        "ci_digest_empty_testgen_slot_selection_list_tag",
        "ci_digest_empty_node_list_tag",
        "ci_extent_whole_file_tag",
        "ci_extent_byte_range_tag",
        "ci_locus_textual_tag",
        "ci_locus_node_tag",
        "ci_locus_port_tag",
        "byte_offset_cache_key_projection_node(i: start)",
        "byte_offset_cache_key_projection_node(i: end)",
        "bag_hash_digest(",
        "xs: map(xs, diagnostic => ci_diagnostic_digest(diagnostic: diagnostic))",
        "canonical_hash_of_connective(c: connective)",
        "canonical_hash_of_behavior(b: behavior)",
        "ci_no_correction_user_input_boundary_tag",
        "ci_no_correction_ambiguous_intent_tag",
        "ci_no_correction_external_contract_unknown_tag",
        "ci_upsert_input_ref_list_projection_node(refs: row.inputs_consulted)",
        "ci_affected_dependency_list_digest(xs: row.affected_intersection)",
        "ci_symbol_digest(sym: row.reason)",
        "ci_claim_anchor_digest(anchor: row.anchor)",
        "ci_string_digest(s: row.label)",
        "ci_testclaim_variant_digest(variant: row.coproduct_variant)",
        "ci_bool_digest(value: row.selected)",
        "ci_test_classification_digest(classification: row.generator.classification)",
        "ci_claim_anchor_digest(anchor: row.generator.anchor)",
        "ci_testgen_concept_digest(concept: row.generator.slot)",
        "ci_symbol_digest(sym: row.generator.provenance.generator_id)",
        "ci_generated_artifact_digest(artifact: row.generator.provenance.artifact)",
        "ci_symbol_digest(sym: row.generator.profile_metadata.profile_ref)",
        "ci_claim_anchor_digest(anchor: row.emits_claim_anchor)",
        "receipt: ci_selection_receipt_shadow_fixture_receipt",
        "data ci_selection_receipt_shadow_fixture_lookup: CiSelectionReceiptLookup",
        "ci_selection_receipt_lookup(",
        "init: CiSelectionReceiptMissing { key: key }",
        "if ci_selection_receipt_storage_key(receipt: row.receipt) == missing_key",
        "CiSelectionReceiptFound { receipt: row.receipt }",
        "ci_selection_receipt_storage_key(receipt: receipt) == ci_selection_receipt_shadow_fixture_storage_key",
        "CiSelectionReceiptMissing { key: _ } => false",
        "data ci_selection_receipt_shadow_fixture_persistence_lookup_ok: Bool",
        "feature:f11c-ci-selection-receipt-persistence",
        "caller-chosen symbols as lookup authority",
        "Forbidden: treating transient fixture construction as persisted receipt evidence",
    ] {
        assert!(
            CI_DAG.contains(needle),
            "{CI_DAG_PATH}: F.11c receipt persistence + lookup must carry `{needle}`"
        );
    }
    assert!(
        !CI_DAG.contains("fallback: ci_wave3_shadow_fixture_fail_closed_receipt")
            && !CI_DAG.contains("data ci_selection_receipt_shadow_fixture_storage_key: Symbol"),
        "{CI_DAG_PATH}: F.11c lookup misses must not expose fallback receipts or caller-authored Symbol keys"
    );
    assert!(
        !CI_DAG.contains("type CiSelectionReceiptStoreRow {\n  key: Hash")
            && !CI_DAG.contains("key: ci_selection_receipt_storage_key(receipt: receipt)"),
        "{CI_DAG_PATH}: F.11c store rows must not carry a driftable key beside the receipt"
    );
    assert!(
        !CI_DAG.contains("if row.key == missing_key"),
        "{CI_DAG_PATH}: F.11c lookup must recompute receipt keys instead of trusting store row keys"
    );
    for overloaded_seed in [
        "empty: ci_symbol_digest(sym: ci_change_kind_node_changed_tag)",
        "empty: ci_symbol_digest(sym: ci_affected_set_produced_tag)",
        "empty: ci_symbol_digest(sym: ci_affected_set_diagnostics_tag)",
        "empty: ci_symbol_digest(sym: ci_dependency_kind_contains_tag)",
        "empty: ci_symbol_digest(sym: ci_selection_decision_run_tag)",
        "empty: ci_symbol_digest(sym: ci_projection_command_claim_ids_edge)",
        "empty: ci_symbol_digest(sym: ci_selection_testgen_slot_tag)",
        "UserInputBoundary => ci_symbol_digest(sym: ci_receipt_inputs_fail_closed_reason)",
        "AmbiguousIntent => ci_symbol_digest(sym: ci_workflow_receipt_unexpected_verdict)",
        "ExternalContractUnknown => ci_symbol_digest(sym: ci_selection_shadow_reason)",
        "WholeFile => ci_symbol_digest(sym: ci_projection_char_offset_authority_edge)",
        "content_hash(n: ci_char_projection_node(c: start))",
        "content_hash(n: ci_char_projection_node(c: end))",
        "NodeLocus { anchor } => content_hash(n: anchor.at)",
        "PortLocus { anchor } => ci_symbol_digest(sym: anchor.at)",
        "fn ci_diagnostics_list_digest(xs: List<Diagnostic>) -> Hash {\n  fold_list(",
    ] {
        assert!(
            !CI_DAG.contains(overloaded_seed),
            "{CI_DAG_PATH}: F.11c receipt digest must not overload `{overloaded_seed}`"
        );
    }
    assert!(
        !CI_DAG.contains("let decisions_match = length(xs: receipt.decisions)")
            && !CI_DAG.contains("let testclaims_match = length(xs: receipt.testclaim_decisions)")
            && !CI_DAG.contains("let testgen_match = length(xs: receipt.testgen_slots)"),
        "{CI_DAG_PATH}: F.11c persistence witness must compare canonical receipt keys, not partition lengths"
    );
}
