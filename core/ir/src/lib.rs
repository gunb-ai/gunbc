//! gunbc-ir: Core IR types for the gunbc DAG framework.
//!
//! This crate provides the fundamental types:
//! - [`Node`]: A node in the DAG (opaque operation or sub-DAG)
//! - [`Dag`]: A directed acyclic graph of nodes
//! - [`Edge`]: Connection between output and input ports
//! - [`Port`]: Input or output port with type and cardinality
//! - [`Value`]: Runtime values flowing through the DAG
//! - [`detect_boundaries`]: Find outputs that leave the DAG (world writes)
//! - [`detect_entrypoints`]: Find inputs that enter the DAG (world reads)
//! - [`transport`]: Transport layer types for I/O abstraction
//!
//! # Core Insight
//!
//! **World I/O is structural, not annotated.**
//!
//! - An output port with no downstream edge is a *boundary* — data leaving
//!   that port exits the DAG and goes to the world (world write).
//! - An input port with no upstream edge is an *entrypoint* — data entering
//!   that port comes from outside the DAG (world read).
//!
//! These are detected by [`detect_boundaries`] and [`detect_entrypoints`],
//! not by annotations on nodes.
//!
//! # Types as DAGs
//!
//! Types are also DAGs (`Dag<TypeOp>`) that describe validation and transformation
//! of values. This unifies types with workflows — same infrastructure, same
//! composition rules. See [`type_op`] and [`type_lib`] modules.
//!
//! # No Meta-Annotations
//!
//! Conditional execution is modeled through explicit Branch patterns and
//! optional types (ZeroOrOne cardinality), not through guards on ports.
//! This keeps the type system closed and self-consistent.
//!
//! # Transport Layer
//!
//! All world I/O can be modeled as transport requests/responses:
//! - REST/HTTP for web APIs
//! - File operations for filesystem I/O
//! - TCP for raw network connections
//! - Shell for command execution
//!
//! This allows business logic to remain pure while transport execution
//! happens at well-defined boundaries.

pub mod algebra;
pub mod boundary;
pub mod builder;
pub mod cargo;
pub mod code_ir;
pub mod codegen_bridge;
pub mod coerce;
pub mod compose;
pub mod contract;
pub mod dag;
pub mod dag_diff;
pub mod dag_mermaid;
pub mod dag_topology;
pub mod effect;
pub mod entrypoint;
pub mod git;
pub mod language;
pub mod layout;
pub mod log_detail;
pub mod makefile_render;
pub mod node;
pub mod patterns;
pub mod plain_render;
pub mod platform;
pub mod port_type;
pub mod render_ir;
pub mod resource;
pub mod signature;
pub mod symbols;
pub mod transport;
pub mod type_lib;
pub mod type_op;
pub mod type_registry;
pub mod types;
pub mod validate;
pub mod value;
pub mod value_bridge;
pub mod value_expr;
pub mod workspace_layout;

// ── DSL codegen IR tiers (dsl-codegen-tasks.md) ────────────────────
pub mod c_ir; // Task 5: C-level AST types (CStyleIR)
pub mod go_ir; // Task 7: Go-specific code_ir extensions (ManagedIR)
pub mod register_ir; // Task 6: MIPS instruction model (RegisterIR)

// Codegen output locations used by the bootstrapper and codegen DAG.
pub const CODEGEN_OUT_DIR: &str = "target/codegen";
pub const CODEGEN_BIN_DIR: &str = "target/codegen/bin";
pub const CODEGEN_LIB_DIR: &str = "target/codegen/lib";
pub const CODEGEN_STAMP_PATH: &str = "target/codegen/.codegen-stamp";

