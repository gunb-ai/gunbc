//! Single authority for the `generated.method_template_projection` `.dag` source.
//!
//! v2 self-compile imports `generated.method_template_projection` (see
//! `src/v1/05_emit*.dag`); this const is the v2-resident source for it, produced
//! ephemerally into a source root (never committed — the bootstrap ratchet forbids a
//! committed `src/generated/method_template_projection.dag`). Consumed by `regen_stage0`
//! and the self-compile test helper, so neither depends on an external producer.

use std::path::{Path, PathBuf};

pub const GENERATED_METHOD_TEMPLATE_PROJECTION_DAG: &str = r#"module generated.method_template_projection

data rust_method_template_emit: Map<String, String> = {
  "chars": "\{recv\}.chars().map(|c| c as i64).collect::<Vec<_>>()",
  "count": "(\{recv\}.len() as i64)",
  "enumerate": "\{recv\}.iter().cloned().enumerate().map(|(i, v)| (i as i64, v)).collect::<Vec<_>>()",
  "first": "\{recv\}.first().cloned()",
  "join": "\{recv\}.join(&\{arg\})",
  "last": "\{recv\}.last().cloned()",
  "skip": "\{recv\}.iter().cloned().skip(\{arg\} as usize).collect::<Vec<_>>()",
  "split": "\{recv\}.split(&\{arg\}).map(|s| s.to_string()).collect::<Vec<_>>()",
  "take": "\{recv\}.iter().cloned().take(\{arg\} as usize).collect::<Vec<_>>()",
}

data python_method_template_emit: Map<String, String> = {
  "all": "all(\{arg\}(x) for x in \{recv\})",
  "any": "any(\{arg\}(x) for x in \{recv\})",
  "append": "\{recv\} + [\{arg\}]",
  "chars": "[ord(c) for c in \{recv\}]",
  "count": "len(\{recv\})",
  "enumerate": "list(enumerate(\{recv\}))",
  "filter": "[x for x in \{recv\} if \{arg\}(x)]",
  "first": "\{recv\}[0] if \{recv\} else None",
  "flat_map": "[y for x in \{recv\} for y in \{arg\}(x)]",
  "join": "\{arg\}.join(\{recv\})",
  "last": "\{recv\}[-1] if \{recv\} else None",
  "skip": "\{recv\}[\{arg\}:]",
  "sort_by": "sorted(\{recv\}, key=\{arg\})",
  "split": "\{recv\}.split(\{arg\})",
  "string_contains": "\{arg\} in \{recv\}",
  "take": "\{recv\}[:\{arg\}]",
}

data go_method_template_emit: Map<String, String> = {
  "all": "v2rt.All(\{recv\}, \{arg\})",
  "any": "v2rt.Any(\{recv\}, \{arg\})",
  "append": "append(\{recv\}, \{arg\})",
  "count": "len(\{recv\})",
  "filter": "v2rt.Filter(\{recv\}, \{arg\})",
  "flat_map": "v2rt.FlatMap(\{recv\}, \{arg\})",
  "join": "strings.Join(\{recv\}, \{arg\})",
  "skip": "\{recv\}[\{arg\}:]",
  "sort_by": "v2rt.SortBy(\{recv\}, \{arg\})",
  "split": "strings.Split(\{recv\}, \{arg\})",
  "string_contains": "strings.Contains(\{recv\}, \{arg\})",
  "take": "\{recv\}[:\{arg\}]",
}
"#;

/// Write the projection source into `<generated_root>/generated/method_template_projection.dag`,
/// creating the `generated/` dir. Returns the generated-source-root path (pass as `--source-root`).
pub fn write_method_template_projection_dag(generated_root: &Path) -> std::io::Result<PathBuf> {
    let generated_dir = generated_root.join("generated");
    std::fs::create_dir_all(&generated_dir)?;
    std::fs::write(
        generated_dir.join("method_template_projection.dag"),
        GENERATED_METHOD_TEMPLATE_PROJECTION_DAG,
    )?;
    Ok(generated_root.to_path_buf())
}
