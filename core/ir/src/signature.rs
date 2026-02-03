//! Workflow signature: declared vs inferred interface validation.
//!
//! A workflow signature explicitly declares the inputs and outputs of a DAG.
//! This prevents silent interface drift where forgotten edges become accidental
//! public API.
//!
//! # Example
//!
//! ```rust,ignore
//! use gunbc_ir::{Dag, WorkflowSignature, SignaturePort};
//! use gunbc_ir::types::Cardinality;
//!
//! let signature = WorkflowSignature::new()
//!     .with_input("files", "PathList", Cardinality::ONE_OR_MORE)
//!     .with_output("result", "String", Cardinality::ONE);
//!
//! // Validate that the DAG matches the declared signature
//! signature.validate(&dag)?;
//! ```

use crate::boundary::detect_boundaries;
use crate::dag::Dag;
use crate::entrypoint::detect_entrypoints;
use crate::types::{Cardinality, PortName, TypeId};
use std::collections::HashSet;
use std::fmt;

/// A port in a workflow signature (input or output).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignaturePort {
    pub name: PortName,
    pub type_id: TypeId,
    pub cardinality: Cardinality,
}

impl SignaturePort {
    pub fn new(
        name: impl Into<PortName>,
        type_id: impl Into<TypeId>,
        cardinality: Cardinality,
    ) -> Self {
        Self {
            name: name.into(),
            type_id: type_id.into(),
            cardinality,
        }
    }

    /// Create a scalar (required) port.
    pub fn scalar(name: impl Into<PortName>, type_id: impl Into<TypeId>) -> Self {
        Self::new(name, type_id, Cardinality::ONE)
    }

    /// Create an optional port.
    pub fn optional(name: impl Into<PortName>, type_id: impl Into<TypeId>) -> Self {
        Self::new(name, type_id, Cardinality::ZERO_OR_ONE)
    }

    /// Create a list port.
    pub fn list(name: impl Into<PortName>, type_id: impl Into<TypeId>) -> Self {
        Self::new(name, type_id, Cardinality::ZERO_OR_MORE)
    }

    /// Create a non-empty list port.
    pub fn non_empty_list(name: impl Into<PortName>, type_id: impl Into<TypeId>) -> Self {
        Self::new(name, type_id, Cardinality::ONE_OR_MORE)
    }
}

/// A workflow's declared interface.
///
/// The signature explicitly declares what inputs the workflow expects and
/// what outputs it produces. This is validated against the inferred signature
/// (computed from unconnected ports) to catch interface drift.
#[derive(Debug, Clone, Default)]
pub struct WorkflowSignature {
    pub inputs: Vec<SignaturePort>,
    pub outputs: Vec<SignaturePort>,
}

impl WorkflowSignature {
    /// Create a new empty signature.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an input port to the signature.
    pub fn with_input(
        mut self,
        name: impl Into<PortName>,
        type_id: impl Into<TypeId>,
        cardinality: Cardinality,
    ) -> Self {
        self.inputs.push(SignaturePort::new(name, type_id, cardinality));
        self
    }

    /// Add an output port to the signature.
    pub fn with_output(
        mut self,
        name: impl Into<PortName>,
        type_id: impl Into<TypeId>,
        cardinality: Cardinality,
    ) -> Self {
        self.outputs.push(SignaturePort::new(name, type_id, cardinality));
        self
    }

    /// Add multiple input ports.
    pub fn with_inputs(mut self, inputs: impl IntoIterator<Item = SignaturePort>) -> Self {
        self.inputs.extend(inputs);
        self
    }

    /// Add multiple output ports.
    pub fn with_outputs(mut self, outputs: impl IntoIterator<Item = SignaturePort>) -> Self {
        self.outputs.extend(outputs);
        self
    }

