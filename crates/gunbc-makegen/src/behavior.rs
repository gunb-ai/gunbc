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

fn produces_one() -> Vec<(Cardinality, ProducesCase)> {
    vec![(Cardinality::One, ProducesCase::Ok(Cardinality::One))]
}

pub fn tool_spec() -> ToolSpec {
    ToolSpec {
        tool_name: "makegen",
        blocks: vec![
            context_block(),
            check_block(),
            compose_block(),
            resolve_block(),
            sink_block(true),
        ],
        patterns: vec![makegen_pattern()],
        type_domains: vec![
            (SpecTypeId("String"), DomainSpec::Scalar),
            (SpecTypeId("Bool"), DomainSpec::Scalar),
        ],
        ports: vec![
            // context outputs
            output_port("context", "workspace_path", None, produces_one),
            output_port("context", "output_path", None, produces_one),
            output_port("context", "force", None, produces_one),

            // check inputs
            input_port("check", "workspace_path"),
            input_port("check", "output_path"),
            input_port("check", "force"),

            // check outputs
            output_port("check", "input_hash", Some("workspace_path"), produces_one),
            output_port("check", "makefile_path", Some("workspace_path"), produces_one),
            output_port("check", "needs_generate", Some("workspace_path"), produces_one),
            output_port("check", "file_exists", Some("workspace_path"), produces_one),

            // compose inputs
            input_port("compose", "input_hash"),

            // compose outputs
            output_port("compose", "content", Some("input_hash"), produces_one),

            // resolve inputs
            input_port("resolve", "content"),
            input_port("resolve", "input_hash"),
            input_port("resolve", "makefile_path"),
            input_port("resolve", "needs_generate"),
            input_port("resolve", "file_exists"),

            // resolve outputs
            output_port("resolve", "content", Some("content"), produces_one),
            output_port("resolve", "hash", Some("content"), produces_one),
            output_port("resolve", "needs_write", Some("content"), produces_one),
            output_port("resolve", "makefile_path", Some("content"), produces_one),
            output_port("resolve", "file_existed", Some("content"), produces_one),

            // sink inputs
            input_port("sink", "content"),
            input_port("sink", "needs_write"),
            input_port("sink", "makefile_path"),
            input_port("sink", "file_existed"),

            // sink output
            output_port("sink", "status", Some("content"), produces_one),
        ],
    }
}
