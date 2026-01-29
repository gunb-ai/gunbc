//! Markdown operations.
//!
//! Operations for generating and working with markdown content.
//!
//! # Example
//!
//! ```ignore
//! use gunbc_ops::markdown::render_code_snapshot;
//! use std::collections::BTreeMap;
//!
//! let mut contents = BTreeMap::new();
//! contents.insert("src/lib.rs".to_string(), "fn main() {}".to_string());
//!
//! let markdown = render_code_snapshot(&contents);
//! println!("{}", markdown);
//! ```

use gunbc_exec::{ExecError, Executable};
use gunbc_ir::Value;
use std::collections::{BTreeMap, HashMap};

/// Markdown operations for use in DAG nodes.
#[derive(Debug, Clone)]
pub enum MarkdownOp {
    /// Render files as a markdown code snapshot
    RenderCodeSnapshot,
}

impl Executable for MarkdownOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            MarkdownOp::RenderCodeSnapshot => {
                let contents = inputs
                    .get("contents")
                    .and_then(|v| v.as_map_str_str())
                    .ok_or_else(|| ExecError::new("missing or invalid 'contents' input"))?;

                let markdown = render_code_snapshot(&contents);

                let mut out = HashMap::new();
                out.insert("markdown".to_string(), Value::Str(markdown));
                Ok(out)
            }
        }
    }
}

// ============================================================================
// Standalone helper functions
// ============================================================================

/// Render file contents as a markdown code snapshot.
///
/// Creates a markdown document with each file in a fenced code block,
/// with language detection based on file extension.
///
/// # Example
///
/// ```ignore
/// let mut contents = BTreeMap::new();
/// contents.insert("lib.rs".to_string(), "fn main() {}".to_string());
///
/// let md = render_code_snapshot(&contents);
/// assert!(md.contains("```rust"));
/// ```
pub fn render_code_snapshot(contents: &BTreeMap<String, String>) -> String {
    let mut markdown = String::new();
    markdown.push_str("# Code Snapshot\n\n");

    for (filename, content) in contents {
        let lang = detect_language(filename);

        markdown.push_str(&format!("## `{}`\n\n", filename));
        markdown.push_str(&format!("```{}\n", lang));
        markdown.push_str(content);
        if !content.ends_with('\n') {
            markdown.push('\n');
        }
        markdown.push_str("```\n\n");
    }

    markdown
}

/// Detect programming language from file extension.
///
/// Returns the language identifier for use in markdown fenced code blocks.
pub fn detect_language(filename: &str) -> &'static str {
    if filename.ends_with(".rs") {
        "rust"
    } else if filename.ends_with(".py") {
        "python"
    } else if filename.ends_with(".js") {
        "javascript"
    } else if filename.ends_with(".ts") {
        "typescript"
    } else if filename.ends_with(".go") {
        "go"
    } else if filename.ends_with(".md") {
        "markdown"
    } else if filename.ends_with(".toml") {
        "toml"
    } else if filename.ends_with(".json") {
        "json"
    } else if filename.ends_with(".yaml") || filename.ends_with(".yml") {
        "yaml"
    } else if filename.ends_with(".sh") || filename.ends_with(".bash") {
        "bash"
    } else if filename.ends_with(".c") || filename.ends_with(".h") {
        "c"
    } else if filename.ends_with(".cpp") || filename.ends_with(".hpp") {
        "cpp"
    } else if filename.ends_with(".java") {
        "java"
    } else if filename.ends_with(".rb") {
        "ruby"
    } else if filename.ends_with(".html") {
        "html"
    } else if filename.ends_with(".css") {
        "css"
    } else if filename.ends_with(".sql") {
        "sql"
    } else {
        ""
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_language() {
        assert_eq!(detect_language("foo.rs"), "rust");
        assert_eq!(detect_language("bar.py"), "python");
        assert_eq!(detect_language("baz.ts"), "typescript");
        assert_eq!(detect_language("unknown.xyz"), "");
    }

    #[test]
    fn test_render_code_snapshot() {
        let mut contents = BTreeMap::new();
        contents.insert("test.rs".to_string(), "fn main() {}".to_string());

        let md = render_code_snapshot(&contents);

        assert!(md.contains("# Code Snapshot"));
        assert!(md.contains("## `test.rs`"));
        assert!(md.contains("```rust"));
        assert!(md.contains("fn main() {}"));
    }

    #[test]
    fn test_markdown_op() {
        let mut contents = BTreeMap::new();
        contents.insert("lib.rs".to_string(), "// code".to_string());

        let mut inputs = HashMap::new();
        inputs.insert("contents".to_string(), Value::MapStrStr(contents));

        let op = MarkdownOp::RenderCodeSnapshot;
        let result = op.execute(inputs).unwrap();

        match result.get("markdown") {
            Some(Value::Str(md)) => {
                assert!(md.contains("lib.rs"));
            }
            _ => panic!("expected markdown"),
        }
    }
}
