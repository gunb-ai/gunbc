use std::fs;
use std::path::{Path, PathBuf};

use daglang_syntax::ast::{Item, SourceFile, TypeBody};

fn dsl_file(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../dsl")
        .join(path)
}

// Test infrastructure: filesystem access for test fixtures
#[allow(clippy::disallowed_methods, clippy::disallowed_types)]
fn parse_dsl(path: &str) -> daglang_syntax::ast::SourceFile {
    let file = dsl_file(path);
    let source = fs::read_to_string(&file).expect("failed to read representative .dag file");
    daglang_syntax::parser::parse(&source)
        .unwrap_or_else(|errors| panic!("failed to parse {}: {errors:?}", file.display()))
}

fn item_signatures(source: &SourceFile) -> Vec<String> {
    source
        .items
        .iter()
        .map(|item| match &item.node {
            Item::TypeDef(def) => format!("type {}", def.name),
            Item::FnDef(def) => format!("fn {}", def.name),
            Item::FuncDef(def) => format!("func {}", def.name),
            Item::PatternDef(def) => format!("pattern {}", def.name),
            Item::ServiceDef(def) => format!("service {}", def.name),
            Item::ResourceDef(def) => format!("resource {}", def.name),
            Item::InterfaceDef(def) => format!("interface {}", def.name),
            Item::PipelineDef(def) => format!("pipeline {}", def.name),
            Item::ProfileDef(def) => format!("profile {}", def.name),
            Item::TestDef(def) => format!("test {}", def.name),
            Item::FixtureDef(def) => format!("fixture {}", def.name),
            Item::ProjectDef(def) => format!("project {}", def.name),
            Item::FeatureDef(def) => format!("feature {}", def.name),
            Item::TaskDef(def) => format!("task {}", def.name),
            Item::DesignDef(def) => format!("design {}", def.name),
            Item::ComponentDef(def) => format!("component {}", def.name),
            Item::EnvironmentDef(def) => format!("environment {}", def.name),
            Item::ParamDecl(decl) => format!("param {}", decl.name),
            Item::DataDef(def) => format!("data {}", def.name),
            Item::ExternAssetDecl(def) => format!("extern asset {}", def.name),
        })
        .collect()
}

#[test]
fn types_file_contains_record_sum_and_alias_definitions() {
    let source = parse_dsl("std/types.dag");
    assert_eq!(
        source.items.len(),
        123,
        "std/types.dag item count changed unexpectedly"
    );
    assert_eq!(
        source
            .module_path
            .as_ref()
            .map(|module| module.node.segments.clone()),
        Some(vec!["std".into(), "types".into()])
    );

    let mut saw_record = false;
    let mut saw_sum = false;
    let mut saw_alias = false;
    for item in &source.items {
        if let Item::TypeDef(typedef) = &item.node {
            match typedef.body {
                TypeBody::Record(_) => saw_record = true,
                TypeBody::Sum(_) => saw_sum = true,
                TypeBody::Alias(_) => saw_alias = true,
            }
        }
    }

    assert!(saw_record, "expected at least one record type definition");
    assert!(saw_sum, "expected at least one sum type definition");
    assert!(saw_alias, "expected at least one alias type definition");
    assert_eq!(
        item_signatures(&source),
        vec![
            "type Char",
            "type CommitSha",
            "type Sha256",
            "type RetryCount",
            "type HttpStatus",
            "type Email",
            "type Port",
            "type GistId",
            "type SecretValue",
            "type Url",
            "type SemVer",
            "type NonEmptyStr",
            "type LanguageId",
            "type PathSegment",
            "type GlobSegment",
            "type FilePathParts",
            "type GlobPattern",
            "type FilePath",
            "type Timestamp",
            "type EpochMs",
            "type Duration",
            "type Milliseconds",
            "type Seconds",
            "type IntentId",
            "type IssueId",
            "type RunKey",
            "type ArtifactId",
            "type LeaseToken",
            "type WorkerId",
            "type CommentId",
            "type SignalKey",
            "type ContentHash",
            "type GitRef",
            "type ProjectId",
            "type ServiceAccountEmail",
            "type WarningPolicy",
            "type CloudRuntime",
            "type Platform",
            "type TopologyNodeKind",
            "type ArtifactKind",
            "type DocSourceKind",
            "type AuthorSource",
            "type ReviewSource",
            "type SeverityLevel",
            "type ReviewDimension",
            "type FermiDepth",
            "type ReviewConcern",
            "type DimensionReviewOutput",
            "type MergedReviewOutput",
            "type CredentialFlow",
            "type Arch",
            "type Vendor",
            "type Os",
            "type AbiEnv",
            "type ExecutionEnv",
            "type TargetTriple",
            "type RuntimePlatform",
            "type EntryKind",
            "type SymlinkTarget",
            "type ContentEncoding",
            "type TextFilePath",
            "type BinaryFilePath",
            "type FileClassification",
            "type MimeType",
            "type AuthScheme",
            "type AccessToken",
            "type CloudSecretConfig",
            "type Credential",
            "type TestResult",
            "type Summary",
            "type StageResult",
            "type DocumentLine",
            "type DocumentSection",
            "type Document",
            "type TextFile",
            "type RenderedTextFile",
            "type ToolEntry",
            "type ToolRegistry",
            "type DagTopology",
            "type TopologyNode",
            "type TopologyEdge",
            "type DagDiff",
            "type CodegenTarget",
            "type CodegenBackend",
            "type PragmaDirective",
            "type IssueLifecycleStage",
            "type TrackedIssue",
            "type DesignOutput",
            "type DesignSections",
            "type DesignReviewOutput",
            "type DesignFinding",
            "type ImplementationPlan",
            "type ImplementationTask",
            "type PipelineRun",
            "type PipelineArtifact",
            "type IssueState",
            "type BindingStatus",
            "type IntentSheet",
            "type IssueBinding",
            "type StageRunKey",
            "type ClaimLease",
            "type OutcomeStatus",
            "type StageOutcome",
            "type ArtifactType",
            "type ArtifactPayload",
            "type Artifact",
            "type MarkerKind",
            "type ArtifactMarker",
            "type SignalType",
            "type Signal",
            "type RuntimeProfile",
            "type LaunchConfig",
            "type InfraIntent",
            "type FailureClass",
            "type AgentStatus",
            "type ApprovalMode",
            "type CredentialIntent",
            "type CredentialResolution",
            "type CredentialBinding",
            "type ExecutionMetrics",
            "type AuditAction",
            "type AuditEntry",
            "type DocSource",
        ]
    );
}

