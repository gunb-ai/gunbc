use gunbc_spec::{
    DomainSpec, InputPortSpec, OutputPortSpec, PortBehaviorSpec, SpecNodeId, SpecPortName,
    SpecTypeId, ToolSpec,
};
use gunbc_test::{Cardinality, ProducesCase};

use crate::contracts::*;

fn input_port(node: &'static str, port: &'static str) -> PortBehaviorSpec {
    PortBehaviorSpec::Input(InputPortSpec {
        node: SpecNodeId(node),
        port: SpecPortName(port),
        domain_override: None,
        accepts: None,
        rejects: None,
        set_cases: None,
    })
}

fn input_port_with(
    node: &'static str,
    port: &'static str,
    domain_override: Option<DomainSpec>,
    accepts: Option<fn() -> Vec<Cardinality>>,
    rejects: Option<fn() -> Vec<Cardinality>>,
) -> PortBehaviorSpec {
    PortBehaviorSpec::Input(InputPortSpec {
        node: SpecNodeId(node),
        port: SpecPortName(port),
        domain_override,
        accepts,
        rejects,
        set_cases: None,
    })
}

fn output_port(
    node: &'static str,
    port: &'static str,
    driver_input: Option<&'static str>,
    produces: fn() -> Vec<(Cardinality, ProducesCase)>,
) -> PortBehaviorSpec {
    PortBehaviorSpec::Output(OutputPortSpec {
        node: SpecNodeId(node),
        port: SpecPortName(port),
        domain_override: None,
        driver_input: driver_input.map(SpecPortName),
        produces,
    })
}

fn output_port_with(
    node: &'static str,
    port: &'static str,
    domain_override: Option<DomainSpec>,
    driver_input: Option<&'static str>,
    produces: fn() -> Vec<(Cardinality, ProducesCase)>,
) -> PortBehaviorSpec {
    PortBehaviorSpec::Output(OutputPortSpec {
        node: SpecNodeId(node),
        port: SpecPortName(port),
        domain_override,
        driver_input: driver_input.map(SpecPortName),
        produces,
    })
}

fn produces_one() -> Vec<(Cardinality, ProducesCase)> {
    vec![(Cardinality::One, ProducesCase::Ok(Cardinality::One))]
}

fn produces_collection_identity() -> Vec<(Cardinality, ProducesCase)> {
    vec![
        (Cardinality::Zero, ProducesCase::Ok(Cardinality::Zero)),
        (Cardinality::One, ProducesCase::Ok(Cardinality::One)),
        (Cardinality::N, ProducesCase::Ok(Cardinality::N)),
        (Cardinality::Null, ProducesCase::Err),
    ]
}

fn produces_collection_any_from_scalar() -> Vec<(Cardinality, ProducesCase)> {
    vec![
        (Cardinality::One, ProducesCase::Ok(Cardinality::Zero)),
        (Cardinality::One, ProducesCase::Ok(Cardinality::One)),
        (Cardinality::One, ProducesCase::Ok(Cardinality::N)),
        (Cardinality::Null, ProducesCase::Err),
    ]
}

fn produces_filter_files() -> Vec<(Cardinality, ProducesCase)> {
    vec![
        (Cardinality::Zero, ProducesCase::Ok(Cardinality::Zero)),
        (Cardinality::One, ProducesCase::Ok(Cardinality::Zero)),
        (Cardinality::One, ProducesCase::Ok(Cardinality::One)),
        (Cardinality::N, ProducesCase::Ok(Cardinality::Zero)),
        (Cardinality::N, ProducesCase::Ok(Cardinality::One)),
        (Cardinality::N, ProducesCase::Ok(Cardinality::N)),
        (Cardinality::Null, ProducesCase::Err),
    ]
}

fn produces_snapshot_from_contents() -> Vec<(Cardinality, ProducesCase)> {
    vec![
        (Cardinality::Zero, ProducesCase::Ok(Cardinality::One)),
        (Cardinality::One, ProducesCase::Ok(Cardinality::One)),
        (Cardinality::N, ProducesCase::Ok(Cardinality::One)),
        (Cardinality::Null, ProducesCase::Err),
    ]
}

fn produces_files_from_snapshot() -> Vec<(Cardinality, ProducesCase)> {
    vec![
        (Cardinality::One, ProducesCase::Ok(Cardinality::One)),
        (Cardinality::Null, ProducesCase::Err),
    ]
}

