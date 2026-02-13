//! Blob content acquisition.
//!
//! Unified content acquisition following the same patterns as tool acquisition:
//! - Prepare/Parse sandwich around TransportOps::Execute
//! - Sealed handles (BlobHandle) requiring acquisition
//! - ResourceId integration for conflict detection
//!
//! All operations are PURE (no I/O). I/O happens through TransportOps::Execute nodes.

#![deny(dead_code)]
use gunbc_exec::{
    optional_bool_strict, optional_json_strict, optional_str_strict, require_json,
    require_response, ExecError, Executable, IntoExecResult, OutputMap,
};
use gunbc_infra::hash::ContentHash;
use gunbc_ir::resource::{AccessMode, ResourceId};
use gunbc_ir::transport::{FileOp, FileRequest, ShellRequest, TransportRequest, TransportResponse};
use gunbc_ir::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::path::PathBuf;

// ============================================================================
// Blob Source (declarative definition, like CliToolDef)
// ============================================================================

/// Declarative blob source definition.
///
/// Specifies where to fetch content from. Similar to `CliToolDef` for tools.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobSource {
    /// Where to get the content
    pub source: SourceSpec,
    /// Cache key for dedup (optional - derived from source if not provided)
    #[serde(default)]
    pub cache_key: Option<String>,
}

impl BlobSource {
    /// Create a new blob source from a source spec.
    pub fn new(source: SourceSpec) -> Self {
        Self {
            source,
            cache_key: None,
        }
    }

    /// Create an inline blob (no I/O needed).
    pub fn inline(data: impl Into<String>) -> Self {
        Self::new(SourceSpec::Inline {
            data: data.into(),
            content_type: None,
        })
    }

    /// Create a file blob.
    pub fn file(path: impl Into<PathBuf>) -> Self {
        Self::new(SourceSpec::File { path: path.into() })
    }

    /// Create a git blob (file at a specific ref).
    pub fn git_blob(ref_: impl Into<String>, path: impl Into<String>) -> Self {
        Self::new(SourceSpec::GitBlob {
            ref_: ref_.into(),
            path: path.into(),
        })
    }

    /// Set a custom cache key.
    pub fn with_cache_key(mut self, key: impl Into<String>) -> Self {
        self.cache_key = Some(key.into());
        self
    }

    /// Get the resource ID for conflict detection.
    ///
    /// All blobs have `AccessMode::Read` - safe for parallel access.
    pub fn resource_id(&self) -> ResourceId {
        match &self.source {
            SourceSpec::Inline { .. } => ResourceId::new("blob:inline"),
            SourceSpec::File { path } => ResourceId::file(path.to_string_lossy()),
            SourceSpec::GitBlob { ref_, path } => {
                ResourceId::new(format!("blob:git:{}:{}", ref_, path))
            }
            SourceSpec::S3 { bucket, key, .. } => {
                ResourceId::new(format!("blob:s3:{}/{}", bucket, key))
            }
            SourceSpec::Http { url, .. } => ResourceId::new(format!("blob:http:{}", url)),
        }
    }

    /// Access mode is always Read for blobs.
    pub fn access_mode(&self) -> AccessMode {
        AccessMode::Read
    }
}

/// Where to get blob content from.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SourceSpec {
    /// Direct inline content (no I/O needed)
    Inline {
        data: String,
        #[serde(default)]
        content_type: Option<String>,
    },

    /// Local filesystem
    File { path: PathBuf },

    /// Git object at ref (uses `git show ref:path`)
    GitBlob { ref_: String, path: String },

    /// S3-compatible storage
    S3 {
        bucket: String,
        key: String,
        #[serde(default)]
        region: Option<String>,
    },

    /// HTTP GET
    Http {
        url: String,
        #[serde(default)]
        headers: Option<HashMap<String, String>>,
    },
}

// ============================================================================
// Blob Metadata
// ============================================================================

/// Metadata about acquired blob content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobMeta {
    /// Size in bytes
    pub size: usize,
    /// Content hash (SHA256) for caching/dedup
    #[serde(default)]
    pub hash: Option<String>,
    /// Content type (MIME type)
    #[serde(default)]
    pub content_type: Option<String>,
    /// ETag for HTTP/S3 caching
    #[serde(default)]
    pub etag: Option<String>,
}