#[test]
fn patterns_file_contains_pattern_defs() {
    let source = parse_dsl("std/patterns.dag");
    assert_eq!(
        source.items.len(),
        18,
        "std/patterns.dag item count changed unexpectedly"
    );
    assert_eq!(
        source
            .module_path
            .as_ref()
            .map(|module| module.node.segments.clone()),
        Some(vec!["std".into(), "patterns".into()])
    );
    assert!(source
        .items
        .iter()
        .any(|item| matches!(item.node, Item::PatternDef(_))));
    assert_eq!(
        item_signatures(&source),
        vec![
            "pattern file_content_matches",
            "pattern classify_files",
            "pattern read_text_files",
            "pattern read_binary_files",
            "pattern acquire_subject_token",
            "pattern optional_impersonation",
            "pattern ensure",
            "pattern upsert",
            "pattern content_upsert",
            "pattern credential_chain",
            "pattern transaction",
            "pattern retry",
            "func github_oidc",
            "func metadata_oidc",
            "func local_auth",
            "fn check_iam_binding",
            "fn add_iam_binding",
            "func iam_preflight_check",
        ]
    );
}

#[test]
fn shell_service_file_contains_service_defs() {
    let source = parse_dsl("extdeps/shell.dag");
    assert_eq!(
        source.items.len(),
        4,
        "extdeps/shell.dag item count changed unexpectedly"
    );
    assert_eq!(
        source
            .module_path
            .as_ref()
            .map(|module| module.node.segments.clone()),
        Some(vec!["extdeps".into(), "shell".into()])
    );
    assert!(source
        .items
        .iter()
        .any(|item| matches!(item.node, Item::ServiceDef(_))));
    assert_eq!(
        item_signatures(&source),
        vec![
            "service shell.Find",
            "service shell.Env",
            "service shell.Which",
            "service shell.Exec",
        ]
    );
}

#[test]
fn resources_file_contains_resource_defs() {
    let source = parse_dsl("std/resources.dag");
    assert_eq!(
        source.items.len(),
        4,
        "std/resources.dag item count changed unexpectedly"
    );
    assert_eq!(
        source
            .module_path
            .as_ref()
            .map(|module| module.node.segments.clone()),
        Some(vec!["std".into(), "resources".into()])
    );
    assert!(source
        .items
        .iter()
        .any(|item| matches!(item.node, Item::ResourceDef(_))));
    assert_eq!(
        item_signatures(&source),
        vec![
            "resource Filesystem",
            "resource Network",
            "resource Clock",
            "resource AuthContext",
        ]
    );
}
