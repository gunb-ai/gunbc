//! Value simulators for property-based testing.
//!
//! Simulators provide two capabilities:
//! 1. **Generate**: Produce random values satisfying constraints
//! 2. **Validate**: Check if a value falls within expected range
//!
//! # Use Cases
//!
//! ## Input Simulation
//! Generate constrained random inputs to test node behavior:
//! ```ignore
//! let sim = Simulator::non_empty_string();
//! for _ in 0..100 {
//!     let input = sim.generate();
//!     let output = node.execute(input);
//!     assert!(output_validator.validate(&output).is_ok());
//! }
//! ```
//!
//! ## Output Range Validation
//! Verify outputs fall within expected bounds:
//! ```ignore
//! let sim = Simulator::exit_code(); // 0-255
//! assert!(sim.validate(&Value::Int(0)).is_ok());
//! assert!(sim.validate(&Value::Int(256)).is_err());
//! ```
//!
//! ## I↔O Contract Testing
//! Put simulator on input, verify output matches expected range:
//! ```ignore
//! let input_sim = Simulator::shell_command();
//! let output_sim = Simulator::shell_response();
//!
//! for input in input_sim.generate_many(100) {
//!     let output = execute_node(&input);
//!     assert!(output_sim.validate(&output).is_ok());
//! }
//! ```

use gunbc_ir::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// A simulator that can generate and validate values.
#[derive(Clone)]
#[allow(clippy::type_complexity)]
pub struct Simulator {
    /// Human-readable description
    pub description: String,
    /// Generator function (returns random value satisfying constraints)
    generator: Option<Arc<dyn Fn() -> Value + Send + Sync>>,
    /// Validator function (checks if value is in expected range)
    validator: Option<Arc<dyn Fn(&Value) -> Result<(), String> + Send + Sync>>,
}

impl std::fmt::Debug for Simulator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Simulator")
            .field("description", &self.description)
            .field("can_generate", &self.generator.is_some())
            .field("can_validate", &self.validator.is_some())
            .finish()
    }
}