impl BlobMeta {
    /// Create metadata from content.
    pub fn from_content(content: &str) -> Self {
        Self {
            size: content.len(),
            hash: Some(Self::compute_hash(content)),
            content_type: None,
            etag: None,
        }
    }

    /// Compute SHA256 hash of content.
    fn compute_hash(content: &str) -> String {
        ContentHash::from_bytes(content.as_bytes())
            .as_str()
            .to_string()
    }
}

// ============================================================================
// Blob Handle (capability-based, like ToolHandle)
// ============================================================================

/// Handle to acquired blob content.
///
/// Like `ToolHandle`, this is a capability that requires acquisition.
/// The `PhantomData` prevents direct construction outside acquisition.
#[derive(Debug, Clone)]
pub struct BlobHandle {
    /// The source this was acquired from
    source: BlobSource,
    /// The actual content
    data: String,
    /// Metadata about the content
    meta: BlobMeta,
    /// Prevents direct construction
    _acquired: PhantomData<()>,
}

impl BlobHandle {
    /// Acquire a blob handle (framework use only).
    ///
    /// This should only be called after successfully fetching content.
    pub fn acquire(source: BlobSource, data: String) -> Self {
        let meta = BlobMeta::from_content(&data);
        Self {
            source,
            data,
            meta,
            _acquired: PhantomData,
        }
    }

    /// Create a mock handle for DryRun/testing.
    pub fn mock(source: BlobSource) -> Self {
        let data = format!("[mock blob from {:?}]", source.source);
        Self::acquire(source, data)
    }

    /// Get the blob content.
    pub fn data(&self) -> &str {
        &self.data
    }

    /// Get blob metadata.
    pub fn meta(&self) -> &BlobMeta {
        &self.meta
    }

    /// Get the source this was acquired from.
    pub fn source(&self) -> &BlobSource {
        &self.source
    }

    /// Encode handle for transmission through DAG edges.
    pub fn encode(&self) -> Value {
        Value::Json(serde_json::json!({
            "type": "blob_handle",
            "source": self.source,
            "data": self.data,
            "meta": self.meta,
        }))
    }

    /// Decode handle from DAG edge value.
    pub fn decode(value: &Value) -> Result<Self, BlobHandleError> {
        let json = value
            .as_json()
            .ok_or_else(|| BlobHandleError::new("expected JSON value"))?;

        let type_field = json
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| BlobHandleError::new("missing 'type' field"))?;

        if type_field != "blob_handle" {
            return Err(BlobHandleError::new(format!(
                "expected type 'blob_handle', got '{}'",
                type_field
            )));
        }

        let source: BlobSource = serde_json::from_value(
            json.get("source")
                .cloned()
                .ok_or_else(|| BlobHandleError::new("missing 'source' field"))?,
        )
        .map_err(|e| BlobHandleError::new(format!("invalid source: {}", e)))?;

        let data = json
            .get("data")
            .and_then(|v| v.as_str())
            .ok_or_else(|| BlobHandleError::new("missing 'data' field"))?
            .to_string();

        let meta: BlobMeta = serde_json::from_value(
            json.get("meta")
                .cloned()
                .ok_or_else(|| BlobHandleError::new("missing 'meta' field"))?,
        )
        .map_err(|e| BlobHandleError::new(format!("invalid meta: {}", e)))?;

        Ok(Self {
            source,
            data,
            meta,
            _acquired: PhantomData,
        })
    }
}

/// Error when parsing a BlobHandle from a Value.
#[derive(Debug)]
pub struct BlobHandleError {
    pub message: String,
}

impl BlobHandleError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for BlobHandleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlobHandle error: {}", self.message)
    }
}

impl std::error::Error for BlobHandleError {}

// ============================================================================
// Blob Operations (Executable trait)
// ============================================================================

