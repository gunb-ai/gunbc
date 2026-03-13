//! Lambda exposure stub.
//!
//! Maps entrypoint parameters to Lambda event payload fields. Not yet
//! implemented — this module defines the types and conventions for when
//! Lambda handler codegen is wired up.

use crate::entrypoint::{EntrypointDef, EntrypointParam, ExposureCodegen, ExposureContext};
use gunbc_ir::code_ir::SourceFile;

/// Lambda-specific exposure config for one parameter.
#[derive(Debug, Clone)]
pub struct LambdaParamConfig {
    /// JSON path in the Lambda event payload.
    pub event_path: String,
    /// Whether this param is required in the event.
    pub required: bool,
}

/// Lambda event source type.
#[derive(Debug, Clone)]
pub enum LambdaEventSource {
    /// Direct invocation (event JSON = params).
    Direct,
    /// API Gateway (event = HTTP request wrapper).
    ApiGateway,
}

/// Lambda exposure implementation.
pub struct LambdaExposure {
    /// Event source type.
    pub event_source: LambdaEventSource,
}

impl ExposureCodegen for LambdaExposure {
    type ParamConfig = LambdaParamConfig;

    fn derive_param_config(
        &self,
        param: &EntrypointParam,
        _context: &ExposureContext,
    ) -> LambdaParamConfig {
        LambdaParamConfig {
            event_path: param.port_name.clone(),
            required: !param.cardinality.allows_empty(),
        }
    }

    fn generate(
        &self,
        _entrypoint: &EntrypointDef,
        _params: &[(EntrypointParam, LambdaParamConfig)],
    ) -> SourceFile {
        todo!("Lambda handler codegen not yet implemented")
    }
}
