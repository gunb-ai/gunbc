//! Shared CLI argument parser for gunbc binary entry points.
//!
//! Replaces the hand-rolled while-loop parsers in each `gunbc-dag/src/bin/*.rs`.
//! All binaries share `-n`/`--dry-run` and `-h`/`--help`; `--mode` and
//! `--check` (deprecated) are opt-in via builder methods.

use std::collections::HashMap;

use gunbc_ir::resource::ExecMode;

use crate::ParseError;

/// Definition for a string-valued CLI parameter.
struct StringParamDef {
    name: String,
    long: String,
    short: Option<char>,
    default: Option<String>,
}

/// Builder for binary CLI argument parsing.
///
/// All binaries get `-n`/`--dry-run` and `-h`/`--help` for free.
/// Optional features are enabled via builder methods:
/// - `with_mode()` — `--mode=VALUE` / `--mode VALUE`
/// - `with_check_deprecated()` — `-c`/`--check` (deprecated alias for `--mode=verify`)
/// - `with_string_param()` — arbitrary `--long VALUE` / `-s VALUE` parameters
pub struct BinaryArgs {
    enable_mode: bool,
    enable_check: bool,
    string_params: Vec<StringParamDef>,
}

/// Result of parsing binary CLI arguments.
pub struct ParsedBinaryArgs {
    /// Whether `--dry-run` / `-n` was present.
    pub dry_run: bool,
    /// Whether `--help` / `-h` was present.
    pub help: bool,
    /// Parsed resource mode from `--mode` or `--check`.
    pub resource_mode: Option<ExecMode>,
    /// Whether the deprecated `--check` flag was used.
    pub check_deprecated: bool,
    /// String parameter values keyed by name.
    string_values: HashMap<String, String>,
}

impl Default for BinaryArgs {
    fn default() -> Self {
        Self::new()
    }
}

impl BinaryArgs {
    /// Create a new builder with `-n`/`--dry-run` and `-h`/`--help` always enabled.
    pub fn new() -> Self {
        Self {
            enable_mode: false,
            enable_check: false,
            string_params: Vec::new(),
        }
    }

    /// Enable `--mode=VALUE` / `--mode VALUE` parsing.
    pub fn with_mode(mut self) -> Self {
        self.enable_mode = true;
        self
    }

    /// Enable `-c`/`--check` as a deprecated alias for `--mode=verify`.
    pub fn with_check_deprecated(mut self) -> Self {
        self.enable_check = true;
        self
    }

    /// Add a string-valued parameter.
    pub fn with_string_param(
        mut self,
        name: &str,
        long: &str,
        short: Option<char>,
        default: Option<&str>,
    ) -> Self {
        self.string_params.push(StringParamDef {
            name: name.to_string(),
            long: long.to_string(),
            short,
            default: default.map(|s| s.to_string()),
        });
        self
    }

