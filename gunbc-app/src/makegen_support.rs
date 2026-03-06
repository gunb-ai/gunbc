//! Makegen-related helper state that still lives in `gunbc-dag`.
//!
//! This module is intentionally narrow:
//! - embedded makegen asset generation

use std::collections::HashMap;

use crate::makegen::shared::render_makefile_from_dsl_discovery;
use daglang_emit::EmbeddedData;

/// Embedded asset key for precomputed makegen content.
pub const MAKEGEN_ASSET_KEY: &str = "tools.makegen::makefile";

/// Build embedded asset map for compile-time codegen.
pub fn build_embedded_data() -> Result<HashMap<String, EmbeddedData>, String> {
    let mut data = HashMap::new();
    data.insert(MAKEGEN_ASSET_KEY.to_string(), makegen_embedded_data()?);
    Ok(data)
}

/// Embedded makegen content payload.
pub fn makegen_embedded_data() -> Result<EmbeddedData, String> {
    Ok(EmbeddedData {
        module: "tools.makegen".to_string(),
        layer1_file_path: "src/embedded_makefile.txt".to_string(),
        layer2_ident: "makegen_content".to_string(),
        content: compute_makegen_content()?,
    })
}

/// Compute makegen content by rendering from discovered tools.
pub fn compute_makegen_content() -> Result<String, String> {
    render_makefile_from_dsl_discovery().map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_makegen_content_renders_discovered_tools() {
        let content = compute_makegen_content().expect("makegen content should render");
        assert!(content.contains("deps:"));
        assert!(content.contains("makegen:"));
        assert!(content.contains("pragma:"));
    }

    #[test]
    fn embedded_data_uses_makegen_asset_key() {
        let data = build_embedded_data().expect("embedded data should build");
        assert!(data.contains_key(MAKEGEN_ASSET_KEY));
    }
}
