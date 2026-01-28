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
        tool_name: "gitignoregen",
        blocks: vec![
            context_block(),
            check_block(),
            compose_block(),
            sink_block(true),
            resolve_block(),
        ],
        patterns: vec![gitignoregen_pattern()],
        type_domains: vec![
            (SpecTypeId("String"), DomainSpec::Scalar),
            (SpecTypeId("Bool"), DomainSpec::Scalar),
        ],
        ports: vec![
            // context outputs
            output_port("context", "file_path", None, produces_one),
            output_port("context", "force", None, produces_one),
            output_port("context", "input_hash", None, produces_one),

            // check inputs
            input_port("check", "file_path"),
            input_port("check", "force"),
            input_port("check", "input_hash"),

            // check outputs
            output_port("check", "input_hash", Some("file_path"), produces_one),
            output_port("check", "file_path", Some("file_path"), produces_one),
            output_port("check", "needs_write", Some("file_path"), produces_one),
            output_port("check", "file_existed", Some("file_path"), produces_one),

            // compose inputs
            input_port("compose", "input_hash"),

            // compose outputs
            output_port("compose", "content", Some("input_hash"), produces_one),

            // sink inputs
            input_port("sink", "content"),
            input_port("sink", "needs_write"),
            input_port("sink", "file_path"),
            input_port("sink", "file_existed"),

            // sink outputs
            output_port("sink", "write_status", Some("content"), produces_one),

            // resolve inputs
            input_port("resolve", "needs_write"),
            input_port("resolve", "write_status"),

            // resolve outputs
            output_port("resolve", "status", Some("write_status"), produces_one),
        ],
    }
}