    /// Parse the given argv slice (index 0 = program name, skipped).
    pub fn parse(self, argv: &[String]) -> Result<ParsedBinaryArgs, ParseError> {
        let mut dry_run = false;
        let mut help = false;
        let mut resource_mode: Option<ExecMode> = None;
        let mut check_deprecated = false;
        let mut string_values: HashMap<String, String> = HashMap::new();

        // Pre-populate defaults
        for param in &self.string_params {
            if let Some(ref default) = param.default {
                string_values.insert(param.name.clone(), default.clone());
            }
        }

        let mut i = 1;
        while i < argv.len() {
            let arg = &argv[i];
            match arg.as_str() {
                "-n" | "--dry-run" => dry_run = true,
                "-h" | "--help" => help = true,
                "-c" | "--check" if self.enable_check => {
                    resource_mode = Some(ExecMode::Verify);
                    check_deprecated = true;
                }
                _ if self.enable_mode && arg == "--mode" => {
                    i += 1;
                    if i >= argv.len() {
                        return Err(ParseError::MissingValue {
                            flag: "--mode".to_string(),
                        });
                    }
                    resource_mode = Some(
                        ExecMode::parse_strict(&argv[i]).map_err(|_| {
                            ParseError::InvalidValue {
                                flag: "--mode".to_string(),
                                value: argv[i].clone(),
                            }
                        })?,
                    );
                }
                _ if self.enable_mode && arg.starts_with("--mode=") => {
                    let mode_str = arg.trim_start_matches("--mode=");
                    resource_mode = Some(
                        ExecMode::parse_strict(mode_str).map_err(|_| {
                            ParseError::InvalidValue {
                                flag: "--mode".to_string(),
                                value: mode_str.to_string(),
                            }
                        })?,
                    );
                }
                _ => {
                    // Check string params
                    let mut matched = false;
                    for param in &self.string_params {
                        let long = format!("--{}", param.long);
                        let short_match = param.short.map(|c| format!("-{}", c));
                        if arg == &long || short_match.as_deref() == Some(arg.as_str()) {
                            i += 1;
                            if i >= argv.len() {
                                return Err(ParseError::MissingValue {
                                    flag: arg.clone(),
                                });
                            }
                            string_values.insert(param.name.clone(), argv[i].clone());
                            matched = true;
                            break;
                        }
                    }
                    if !matched && arg.starts_with('-') {
                        return Err(ParseError::UnknownFlag {
                            flag: arg.clone(),
                        });
                    }
                }
            }
            i += 1;
        }

        Ok(ParsedBinaryArgs {
            dry_run,
            help,
            resource_mode,
            check_deprecated,
            string_values,
        })
    }

    /// Parse from `std::env::args()`, printing errors and exiting on failure.
    ///
    /// Also prints the `--check` deprecation warning if applicable.
    pub fn parse_env(self) -> ParsedBinaryArgs {
        let argv: Vec<String> = std::env::args().collect();
        let parsed = match self.parse(&argv) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("error: {}", e);
                std::process::exit(1);
            }
        };
        if parsed.check_deprecated {
            eprintln!("Warning: --check is deprecated; use --mode=verify");
        }
        parsed
    }
}

