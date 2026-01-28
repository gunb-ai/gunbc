use std::fmt;

/// Unique identifier for a node within a DAG.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeId(pub String);

/// Unique identifier for a port on a node.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PortName(pub String);

/// Type identifier for port type checking.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeId(pub String);

/// A secret value that redacts its contents in Debug output and has no Display.
#[derive(Clone, PartialEq, Eq)]
pub struct Secret<T>(pub T);

impl<T> fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<REDACTED>")
    }
}

impl<T> Secret<T> {
    pub fn into_inner(self) -> T {
        self.0
    }

    pub fn as_inner(&self) -> &T {
        &self.0
    }
}

/// Whether a pattern-based node was instantiated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternDecision {
    Instantiated,
    NotApplicable { reason: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_redaction() {
        let secret = Secret("my_password".to_string());
        let debug_output = format!("{:?}", secret);
        assert_eq!(debug_output, "<REDACTED>");
        assert!(!debug_output.contains("my_password"));
        assert_eq!(secret.as_inner(), "my_password");
    }
}
