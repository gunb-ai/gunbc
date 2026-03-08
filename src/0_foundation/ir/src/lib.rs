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

/// DSL-generated types from `dsl/std/{symbols,render,box_draw}.dag`.
///
/// Regenerate:
/// ```sh
/// daglang gen-types dsl/std \
///   --module std.symbols --module std.render --module std.box_draw \
///   --output src/0_foundation/ir/src/generated/mod.rs
/// ```
pub mod generated;

pub mod algebra;
pub mod boundary;
pub mod builder;
pub mod cargo;
pub mod code_ir;
pub mod codegen_bridge;
pub mod coerce;
pub mod contract;
pub mod dag;
pub mod dag_topology;
pub mod entrypoint;
pub mod git;
pub mod invocation_contract;
pub mod language;
pub mod layout;
pub mod log_detail;
pub mod makefile_render;
pub mod node;
pub mod patterns;
pub mod plain_render;
pub mod platform;
pub mod render_ir;
pub mod resource;
pub mod signature;
pub mod symbol;
pub mod symbols;
pub mod system_model;
pub mod transport;
pub mod type_lib;
pub mod type_op;
pub mod type_registry;
pub mod type_shape;
pub mod typed_io;
pub mod types;
pub mod validate;
pub mod value;
pub mod value_bridge;
pub mod value_expr;
pub mod verified;
pub mod workspace_layout;

