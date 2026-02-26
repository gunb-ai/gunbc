use std::collections::HashMap;

use daglang_emit::EmbeddedData;

use crate::makegen::ToolRegistry;

pub const MAKEGEN_ASSET_KEY: &str = "tools.makegen::makefile";

pub fn build_embedded_data() -> HashMap<String, EmbeddedData> {
    let mut data = HashMap::new();
    data.insert(MAKEGEN_ASSET_KEY.to_string(), makegen_embedded_data());
    data
}

pub fn makegen_embedded_data() -> EmbeddedData {
    EmbeddedData {
        module: "tools.makegen".to_string(),
        layer1_file_path: "src/embedded_makefile.txt".to_string(),
        layer2_ident: "makegen_content".to_string(),
        content: compute_makegen_content(),
    }
}

pub fn compute_makegen_content() -> String {
    let registry = ToolRegistry::default_registry();
    crate::render_makefile(&registry)
}
