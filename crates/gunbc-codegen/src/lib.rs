pub mod emit_io;
pub mod emit_graph;
pub mod emit_constants;
pub mod emit_entrypoint;
pub mod emit_cli;
pub mod verify;

pub use emit_io::emit_io_structs;
pub use emit_graph::emit_subdag_builder;
pub use emit_constants::emit_port_constants;
pub use emit_entrypoint::{
    analyze_entrypoint, detect_entrypoint_kind, extract_layer_name,
    find_all_boundaries_recursive, find_leaf_nodes, find_root_nodes,
    is_cli_sink, is_cli_source, is_external_type, is_http_sink, is_http_source,
    is_sink_node, is_source_node,
    CliArgSpec, EntrypointInfo, EntrypointKind,
};
pub use emit_cli::{
    derive_execution_mode_flags, emit_cli_main, emit_cli_main_explicit, emit_cli_struct,
    emit_main_function, CliCodegenConfig,
};
pub use verify::{verify_acyclic, verify_type_agreement, verify_export_alignment, verify_port_saturation};