// Re-exports for convenience
pub use algebra::{BoundedLattice, JoinSemilattice, Lattice, MeetSemilattice, PartialOrder};
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
    AppliedCoercion, CardinalityCoercion, CardinalityDrift, CoercionError, CoercionKind,
    CoercionReport,
};
pub use contract::{
    cross_product_witnesses, generate_compliance_test_suite, generate_contract_test_body,
    generate_contract_test_fn, generate_interface_contract_tests, generate_response_contract_tests,
    validate_provider_compliance, validate_response_contract_coverage, variant_witness_for,
    variant_witnesses, BoundaryWitness, CodegenPlatformRepr, CodegenTypeShape, Platform,
    ProtocolLayer, ProtocolLayerKind, ProtocolStack, ProtocolStackError, ProviderBinding,
    ProviderResponseContract, ScalarKind, StatusSemantic, TypeContract, TypeLayer,
};
pub use dag::{
    build, canonical_edge_order, edges_to_port, Dag, DagEdgePorts, DagInputPort, DagOutputPort,
    Edge, EdgeKind, Guard, Port, ReachableDag,
};
pub use dag_topology::{DagTopology, EdgeTopology, NodeTopology, PortTopology};
pub use entrypoint::{detect_entrypoints, EntrypointInfo};
pub use git::GitConfig;
pub use invocation_contract::InvocationContract;
pub use layout::{
    compute_layout, compute_levels, ConnectorCell, DagLayout, EdgeLayout, EdgeOrientation,
    NodeLayout, OverflowState, OverflowStrategy, Viewport, ViewportUnit,
};
pub use log_detail::LogDetailLevel;
pub use makefile_render::MakefileStructuredRenderer;
pub use node::{
    Node, NodeBody, NodeIoExample, NodeKind, NodeOrigin, ServiceTransportClass, SubDagKind,
};
pub use patterns::{
    canonical_authenticate_chain,
    content_upsert::{add_content_upsert_chain, ContentUpsertChain},
    transport_triplet::{
        add_skippable_transport_triplet, add_skippable_transport_triplet_typed,
        add_transport_triplet, add_transport_triplet_named_with_passthrough,
        add_transport_triplet_named_with_passthrough_typed, add_transport_triplet_typed,
        TransportPortTypes,
    },
    validate_authenticate_bindings, validate_authenticate_chain, AtomicBuilder, AuthenticatePhase,
    AuthenticatePhaseBinding, BackoffStrategy, FailureClassifier, PatternOp, PollBuilder,
    RepeatPolicy, ResourceInput, RetryBuilder, TransactionBuilder, UpsertBuilder, WhileBuilder,
};
pub use plain_render::PlainStructuredRenderer;
pub use platform::{
    AbiEnv, Arch, ExecutionEnv, Os, RuntimePlatform, TargetTriple, ToolchainCommands, Vendor,
};
pub use render_ir::{
    AnsiText, Block, Category, CodeRenderer, CursorAction, DataNode, DataValue, Document,
    DocumentBody, FileHeader, Frame, FrameRenderer, GraphicsElement, HtmlText, Line, MarkupNode,
    OutputMedium, PlainText, RenderSurface, Span, SpanStyle, StructuredBlock, StructuredRenderer,
    Target, TextMedium,
};
pub use resource::{
    derive_resource_accesses, detect_resource_conflicts, normalize_resource_id, resource_api_port,
    resource_file_port, resource_port, resource_target_port, AccessMode, DagResource, Resource,
    ResourceAccess, ResourceAccessError, ResourceConflict, ResourceId, ResourceKind, Timestamp,
    API_NETWORK_HANDLE_PORT, FILE_HANDLE_READ_PORT, FILE_HANDLE_WRITE_PORT, RESOURCE_API_NETWORK,
    RESOURCE_CREDENTIAL, RESOURCE_FILE, RESOURCE_FILE_PREFIX, RESOURCE_PORT_PREFIX, RESOURCE_REPO,
    RESOURCE_TARGET,
};
pub use signature::{infer_signature, SignatureError, SignaturePort, WorkflowSignature};
pub use symbol::ProgramSymbolId;
pub use symbols::{SemanticColor, Symbol, SymbolId, SymbolOp, SymbolSet, Tier, STANDARD};
pub use system_model::{
    default_system_models, derive_contract_test_specs, generate_contract_test_harnesses,
    get_registered_system_model, iter_registered_system_models, render_contract_test_harness,
    validate_dependency_graph_acyclic, validate_store_behavior_mapping, validate_system_model,
    Behavior, BehaviorInput, BehaviorOutput, ContractTestSpec, Dependency, DependencyKind,
    InputType, Invocation, OutputType, Property, SecretDependencyId, SystemDependencyId,
    SystemKind, SystemModel, SystemModelDef, UpsertPhase,
};
pub use transport::{
    default_transport_behaviors, AuthScheme, CircuitBreakerConfig, Credential, CredentialConfig,
    CredentialError, CredentialInjection, CredentialIntent, CredentialProvider, FieldRouteSpec,
    RateLimitAlgorithm, RateLimitConfig, ResponseClassification, ResponseProvider, RetryBackoff,
    RetryConfig, ScopeContract, ScopeContractError, Secret, SecretSource, TransportBehavior,
    TransportKind, TransportMiddlewareConfig, TransportRequest, TransportResponse,
};
pub use type_op::{
    BaseType, Coercion, ContentEncoding, MetadataPayload, PlatformRepr, Predicate, PredicateValue,
    TypeOp, WrapperKind,
};
pub use type_registry::{TypeNotFoundError, TypeRegistry};
pub use type_shape::{type_shape, ContainerShape, TypeShape};
pub use typed_io::{
    typed_input, typed_output, typed_port, AnyTag, CredentialTag, FilePathTag, FilesystemHandleTag,
    ListTag, NetworkHandleTag, NonEmptyListTag, OptionalTag, PlatformTag, PortTypeTag, SecretTag,
    TimestampTag, ToolHandleTag, TransportRequestTag, TransportResponseTag, TypedInput,
    TypedOutput, TypedPort, UrlTag,
};
pub use types::{
    boundary_label, parse_map_type_id, seed_placeholder_policy_for_type_id,
    semantic_carrier_class_for_type_id, semantic_carrier_compatible,
    semantic_carrier_kind_for_type_id, value_backing_for_type_id, value_compatible_with_type_id,
    value_kind_name, Cardinality, CardinalityMismatch, CardinalitySamplingStrategy,
    InputProvenance, NodeId, OperationKey, PortCategory, PortName, SeedPlaceholderPolicy,
    SemanticCarrierClass, SemanticCarrierKind, StaticFingerprint, TypeId, ValueBacking,
};
pub use validate::{
    validate_fingerprint_uniqueness, validate_required_inputs, validate_resource_wiring,
    validate_resource_wiring_recursive, validate_subdag_interfaces, verify_dag,
    FingerprintConflict, PortDirection, SubDagError, UnwiredInputError, UnwiredResource,
    VerifyError,
};
pub use value::{
    SecretHint, SecretString, Value, ValueKind, HUMAN_TEXT_MAX_LINES, HUMAN_TEXT_MAX_LINE_WIDTH,
};
pub use value_bridge::{
    classify_value, from_bridge_json, from_bridge_json_typed, to_bridge_json, ValueCategory,
};
pub use value_expr::ValueExpr;
pub use verified::VerifiedDag;
pub use workspace_layout::{WorkspaceLayout, WorkspaceLayoutError};

// Re-exports from language module for common use
pub use language::{
    build_languages_dag, detect_language_from_file, html_comment, markdown_comment,
    markdown_language_id, render_code_block, render_html_document, rust_type, GitignoreConfig,
    HtmlConfig, LanguageOp, MakeTarget, MakefileConfig, MarkdownConfig, NamingCase, RustConfig,
    DEFAULT_GITIGNORE_FILENAME, DEFAULT_MAKEFILE_FILENAME, GITIGNORE, HTML, MAKEFILE, MARKDOWN,
    RUST,
};
