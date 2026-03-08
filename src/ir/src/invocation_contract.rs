//! Shared invocation contract model (M16).
//!
//! This type is the single semantic authority for invocation-shape contracts
//! consumed by both SystemModel behaviors and transport behavior specs.

use serde::{Deserialize, Serialize};

/// Shared invocation contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvocationContract {
    Cli {
        command: String,
        docs: String,
    },
    Rest {
        method: String,
        path: String,
        docs: String,
    },
    Sdk {
        function: String,
        docs: String,
    },
    Protocol {
        protocol: String,
        docs: String,
    },
}

impl InvocationContract {
    pub fn cli(command: impl Into<String>, docs: impl Into<String>) -> Self {
        Self::Cli {
            command: command.into(),
            docs: docs.into(),
        }
    }

    pub fn rest(
        method: impl Into<String>,
        path: impl Into<String>,
        docs: impl Into<String>,
    ) -> Self {
        Self::Rest {
            method: method.into(),
            path: path.into(),
            docs: docs.into(),
        }
    }

    pub fn sdk(function: impl Into<String>, docs: impl Into<String>) -> Self {
        Self::Sdk {
            function: function.into(),
            docs: docs.into(),
        }
    }

    pub fn protocol(protocol: impl Into<String>, docs: impl Into<String>) -> Self {
        Self::Protocol {
            protocol: protocol.into(),
            docs: docs.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::InvocationContract;

    #[test]
    fn builders_construct_expected_variants() {
        assert!(matches!(
            InvocationContract::cli("gh", "docs"),
            InvocationContract::Cli { .. }
        ));
        assert!(matches!(
            InvocationContract::rest("GET", "/path", "docs"),
            InvocationContract::Rest { .. }
        ));
        assert!(matches!(
            InvocationContract::sdk("call", "docs"),
            InvocationContract::Sdk { .. }
        ));
        assert!(matches!(
            InvocationContract::protocol("rest", "docs"),
            InvocationContract::Protocol { .. }
        ));
    }
}