impl Simulator {
    /// Create a new simulator with description.
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            generator: None,
            validator: None,
        }
    }

    /// Add a generator function.
    pub fn with_generator<F>(mut self, f: F) -> Self
    where
        F: Fn() -> Value + Send + Sync + 'static,
    {
        self.generator = Some(Arc::new(f));
        self
    }

    /// Add a validator function.
    pub fn with_validator<F>(mut self, f: F) -> Self
    where
        F: Fn(&Value) -> Result<(), String> + Send + Sync + 'static,
    {
        self.validator = Some(Arc::new(f));
        self
    }

    /// Generate a random value (panics if no generator).
    pub fn generate(&self) -> Value {
        self.generator
            .as_ref()
            .expect("Simulator has no generator")()
    }

    /// Generate multiple random values.
    pub fn generate_many(&self, count: usize) -> Vec<Value> {
        (0..count).map(|_| self.generate()).collect()
    }

    /// Validate a value against expected range.
    pub fn validate(&self, value: &Value) -> Result<(), String> {
        match &self.validator {
            Some(f) => f(value),
            None => Ok(()), // No validator = accept anything
        }
    }

    /// Check if this simulator can generate values.
    pub fn can_generate(&self) -> bool {
        self.generator.is_some()
    }

    /// Check if this simulator can validate values.
    pub fn can_validate(&self) -> bool {
        self.validator.is_some()
    }

    // =========================================================================
    // Built-in Simulators
    // =========================================================================

    /// Non-empty string simulator.
    pub fn non_empty_string() -> Self {
        Self::new("non-empty string")
            .with_generator(|| {
                use std::time::{SystemTime, UNIX_EPOCH};
                let seed = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos();
                Value::Str(format!("generated_{}", seed % 10000))
            })
            .with_validator(|v| match v {
                Value::Str(s) if !s.is_empty() => Ok(()),
                Value::Str(_) => Err("string is empty".into()),
                _ => Err(format!("expected string, got {:?}", v)),
            })
    }

    /// Boolean simulator.
    pub fn boolean() -> Self {
        Self::new("boolean")
            .with_generator(|| {
                use std::time::{SystemTime, UNIX_EPOCH};
                let seed = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos();
                Value::Bool(seed.is_multiple_of(2))
            })
            .with_validator(|v| match v {
                Value::Bool(_) => Ok(()),
                _ => Err(format!("expected bool, got {:?}", v)),
            })
    }

    /// Exit code simulator (0-255).
    pub fn exit_code() -> Self {
        Self::new("exit code (0-255)")
            .with_generator(|| {
                use std::time::{SystemTime, UNIX_EPOCH};
                let seed = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos();
                Value::Int((seed % 256) as i64)
            })
            .with_validator(|v| match v {
                Value::Int(i) if *i >= 0 && *i <= 255 => Ok(()),
                Value::Int(i) => Err(format!("exit code {} out of range 0-255", i)),
                _ => Err(format!("expected int, got {:?}", v)),
            })
    }

    /// Success exit code (0).
    pub fn success_exit_code() -> Self {
        Self::new("success exit code (0)")
            .with_generator(|| Value::Int(0))
            .with_validator(|v| match v {
                Value::Int(0) => Ok(()),
                Value::Int(i) => Err(format!("expected 0, got {}", i)),
                _ => Err(format!("expected int, got {:?}", v)),
            })
    }

    /// Failure exit code (non-zero).
    pub fn failure_exit_code() -> Self {
        Self::new("failure exit code (non-zero)")
            .with_generator(|| {
                use std::time::{SystemTime, UNIX_EPOCH};
                let seed = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos();
                Value::Int(1 + (seed % 255) as i64)
            })
            .with_validator(|v| match v {
                Value::Int(i) if *i != 0 => Ok(()),
                Value::Int(0) => Err("expected non-zero exit code".into()),
                _ => Err(format!("expected int, got {:?}", v)),
            })
    }

    /// Integer in range [min, max].
    pub fn int_range(min: i64, max: i64) -> Self {
        Self::new(format!("int in [{}, {}]", min, max))
            .with_generator(move || {
                use std::time::{SystemTime, UNIX_EPOCH};
                let seed = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos() as i64;
                let range = max - min + 1;
                Value::Int(min + (seed.abs() % range))
            })
            .with_validator(move |v| match v {
                Value::Int(i) if *i >= min && *i <= max => Ok(()),
                Value::Int(i) => Err(format!("{} not in range [{}, {}]", i, min, max)),
                _ => Err(format!("expected int, got {:?}", v)),
            })
    }

    /// JSON object simulator.
    pub fn json_object() -> Self {
        Self::new("JSON object")
            .with_generator(|| {
                let mut map = serde_json::Map::new();
                map.insert("key".into(), serde_json::Value::String("value".into()));
                Value::Json(serde_json::Value::Object(map))
            })
            .with_validator(|v| match v {
                Value::Json(j) if j.is_object() => Ok(()),
                Value::Json(_) => Err("expected JSON object".into()),
                _ => Err(format!("expected JSON, got {:?}", v)),
            })
    }

    /// One of specific values.
    pub fn one_of(values: Vec<Value>) -> Self {
        let values_clone = values.clone();
        Self::new(format!("one of {} values", values.len()))
            .with_generator(move || {
                use std::time::{SystemTime, UNIX_EPOCH};
                let seed = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos() as usize;
                values[seed % values.len()].clone()
            })
            .with_validator(move |v| {
                if values_clone.contains(v) {
                    Ok(())
                } else {
                    Err(format!("value {:?} not in allowed set", v))
                }
            })
    }

    /// Any value (no constraints).
    pub fn any() -> Self {
        Self::new("any value")
            .with_generator(|| Value::Str("any".into()))
            .with_validator(|_| Ok(()))
    }
}

