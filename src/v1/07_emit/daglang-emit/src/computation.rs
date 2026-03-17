//! Target-independent computation model.
//!
//! Describes *what* each DAG node does, not *how* it's expressed in any language.
//! Every codegen backend consumes `Computation`, not `LoweredOp` directly.
//!
//! # Design
//!
//! The `Computation` enum classifies DAG node semantics into four categories:
//!
//! - **Pure**: deterministic transform with no side effects (string ops, JSON
//!   extraction, template rendering, comparisons).
//! - **Transport**: I/O boundary crossing (file read/write, shell exec, HTTP).
//! - **ResourceAcquire**: obtain a handle to a shared resource (filesystem,
//!   credential, service connection).
//! - **Collection**: apply an operation element-wise over a list.
//!
//! This classification is consumed by [`super::plan`] to build an `EmitPlan`,
//! which is then lowered to target-specific IR.

// ---------------------------------------------------------------------------
// EmitCollectionFamily — re-exported from gunbc_ir (S11)
// ---------------------------------------------------------------------------

/// Re-export from `gunbc_ir::patterns` — single source of truth (S11).
pub use gunbc_ir::patterns::EmitCollectionFamily;

/// Classify a collection op kind into an emit-level family.
///
/// Delegates to [`CollectionKind::emit_family`] — single source of truth (S11).
pub fn collection_emit_family(kind: &daglang_lower::CollectionOpKind) -> EmitCollectionFamily {
    kind.emit_family()
}

// ---------------------------------------------------------------------------
// Computation
// ---------------------------------------------------------------------------

/// What a DAG node does, independent of any target language.
#[derive(Debug, Clone)]
pub enum Computation {
    /// Pure deterministic transform: read inputs, apply body, produce outputs.
    /// No side effects, no I/O.
    Pure {
        inputs: Vec<TypedPort>,
        outputs: Vec<TypedPort>,
        body: PureBody,
    },
    /// Transport boundary: crosses the I/O boundary to interact with the world.
    /// Modeled as prepare → execute → parse.
    Transport {
        prepare: RequestSpec,
        execute: TransportKind,
        parse: ResponseSpec,
    },
    /// Resource acquisition: produce a handle to a shared resource.
    ResourceAcquire {
        handle_type: String,
        handle_value: String,
    },
    /// Collection operation: apply an operation to each element of a list.
    Collection {
        family: EmitCollectionFamily,
        element_type: String,
    },
}

// ---------------------------------------------------------------------------
// TypedPort (A1.2)
// ---------------------------------------------------------------------------

/// A typed port on a computation node.
///
/// Captures the port's name, abstract type, and whether it carries a single
/// value or a list of values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedPort {
    /// Port name (e.g., "registry", "content", "result").
    pub name: String,
    /// Abstract type identifier (e.g., "String", "ToolRegistry", "Bool").
    /// Not a language-level type — this is the DAG's type vocabulary.
    pub abstract_type: String,
    /// Whether this port carries a single value or a list.
    pub cardinality: Cardinality,
}

/// Value cardinality on a port.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cardinality {
    /// Exactly one value.
    Scalar,
    /// Zero or more values (list).
    List,
    /// Zero or one value (optional).
    Optional,
}

// ---------------------------------------------------------------------------
// PureBody — what a Pure computation actually computes
// ---------------------------------------------------------------------------

/// The specific transform applied by a [`Computation::Pure`] node.
#[derive(Debug, Clone)]
pub enum PureBody {
    /// Hardcoded value (e.g., LoadRegistry, FsEnv configuration).
    Literal(serde_json::Value),

    /// Build a transport request payload from step inputs.
    PrepareTransport { kind: TransportKind },

    /// String interpolation: fill variables into a pattern.
    /// `pattern` uses `{var}` placeholders; `vars` lists the names to substitute.
    Template { pattern: String, vars: Vec<String> },

    /// String operation (concat, split, filter, map).
    StringOp(StringOpKind),

    /// JSON operation (extract field, parse string, serialize value).
    JsonOp(JsonOpKind),

    /// Content comparison: check whether `left` and `right` are identical.
    /// Used for freshness checks (compare file content vs generated content).
    Compare { left: String, right: String },

    /// Conditional routing: emit to `then_port` or `else_port` based on condition.
    Conditional {
        condition: String,
        then_port: String,
        else_port: Option<String>,
    },

    /// Multi-input aggregation: combine several inputs into one output.
    Aggregate {
        inputs: Vec<String>,
        strategy: AggregateKind,
    },

    /// Service call: computation delegated to a service handler.
    /// The metadata identifies which service and method.
    ServiceCall(ServiceCallMetadata),
}

// ---------------------------------------------------------------------------
// TransportKind — I/O boundary crossings
// ---------------------------------------------------------------------------

/// The kind of I/O a [`Computation::Transport`] performs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportKind {
    /// Read file content from disk.
    FileRead,
    /// Write content to a file on disk.
    FileWrite,
    /// Check whether a file exists.
    FileExists,
    /// Execute a shell command and capture output.
    ShellExec,
    /// Make an HTTP request.
    HttpRequest,
    /// List directory entries.
    DirectoryList,
}

// ---------------------------------------------------------------------------
// Supporting types
// ---------------------------------------------------------------------------

/// How to prepare a transport request.
#[derive(Debug, Clone)]
pub struct RequestSpec {
    /// Which input ports feed the request.
    pub input_ports: Vec<String>,
    /// How to construct the request from inputs.
    pub kind: RequestKind,
}

/// Request construction strategy.
#[derive(Debug, Clone)]
pub enum RequestKind {
    /// Build from a file path input.
    FilePath { path_port: String },
    /// Build from shell command parts.
    ShellCommand {
        command_port: String,
        args_port: Option<String>,
    },
    /// Build from HTTP method + URL + body.
    Http {
        method: String,
        url_port: String,
        body_port: Option<String>,
    },
}

/// How to parse a transport response.
#[derive(Debug, Clone)]
pub struct ResponseSpec {
    /// Which output ports receive parsed data.
    pub output_ports: Vec<String>,
    /// How to extract data from the response.
    pub kind: ResponseKind,
}

/// Response parsing strategy.
#[derive(Debug, Clone)]
pub enum ResponseKind {
    /// The raw response content becomes the output.
    RawContent,
    /// Parse the response as JSON.
    JsonParse,
    /// Extract the exit code / success status.
    ExitStatus,
    /// Extract a boolean (file exists / doesn't exist).
    BooleanCheck,
}

