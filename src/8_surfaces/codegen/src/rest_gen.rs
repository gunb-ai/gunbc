//! REST exposure stub.
//!
//! Maps entrypoint parameters to HTTP request bindings (path, query, body,
//! header). Not yet implemented — this module defines the types and conventions
//! for when REST handler codegen is wired up.

use crate::entrypoint::{EntrypointDef, EntrypointParam, ExposureCodegen, ExposureContext};
use gunbc_ir::code_ir::SourceFile;

/// How a parameter is bound in a REST request.
#[derive(Debug, Clone)]
pub enum RestParamBinding {
    /// Query parameter: `?param_name=value`
    Query,
    /// JSON body field: `{ "param_name": value }`
    Body,
}

/// REST-specific exposure config for one parameter.
#[derive(Debug, Clone)]
pub struct RestParamConfig {
    /// Where this param appears in the HTTP request.
    pub binding: RestParamBinding,
    /// JSON field name (may differ from port_name by convention).
    pub json_name: String,
}

/// REST exposure implementation.
pub struct RestExposure {
    /// HTTP method for this entrypoint.
    pub method: HttpMethod,
    /// Path template (e.g., "/api/v1/gist").
    pub path: String,
}

/// HTTP method.
#[derive(Debug, Clone, Copy)]
pub enum HttpMethod {
    Get,
    Post,
}

impl ExposureCodegen for RestExposure {
    type ParamConfig = RestParamConfig;

    fn derive_param_config(
        &self,
        param: &EntrypointParam,
        _context: &ExposureContext,
    ) -> RestParamConfig {
        // Convention:
        // - List<T> → body (too complex for query params)
        // - Scalar types → query params
        let binding = if param.cardinality.allows_many() {
            RestParamBinding::Body
        } else {
            RestParamBinding::Query
        };
        RestParamConfig {
            binding,
            json_name: param.port_name.clone(),
        }
    }

    fn generate(
        &self,
        _entrypoint: &EntrypointDef,
        _params: &[(EntrypointParam, RestParamConfig)],
    ) -> SourceFile {
        todo!("REST handler codegen not yet implemented")
    }
}