/// Blob operations for use in DAG nodes.
///
/// All operations are PURE - no I/O. Use TransportOps::Execute for actual I/O.
#[derive(Debug, Clone)]
pub enum BlobOps {
    /// Prepare a fetch request from source spec (PURE - no I/O).
    ///
    /// For inline sources, this returns the data directly (no request needed).
    PrepareFetch,

    /// Parse fetch response into BlobHandle (PURE - no I/O).
    ParseFetch,

    /// Compare expected content against actual content for equality.
    ///
    /// This is the universal content comparison op — it accepts BlobHandles,
    /// raw strings, or file read responses and unifies them into a single
    /// comparison. Uses SHA-256 hash for O(1) comparison when available.
    ///
    /// **Actual content** (resolution priority):
    /// 1. `actual` (Json) — encoded BlobHandle (blob pipeline)
    /// 2. `actual_content` (String) — pre-extracted content
    /// 3. `response` (TransportResponse::File) — extract from file read
    ///
    /// **Expected content** (resolution priority):
    /// 1. `expected` (Json) — encoded BlobHandle (blob pipeline)
    /// 2. `expected_hash` (String) — SHA-256 hex for hash-only comparison
    /// 3. `expected_content` (String) — full content for string comparison
    ///
    /// When both sides have hashes, comparison is O(1). Otherwise falls back
    /// to string comparison.
    ///
    /// Inputs (all optional, at least one actual + one expected required):
    /// - `actual`: Json (encoded BlobHandle)
    /// - `actual_content`: String
    /// - `response`: TransportResponse (file read)
    /// - `expected`: Json (encoded BlobHandle)
    /// - `expected_hash`: String (SHA-256 hex)
    /// - `expected_content`: String
    /// - `check_mode`: Bool (optional) — if true, forces skip=true
    ///
    /// Outputs:
    /// - `fresh`: Bool — true if content matches
    /// - `skip`: Bool — true if write should be skipped (fresh || check_mode)
    /// - `skip_reason`: String — explanation
    CompareContent,
}

impl Executable for BlobOps {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            BlobOps::PrepareFetch => {
                let source_json = require_json(&inputs, "source")?;

                let source: BlobSource =
                    serde_json::from_value(source_json.clone()).exec_context("invalid source")?;

                match &source.source {
                    // Inline: no I/O needed, return handle directly
                    SourceSpec::Inline { data, .. } => {
                        let handle = BlobHandle::acquire(source.clone(), data.clone());
                        OutputMap::new()
                            .value("handle", handle.encode())
                            .bool("skip_fetch", true)
                            .bool("skip", true)
                            .ok()
                    }

                    // File: prepare file read request
                    SourceSpec::File { path } => {
                        let request =
                            TransportRequest::File(FileRequest::read(path.to_string_lossy()));
                        OutputMap::new()
                            .request("request", request)
                            .bool("skip_fetch", false)
                            .bool("skip", false)
                            .json("source", serde_json::to_value(&source).unwrap())
                            .ok()
                    }

                    // Git blob: prepare git show command
                    SourceSpec::GitBlob { ref_, path } => {
                        let request = ShellRequest::new("git")
                            .args(["show", &format!("{}:{}", ref_, path)])
                            .into_transport_request();
                        OutputMap::new()
                            .request("request", request)
                            .bool("skip_fetch", false)
                            .bool("skip", false)
                            .json("source", serde_json::to_value(&source).unwrap())
                            .ok()
                    }

                    // S3/HTTP: not yet implemented
                    SourceSpec::S3 { .. } | SourceSpec::Http { .. } => {
                        Err(ExecError::new("S3/HTTP sources not yet implemented"))
                    }
                }
            }

