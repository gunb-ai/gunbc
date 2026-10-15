//! Seed mirror of `v2.compiler.source_authority` `module_file` over
//! `gunbc.source_layout`: module `a.b.c` is stored at `<root(a)>/b/c.dag`.
//!
//! This is the name-to-path lookup `gunbc run` uses to acquire an entry's closure
//! without walking the governed trees. The policy rows live in
//! `gunbc.source_layout`; this file does not invent a second layout.

#![allow(dead_code)]

/// Relative path of the file that stores `module`, or a refusal if the name is
/// not a dotted identifier sequence (or is the elided `v2` namespace alone).
pub(crate) fn module_storage_relative(module: &str) -> Result<String, String> {
    if module.is_empty() {
        return Err(
            "a module name is a nonempty dotted sequence of identifier segments".to_string(),
        );
    }
    let segments: Vec<&str> = module.split('.').collect();
    if segments
        .iter()
        .any(|s| !module_name_segment_is_identifier(s))
    {
        return Err(
            "a module name is a nonempty dotted sequence of identifier segments (letters, digits and underscore, not starting with a digit)".to_string(),
        );
    }
    if segments[0] == "v2" {
        if segments.len() == 1 {
            return Err(
                "the namespace v2 is a directory of its tree and has no file of its own"
                    .to_string(),
            );
        }
        return Ok(format!("src/v2/{}.dag", segments[1..].join("/")));
    }
    Ok(format!("dag/{}.dag", segments.join("/")))
}

fn module_name_segment_is_identifier(segment: &str) -> bool {
    let mut chars = segment.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    ident_start(first) && chars.all(ident_continue)
}

fn ident_start(c: char) -> bool {
    c == '_' || c.is_ascii_alphabetic()
}

fn ident_continue(c: char) -> bool {
    ident_start(c) || c.is_ascii_digit()
}

#[cfg(test)]
mod tests {
    use super::module_storage_relative;

    #[test]
    fn v2_elides_the_namespace_directory() {
        assert_eq!(
            module_storage_relative("v2.compiler.compile").unwrap(),
            "src/v2/compiler/compile.dag"
        );
    }

    #[test]
    fn default_tree_is_dag() {
        assert_eq!(
            module_storage_relative("gunbc.source_layout").unwrap(),
            "dag/gunbc/source_layout.dag"
        );
    }

    #[test]
    fn v2_namespace_alone_has_no_file() {
        assert!(module_storage_relative("v2").is_err());
    }
}