    /// Validate that this signature matches the DAG's inferred signature.
    ///
    /// Returns `Ok(())` if the declared signature matches the inferred signature.
    /// Returns `Err(SignatureError)` with details about the mismatch.
    pub fn validate<T>(&self, dag: &Dag<T>) -> Result<(), SignatureError> {
        let inferred = infer_signature(dag);
        
        // Check inputs
        let declared_inputs: HashSet<_> = self.inputs.iter()
            .map(|p| (&p.name, &p.type_id, p.cardinality))
            .collect();
        let inferred_inputs: HashSet<_> = inferred.inputs.iter()
            .map(|p| (&p.name, &p.type_id, p.cardinality))
            .collect();

        let missing_inputs: Vec<_> = inferred_inputs
            .difference(&declared_inputs)
            .map(|(name, type_id, card)| SignaturePort::new((*name).clone(), (*type_id).clone(), *card))
            .collect();

        let extra_inputs: Vec<_> = declared_inputs
            .difference(&inferred_inputs)
            .map(|(name, type_id, card)| SignaturePort::new((*name).clone(), (*type_id).clone(), *card))
            .collect();

        // Check outputs
        let declared_outputs: HashSet<_> = self.outputs.iter()
            .map(|p| (&p.name, &p.type_id, p.cardinality))
            .collect();
        let inferred_outputs: HashSet<_> = inferred.outputs.iter()
            .map(|p| (&p.name, &p.type_id, p.cardinality))
            .collect();

        let missing_outputs: Vec<_> = inferred_outputs
            .difference(&declared_outputs)
            .map(|(name, type_id, card)| SignaturePort::new((*name).clone(), (*type_id).clone(), *card))
            .collect();

        let extra_outputs: Vec<_> = declared_outputs
            .difference(&inferred_outputs)
            .map(|(name, type_id, card)| SignaturePort::new((*name).clone(), (*type_id).clone(), *card))
            .collect();

        if missing_inputs.is_empty() && extra_inputs.is_empty() 
            && missing_outputs.is_empty() && extra_outputs.is_empty() 
        {
            Ok(())
        } else {
            Err(SignatureError {
                missing_inputs,
                extra_inputs,
                missing_outputs,
                extra_outputs,
            })
        }
    }
}

/// Infer a signature from a DAG by detecting unconnected ports.
///
/// - Inputs: entrypoint ports (input ports with no upstream edge)
/// - Outputs: boundary ports (output ports with no downstream edge)
pub fn infer_signature<T>(dag: &Dag<T>) -> WorkflowSignature {
    let entrypoints = detect_entrypoints(dag);
    let boundaries = detect_boundaries(dag);

    let mut inputs = Vec::new();
    let mut outputs = Vec::new();

    // Collect entrypoint ports as inputs
    // entrypoint_ports is Vec<(NodeId, PortName, TypeId)>
    // Exclude tool ports (tool:*) - these are framework-provided, not user inputs
    for (node_id, port_name, _type_id) in &entrypoints.entrypoint_ports {
        // Skip tool capability ports - they're provided by the framework, not users
        if port_name.0.starts_with("tool:") {
            continue;
        }
        
        if let Some(node) = dag.get_node(node_id) {
            if let Some(port) = node.inputs.iter().find(|p| &p.name == port_name) {
                inputs.push(SignaturePort::new(
                    port.name.clone(),
                    port.type_id.clone(),
                    port.cardinality,
                ));
            }
        }
    }

    // Collect boundary ports as outputs
    // boundary_ports is Vec<(NodeId, PortName)>
    for (node_id, port_name) in &boundaries.boundary_ports {
        if let Some(node) = dag.get_node(node_id) {
            if let Some(port) = node.outputs.iter().find(|p| &p.name == port_name) {
                outputs.push(SignaturePort::new(
                    port.name.clone(),
                    port.type_id.clone(),
                    port.cardinality,
                ));
            }
        }
    }

    WorkflowSignature { inputs, outputs }
}

/// Error when declared signature doesn't match inferred signature.
#[derive(Debug, Clone)]
pub struct SignatureError {
    /// Inputs that exist in DAG but weren't declared
    pub missing_inputs: Vec<SignaturePort>,
    /// Inputs that were declared but don't exist in DAG
    pub extra_inputs: Vec<SignaturePort>,
    /// Outputs that exist in DAG but weren't declared
    pub missing_outputs: Vec<SignaturePort>,
    /// Outputs that were declared but don't exist in DAG
    pub extra_outputs: Vec<SignaturePort>,
}

