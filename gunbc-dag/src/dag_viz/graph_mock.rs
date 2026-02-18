//! Mock specifications for dag-viz modes.

use crate::dag_viz::{build_dag_viz_graph, DagVizMode};
use gunbc_test::{MockSpec, OutputMatcher};

fn dag_viz_mock_spec(mode: &DagVizMode) -> MockSpec {
    let dag = build_dag_viz_graph(mode.clone()).expect("dag_viz graph should build");
    crate::mock_defaults::auto_mock_spec(&dag, "dag_viz")
        .live_expected_output(
            "gist_upload/parse_gist_response",
            "url",
            OutputMatcher::non_empty(),
        )
        .live_expected_output(
            "gist_upload/cloud_credential/gcp_wif_secret/parse_set_iam",
            "ok",
            OutputMatcher::IsBool,
        )
}

#[gunbc_testgen_registry_macros::testgen_target(
    name = "dag-viz-snapshot",
    output = "gunbc-dag/src/dag_viz/generated_tests_snapshot.rs",
    module = "dag_viz_snapshot_generated_tests",
    builder = "crate::dag_viz::build_dag_viz_graph(crate::dag_viz::DagVizMode::Snapshot).unwrap()",
    signature = "crate::dag_viz::dag_viz_signature(&crate::dag_viz::DagVizMode::Snapshot)",
    tool = "dag-viz"
)]
pub fn dag_viz_snapshot_mock_spec() -> MockSpec {
    dag_viz_mock_spec(&DagVizMode::Snapshot)
}

#[gunbc_testgen_registry_macros::testgen_target(
    name = "dag-viz-diff",
    output = "gunbc-dag/src/dag_viz/generated_tests_diff.rs",
    module = "dag_viz_diff_generated_tests",
    builder = "crate::dag_viz::build_dag_viz_graph(crate::dag_viz::DagVizMode::Diff { base_ref: \"main\".to_string() }).unwrap()",
    signature = "crate::dag_viz::dag_viz_signature(&crate::dag_viz::DagVizMode::Diff { base_ref: \"main\".to_string() })",
    tool = "dag-viz-diff"
)]
pub fn dag_viz_diff_mock_spec() -> MockSpec {
    dag_viz_mock_spec(&DagVizMode::Diff {
        base_ref: "main".to_string(),
    })
}

#[gunbc_testgen_registry_macros::testgen_target(
    name = "dag-viz-recent",
    output = "gunbc-dag/src/dag_viz/generated_tests_recent.rs",
    module = "dag_viz_recent_generated_tests",
    builder = "crate::dag_viz::build_dag_viz_graph(crate::dag_viz::DagVizMode::Recent).unwrap()",
    signature = "crate::dag_viz::dag_viz_signature(&crate::dag_viz::DagVizMode::Recent)",
    tool = "dag-viz-recent"
)]
pub fn dag_viz_recent_mock_spec() -> MockSpec {
    dag_viz_mock_spec(&DagVizMode::Recent)
}

#[gunbc_testgen_registry_macros::testgen_target(
    name = "dag-snapshot",
    output = "gunbc-dag/src/dag_viz/generated_tests_snapshot_save.rs",
    module = "dag_snapshot_generated_tests",
    builder = "crate::dag_viz::build_dag_viz_graph(crate::dag_viz::DagVizMode::SaveSnapshot).unwrap()",
    signature = "crate::dag_viz::dag_viz_signature(&crate::dag_viz::DagVizMode::SaveSnapshot)",
    tool = "dag-snapshot"
)]
pub fn dag_viz_save_snapshot_mock_spec() -> MockSpec {
    dag_viz_mock_spec(&DagVizMode::SaveSnapshot)
}