impl ParsedBinaryArgs {
    /// Get a string parameter value by name.
    pub fn get_string(&self, name: &str) -> Option<&str> {
        self.string_values.get(name).map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(args: &[&str]) -> Vec<String> {
        args.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_simple_dry_run() {
        let parsed = BinaryArgs::new()
            .parse(&argv(&["prog", "-n"]))
            .unwrap();
        assert!(parsed.dry_run);
        assert!(!parsed.help);
    }

    #[test]
    fn test_simple_dry_run_long() {
        let parsed = BinaryArgs::new()
            .parse(&argv(&["prog", "--dry-run"]))
            .unwrap();
        assert!(parsed.dry_run);
    }

    #[test]
    fn test_simple_help() {
        let parsed = BinaryArgs::new()
            .parse(&argv(&["prog", "--help"]))
            .unwrap();
        assert!(parsed.help);
        assert!(!parsed.dry_run);
    }

    #[test]
    fn test_simple_help_short() {
        let parsed = BinaryArgs::new()
            .parse(&argv(&["prog", "-h"]))
            .unwrap();
        assert!(parsed.help);
    }

    #[test]
    fn test_unknown_flag_errors() {
        let result = BinaryArgs::new().parse(&argv(&["prog", "--unknown"]));
        assert!(matches!(result, Err(ParseError::UnknownFlag { .. })));
    }

    #[test]
    fn test_mode_equals() {
        let parsed = BinaryArgs::new()
            .with_mode()
            .parse(&argv(&["prog", "--mode=verify"]))
            .unwrap();
        assert_eq!(parsed.resource_mode, Some(ExecMode::Verify));
    }

    #[test]
    fn test_mode_space() {
        let parsed = BinaryArgs::new()
            .with_mode()
            .parse(&argv(&["prog", "--mode", "ensure"]))
            .unwrap();
        assert_eq!(parsed.resource_mode, Some(ExecMode::Ensure));
    }

    #[test]
    fn test_mode_missing_value() {
        let result = BinaryArgs::new()
            .with_mode()
            .parse(&argv(&["prog", "--mode"]));
        assert!(matches!(result, Err(ParseError::MissingValue { .. })));
    }

    #[test]
    fn test_mode_invalid_value() {
        let result = BinaryArgs::new()
            .with_mode()
            .parse(&argv(&["prog", "--mode=bogus"]));
        assert!(matches!(result, Err(ParseError::InvalidValue { .. })));
    }

    #[test]
    fn test_check_deprecated() {
        let parsed = BinaryArgs::new()
            .with_mode()
            .with_check_deprecated()
            .parse(&argv(&["prog", "-c"]))
            .unwrap();
        assert_eq!(parsed.resource_mode, Some(ExecMode::Verify));
        assert!(parsed.check_deprecated);
    }

    #[test]
    fn test_check_long_deprecated() {
        let parsed = BinaryArgs::new()
            .with_mode()
            .with_check_deprecated()
            .parse(&argv(&["prog", "--check"]))
            .unwrap();
        assert_eq!(parsed.resource_mode, Some(ExecMode::Verify));
        assert!(parsed.check_deprecated);
    }

    #[test]
    fn test_check_without_enable_is_unknown() {
        let result = BinaryArgs::new().parse(&argv(&["prog", "-c"]));
        assert!(matches!(result, Err(ParseError::UnknownFlag { .. })));
    }

    #[test]
    fn test_string_param_short() {
        let parsed = BinaryArgs::new()
            .with_string_param("path", "path", Some('o'), Some("Makefile"))
            .parse(&argv(&["prog", "-o", "output.mk"]))
            .unwrap();
        assert_eq!(parsed.get_string("path"), Some("output.mk"));
    }

    #[test]
    fn test_string_param_default() {
        let parsed = BinaryArgs::new()
            .with_string_param("path", "path", Some('o'), Some("Makefile"))
            .parse(&argv(&["prog"]))
            .unwrap();
        assert_eq!(parsed.get_string("path"), Some("Makefile"));
    }

    #[test]
    fn test_string_param_long() {
        let parsed = BinaryArgs::new()
            .with_string_param("path", "path", Some('o'), None)
            .parse(&argv(&["prog", "--path", "foo"]))
            .unwrap();
        assert_eq!(parsed.get_string("path"), Some("foo"));
    }

    #[test]
    fn test_string_param_missing_value() {
        let result = BinaryArgs::new()
            .with_string_param("path", "path", Some('o'), None)
            .parse(&argv(&["prog", "--path"]));
        assert!(matches!(result, Err(ParseError::MissingValue { .. })));
    }

    #[test]
    fn test_combined_flags() {
        let parsed = BinaryArgs::new()
            .with_mode()
            .with_check_deprecated()
            .with_string_param("path", "path", Some('o'), Some("Makefile"))
            .parse(&argv(&["prog", "-n", "--mode=verify", "-o", "out.mk"]))
            .unwrap();
        assert!(parsed.dry_run);
        assert_eq!(parsed.resource_mode, Some(ExecMode::Verify));
        assert_eq!(parsed.get_string("path"), Some("out.mk"));
    }

    #[test]
    fn test_no_args() {
        let parsed = BinaryArgs::new().parse(&argv(&["prog"])).unwrap();
        assert!(!parsed.dry_run);
        assert!(!parsed.help);
        assert_eq!(parsed.resource_mode, None);
    }

    #[test]
    fn test_mode_without_enable_is_unknown() {
        let result = BinaryArgs::new().parse(&argv(&["prog", "--mode=verify"]));
        assert!(matches!(result, Err(ParseError::UnknownFlag { .. })));
    }

    #[test]
    fn test_string_param_no_default_absent() {
        let parsed = BinaryArgs::new()
            .with_string_param("path", "path", Some('o'), None)
            .parse(&argv(&["prog"]))
            .unwrap();
        assert_eq!(parsed.get_string("path"), None);
    }
}