impl fmt::Display for SignatureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Signature mismatch:")?;
        
        if !self.missing_inputs.is_empty() {
            writeln!(f, "  Missing input declarations (found in DAG but not declared):")?;
            for port in &self.missing_inputs {
                writeln!(f, "    - {} ({}, {:?})", port.name, port.type_id, port.cardinality)?;
            }
        }
        
        if !self.extra_inputs.is_empty() {
            writeln!(f, "  Extra input declarations (declared but not in DAG):")?;
            for port in &self.extra_inputs {
                writeln!(f, "    - {} ({}, {:?})", port.name, port.type_id, port.cardinality)?;
            }
        }
        
        if !self.missing_outputs.is_empty() {
            writeln!(f, "  Missing output declarations (found in DAG but not declared):")?;
            for port in &self.missing_outputs {
                writeln!(f, "    - {} ({}, {:?})", port.name, port.type_id, port.cardinality)?;
            }
        }
        
        if !self.extra_outputs.is_empty() {
            writeln!(f, "  Extra output declarations (declared but not in DAG):")?;
            for port in &self.extra_outputs {
                writeln!(f, "    - {} ({}, {:?})", port.name, port.type_id, port.cardinality)?;
            }
        }
        
        Ok(())
    }
}

impl std::error::Error for SignatureError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dag::Port;
    use crate::node::Node;

    fn test_node(id: &str, inputs: Vec<(&str, &str)>, outputs: Vec<(&str, &str)>) -> Node<String> {
        Node::opaque(
            id,
            inputs.into_iter().map(|(name, ty)| Port::new(name, ty)).collect(),
            outputs.into_iter().map(|(name, ty)| Port::new(name, ty)).collect(),
            format!("op_{}", id),
        )
    }

    #[test]
    fn test_infer_signature_simple() {
        let mut dag: Dag<String> = Dag::new();
        
        // A -> B -> C
        // A has entrypoint input, C has boundary output
        dag.add_node(test_node("a", vec![("in", "String")], vec![("out", "String")]));
        dag.add_node(test_node("b", vec![("in", "String")], vec![("out", "String")]));
        dag.add_node(test_node("c", vec![("in", "String")], vec![("result", "String")]));
        
        dag.add_edge(crate::dag::Edge::new("a", "out", "b", "in"));
        dag.add_edge(crate::dag::Edge::new("b", "out", "c", "in"));
        
        let sig = infer_signature(&dag);
        
        assert_eq!(sig.inputs.len(), 1);
        assert_eq!(sig.inputs[0].name.0, "in");
        assert_eq!(sig.inputs[0].type_id.0, "String");
        
        assert_eq!(sig.outputs.len(), 1);
        assert_eq!(sig.outputs[0].name.0, "result");
        assert_eq!(sig.outputs[0].type_id.0, "String");
    }

    #[test]
    fn test_validate_signature_match() {
        let mut dag: Dag<String> = Dag::new();
        dag.add_node(test_node("a", vec![("in", "String")], vec![("out", "String")]));
        
        let sig = WorkflowSignature::new()
            .with_input("in", "String", Cardinality::ONE)
            .with_output("out", "String", Cardinality::ONE);
        
        assert!(sig.validate(&dag).is_ok());
    }

    #[test]
    fn test_validate_signature_missing_input() {
        let mut dag: Dag<String> = Dag::new();
        dag.add_node(test_node("a", vec![("in", "String")], vec![("out", "String")]));
        
        // Signature missing the input declaration
        let sig = WorkflowSignature::new()
            .with_output("out", "String", Cardinality::ONE);
        
        let err = sig.validate(&dag).unwrap_err();
        assert_eq!(err.missing_inputs.len(), 1);
        assert_eq!(err.missing_inputs[0].name.0, "in");
    }

    #[test]
    fn test_validate_signature_extra_input() {
        let mut dag: Dag<String> = Dag::new();
        dag.add_node(test_node("a", vec![], vec![("out", "String")]));  // No input port
        
        // Signature declares an input that doesn't exist
        let sig = WorkflowSignature::new()
            .with_input("phantom", "String", Cardinality::ONE)
            .with_output("out", "String", Cardinality::ONE);
        
        let err = sig.validate(&dag).unwrap_err();
        assert_eq!(err.extra_inputs.len(), 1);
        assert_eq!(err.extra_inputs[0].name.0, "phantom");
    }

    #[test]
    fn test_validate_signature_missing_output() {
        let mut dag: Dag<String> = Dag::new();
        dag.add_node(test_node("a", vec![("in", "String")], vec![("out", "String")]));
        
        // Signature missing the output declaration
        let sig = WorkflowSignature::new()
            .with_input("in", "String", Cardinality::ONE);
        
        let err = sig.validate(&dag).unwrap_err();
        assert_eq!(err.missing_outputs.len(), 1);
        assert_eq!(err.missing_outputs[0].name.0, "out");
    }

    #[test]
    fn test_validate_signature_cardinality_mismatch() {
        let mut dag: Dag<String> = Dag::new();
        let node = Node::opaque(
            "a",
            vec![Port::scalar("in", "String")],  // One
            vec![Port::optional("out", "String")],  // ZeroOrOne
            "op_a".to_string(),
        );
        dag.add_node(node);
        
        // Signature declares wrong cardinality
        let sig = WorkflowSignature::new()
            .with_input("in", "String", Cardinality::ONE)
            .with_output("out", "String", Cardinality::ONE);  // Wrong! Should be ZeroOrOne
        
        let err = sig.validate(&dag).unwrap_err();
        // The ZeroOrOne output is "missing" (we declared One instead)
        assert_eq!(err.missing_outputs.len(), 1);
        assert_eq!(err.missing_outputs[0].cardinality, Cardinality::ZERO_OR_ONE);
        // The One output is "extra" (we declared it but it doesn't exist)
        assert_eq!(err.extra_outputs.len(), 1);
        assert_eq!(err.extra_outputs[0].cardinality, Cardinality::ONE);
    }

    #[test]
    fn test_validate_signature_type_mismatch() {
        let mut dag: Dag<String> = Dag::new();
        dag.add_node(test_node("a", vec![("in", "String")], vec![("out", "Int")]));
        
        // Signature declares wrong type
        let sig = WorkflowSignature::new()
            .with_input("in", "String", Cardinality::ONE)
            .with_output("out", "String", Cardinality::ONE);  // Wrong! Should be Int
        
        let err = sig.validate(&dag).unwrap_err();
        assert_eq!(err.missing_outputs.len(), 1);
        assert_eq!(err.missing_outputs[0].type_id.0, "Int");
        assert_eq!(err.extra_outputs.len(), 1);
        assert_eq!(err.extra_outputs[0].type_id.0, "String");
    }

    #[test]
    fn test_signature_error_display() {
        let err = SignatureError {
            missing_inputs: vec![SignaturePort::scalar("missing_in", "String")],
            extra_inputs: vec![],
            missing_outputs: vec![],
            extra_outputs: vec![SignaturePort::scalar("extra_out", "Int")],
        };
        
        let msg = format!("{}", err);
        assert!(msg.contains("missing_in"));
        assert!(msg.contains("extra_out"));
    }

    #[test]
    fn test_signature_builder_fluent() {
        let sig = WorkflowSignature::new()
            .with_input("a", "String", Cardinality::ONE)
            .with_input("b", "Int", Cardinality::ZERO_OR_ONE)
            .with_output("result", "Bool", Cardinality::ONE);
        
        assert_eq!(sig.inputs.len(), 2);
        assert_eq!(sig.outputs.len(), 1);
    }

    #[test]
    fn test_signature_port_helpers() {
        let scalar = SignaturePort::scalar("s", "String");
        assert_eq!(scalar.cardinality, Cardinality::ONE);
        
        let optional = SignaturePort::optional("o", "String");
        assert_eq!(optional.cardinality, Cardinality::ZERO_OR_ONE);
        
        let list = SignaturePort::list("l", "String");
        assert_eq!(list.cardinality, Cardinality::ZERO_OR_MORE);
        
        let non_empty = SignaturePort::non_empty_list("n", "String");
        assert_eq!(non_empty.cardinality, Cardinality::ONE_OR_MORE);
    }
}
