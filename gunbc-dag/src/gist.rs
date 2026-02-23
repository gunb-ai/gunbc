//! Gist tool_target registrations.
//!
//! Three modes: snapshot (default), diff, recent.
//! All backed by `dsl/tools/gist.dag`.

#[gunbc_tool_registry_macros::tool_target(
    name = "gist",
    crate_name = "gunbc-dag",
    description = "Create a GitHub gist from workspace snapshot",
    builder = "build_gist_snapshot_graph_dsl",
    import = "use gunbc_dag::build_gist_snapshot_graph_dsl;",
    package = "dag",
    binary = "gist",
    mock_spec = r#"gunbc_dag::mock_defaults::auto_mock_spec(&dag, "gist")"#,
    entrypoints = r#"[{"port_name":"repo_path","type_id":"String","short":"r","default":".","help":"Repository path to scan","make_var":"REPO"},{"port_name":"extensions","type_id":"String","cardinality":"ZERO_OR_MORE","short":"e","help":"File extensions to include (can be repeated)","make_var":"EXT"},{"port_name":"public","type_id":"Bool","short":"p","help":"Make gist public"}]"#,
    dsl_module = "gist",
    has_invocation,
    returns_result
)]
pub fn gist_snapshot_tool() {}

#[gunbc_tool_registry_macros::tool_target(
    name = "gist-diff",
    crate_name = "gunbc-dag",
    description = "Create a GitHub gist from branch diff",
    builder = "build_gist_diff_graph_dsl",
    import = "use gunbc_dag::build_gist_diff_graph_dsl;",
    package = "dag",
    binary = "gist-diff",
    mock_spec = r#"gunbc_dag::mock_defaults::auto_mock_spec(&dag, "gist-diff")"#,
    entrypoints = r#"[{"port_name":"repo_path","type_id":"String","short":"r","default":".","help":"Repository path to scan","make_var":"REPO"},{"port_name":"base_ref","type_id":"String","short":"b","default":"main","help":"Base branch for diff","make_var":"BASE"},{"port_name":"extensions","type_id":"String","cardinality":"ZERO_OR_MORE","short":"e","help":"File extensions to include (can be repeated)","make_var":"EXT"},{"port_name":"public","type_id":"Bool","short":"p","help":"Make gist public"}]"#,
    dsl_module = "gist",
    has_invocation,
    returns_result
)]
pub fn gist_diff_tool() {}

#[gunbc_tool_registry_macros::tool_target(
    name = "gist-recent",
    crate_name = "gunbc-dag",
    description = "Create a GitHub gist from recent changes",
    builder = "build_gist_recent_graph_dsl",
    import = "use gunbc_dag::build_gist_recent_graph_dsl;",
    package = "dag",
    binary = "gist-recent",
    mock_spec = r#"gunbc_dag::mock_defaults::auto_mock_spec(&dag, "gist-recent")"#,
    entrypoints = r#"[{"port_name":"repo_path","type_id":"String","short":"r","default":".","help":"Repository path to scan","make_var":"REPO"},{"port_name":"extensions","type_id":"String","cardinality":"ZERO_OR_MORE","short":"e","help":"File extensions to include (can be repeated)","make_var":"EXT"},{"port_name":"public","type_id":"Bool","short":"p","help":"Make gist public"}]"#,
    dsl_module = "gist",
    has_invocation,
    returns_result
)]
pub fn gist_recent_tool() {}