/// String operation variants.
#[derive(Debug, Clone)]
pub enum StringOpKind {
    /// Concatenate multiple strings.
    Concat { separator: Option<String> },
    /// Split a string by a delimiter.
    Split { delimiter: String },
    /// Filter lines/items matching a pattern.
    Filter { pattern: String },
    /// Map each element through a transform.
    Map { transform: String },
    /// Join a list of strings.
    Join { separator: String },
}

/// JSON operation variants.
#[derive(Debug, Clone)]
pub enum JsonOpKind {
    /// Extract a field by path (e.g., `"response.data.items"`).
    Extract { path: String },
    /// Parse a JSON string into a value.
    Parse,
    /// Serialize a value to a JSON string.
    Serialize,
}

/// How multiple inputs are combined.
#[derive(Debug, Clone)]
pub enum AggregateKind {
    /// Logical AND of boolean inputs.
    AllTrue,
    /// Logical OR of boolean inputs.
    AnyTrue,
    /// Concatenate string inputs.
    Concat,
    /// Merge map/object inputs (last wins on conflict).
    Merge,
    /// Collect into a list.
    CollectList,
}

/// Metadata for a service-delegated computation.
#[derive(Debug, Clone)]
pub struct ServiceCallMetadata {
    /// Service name (e.g., "github", "gcp").
    pub service: String,
    /// Method or operation name.
    pub method: String,
    /// Additional config keys.
    pub config: Vec<(String, String)>,
}

// ===========================================================================
// Classification (A1.3)
// ===========================================================================

use daglang_lower::{
    LoweredOp, ObligationCategory, PrimitiveLiteral, PrimitiveOpKind, ServiceTransportClass,
};
use gunbc_ir::node::{Node, NodeBody, NodeKind};
use gunbc_ir::Port;

/// Error during computation classification.
#[derive(Debug, Clone)]
pub enum ClassifyError {
    /// Node is a SubDag — classify its inner nodes instead.
    SubDagNode(String),
    /// Unrecognizable operation — no heuristic matched.
    UnrecognizedOp { node_id: String, detail: String },
}

impl std::fmt::Display for ClassifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SubDagNode(id) => write!(f, "node `{id}` is a SubDag — classify inner nodes"),
            Self::UnrecognizedOp { node_id, detail } => {
                write!(f, "cannot classify node `{node_id}`: {detail}")
            }
        }
    }
}

/// Classify a DAG node's operation as a target-independent [`Computation`].
///
/// Dispatches on `node.kind` (stamped by the lowerer) for the top-level
/// classification (Pure/Transport/ResourceAcquire/Collection), then drills
/// into `LoweredOp` only for body details within each variant.
///
/// This eliminates the duplicated `ObligationCategory` dispatch that
/// previously existed between the lowerer and this function (S68).
pub fn classify_computation(node: &Node<LoweredOp>) -> Result<Computation, ClassifyError> {
    let op = match &node.body {
        NodeBody::Opaque(op) => op,
        NodeBody::SubDag(..) => return Err(ClassifyError::SubDagNode(node.id.0.clone())),
    };

    let inputs: Vec<TypedPort> = node.inputs.iter().map(port_to_typed).collect();
    let outputs: Vec<TypedPort> = node.outputs.iter().map(port_to_typed).collect();

    match node.kind {
        // Collection nodes: stamped by the lowerer from LoweredOp::Collection.
        NodeKind::Collection => {
            if let LoweredOp::Collection { kind, .. } = op {
                classify_collection(&inputs, kind)
            } else {
                Err(ClassifyError::UnrecognizedOp {
                    node_id: node.id.0.clone(),
                    detail: "NodeKind::Collection but LoweredOp is not Collection".into(),
                })
            }
        }

        // Transport execute: the I/O boundary crossing.
        NodeKind::TransportExecute => classify_transport_execute(op, &node.id.0, inputs, outputs),

        // Transport prepare/parse: pure computations that build requests or
        // parse responses, classified as Pure with ServiceCall body.
        NodeKind::TransportPrepare => classify_transport_phase(op, "prepare", inputs, outputs),
        NodeKind::TransportParse => classify_transport_phase(op, "parse", inputs, outputs),

        // Resource acquisition nodes.
        NodeKind::ResourceAcquire | NodeKind::ResourceEnvironment => {
            classify_resource_acquire(op, &node.id.0, &outputs)
        }

        // Remaining node kinds are all pure computations with varying bodies.
        // DataDeclaration nodes are metadata-only and filtered before emit,
        // but the match must be exhaustive.
        NodeKind::ResourceRelease
        | NodeKind::ParamSource
        | NodeKind::ToolEnvironment
        | NodeKind::ToolConsumer
        | NodeKind::DataDeclaration
        | NodeKind::Pure => classify_pure_body(op, inputs, outputs),
    }
}

/// Classify a transport execute node as `Computation::Transport`.
fn classify_transport_execute(
    op: &LoweredOp,
    node_id: &str,
    inputs: Vec<TypedPort>,
    outputs: Vec<TypedPort>,
) -> Result<Computation, ClassifyError> {
    let service_metadata = match op {
        LoweredOp::Transport {
            service_metadata, ..
        } => Some(service_metadata),
        LoweredOp::Primitive {
            kind: PrimitiveOpKind::IoExecuteFileRead,
            ..
        } => {
            return Ok(Computation::Transport {
                prepare: RequestSpec {
                    input_ports: inputs.iter().map(|p| p.name.clone()).collect(),
                    kind: RequestKind::FilePath {
                        path_port: "request".to_string(),
                    },
                },
                execute: TransportKind::FileRead,
                parse: ResponseSpec {
                    output_ports: outputs.iter().map(|p| p.name.clone()).collect(),
                    kind: ResponseKind::RawContent,
                },
            });
        }
        LoweredOp::Primitive {
            kind: PrimitiveOpKind::IoExecuteFileWrite,
            ..
        } => {
            return Ok(Computation::Transport {
                prepare: RequestSpec {
                    input_ports: inputs.iter().map(|p| p.name.clone()).collect(),
                    kind: RequestKind::FilePath {
                        path_port: "request".to_string(),
                    },
                },
                execute: TransportKind::FileWrite,
                parse: ResponseSpec {
                    output_ports: outputs.iter().map(|p| p.name.clone()).collect(),
                    kind: ResponseKind::ExitStatus,
                },
            });
        }
        _ => None,
    };

    let meta = service_metadata.ok_or_else(|| ClassifyError::UnrecognizedOp {
        node_id: node_id.to_string(),
        detail: "TransportExecute node without service metadata".into(),
    })?;
    let transport_kind = infer_transport_kind(node_id, meta)?;
    Ok(Computation::Transport {
        prepare: RequestSpec {
            input_ports: inputs.iter().map(|p| p.name.clone()).collect(),
            kind: infer_request_kind(&inputs, transport_kind),
        },
        execute: transport_kind,
        parse: ResponseSpec {
            output_ports: outputs.iter().map(|p| p.name.clone()).collect(),
            kind: ResponseKind::RawContent,
        },
    })
}

