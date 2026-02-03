//! VariableSyntax SubDag: Template variable expansion.
//!
//! Handles Makefile-style variable expansion: $(VAR), ${VAR}, $@, $<, etc.
//!
//! # I/O Contract
//!
//! Inputs:
//! - `template`: String - Template with variables
//! - `variables`: MapStrStr - Variable name → value mapping
//!
//! Outputs:
//! - `expanded`: String - Expanded template

use crate::dag::{Dag, Port};
use crate::node::Node;
use crate::language::LanguageOp;
use std::collections::HashMap;

/// Variable syntax styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableStyle {
    /// Makefile style: $(VAR), ${VAR}, $@, $<, $^
    Make,
    /// Shell style: $VAR, ${VAR}
    Shell,
    /// Mustache style: {{VAR}}
    Mustache,
}

/// Build the VariableSyntax SubDag node.
///
/// This SubDag provides template expansion for various variable syntaxes.
///
/// # Supported Syntaxes
///
/// - `$(VAR)` - Makefile parentheses syntax
/// - `${VAR}` - Makefile/shell brace syntax
/// - `$@` - Makefile automatic variable (target)
/// - `$<` - Makefile automatic variable (first prerequisite)
/// - `$^` - Makefile automatic variable (all prerequisites)
///
/// # Example
///
/// ```ignore
/// let var_node = build_variable_syntax_subdag();
/// // Execute with template = "$(CC) -o $@ $<", variables = {"CC": "gcc"}
/// // → expanded = "gcc -o $@ $<" (automatic vars need context)
/// ```
pub fn build_variable_syntax_subdag() -> Node<LanguageOp> {
    let mut inner = Dag::new();

    // Expand node: expands variables in template
    inner.add_node(Node::opaque(
        "expand",
        vec![
            Port::scalar("template", "String"),
            Port::scalar("variables", "MapStrStr"),
        ],
        vec![Port::scalar("expanded", "String")],
        LanguageOp::ExpandVariables,
    ));

    // Create the SubDag node with interface
    Node::subdag(
        "variable_syntax",
        inner,
    )
}

/// Expand variables in a template string.
///
/// Supports both $(VAR) and ${VAR} syntax.
pub fn expand_variables(template: &str, variables: &HashMap<String, String>) -> String {
    let mut result = String::new();
    let mut chars = template.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '$' {
            match chars.peek() {
                Some('(') => {
                    chars.next(); // consume '('
                    let var_name: String = chars.by_ref().take_while(|&c| c != ')').collect();
                    if let Some(value) = variables.get(&var_name) {
                        result.push_str(value);
                    } else {
                        // Keep original if variable not found
                        result.push_str(&format!("$({})", var_name));
                    }
                }
                Some('{') => {
                    chars.next(); // consume '{'
                    let var_name: String = chars.by_ref().take_while(|&c| c != '}').collect();
                    if let Some(value) = variables.get(&var_name) {
                        result.push_str(value);
                    } else {
                        // Keep original if variable not found
                        result.push_str(&format!("${{{}}}", var_name));
                    }
                }
                Some(&c2) if c2 == '@' || c2 == '<' || c2 == '^' || c2 == '*' => {
                    // Makefile automatic variables - pass through
                    chars.next();
                    result.push('$');
                    result.push(c2);
                }
                _ => {
                    result.push('$');
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    #![allow(unused_imports)]
    use super::*;
    use crate::node::NodeBody;

    #[test]
    fn test_variable_syntax_subdag_is_subdag() {
        let node = build_variable_syntax_subdag();
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "variable_syntax");
    }

    #[test]
    fn test_variable_syntax_subdag_interface() {
        let node = build_variable_syntax_subdag();

        // Check inputs
        assert_eq!(node.inputs.len(), 2);
        assert_eq!(node.inputs[0].name.0, "template");
        assert_eq!(node.inputs[1].name.0, "variables");

        // Check outputs
        assert_eq!(node.outputs.len(), 1);
        assert_eq!(node.outputs[0].name.0, "expanded");
    }

    #[test]
    fn test_expand_variables_parens() {
        let mut vars = HashMap::new();
        vars.insert("CC".to_string(), "gcc".to_string());
        vars.insert("CFLAGS".to_string(), "-Wall".to_string());

        assert_eq!(
            expand_variables("$(CC) $(CFLAGS) -o main", &vars),
            "gcc -Wall -o main"
        );
    }

    #[test]
    fn test_expand_variables_braces() {
        let mut vars = HashMap::new();
        vars.insert("NAME".to_string(), "test".to_string());

        assert_eq!(
            expand_variables("${NAME}.txt", &vars),
            "test.txt"
        );
    }

    #[test]
    fn test_expand_variables_automatic() {
        let vars = HashMap::new();

        // Automatic variables should pass through
        assert_eq!(expand_variables("$@ $< $^", &vars), "$@ $< $^");
    }

    #[test]
    fn test_expand_variables_missing() {
        let vars = HashMap::new();

        // Missing variables should keep original syntax
        assert_eq!(expand_variables("$(MISSING)", &vars), "$(MISSING)");
    }
}