/// Contract between input and output simulators.
///
/// Defines: "Given inputs from input_sim, outputs should satisfy output_sim"
#[derive(Debug)]
pub struct IoContract {
    /// Name of this contract
    pub name: String,
    /// Input simulator (generates test inputs)
    pub input: HashMap<String, Simulator>,
    /// Output validator (checks outputs are in range)
    pub output: HashMap<String, Simulator>,
}

impl IoContract {
    /// Create a new I/O contract.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            input: HashMap::new(),
            output: HashMap::new(),
        }
    }

    /// Add an input simulator.
    pub fn input(mut self, port: impl Into<String>, sim: Simulator) -> Self {
        self.input.insert(port.into(), sim);
        self
    }

    /// Add an output validator.
    pub fn output(mut self, port: impl Into<String>, sim: Simulator) -> Self {
        self.output.insert(port.into(), sim);
        self
    }

    /// Generate a set of test inputs.
    pub fn generate_inputs(&self) -> HashMap<String, Value> {
        self.input
            .iter()
            .map(|(k, sim)| (k.clone(), sim.generate()))
            .collect()
    }

    /// Validate outputs against expected ranges.
    pub fn validate_outputs(&self, outputs: &HashMap<String, Value>) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        for (port, sim) in &self.output {
            if let Some(value) = outputs.get(port) {
                if let Err(e) = sim.validate(value) {
                    errors.push(format!("{}: {}", port, e));
                }
            } else {
                errors.push(format!("{}: missing output", port));
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_non_empty_string_generator() {
        let sim = Simulator::non_empty_string();
        let value = sim.generate();
        assert!(matches!(value, Value::Str(s) if !s.is_empty()));
    }

    #[test]
    fn test_non_empty_string_validator() {
        let sim = Simulator::non_empty_string();
        assert!(sim.validate(&Value::Str("hello".into())).is_ok());
        assert!(sim.validate(&Value::Str("".into())).is_err());
        assert!(sim.validate(&Value::Int(42)).is_err());
    }

    #[test]
    fn test_exit_code_range() {
        let sim = Simulator::exit_code();
        assert!(sim.validate(&Value::Int(0)).is_ok());
        assert!(sim.validate(&Value::Int(255)).is_ok());
        assert!(sim.validate(&Value::Int(256)).is_err());
        assert!(sim.validate(&Value::Int(-1)).is_err());
    }

    #[test]
    fn test_int_range() {
        let sim = Simulator::int_range(10, 20);
        assert!(sim.validate(&Value::Int(10)).is_ok());
        assert!(sim.validate(&Value::Int(15)).is_ok());
        assert!(sim.validate(&Value::Int(20)).is_ok());
        assert!(sim.validate(&Value::Int(9)).is_err());
        assert!(sim.validate(&Value::Int(21)).is_err());
    }

    #[test]
    fn test_one_of() {
        let sim = Simulator::one_of(vec![
            Value::Str("a".into()),
            Value::Str("b".into()),
            Value::Str("c".into()),
        ]);
        assert!(sim.validate(&Value::Str("a".into())).is_ok());
        assert!(sim.validate(&Value::Str("b".into())).is_ok());
        assert!(sim.validate(&Value::Str("d".into())).is_err());
    }

    #[test]
    fn test_io_contract() {
        let contract = IoContract::new("parse_exit_code")
            .input("exit_code", Simulator::exit_code())
            .output("success", Simulator::boolean());

        // Generate inputs
        let inputs = contract.generate_inputs();
        assert!(inputs.contains_key("exit_code"));

        // Validate good outputs
        let mut outputs = HashMap::new();
        outputs.insert("success".into(), Value::Bool(true));
        assert!(contract.validate_outputs(&outputs).is_ok());

        // Validate bad outputs
        outputs.insert("success".into(), Value::Int(42));
        assert!(contract.validate_outputs(&outputs).is_err());
    }

    #[test]
    fn test_generate_many() {
        let sim = Simulator::boolean();
        let values = sim.generate_many(10);
        assert_eq!(values.len(), 10);
        for v in values {
            assert!(matches!(v, Value::Bool(_)));
        }
    }
}