/// Classify a transport prepare or parse node as `Computation::Pure` with a
/// `ServiceCall` body.
fn classify_transport_phase(
    op: &LoweredOp,
    phase: &str,
    inputs: Vec<TypedPort>,
    outputs: Vec<TypedPort>,
) -> Result<Computation, ClassifyError> {
    // Primitive transport prepare/parse (file I/O).
    if let LoweredOp::Primitive { kind, .. } = op {
        match kind {
            PrimitiveOpKind::IoPrepareFileRead => {
                return Ok(Computation::Pure {
                    inputs,
                    outputs,
                    body: PureBody::PrepareTransport {
                        kind: TransportKind::FileRead,
                    },
                });
            }
            PrimitiveOpKind::IoPrepareFileWrite => {
                return Ok(Computation::Pure {
                    inputs,
                    outputs,
                    body: PureBody::PrepareTransport {
                        kind: TransportKind::FileWrite,
                    },
                });
            }
            _ => {}
        }
    }

    let body = match op {
        LoweredOp::Transport {
            service_metadata, ..
        } => PureBody::ServiceCall(ServiceCallMetadata {
            service: service_metadata.service.clone(),
            method: service_metadata.operation.clone(),
            config: vec![(String::from("phase"), phase.to_string())],
        }),
        _ => PureBody::Literal(serde_json::Value::Null),
    };
    Ok(Computation::Pure {
        inputs,
        outputs,
        body,
    })
}

/// Classify a resource acquire/environment node as `Computation::ResourceAcquire`.
fn classify_resource_acquire(
    op: &LoweredOp,
    node_id: &str,
    outputs: &[TypedPort],
) -> Result<Computation, ClassifyError> {
    let handle_type = outputs
        .first()
        .map(|p| p.abstract_type.clone())
        .unwrap_or_else(|| "Unknown".to_string());
    let handle_value = match op {
        LoweredOp::Callable { name, .. }
        | LoweredOp::Transport { name, .. }
        | LoweredOp::Primitive { name, .. } => name.clone(),
        _ => node_id.to_string(),
    };
    Ok(Computation::ResourceAcquire {
        handle_type,
        handle_value,
    })
}

/// Classify a pure-kind node's body from its `LoweredOp`.
fn classify_pure_body(
    op: &LoweredOp,
    inputs: Vec<TypedPort>,
    outputs: Vec<TypedPort>,
) -> Result<Computation, ClassifyError> {
    match op {
        LoweredOp::Pipeline { stage_names, .. } => Ok(Computation::Pure {
            inputs,
            outputs,
            body: PureBody::Aggregate {
                inputs: stage_names.clone(),
                strategy: AggregateKind::Concat,
            },
        }),
        LoweredOp::Primitive { module, name, kind } => {
            classify_primitive(module, name, kind, inputs, outputs)
        }
        LoweredOp::Callable {
            module,
            name,
            obligation,
            ..
        } => classify_callable_pure(module, name, (*obligation).into(), None, inputs, outputs),
        LoweredOp::Transport {
            module,
            name,
            obligation,
            service_metadata,
            ..
        } => classify_callable_pure(
            module,
            name,
            (*obligation).into(),
            Some(service_metadata),
            inputs,
            outputs,
        ),
        LoweredOp::Pattern(_) => Ok(Computation::Pure {
            inputs,
            outputs,
            body: PureBody::Literal(serde_json::Value::Null),
        }),
        LoweredOp::UnsupportedPattern { name } => Err(ClassifyError::SubDagNode(format!(
            "unsupported pattern: {name}"
        ))),
        // Collection should not reach here (handled by NodeKind::Collection).
        LoweredOp::Collection { .. } => Err(ClassifyError::UnrecognizedOp {
            node_id: "unknown".to_string(),
            detail: "Collection node with non-Collection NodeKind".into(),
        }),
    }
}

fn classify_primitive(
    _module: &str,
    name: &str,
    kind: &PrimitiveOpKind,
    inputs: Vec<TypedPort>,
    outputs: Vec<TypedPort>,
) -> Result<Computation, ClassifyError> {
    match kind {
        PrimitiveOpKind::FsEnv => {
            let handle_type = outputs
                .first()
                .map(|p| p.abstract_type.clone())
                .unwrap_or_else(|| "Unknown".to_string());
            Ok(Computation::ResourceAcquire {
                handle_type,
                handle_value: name.to_string(),
            })
        }
        PrimitiveOpKind::CallLiteralSource { literal } => Ok(Computation::Pure {
            inputs,
            outputs,
            body: PureBody::Literal(primitive_literal_to_json(literal)),
        }),
        PrimitiveOpKind::IoPrepareFileRead => Ok(Computation::Pure {
            inputs,
            outputs,
            body: PureBody::PrepareTransport {
                kind: TransportKind::FileRead,
            },
        }),
        PrimitiveOpKind::IoExecuteFileRead => Ok(Computation::Transport {
            prepare: RequestSpec {
                input_ports: inputs.iter().map(|p| p.name.clone()).collect(),
                kind: RequestKind::FilePath {
                    path_port: "request".to_string(),
                },
            },
            execute: TransportKind::FileRead,
            parse: ResponseSpec {
                output_ports: outputs.iter().map(|p| p.name.clone()).collect(),
                kind: ResponseKind::RawContent,
            },
        }),
        PrimitiveOpKind::CompareEquality => Ok(Computation::Pure {
            inputs,
            outputs,
            body: PureBody::Compare {
                left: "expected_content".to_string(),
                right: "response".to_string(),
            },
        }),
        PrimitiveOpKind::IoPrepareFileWrite => Ok(Computation::Pure {
            inputs,
            outputs,
            body: PureBody::PrepareTransport {
                kind: TransportKind::FileWrite,
            },
        }),
        PrimitiveOpKind::IoExecuteFileWrite => Ok(Computation::Transport {
            prepare: RequestSpec {
                input_ports: inputs.iter().map(|p| p.name.clone()).collect(),
                kind: RequestKind::FilePath {
                    path_port: "request".to_string(),
                },
            },
            execute: TransportKind::FileWrite,
            parse: ResponseSpec {
                output_ports: outputs.iter().map(|p| p.name.clone()).collect(),
                kind: ResponseKind::ExitStatus,
            },
        }),
        PrimitiveOpKind::CallParamSource { .. } => Ok(Computation::Pure {
            inputs,
            outputs,
            body: PureBody::Literal(serde_json::Value::Null),
        }),
        // FC-7: Output path annotation nodes are metadata-only, no computation.
        PrimitiveOpKind::ContentUpsertOutputPath { .. } => Ok(Computation::Pure {
            inputs,
            outputs,
            body: PureBody::Literal(serde_json::Value::Null),
        }),
        // C24: GetField is an interpreter-only operation resolved at runtime
        // by the resolve layer. It must not reach the emitter.
        PrimitiveOpKind::GetField { field } => Err(ClassifyError::UnrecognizedOp {
            node_id: name.to_string(),
            detail: format!("GetField({field}) is interpreter-only and cannot be emitted"),
        }),
        // C24: All remaining structural primitive ops are interpreter-only
        // (resolved at runtime by the resolve layer). They must not reach the emitter.
        _ => {
            debug_assert!(
                kind.is_structural(),
                "unhandled non-structural primitive: {kind:?}"
            );
            Err(ClassifyError::UnrecognizedOp {
                node_id: name.to_string(),
                detail: format!("{kind:?} is interpreter-only and cannot be emitted"),
            })
        }
    }
}

