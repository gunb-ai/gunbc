//! Simple string template rendering.
//!
//! No external dependencies — just basic variable substitution.

use std::collections::HashMap;

/// A simple template with variable substitution.
///
/// Variables are marked with `{{name}}` syntax.
#[derive(Debug, Clone)]
pub struct Template {
    content: String,
}

impl Template {
    /// Create a new template from a string.
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
        }
    }

    /// Render the template with the given variables.
    ///
    /// Variables in the template are replaced with their values.
    /// Unknown variables are left as-is.
    pub fn render(&self, vars: &HashMap<String, String>) -> String {
        let mut result = self.content.clone();
        for (key, value) in vars {
            let pattern = format!("{{{{{}}}}}", key);
            result = result.replace(&pattern, value);
        }
        result
    }

    /// Render with a builder pattern.
    pub fn render_with(&self) -> TemplateRenderer<'_> {
        TemplateRenderer {
            template: self,
            vars: HashMap::new(),
        }
    }
}

/// Builder for rendering templates.
pub struct TemplateRenderer<'a> {
    template: &'a Template,
    vars: HashMap<String, String>,
}

impl<'a> TemplateRenderer<'a> {
    /// Set a variable value.
    pub fn var(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.vars.insert(name.into(), value.into());
        self
    }

    /// Render the template.
    pub fn finish(self) -> String {
        self.template.render(&self.vars)
    }
}

/// Convenience macro for building variable maps.
#[macro_export]
macro_rules! vars {
    ($($key:expr => $value:expr),* $(,)?) => {{
        let mut map = std::collections::HashMap::new();
        $(
            map.insert($key.to_string(), $value.to_string());
        )*
        map
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_substitution() {
        let template = Template::new("Hello, {{name}}!");
        let result = template.render_with().var("name", "World").finish();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_multiple_vars() {
        let template = Template::new("{{greeting}}, {{name}}!");
        let result = template
            .render_with()
            .var("greeting", "Hi")
            .var("name", "Alice")
            .finish();
        assert_eq!(result, "Hi, Alice!");
    }

    #[test]
    fn test_unknown_var_unchanged() {
        let template = Template::new("Hello, {{name}}! {{unknown}}");
        let result = template.render_with().var("name", "World").finish();
        assert_eq!(result, "Hello, World! {{unknown}}");
    }

    #[test]
    fn test_vars_macro() {
        let template = Template::new("{{a}} + {{b}}");
        let vars = vars! {
            "a" => "1",
            "b" => "2",
        };
        let result = template.render(&vars);
        assert_eq!(result, "1 + 2");
    }

    #[test]
    fn test_multiline_template() {
        let template = Template::new(
            r#"# {{title}}

Author: {{author}}
Date: {{date}}
"#,
        );
        let result = template
            .render_with()
            .var("title", "My Document")
            .var("author", "Alice")
            .var("date", "2024-01-01")
            .finish();

        assert!(result.contains("# My Document"));
        assert!(result.contains("Author: Alice"));
        assert!(result.contains("Date: 2024-01-01"));
    }
}