            BlobOps::ParseFetch => {
                let skip = optional_bool_strict(&inputs, "skip")?.unwrap_or(false);
                if skip {
                    let handle_json = require_json(&inputs, "handle")?;
                    let handle = BlobHandle::decode(&Value::Json(handle_json.clone()))
                        .map_err(|e| ExecError::new(format!("invalid handle: {}", e)))?;
                    return OutputMap::new()
                        .value("handle", handle.encode())
                        .json("meta", serde_json::to_value(handle.meta()).unwrap())
                        .ok();
                }

                let source_json = require_json(&inputs, "source")?;

                let source: BlobSource =
                    serde_json::from_value(source_json.clone()).exec_context("invalid source")?;

                let response = require_response(&inputs, "response")?;

                let data = match response {
                    TransportResponse::File(file_resp) if file_resp.operation == FileOp::Read => {
                        file_resp.content.clone().ok_or_else(|| {
                            ExecError::new(format!(
                                "file read failed: {}",
                                file_resp.error.as_deref().unwrap_or("unknown error")
                            ))
                        })?
                    }
                    TransportResponse::Shell(shell) => {
                        if !shell.success() {
                            return Err(ExecError::new(format!("fetch failed: {}", shell.stderr)));
                        }
                        shell.stdout.clone()
                    }
                    _ => {
                        return Err(ExecError::new("unexpected response type"));
                    }
                };

                let handle = BlobHandle::acquire(source, data);

                OutputMap::new()
                    .value("handle", handle.encode())
                    .json("meta", serde_json::to_value(handle.meta()).unwrap())
                    .ok()
            }

            BlobOps::CompareContent => {
                let check_mode = optional_bool_strict(&inputs, "check_mode")?.unwrap_or(false);

                // --- Resolve actual content + hash ---
                // Priority: actual (BlobHandle) > actual_content (String) > response
                let (actual_data, actual_hash): (Option<String>, Option<String>) =
                    if let Some(json) = optional_json_strict(&inputs, "actual")? {
                        let handle = BlobHandle::decode(&Value::Json(json.clone()))
                            .map_err(|e| ExecError::new(format!("invalid actual handle: {}", e)))?;
                        (Some(handle.data().to_string()), handle.meta().hash.clone())
                    } else if let Some(s) = optional_str_strict(&inputs, "actual_content")? {
                        (Some(s.to_string()), None)
                    } else {
                        // Extract from file read response (string compat path)
                        match inputs.get("response") {
                            Some(Value::Response(TransportResponse::File(f))) if f.success => {
                                (f.content.clone(), None)
                            }
                            _ => (None, None),
                        }
                    };

                // --- Resolve expected content + hash ---
                // Priority: expected (BlobHandle) > expected_hash > expected_content
                let (expected_data, expected_hash): (Option<String>, Option<String>) =
                    if let Some(json) = optional_json_strict(&inputs, "expected")? {
                        let handle =
                            BlobHandle::decode(&Value::Json(json.clone())).map_err(|e| {
                                ExecError::new(format!("invalid expected handle: {}", e))
                            })?;
                        (Some(handle.data().to_string()), handle.meta().hash.clone())
                    } else if let Some(h) = optional_str_strict(&inputs, "expected_hash")? {
                        (None, Some(h.to_string()))
                    } else if let Some(c) = optional_str_strict(&inputs, "expected_content")? {
                        (Some(c.to_string()), None)
                    } else {
                        (None, None)
                    };

                // --- Compare ---
                let (fresh, detail) =
                    match (&actual_data, &actual_hash, &expected_data, &expected_hash) {
                        // Both sides have hashes → O(1) comparison
                        (_, Some(ah), _, Some(eh)) => {
                            if ah == eh {
                                (true, "content hash matches".to_string())
                            } else {
                                (false, "content hash differs".to_string())
                            }
                        }
                        // Expected has hash, actual has data → compute actual hash
                        (Some(ad), None, _, Some(eh)) => {
                            let ah = ContentHash::from_bytes(ad.as_bytes()).as_str().to_string();
                            if ah == *eh {
                                (true, "content hash matches".to_string())
                            } else {
                                (false, "content hash differs".to_string())
                            }
                        }
                        // Both have data → string comparison
                        (Some(ad), _, Some(ed), _) => {
                            if ad == ed {
                                (true, "disk content matches expected".to_string())
                            } else {
                                (false, "disk content differs from expected".to_string())
                            }
                        }
                        // Missing actual
                        (None, None, _, _) => {
                            let reason = match inputs.get("response") {
                                Some(Value::Response(TransportResponse::File(f))) if !f.success => {
                                    "file read failed".to_string()
                                }
                                Some(Value::Skipped) => "upstream read was skipped".to_string(),
                                _ => "no actual content available".to_string(),
                            };
                            (false, reason)
                        }
                        // Missing expected
                        _ => (false, "no expected content or hash provided".to_string()),
                    };

                let skip = fresh || check_mode;
                let skip_reason = if fresh {
                    "content is fresh — write skipped".to_string()
                } else if check_mode {
                    format!("check mode — would write ({})", detail)
                } else {
                    String::new()
                };

                OutputMap::new()
                    .bool("fresh", fresh)
                    .bool("skip", skip)
                    .str("skip_reason", skip_reason)
                    .ok()
            }
        }
    }
}

