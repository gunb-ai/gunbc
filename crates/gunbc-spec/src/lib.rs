use gunbc_contracts::{BlockContract, PatternContract};
use gunbc_test::{Cardinality, ProducesCase, SetSpecCase};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpecNodeId(pub &'static str);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpecPortName(pub &'static str);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpecTypeId(pub &'static str);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainSpec {
    Scalar,
    OptionalScalar,
    Collection,
    OptionalCollection,
}

impl DomainSpec {
    pub fn cardinalities(self) -> &'static [Cardinality] {
        use Cardinality::*;
        match self {
            DomainSpec::Scalar => &[One],
            DomainSpec::OptionalScalar => &[Zero, One, Null],
            DomainSpec::Collection => &[Zero, One, N],
            DomainSpec::OptionalCollection => &[Zero, One, N, Null],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortDir {
    Input,
    Output,
}

#[derive(Clone, Copy)]
pub struct InputPortSpec {
    pub node: SpecNodeId,
    pub port: SpecPortName,
    pub domain_override: Option<DomainSpec>,
    pub accepts: Option<fn() -> Vec<Cardinality>>,
    pub rejects: Option<fn() -> Vec<Cardinality>>,
    pub set_cases: Option<fn() -> Vec<SetSpecCase>>,
}

#[derive(Clone, Copy)]
pub struct OutputPortSpec {
    pub node: SpecNodeId,
    pub port: SpecPortName,
    pub domain_override: Option<DomainSpec>,
    pub driver_input: Option<SpecPortName>,
    pub produces: fn() -> Vec<(Cardinality, ProducesCase)>,
}

#[derive(Clone, Copy)]
pub enum PortBehaviorSpec {
    Input(InputPortSpec),
    Output(OutputPortSpec),
}

impl PortBehaviorSpec {
    pub fn dir(&self) -> PortDir {
        match self {
            PortBehaviorSpec::Input(_) => PortDir::Input,
            PortBehaviorSpec::Output(_) => PortDir::Output,
        }
    }

    pub fn node(&self) -> SpecNodeId {
        match self {
            PortBehaviorSpec::Input(spec) => spec.node,
            PortBehaviorSpec::Output(spec) => spec.node,
        }
    }

    pub fn port(&self) -> SpecPortName {
        match self {
            PortBehaviorSpec::Input(spec) => spec.port,
            PortBehaviorSpec::Output(spec) => spec.port,
        }
    }
}

pub struct ToolSpec {
    pub tool_name: &'static str,
    pub patterns: Vec<PatternContract>,
    pub blocks: Vec<BlockContract>,
    pub type_domains: Vec<(SpecTypeId, DomainSpec)>,
    pub ports: Vec<PortBehaviorSpec>,
}
