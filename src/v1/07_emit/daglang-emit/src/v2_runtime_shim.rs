//! Rust source text for the `v2_rt` runtime shim module.
//!
//! The emitted runtime now comes from a real Rust source file so the generated
//! crate and the tested runtime share one implementation.

/// The Rust source code for the v2 runtime shim module.
pub const V2_RUNTIME_SOURCE: &str = include_str!("v2_runtime_source.rs");

#[cfg(test)]
#[allow(dead_code)]
#[path = "v2_runtime_source.rs"]
mod compiled_runtime;

#[cfg(test)]
mod tests {
    use super::compiled_runtime as rt;

    #[test]
    fn char_at_uses_character_indices_for_non_ascii() {
        assert_eq!(rt::char_at("AB⟦CD", 2), "⟦");
        assert_eq!(rt::char_at("AB⟦CD", 3), "C");
    }

    #[test]
    fn string_helpers_preserve_character_index_contract_for_non_ascii() {
        assert_eq!(rt::string_length("AB⟦CD"), 5);
        assert_eq!(rt::substring("AB⟦CD", 1, 4), "B⟦C");
        assert_eq!(rt::substring("AB⟦CD", 4, 2), "");
    }

    #[test]
    fn scanner_helpers_use_character_indices_after_non_ascii_prefix() {
        assert_eq!(rt::skip_horizontal_ws("π\t x", 1), 3);
        assert_eq!(rt::scan_to_eol("πab\nz", 1), 3);
        assert_eq!(rt::scan_string_end("π\"ab\\\"c\"", 2), 8);
        assert_eq!(
            rt::scan_while("π123x", 1, |ch| ch.chars().all(|c| c.is_ascii_digit())),
            4
        );
    }
}