// ============================================================================
// Convenience functions
// ============================================================================

/// Prepare a blob fetch request.
///
/// Returns `(Option<TransportRequest>, Option<BlobHandle>)`:
/// - For inline sources: `(None, Some(handle))` - no fetch needed
/// - For other sources: `(Some(request), None)` - fetch needed
pub fn prepare_blob_fetch(source: &BlobSource) -> (Option<TransportRequest>, Option<BlobHandle>) {
    match &source.source {
        SourceSpec::Inline { data, .. } => {
            let handle = BlobHandle::acquire(source.clone(), data.clone());
            (None, Some(handle))
        }
        SourceSpec::File { path } => {
            let request = TransportRequest::File(FileRequest::read(path.to_string_lossy()));
            (Some(request), None)
        }
        SourceSpec::GitBlob { ref_, path } => {
            let request = ShellRequest::new("git")
                .args(["show", &format!("{}:{}", ref_, path)])
                .into_transport_request();
            (Some(request), None)
        }
        SourceSpec::S3 { .. } | SourceSpec::Http { .. } => {
            // Not yet implemented
            (None, None)
        }
    }
}

/// Parse a fetch response into a BlobHandle.
pub fn parse_blob_response(
    source: &BlobSource,
    response: &TransportResponse,
) -> Result<BlobHandle, String> {
    let data = match response {
        TransportResponse::File(file_resp) if file_resp.operation == FileOp::Read => {
            file_resp.content.clone().ok_or_else(|| {
                format!(
                    "file read failed: {}",
                    file_resp.error.as_deref().unwrap_or("unknown error")
                )
            })?
        }
        TransportResponse::Shell(shell) => {
            if !shell.success() {
                return Err(format!("fetch failed: {}", shell.stderr));
            }
            shell.stdout.clone()
        }
        _ => return Err("unexpected response type".to_string()),
    };

    Ok(BlobHandle::acquire(source.clone(), data))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_blob() {
        let source = BlobSource::inline("hello world");
        let (request, handle) = prepare_blob_fetch(&source);

        assert!(request.is_none());
        assert!(handle.is_some());

        let handle = handle.unwrap();
        assert_eq!(handle.data(), "hello world");
        assert_eq!(handle.meta().size, 11);
    }

    #[test]
    fn test_file_blob_request() {
        let source = BlobSource::file("/tmp/test.txt");
        let (request, handle) = prepare_blob_fetch(&source);

        assert!(request.is_some());
        assert!(handle.is_none());

        match request.unwrap() {
            TransportRequest::File(req) if req.operation == FileOp::Read => {
                assert_eq!(req.path, "/tmp/test.txt");
            }
            _ => panic!("expected file read request"),
        }
    }

    #[test]
    fn test_git_blob_request() {
        let source = BlobSource::git_blob("HEAD", "src/main.rs");
        let (request, handle) = prepare_blob_fetch(&source);

        assert!(request.is_some());
        assert!(handle.is_none());

        match request.unwrap() {
            TransportRequest::Shell(req) => {
                assert_eq!(req.command, "git");
                assert_eq!(req.args, vec!["show", "HEAD:src/main.rs"]);
            }
            _ => panic!("expected shell request"),
        }
    }

    #[test]
    fn test_blob_handle_encode_decode() {
        let source = BlobSource::inline("test content");
        let handle = BlobHandle::acquire(source, "test content".to_string());

        let encoded = handle.encode();
        let decoded = BlobHandle::decode(&encoded).unwrap();

        assert_eq!(decoded.data(), "test content");
        assert_eq!(decoded.meta().size, 12);
    }

    #[test]
    fn test_resource_id() {
        let inline = BlobSource::inline("x");
        assert_eq!(inline.resource_id().0, "blob:inline");

        let file = BlobSource::file("/tmp/test.txt");
        assert_eq!(file.resource_id().0, "file:/tmp/test.txt");

        let git = BlobSource::git_blob("main", "src/lib.rs");
        assert_eq!(git.resource_id().0, "blob:git:main:src/lib.rs");
    }

    #[test]
    fn test_blob_ops_prepare_inline() {
        let source = BlobSource::inline("inline content");
        let mut inputs = HashMap::new();
        inputs.insert(
            "source".to_string(),
            Value::Json(serde_json::to_value(&source).unwrap()),
        );

        let op = BlobOps::PrepareFetch;
        let result = op.execute(inputs).unwrap();

        assert_eq!(result.get("skip_fetch"), Some(&Value::Bool(true)));
        assert!(result.contains_key("handle"));
    }

    #[test]
    fn test_blob_ops_prepare_file() {
        let source = BlobSource::file("/tmp/test.txt");
        let mut inputs = HashMap::new();
        inputs.insert(
            "source".to_string(),
            Value::Json(serde_json::to_value(&source).unwrap()),
        );

        let op = BlobOps::PrepareFetch;
        let result = op.execute(inputs).unwrap();

        assert_eq!(result.get("skip_fetch"), Some(&Value::Bool(false)));
        assert!(result.contains_key("request"));
    }

    // ========================================================================
    // BlobOps::CompareContent tests
    // ========================================================================

    #[test]
    fn test_blob_compare_matching_content() {
        let source = BlobSource::inline("same content");
        let expected = BlobHandle::acquire(source.clone(), "same content".to_string());
        let actual = BlobHandle::acquire(source, "same content".to_string());

        let mut inputs = HashMap::new();
        inputs.insert("expected".to_string(), expected.encode());
        inputs.insert("actual".to_string(), actual.encode());

        let op = BlobOps::CompareContent;
        let result = op.execute(inputs).unwrap();

        assert_eq!(result.get("fresh"), Some(&Value::Bool(true)));
        assert_eq!(result.get("skip"), Some(&Value::Bool(true)));
    }

    #[test]
    fn test_blob_compare_different_content() {
        let source = BlobSource::inline("x");
        let expected = BlobHandle::acquire(source.clone(), "expected content".to_string());
        let actual = BlobHandle::acquire(source, "actual content".to_string());

        let mut inputs = HashMap::new();
        inputs.insert("expected".to_string(), expected.encode());
        inputs.insert("actual".to_string(), actual.encode());

        let op = BlobOps::CompareContent;
        let result = op.execute(inputs).unwrap();

        assert_eq!(result.get("fresh"), Some(&Value::Bool(false)));
        assert_eq!(result.get("skip"), Some(&Value::Bool(false)));
    }

    #[test]
    fn test_blob_compare_check_mode_override() {
        let source = BlobSource::inline("x");
        let expected = BlobHandle::acquire(source.clone(), "expected".to_string());
        let actual = BlobHandle::acquire(source, "actual".to_string());

        let mut inputs = HashMap::new();
        inputs.insert("expected".to_string(), expected.encode());
        inputs.insert("actual".to_string(), actual.encode());
        inputs.insert("check_mode".to_string(), Value::Bool(true));

        let op = BlobOps::CompareContent;
        let result = op.execute(inputs).unwrap();

        assert_eq!(result.get("fresh"), Some(&Value::Bool(false)));
        assert_eq!(result.get("skip"), Some(&Value::Bool(true)));
        let reason = result.get("skip_reason").and_then(|v| v.as_str()).unwrap();
        assert!(reason.contains("check mode"));
    }

    #[test]
    fn test_blob_compare_uses_hash_when_available() {
        // Both handles have hashes (from BlobHandle::acquire → BlobMeta::from_content)
        let source = BlobSource::inline("x");
        let handle_a = BlobHandle::acquire(source.clone(), "content".to_string());
        let handle_b = BlobHandle::acquire(source, "content".to_string());

        // Verify both have hashes
        assert!(handle_a.meta().hash.is_some());
        assert!(handle_b.meta().hash.is_some());

        let mut inputs = HashMap::new();
        inputs.insert("expected".to_string(), handle_a.encode());
        inputs.insert("actual".to_string(), handle_b.encode());

        let op = BlobOps::CompareContent;
        let result = op.execute(inputs).unwrap();

        assert_eq!(result.get("fresh"), Some(&Value::Bool(true)));
    }

    // ========================================================================
    // BlobOps::CompareContent — string/response compat tests
    // ========================================================================

    #[test]
    fn test_blob_compare_with_response_and_expected_content() {
        // String compat: response + expected_content
        use gunbc_ir::transport::{FileOp, FileResponse};

        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::File(FileResponse {
                path: "Makefile".into(),
                operation: FileOp::Read,
                success: true,
                content: Some("hello world".into()),
                exists: None,
                error: None,
            })),
        );
        inputs.insert(
            "expected_content".to_string(),
            Value::Str("hello world".into()),
        );

        let op = BlobOps::CompareContent;
        let result = op.execute(inputs).unwrap();

        assert_eq!(result.get("fresh"), Some(&Value::Bool(true)));
        assert_eq!(result.get("skip"), Some(&Value::Bool(true)));
    }

    #[test]
    fn test_blob_compare_with_response_mismatch() {
        use gunbc_ir::transport::{FileOp, FileResponse};

        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::File(FileResponse {
                path: "Makefile".into(),
                operation: FileOp::Read,
                success: true,
                content: Some("old".into()),
                exists: None,
                error: None,
            })),
        );
        inputs.insert("expected_content".to_string(), Value::Str("new".into()));

        let op = BlobOps::CompareContent;
        let result = op.execute(inputs).unwrap();

        assert_eq!(result.get("fresh"), Some(&Value::Bool(false)));
        assert_eq!(result.get("skip"), Some(&Value::Bool(false)));
    }

    #[test]
    fn test_blob_compare_response_failed_read() {
        use gunbc_ir::transport::{FileOp, FileResponse};

        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::File(FileResponse {
                path: "Makefile".into(),
                operation: FileOp::Read,
                success: false,
                content: None,
                exists: None,
                error: Some("No such file".into()),
            })),
        );
        inputs.insert("expected_content".to_string(), Value::Str("content".into()));

        let op = BlobOps::CompareContent;
        let result = op.execute(inputs).unwrap();

        assert_eq!(result.get("fresh"), Some(&Value::Bool(false)));
    }

    #[test]
    fn test_blob_compare_response_check_mode() {
        use gunbc_ir::transport::{FileOp, FileResponse};

        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::File(FileResponse {
                path: "Makefile".into(),
                operation: FileOp::Read,
                success: true,
                content: Some("old".into()),
                exists: None,
                error: None,
            })),
        );
        inputs.insert("expected_content".to_string(), Value::Str("new".into()));
        inputs.insert("check_mode".to_string(), Value::Bool(true));

        let op = BlobOps::CompareContent;
        let result = op.execute(inputs).unwrap();

        assert_eq!(result.get("fresh"), Some(&Value::Bool(false)));
        assert_eq!(result.get("skip"), Some(&Value::Bool(true)));
        let reason = result.get("skip_reason").and_then(|v| v.as_str()).unwrap();
        assert!(reason.contains("check mode"));
    }

    #[test]
    fn test_blob_compare_actual_content_string() {
        let mut inputs = HashMap::new();
        inputs.insert("actual_content".to_string(), Value::Str("same".into()));
        inputs.insert("expected_content".to_string(), Value::Str("same".into()));

        let op = BlobOps::CompareContent;
        let result = op.execute(inputs).unwrap();

        assert_eq!(result.get("fresh"), Some(&Value::Bool(true)));
    }
}
