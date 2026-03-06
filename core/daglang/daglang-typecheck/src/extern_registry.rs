//! Registry for validating `extern func` declarations against known symbols.

use std::collections::HashMap;

/// Signature of an extern function for typecheck-time validation.
#[derive(Debug, Clone)]
pub struct ExternSignature {
    pub module: String,
    pub name: String,
    pub input_count: usize,
    pub output_count: usize,
}

/// Registry of known extern function signatures.
///
/// When present during typechecking, validates that `extern func` declarations
/// reference symbols that actually exist in the app's extern binding table.
#[derive(Debug, Clone, Default)]
pub struct ExternRegistry {
    entries: HashMap<(String, String), ExternSignature>,
}

impl ExternRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an extern signature.
    pub fn register(&mut self, sig: ExternSignature) {
        self.entries
            .insert((sig.module.clone(), sig.name.clone()), sig);
    }

    /// Look up an extern by (module, name).
    pub fn lookup(&self, module: &str, name: &str) -> Option<&ExternSignature> {
        self.entries.get(&(module.to_string(), name.to_string()))
    }

    /// Check if an extern is registered.
    pub fn contains(&self, module: &str, name: &str) -> bool {
        self.entries
            .contains_key(&(module.to_string(), name.to_string()))
    }

    /// Build from a list of (module, name, input_count, output_count) tuples.
    pub fn from_symbols(symbols: &[(String, String, usize, usize)]) -> Self {
        let mut reg = Self::new();
        for (module, name, input_count, output_count) in symbols {
            reg.register(ExternSignature {
                module: module.clone(),
                name: name.clone(),
                input_count: *input_count,
                output_count: *output_count,
            });
        }
        reg
    }
}