fn produces_build_gist_request() -> Vec<(Cardinality, ProducesCase)> {
    vec![
        (Cardinality::Zero, ProducesCase::Err),
        (Cardinality::One, ProducesCase::Ok(Cardinality::One)),
        (Cardinality::N, ProducesCase::Ok(Cardinality::One)),
        (Cardinality::Null, ProducesCase::Err),
    ]
}

fn produces_gist_api() -> Vec<(Cardinality, ProducesCase)> {
    vec![
        (Cardinality::Zero, ProducesCase::Err),
        (Cardinality::One, ProducesCase::Ok(Cardinality::One)),
        (Cardinality::N, ProducesCase::Ok(Cardinality::One)),
        (Cardinality::Null, ProducesCase::Err),
    ]
}

fn accepts_nonempty_collection() -> Vec<Cardinality> {
    vec![Cardinality::One, Cardinality::N]
}

fn rejects_empty_and_null() -> Vec<Cardinality> {
    vec![Cardinality::Zero, Cardinality::Null]
}

pub fn tool_spec() -> ToolSpec {
    ToolSpec {
        tool_name: "gistgen",
        blocks: vec![
            auth_check(),
            auth_create(),
            auth_resolve(),
            auth_block(),
            context_block(),
            enumerate_files_block(),
            filter_files_block(),
            read_files_block(),
            compose_snapshot_block(),
            wrap_single_gist_file_block(),
            compose_gist_files_block(),
            build_gist_request_block(),
            gist_block(),
        ],
        patterns: vec![
            upsert_pattern(),
            gistgen_pattern_single_file(),
            gistgen_pattern_file_map(),
        ],
        type_domains: vec![
            (SpecTypeId("String"), DomainSpec::Scalar),
            (SpecTypeId("Bool"), DomainSpec::Scalar),
            (SpecTypeId("I64"), DomainSpec::Scalar),
            (SpecTypeId("Secret"), DomainSpec::Scalar),
            (SpecTypeId("StrList"), DomainSpec::Collection),
            (SpecTypeId("MapStrStr"), DomainSpec::Collection),
            (SpecTypeId("GitHub::Gist::CreateRequest"), DomainSpec::Scalar),
            (SpecTypeId("GitHub::Gist::CreateResponse"), DomainSpec::Scalar),
        ],
        ports: vec![
            // Auth pattern
            output_port("auth_check", "token", None, produces_one),
            output_port("auth_check", "needs_create", None, produces_one),
            input_port("auth_create", "needs_create"),
            output_port("auth_create", "token", Some("needs_create"), produces_one),
            input_port("auth_resolve", "check_token"),
            input_port("auth_resolve", "create_token"),
            output_port("auth_resolve", "token", Some("check_token"), produces_one),

            // Gistgen wrapper
            output_port("auth", "token", None, produces_one),

            // Context
            output_port("context", "repo", None, produces_one),
            output_port("context", "selection_spec", None, produces_one),

            // Enumerate/Filter/Read
            input_port("enumerate_files", "repo"),
            output_port(
                "enumerate_files",
                "files",
                Some("repo"),
                produces_collection_any_from_scalar,
            ),

            input_port("filter_files", "selection_spec"),
            input_port("filter_files", "files"),
            output_port("filter_files", "files", Some("files"), produces_filter_files),

            input_port("read_files", "repo"),
            input_port("read_files", "files"),
            output_port("read_files", "contents", Some("files"), produces_collection_identity),

            // Snapshot / gist files
            input_port("compose_snapshot", "contents"),
            output_port(
                "compose_snapshot",
                "snapshot",
                Some("contents"),
                produces_snapshot_from_contents,
            ),

            input_port("wrap_single_gist_file", "snapshot"),
            output_port(
                "wrap_single_gist_file",
                "files",
                Some("snapshot"),
                produces_files_from_snapshot,
            ),

            input_port("compose_gist_files", "contents"),
            output_port(
                "compose_gist_files",
                "files",
                Some("contents"),
                produces_collection_identity,
            ),

            // Build request
            input_port_with(
                "build_gist_request",
                "files",
                None,
                Some(accepts_nonempty_collection),
                Some(rejects_empty_and_null),
            ),
            output_port_with(
                "build_gist_request",
                "request",
                Some(DomainSpec::Collection),
                Some("files"),
                produces_build_gist_request,
            ),

            // Gist transport node
            input_port_with(
                "gist",
                "request",
                Some(DomainSpec::Collection),
                Some(accepts_nonempty_collection),
                Some(rejects_empty_and_null),
            ),
            input_port("gist", "token"),
            output_port("gist", "gist_url", Some("request"), produces_gist_api),
            output_port("gist", "response", Some("request"), produces_gist_api),
        ],
    }
}