fn primitive_literal_to_json(literal: &PrimitiveLiteral) -> serde_json::Value {
    match literal {
        PrimitiveLiteral::String(value) => serde_json::Value::String(value.clone()),
        PrimitiveLiteral::Int(value) => serde_json::Value::Number((*value).into()),
        PrimitiveLiteral::Bool(value) => serde_json::Value::Bool(*value),
        PrimitiveLiteral::Json(value) => value.clone(),
        PrimitiveLiteral::Unit => serde_json::Value::Null,
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Convert a gunbc-ir `Port` to a [`TypedPort`].
fn port_to_typed(port: &Port) -> TypedPort {
    let cardinality = if port.cardinality.is_scalar() {
        Cardinality::Scalar
    } else if port.cardinality.min == 0 && port.cardinality.max == Some(1) {
        Cardinality::Optional
    } else {
        Cardinality::List
    };
    TypedPort {
        name: port.name.0.clone(),
        abstract_type: port.type_id.0.clone(),
        cardinality,
    }
}

fn classify_collection(
    inputs: &[TypedPort],
    kind: &daglang_lower::CollectionOpKind,
) -> Result<Computation, ClassifyError> {
    let element_type = inputs
        .iter()
        .find(|p| p.cardinality == Cardinality::List)
        .map(|p| p.abstract_type.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    Ok(Computation::Collection {
        family: collection_emit_family(kind),
        element_type,
    })
}

/// Classify a callable/transport node that `node.kind` has already identified
/// as pure. Uses the `ObligationCategory` only to determine the *body* variant
/// (template, literal, compare, service-call), not the top-level Computation
/// category — that decision was made at lowering time (S68).
fn classify_callable_pure(
    module: &str,
    name: &str,
    obligation: ObligationCategory,
    service_metadata: Option<&daglang_lower::ServiceCallMetadata>,
    inputs: Vec<TypedPort>,
    outputs: Vec<TypedPort>,
) -> Result<Computation, ClassifyError> {
    let body = match obligation {
        ObligationCategory::ServiceTransportPrepare => service_metadata
            .map(|meta| {
                PureBody::ServiceCall(ServiceCallMetadata {
                    service: meta.service.clone(),
                    method: meta.operation.clone(),
                    config: vec![("phase".to_string(), "prepare".to_string())],
                })
            })
            .unwrap_or(PureBody::Literal(serde_json::Value::Null)),

        ObligationCategory::ServiceTransportParse => service_metadata
            .map(|meta| {
                PureBody::ServiceCall(ServiceCallMetadata {
                    service: meta.service.clone(),
                    method: meta.operation.clone(),
                    config: vec![("phase".to_string(), "parse".to_string())],
                })
            })
            .unwrap_or(PureBody::Literal(serde_json::Value::Null)),

        ObligationCategory::InterfaceContractVerification => PureBody::Compare {
            left: "expected".to_string(),
            right: "actual".to_string(),
        },

        ObligationCategory::PureRender => {
            let vars = inputs.iter().map(|p| p.name.clone()).collect();
            PureBody::Template {
                pattern: name.to_string(),
                vars,
            }
        }

        ObligationCategory::None => {
            return classify_by_name(module, name, inputs, outputs);
        }

        // All other pure-kind obligations produce a literal body.
        _ => PureBody::Literal(serde_json::Value::Null),
    };

    Ok(Computation::Pure {
        inputs,
        outputs,
        body,
    })
}

/// Derive [`TransportKind`] from required service metadata.
///
/// Uses the DSL service definition's transport binding and `readonly` flag.
/// Returns an error for `Unknown` transport class (the caller must specify a
/// concrete transport) and for `InterfaceStub` (stubs are resolved before
/// emission).
fn infer_transport_kind(
    name: &str,
    service_metadata: &daglang_lower::ServiceCallMetadata,
) -> Result<TransportKind, ClassifyError> {
    match service_metadata.transport {
        ServiceTransportClass::FileBoundary => {
            if service_metadata.readonly {
                Ok(TransportKind::FileRead)
            } else {
                Ok(TransportKind::FileWrite)
            }
        }
        ServiceTransportClass::ShellLocal => Ok(TransportKind::ShellExec),
        ServiceTransportClass::RestNetwork => Ok(TransportKind::HttpRequest),
        ServiceTransportClass::LocalDirect => Ok(TransportKind::ShellExec),
        ServiceTransportClass::Unknown => Err(ClassifyError::UnrecognizedOp {
            node_id: name.to_string(),
            detail:
                "transport class is Unknown — service must declare a concrete transport binding"
                    .into(),
        }),
        ServiceTransportClass::InterfaceStub => Err(ClassifyError::UnrecognizedOp {
            node_id: name.to_string(),
            detail:
                "InterfaceStub transport should be resolved to a concrete binding before emission"
                    .into(),
        }),
    }
}

/// Infer the request construction strategy from port names and transport kind.
fn infer_request_kind(inputs: &[TypedPort], kind: TransportKind) -> RequestKind {
    match kind {
        TransportKind::FileRead
        | TransportKind::FileWrite
        | TransportKind::FileExists
        | TransportKind::DirectoryList => {
            let path_port = inputs
                .iter()
                .find(|p| p.name.contains("path") || p.name == "request")
                .map(|p| p.name.clone())
                .unwrap_or_else(|| "request".to_string());
            RequestKind::FilePath { path_port }
        }
        TransportKind::ShellExec => {
            let command_port = inputs
                .iter()
                .find(|p| p.name.contains("command") || p.name == "request")
                .map(|p| p.name.clone())
                .unwrap_or_else(|| "request".to_string());
            let args_port = inputs
                .iter()
                .find(|p| p.name.contains("args"))
                .map(|p| p.name.clone());
            RequestKind::ShellCommand {
                command_port,
                args_port,
            }
        }
        TransportKind::HttpRequest => {
            let url_port = inputs
                .iter()
                .find(|p| p.name.contains("url") || p.name == "request")
                .map(|p| p.name.clone())
                .unwrap_or_else(|| "request".to_string());
            let body_port = inputs
                .iter()
                .find(|p| p.name.contains("body"))
                .map(|p| p.name.clone());
            RequestKind::Http {
                method: "GET".to_string(),
                url_port,
                body_port,
            }
        }
    }
}

/// Safety-net classifier for callables with `ObligationCategory::None`.
///
/// Most callables are now classified by obligation (assigned in the lowerer).
/// This function handles any remaining untagged callables as a fallback.
fn classify_by_name(
    _module: &str,
    _name: &str,
    inputs: Vec<TypedPort>,
    outputs: Vec<TypedPort>,
) -> Result<Computation, ClassifyError> {
    Ok(Computation::Pure {
        inputs,
        outputs,
        body: PureBody::Literal(serde_json::Value::Null),
    })
}

// ===========================================================================
// Fn Body Classification (CP-11)
// ===========================================================================

/// Classification of a callable's body based on its computational nature.
///
/// Used by the emit pipeline to decide code generation strategy:
/// - **PureRender**: only string/template operations, no I/O — can be
///   const-evaluated or inlined.
/// - **PureCompute**: deterministic computation with data transforms but
///   no transport — can be tested without mocks.
/// - **Effectful**: contains transport or resource operations — requires
///   mocks for testing, I/O runtime at execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FnBodyClassification {
    /// Pure string/template rendering — no computation, no I/O.
    PureRender,
    /// Deterministic computation — no I/O but may include logic/data ops.
    PureCompute,
    /// Contains transport or resource operations — requires I/O runtime.
    Effectful,
}

impl FnBodyClassification {
    /// Whether this classification requires mocks for testing.
    pub fn needs_mocks(&self) -> bool {
        matches!(self, FnBodyClassification::Effectful)
    }

    /// Whether this classification is guaranteed deterministic.
    pub fn is_deterministic(&self) -> bool {
        matches!(
            self,
            FnBodyClassification::PureRender | FnBodyClassification::PureCompute
        )
    }
}

/// Classify a callable's fn body by walking its SubDag nodes.
///
/// Uses `node.kind` (set by the lowerer) instead of string heuristics:
/// - If any node has a transport/resource `NodeKind` → `Effectful`
/// - If any node has non-trivial ports → `PureCompute`
/// - Otherwise → `PureRender`
pub fn classify_fn_body<T: std::fmt::Debug>(dag: &gunbc_ir::Dag<T>) -> FnBodyClassification {
    let mut has_compute = false;
    for node in &dag.nodes {
        match node.kind {
            NodeKind::TransportExecute
            | NodeKind::TransportPrepare
            | NodeKind::TransportParse
            | NodeKind::ResourceAcquire
            | NodeKind::ResourceRelease
            | NodeKind::ResourceEnvironment => {
                return FnBodyClassification::Effectful;
            }
            _ => {}
        }
        if !node.inputs.is_empty() || !node.outputs.is_empty() {
            has_compute = true;
        }
    }
    if has_compute {
        FnBodyClassification::PureCompute
    } else {
        FnBodyClassification::PureRender
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use daglang_lower::{CallableKind, CallableObligation, LoweredOp, TransportObligation};
    use gunbc_ir::Node;

    /// Helper: build an opaque node with the given op.
    fn make_node(
        id: &str,
        inputs: Vec<Port>,
        outputs: Vec<Port>,
        op: LoweredOp,
    ) -> Node<LoweredOp> {
        Node::opaque(id, inputs, outputs, op)
    }

    fn scalar(name: &str, ty: &str) -> Port {
        Port::scalar(name, ty)
    }

    // -- A1.4: makegen node classification tests --

    #[test]
    fn classify_makegen_load_registry() {
        let node = make_node(
            "load_registry",
            vec![],
            vec![scalar("registry", "ToolRegistry")],
            LoweredOp::Callable {
                module: "tools.makegen".into(),
                kind: CallableKind::Fn,
                name: "load_registry".into(),
                obligation: CallableObligation::PureDataLoad,
                is_interactive: false,
                resource_target: None,
                fn_body: None,
            },
        );
        let comp = classify_computation(&node).unwrap();
        assert!(matches!(
            comp,
            Computation::Pure {
                body: PureBody::Literal(_),
                ..
            }
        ));
    }

    #[test]
    fn classify_makegen_render_makefile() {
        let node = make_node(
            "render_makefile",
            vec![scalar("registry", "ToolRegistry")],
            vec![scalar("return", "String")],
            LoweredOp::Callable {
                module: "tools.makegen".into(),
                kind: CallableKind::Fn,
                name: "render_makefile".into(),
                obligation: CallableObligation::PureRender,
                is_interactive: false,
                resource_target: None,
                fn_body: None,
            },
        );
        let comp = classify_computation(&node).unwrap();
        assert!(
            matches!(
                comp,
                Computation::Pure {
                    body: PureBody::Template { .. },
                    ..
                }
            ),
            "expected Template, got {comp:?}"
        );
    }

    #[test]
    fn classify_makegen_prepare_read() {
        let node = make_node(
            "prepare_read_makegen",
            vec![
                scalar("path", "String"),
                scalar("res:file:Makefile", "FilesystemHandle"),
            ],
            vec![scalar("request", "TransportRequest")],
            LoweredOp::Primitive {
                module: "tools.makegen".into(),
                name: "content_upsert::prepare_read_makegen".into(),
                kind: daglang_lower::PrimitiveOpKind::IoPrepareFileRead,
            },
        )
        .with_kind(NodeKind::TransportPrepare);
        let comp = classify_computation(&node).unwrap();
        assert!(
            matches!(
                comp,
                Computation::Pure {
                    body: PureBody::PrepareTransport {
                        kind: TransportKind::FileRead
                    },
                    ..
                }
            ),
            "prepare_read should prepare file-read transport request, got {comp:?}"
        );
    }

    #[test]
    fn classify_makegen_execute_read() {
        let node = make_node(
            "execute_read_makegen",
            vec![scalar("request", "TransportRequest")],
            vec![scalar("response", "TransportResponse")],
            LoweredOp::Primitive {
                module: "tools.makegen".into(),
                name: "content_upsert::execute_read_makegen".into(),
                kind: daglang_lower::PrimitiveOpKind::IoExecuteFileRead,
            },
        )
        .with_kind(NodeKind::TransportExecute);
        let comp = classify_computation(&node).unwrap();
        assert!(
            matches!(
                comp,
                Computation::Transport {
                    execute: TransportKind::FileRead,
                    ..
                }
            ),
            "execute_read should be Transport(FileRead), got {comp:?}"
        );
    }

    #[test]
    fn classify_makegen_compare_content() {
        let node = make_node(
            "compare_makegen_content",
            vec![
                scalar("expected_content", "String"),
                scalar("response", "TransportResponse"),
            ],
            vec![scalar("fresh", "Bool"), scalar("skip", "Bool")],
            LoweredOp::Primitive {
                module: "tools.makegen".into(),
                name: "content_upsert::compare_makegen_content".into(),
                kind: daglang_lower::PrimitiveOpKind::CompareEquality,
            },
        );
        let comp = classify_computation(&node).unwrap();
        assert!(
            matches!(
                comp,
                Computation::Pure {
                    body: PureBody::Compare { .. },
                    ..
                }
            ),
            "compare should be Pure(Compare), got {comp:?}"
        );
    }

    #[test]
    fn classify_makegen_prepare_write() {
        let node = make_node(
            "prepare_write_makegen",
            vec![scalar("content", "String"), scalar("path", "String")],
            vec![scalar("request", "TransportRequest")],
            LoweredOp::Primitive {
                module: "tools.makegen".into(),
                name: "content_upsert::prepare_write_makegen".into(),
                kind: daglang_lower::PrimitiveOpKind::IoPrepareFileWrite,
            },
        )
        .with_kind(NodeKind::TransportPrepare);
        let comp = classify_computation(&node).unwrap();
        assert!(
            matches!(
                comp,
                Computation::Pure {
                    body: PureBody::PrepareTransport {
                        kind: TransportKind::FileWrite
                    },
                    ..
                }
            ),
            "prepare_write should prepare file-write transport request, got {comp:?}"
        );
    }

    #[test]
    fn classify_makegen_execute_transport() {
        let node = make_node(
            "execute_makegen_transport",
            vec![
                scalar("request", "TransportRequest"),
                scalar("skip", "Bool"),
            ],
            vec![scalar("response", "TransportResponse")],
            LoweredOp::Primitive {
                module: "tools.makegen".into(),
                name: "content_upsert::execute_makegen_transport".into(),
                kind: daglang_lower::PrimitiveOpKind::IoExecuteFileWrite,
            },
        )
        .with_kind(NodeKind::TransportExecute);
        let comp = classify_computation(&node).unwrap();
        assert!(
            matches!(
                comp,
                Computation::Transport {
                    execute: TransportKind::FileWrite,
                    ..
                }
            ),
            "execute_transport should be Transport(FileWrite), got {comp:?}"
        );
    }

    #[test]
    fn classify_makegen_fs_env() {
        let node = make_node(
            "fs_env",
            vec![],
            vec![scalar("handle", "FilesystemHandle")],
            LoweredOp::Callable {
                module: "tools.makegen".into(),
                kind: CallableKind::Fn,
                name: "fs_env".into(),
                obligation: CallableObligation::ResourceProvide,
                is_interactive: false,
                resource_target: None,
                fn_body: None,
            },
        )
        .with_kind(NodeKind::ResourceEnvironment);
        let comp = classify_computation(&node).unwrap();
        assert!(
            matches!(comp, Computation::ResourceAcquire { .. }),
            "fs_env with Handle output should be ResourceAcquire, got {comp:?}"
        );
    }

    #[test]
    fn classify_makegen_entrypoint() {
        let node = make_node(
            "makegen",
            vec![scalar("registry", "ToolRegistry")],
            vec![scalar("written", "Bool")],
            LoweredOp::Callable {
                module: "tools.makegen".into(),
                kind: CallableKind::Func,
                name: "makegen".into(),
                obligation: CallableObligation::None,
                is_interactive: false,
                resource_target: None,
                fn_body: None,
            },
        );
        let comp = classify_computation(&node).unwrap();
        // Entrypoint with no special name pattern → generic Pure(Literal).
        assert!(matches!(comp, Computation::Pure { .. }));
    }

    // -- A1.5: pragma node classification tests --

    #[test]
    fn classify_pragma_render_clippy() {
        let node = make_node(
            "render_clippy",
            vec![scalar("registry", "ToolRegistry")],
            vec![scalar("return", "String")],
            LoweredOp::Callable {
                module: "pragma".into(),
                kind: CallableKind::Fn,
                name: "render_clippy".into(),
                obligation: CallableObligation::PureRender,
                is_interactive: false,
                resource_target: None,
                fn_body: None,
            },
        );
        let comp = classify_computation(&node).unwrap();
        assert!(matches!(
            comp,
            Computation::Pure {
                body: PureBody::Template { .. },
                ..
            }
        ));
    }

    #[test]
    fn classify_pragma_render_allowlist() {
        let node = make_node(
            "render_allowlist",
            vec![scalar("registry", "ToolRegistry")],
            vec![scalar("return", "String")],
            LoweredOp::Callable {
                module: "pragma".into(),
                kind: CallableKind::Fn,
                name: "render_allowlist".into(),
                obligation: CallableObligation::PureRender,
                is_interactive: false,
                resource_target: None,
                fn_body: None,
            },
        );
        let comp = classify_computation(&node).unwrap();
        assert!(matches!(
            comp,
            Computation::Pure {
                body: PureBody::Template { .. },
                ..
            }
        ));
    }

    #[test]
    fn classify_pragma_render_lint_policy() {
        let node = make_node(
            "render_lint_policy",
            vec![scalar("registry", "ToolRegistry")],
            vec![scalar("return", "String")],
            LoweredOp::Callable {
                module: "pragma".into(),
                kind: CallableKind::Fn,
                name: "render_lint_policy".into(),
                obligation: CallableObligation::PureRender,
                is_interactive: false,
                resource_target: None,
                fn_body: None,
            },
        );
        let comp = classify_computation(&node).unwrap();
        assert!(matches!(
            comp,
            Computation::Pure {
                body: PureBody::Template { .. },
                ..
            }
        ));
    }

    #[test]
    fn classify_pragma_execute_read_transport() {
        let node = make_node(
            "execute_read_clippy",
            vec![scalar("request", "TransportRequest")],
            vec![scalar("response", "TransportResponse")],
            LoweredOp::Primitive {
                module: "pragma".into(),
                name: "content_upsert::execute_read_clippy".into(),
                kind: daglang_lower::PrimitiveOpKind::IoExecuteFileRead,
            },
        )
        .with_kind(NodeKind::TransportExecute);
        let comp = classify_computation(&node).unwrap();
        assert!(matches!(
            comp,
            Computation::Transport {
                execute: TransportKind::FileRead,
                ..
            }
        ));
    }

    #[test]
    fn classify_pragma_compare_content() {
        let node = make_node(
            "compare_clippy_content",
            vec![
                scalar("expected_content", "String"),
                scalar("response", "TransportResponse"),
            ],
            vec![scalar("fresh", "Bool"), scalar("skip", "Bool")],
            LoweredOp::Primitive {
                module: "pragma".into(),
                name: "content_upsert::compare_clippy_content".into(),
                kind: daglang_lower::PrimitiveOpKind::CompareEquality,
            },
        );
        let comp = classify_computation(&node).unwrap();
        assert!(matches!(
            comp,
            Computation::Pure {
                body: PureBody::Compare { .. },
                ..
            }
        ));
    }

    #[test]
    fn classify_pragma_execute_write_transport() {
        let node = make_node(
            "execute_clippy_transport",
            vec![
                scalar("request", "TransportRequest"),
                scalar("skip", "Bool"),
            ],
            vec![scalar("response", "TransportResponse")],
            LoweredOp::Primitive {
                module: "pragma".into(),
                name: "content_upsert::execute_clippy_transport".into(),
                kind: daglang_lower::PrimitiveOpKind::IoExecuteFileWrite,
            },
        )
        .with_kind(NodeKind::TransportExecute);
        let comp = classify_computation(&node).unwrap();
        assert!(matches!(
            comp,
            Computation::Transport {
                execute: TransportKind::FileWrite,
                ..
            }
        ));
    }

    // -- Obligation-based classification --

    #[test]
    fn classify_resource_acquire() {
        let node = make_node(
            "acquire_fs",
            vec![],
            vec![scalar("handle", "FilesystemHandle")],
            LoweredOp::Callable {
                module: "infra".into(),
                kind: CallableKind::Fn,
                name: "acquire_filesystem".into(),
                obligation: CallableObligation::ResourceAcquire,
                is_interactive: false,
                resource_target: None,
                fn_body: None,
            },
        )
        .with_kind(NodeKind::ResourceAcquire);
        let comp = classify_computation(&node).unwrap();
        assert!(matches!(comp, Computation::ResourceAcquire { .. }));
    }

    #[test]
    fn classify_service_transport_execute_shell() {
        let meta = daglang_lower::ServiceCallMetadata {
            service: "local".into(),
            operation: "run_command".into(),
            transport: ServiceTransportClass::ShellLocal,
            idempotent: true,
            readonly: false,
            spec: None,
        };
        let node = make_node(
            "execute_cmd",
            vec![scalar("request", "TransportRequest")],
            vec![scalar("response", "TransportResponse")],
            LoweredOp::Transport {
                module: "svc.local".into(),
                kind: CallableKind::Func,
                name: "run_command".into(),
                obligation: TransportObligation::Execute,
                service_metadata: Box::new(meta),
                is_interactive: false,
                resource_target: None,
            },
        )
        .with_kind(NodeKind::TransportExecute);
        let comp = classify_computation(&node).unwrap();
        assert!(
            matches!(
                comp,
                Computation::Transport {
                    execute: TransportKind::ShellExec,
                    ..
                }
            ),
            "expected ShellExec, got {comp:?}"
        );
    }

    #[test]
    fn classify_service_transport_execute_http() {
        let meta = daglang_lower::ServiceCallMetadata {
            service: "github".into(),
            operation: "list_repos".into(),
            transport: ServiceTransportClass::RestNetwork,
            idempotent: true,
            readonly: true,
            spec: None,
        };
        let node = make_node(
            "execute_list_repos",
            vec![scalar("request", "TransportRequest")],
            vec![scalar("response", "TransportResponse")],
            LoweredOp::Transport {
                module: "svc.github".into(),
                kind: CallableKind::Func,
                name: "list_repos".into(),
                obligation: TransportObligation::Execute,
                service_metadata: Box::new(meta),
                is_interactive: false,
                resource_target: None,
            },
        )
        .with_kind(NodeKind::TransportExecute);
        let comp = classify_computation(&node).unwrap();
        assert!(matches!(
            comp,
            Computation::Transport {
                execute: TransportKind::HttpRequest,
                ..
            }
        ));
    }

    #[test]
    fn classify_service_transport_prepare() {
        let meta = daglang_lower::ServiceCallMetadata {
            service: "github".into(),
            operation: "list_repos".into(),
            transport: ServiceTransportClass::RestNetwork,
            idempotent: true,
            readonly: true,
            spec: None,
        };
        let node = make_node(
            "prepare_list_repos",
            vec![scalar("token", "String")],
            vec![scalar("request", "TransportRequest")],
            LoweredOp::Transport {
                module: "svc.github".into(),
                kind: CallableKind::Func,
                name: "prepare_list_repos".into(),
                obligation: TransportObligation::Prepare,
                service_metadata: Box::new(meta),
                is_interactive: false,
                resource_target: None,
            },
        )
        .with_kind(NodeKind::TransportPrepare);
        let comp = classify_computation(&node).unwrap();
        assert!(
            matches!(
                comp,
                Computation::Pure {
                    body: PureBody::ServiceCall(_),
                    ..
                }
            ),
            "service prepare should be Pure(ServiceCall), got {comp:?}"
        );
    }

    #[test]
    fn classify_collection_map() {
        let node = make_node(
            "map_items",
            vec![Port::list("items", "String")],
            vec![Port::list("mapped", "String")],
            LoweredOp::Collection {
                module: "tools.makegen".into(),
                callable: "transform".into(),
                kind: daglang_lower::CollectionOpKind::Map,
            },
        )
        .with_kind(NodeKind::Collection);
        let comp = classify_computation(&node).unwrap();
        assert!(matches!(
            comp,
            Computation::Collection {
                family: EmitCollectionFamily::Map,
                ..
            }
        ));
    }

    #[test]
    fn classify_collection_filter() {
        let node = make_node(
            "filter_items",
            vec![Port::list("items", "String")],
            vec![Port::list("filtered", "String")],
            LoweredOp::Collection {
                module: "tools.makegen".into(),
                callable: "is_valid".into(),
                kind: daglang_lower::CollectionOpKind::Filter,
            },
        )
        .with_kind(NodeKind::Collection);
        let comp = classify_computation(&node).unwrap();
        assert!(matches!(
            comp,
            Computation::Collection {
                family: EmitCollectionFamily::Filter,
                ..
            }
        ));
    }

    #[test]
    fn classify_collection_len_as_fold_family() {
        let node = make_node(
            "len_items",
            vec![Port::list("items", "String")],
            vec![scalar("len", "Int")],
            LoweredOp::Collection {
                module: "tools.makegen".into(),
                callable: "len".into(),
                kind: daglang_lower::CollectionOpKind::Len,
            },
        )
        .with_kind(NodeKind::Collection);
        let comp = classify_computation(&node).unwrap();
        assert!(matches!(
            comp,
            Computation::Collection {
                family: EmitCollectionFamily::Fold,
                ..
            }
        ));
    }

    #[test]
    fn classify_collection_dedup_as_sort_family() {
        let node = make_node(
            "dedup_items",
            vec![Port::list("items", "String")],
            vec![Port::list("items", "String")],
            LoweredOp::Collection {
                module: "tools.makegen".into(),
                callable: "dedup".into(),
                kind: daglang_lower::CollectionOpKind::Dedup,
            },
        )
        .with_kind(NodeKind::Collection);
        let comp = classify_computation(&node).unwrap();
        assert!(matches!(
            comp,
            Computation::Collection {
                family: EmitCollectionFamily::Sort,
                ..
            }
        ));
    }

    #[test]
    fn classify_pipeline() {
        let node = make_node(
            "my_pipeline",
            vec![scalar("input", "String")],
            vec![scalar("output", "String")],
            LoweredOp::Pipeline {
                module: "tools.makegen".into(),
                name: "my_pipeline".into(),
                stages: 3,
                stage_names: vec!["a".into(), "b".into(), "c".into()],
            },
        );
        let comp = classify_computation(&node).unwrap();
        assert!(matches!(
            comp,
            Computation::Pure {
                body: PureBody::Aggregate { .. },
                ..
            }
        ));
    }

    #[test]
    fn classify_subdag_returns_error() {
        let inner = gunbc_ir::Dag::new();
        let node = Node::subdag("sub", inner);
        let result = classify_computation(&node);
        assert!(matches!(result, Err(ClassifyError::SubDagNode(_))));
    }

    #[test]
    fn classify_fn_body_empty_dag_is_pure_render() {
        let dag: gunbc_ir::Dag<LoweredOp> = gunbc_ir::Dag::new();
        assert_eq!(classify_fn_body(&dag), FnBodyClassification::PureRender);
    }

    #[test]
    fn classify_fn_body_with_ports_is_pure_compute() {
        let mut dag: gunbc_ir::Dag<LoweredOp> = gunbc_ir::Dag::new();
        dag.add_node(make_node(
            "compute",
            vec![Port::scalar("input", "String")],
            vec![Port::scalar("output", "String")],
            LoweredOp::Callable {
                module: "test".into(),
                kind: CallableKind::Fn,
                name: "transform".into(),
                obligation: CallableObligation::None,
                is_interactive: false,
                resource_target: None,
                fn_body: None,
            },
        ));
        assert_eq!(classify_fn_body(&dag), FnBodyClassification::PureCompute);
    }

    #[test]
    fn classify_fn_body_with_resource_node_is_effectful() {
        let mut dag: gunbc_ir::Dag<LoweredOp> = gunbc_ir::Dag::new();
        dag.add_node(
            make_node(
                "acquire_fs",
                vec![Port::scalar("res:file:path", "String")],
                vec![Port::scalar("output", "String")],
                LoweredOp::Callable {
                    module: "test".into(),
                    kind: CallableKind::Func,
                    name: "read_file".into(),
                    obligation: CallableObligation::None,
                    is_interactive: false,
                    resource_target: None,
                    fn_body: None,
                },
            )
            .with_kind(gunbc_ir::NodeKind::ResourceAcquire),
        );
        assert_eq!(classify_fn_body(&dag), FnBodyClassification::Effectful);
    }

    #[test]
    fn classify_fn_body_with_transport_node_is_effectful() {
        let mut dag: gunbc_ir::Dag<LoweredOp> = gunbc_ir::Dag::new();
        dag.add_node(
            make_node(
                "prepare_transport_github",
                vec![Port::scalar("url", "String")],
                vec![Port::scalar("request", "TransportRequest")],
                LoweredOp::Callable {
                    module: "test".into(),
                    kind: CallableKind::Func,
                    name: "prepare".into(),
                    obligation: CallableObligation::None,
                    is_interactive: false,
                    resource_target: None,
                    fn_body: None,
                },
            )
            .with_kind(gunbc_ir::NodeKind::TransportPrepare),
        );
        assert_eq!(classify_fn_body(&dag), FnBodyClassification::Effectful);
    }

    #[test]
    fn fn_body_classification_needs_mocks() {
        assert!(!FnBodyClassification::PureRender.needs_mocks());
        assert!(!FnBodyClassification::PureCompute.needs_mocks());
        assert!(FnBodyClassification::Effectful.needs_mocks());
    }

    #[test]
    fn fn_body_classification_is_deterministic() {
        assert!(FnBodyClassification::PureRender.is_deterministic());
        assert!(FnBodyClassification::PureCompute.is_deterministic());
        assert!(!FnBodyClassification::Effectful.is_deterministic());
    }
}