// Re-exports for convenience
pub use algebra::{
    BoundedLattice, JoinSemilattice, Lattice, MeetSemilattice, PartialOrder, Semiring,
};
pub use boundary::{detect_boundaries, BoundaryInfo};
pub use builder::{BuilderError, DagBuilder, InputRef, NodeRef, OutputRef, PortKind};
pub use cargo::{
    CargoCommand, CargoEnv, CargoInvocation, Subcommand, TermColor, Warnings,
    PREFIX as CARGO_PREFIX,
};
pub use code_ir::{
    Assert, EnumDef, Expr, FnDef, HelperFn, ImplBlock, Import, Item, MatchArm, SourceFile, Stmt,
    StructDef, TestFile, TestFn, TestSection,
};
pub use codegen_bridge::{BridgeEnum, BridgeField, BridgeFunction, BridgeModule, BridgeStruct};
pub use coerce::{
    audit_cardinality_drift, classify_coercion, detect_coercions, validate_coercions,
    CardinalityCoercion, CardinalityDrift, CoercionError, CoercionKind, CoercionReport,
};
pub use contract::{BoundaryWitness, TypeContract};
pub use dag::{build, canonical_edge_order, edges_to_port, Dag, Edge, EdgeKind, Port};
pub use dag_diff::{diff_topologies, DagDiffResult, NodeChangeSummary, NodeDiffStatus, PortChange};
pub use dag_mermaid::{
    render_changelog, to_mermaid_expanded_diff, to_mermaid_overview_diff, to_mermaid_snapshot,
};
pub use dag_topology::{DagTopology, EdgeTopology, NodeTopology, PortTopology};
pub use effect::Effect;
pub use entrypoint::{detect_entrypoints, EntrypointInfo};
pub use git::GitConfig;
pub use layout::{
    compute_layout, compute_levels, ConnectorCell, DagLayout, EdgeLayout, EdgeOrientation,
    NodeLayout, OverflowState, OverflowStrategy, Viewport, ViewportUnit,
};
pub use log_detail::LogDetailLevel;
pub use makefile_render::MakefileStructuredRenderer;
pub use node::{Node, NodeBody, NodeIoExample};
pub use patterns::{
    content_upsert::{add_content_upsert_chain, ContentUpsertChain},
    transport_triplet::{
        add_skippable_transport_triplet, add_skippable_transport_triplet_typed,
        add_transport_triplet, add_transport_triplet_named_with_passthrough,
        add_transport_triplet_named_with_passthrough_typed, add_transport_triplet_typed,
        TransportPortTypes,
    },
    AtomicBuilder, BackoffStrategy, FailureClassifier, PatternOp, PollBuilder, RepeatPolicy,
    ResourceInput, RetryBuilder, TransactionBuilder, UpsertBuilder, WhileBuilder,
};
pub use plain_render::PlainStructuredRenderer;
pub use platform::{
    AbiEnv, Arch, ExecutionEnv, Os, RuntimePlatform, TargetTriple, ToolchainCommands, Vendor,
};
pub use port_type::PortType;
pub use render_ir::{
    AnsiText, Block, Category, CodeRenderer, CursorAction, DataNode, DataValue, Document,
    DocumentBody, DocumentRenderer, FileHeader, Frame, FrameRenderer, GraphicsElement,
    GraphicsMedium, HtmlText, Line, MarkupNode, MarkupRenderer, OutputMedium, PlainText,
    RenderSurface, Span, SpanStyle, StructuredBlock, StructuredRenderer, Target, TextMedium,
};
pub use resource::{
    derive_resource_accesses, detect_resource_conflicts, normalize_resource_id, resource_api_port,
    resource_file_port, resource_port, resource_target_port, AccessMode, Resource, ResourceAccess,
    ResourceAccessError, ResourceConflict, ResourceId, ResourceKind, Timestamp,
    API_NETWORK_HANDLE_PORT, FILE_HANDLE_READ_PORT, FILE_HANDLE_WRITE_PORT, RESOURCE_API_NETWORK,
    RESOURCE_FILE, RESOURCE_FILE_PREFIX, RESOURCE_PORT_PREFIX, RESOURCE_REPO, RESOURCE_TARGET,
};
pub use signature::{infer_signature, SignatureError, SignaturePort, WorkflowSignature};
pub use symbols::{SemanticColor, Symbol, SymbolId, SymbolOp, SymbolSet, Tier, STANDARD};
pub use transport::{
    AuthScheme, Credential, CredentialError, CredentialIntent, ScopeContract, ScopeContractError,
    Secret, SecretSource, TransportRequest, TransportResponse,
};
pub use type_op::{BaseType, Coercion, Predicate, PredicateValue, TypeOp, WrapperKind};
pub use type_registry::{TypeNotFoundError, TypeRegistry};
pub use types::{
    boundary_label, seed_placeholder_policy_for_type_id, Cardinality, CardinalityMismatch, NodeId,
    PortName, SeedPlaceholderPolicy, TypeId,
};
pub use validate::{
    validate_resource_wiring, validate_resource_wiring_recursive, validate_subdag_interfaces,
    PortDirection, SubDagError, UnwiredResource,
};
pub use value::{SecretString, Value, HUMAN_TEXT_MAX_LINES, HUMAN_TEXT_MAX_LINE_WIDTH};
pub use value_bridge::{classify_value, from_bridge_json, to_bridge_json, ValueCategory};
pub use value_expr::ValueExpr;
pub use workspace_layout::{WorkspaceLayout, WorkspaceLayoutError};

// Re-exports from language module for common use
pub use language::{
    build_languages_dag, detect_language_from_file, html_comment, markdown_comment,
    markdown_language_id, render_code_block, render_html_document, rust_type, GitignoreConfig,
    HtmlConfig, LanguageOp, MakeTarget, MakefileConfig, MarkdownConfig, NamingCase, RustConfig,
    DEFAULT_GITIGNORE_FILENAME, DEFAULT_MAKEFILE_FILENAME, GITIGNORE, HTML, MAKEFILE, MARKDOWN,
    RUST,
};
