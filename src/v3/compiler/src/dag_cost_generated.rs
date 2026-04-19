// AUTO-GENERATED from `src/v3/std/algebra.dag`.
// Regenerate instead of hand-editing.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DegreeAtLeastTwo {
    DegreeTwo,
    DegreeSuccessor { previous: Box<DegreeAtLeastTwo> },
}

impl DegreeAtLeastTwo {
    pub const TWO: Self = Self::DegreeTwo;

    pub fn new(value: i64) -> Option<Self> {
        match value {
            2 => Some(Self::DegreeTwo),
            v if v > 2 => Some(Self::DegreeSuccessor {
                previous: Box::new(Self::new(v - 1)?),
            }),
            _ => None,
        }
    }

    pub fn raw(&self) -> i64 {
        match self {
            Self::DegreeTwo => 2,
            Self::DegreeSuccessor { previous } => previous.raw() + 1,
        }
    }
}

type BoxedSymbolicCostList = NonSingletonList<Box<SymbolicCost>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolicCost {
    ConstantCost { _0: i64 },
    LinearCost { _0: SizeVariable },
    PolynomialCost {
        var: SizeVariable,
        degree: DegreeAtLeastTwo,
    },
    ProductCost { _0: BoxedSymbolicCostList },
    SumCost { _0: BoxedSymbolicCostList },
    LogCost { _0: SizeVariable },
    UnknownCost { _0: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SizeVariable {
    pub source_port: PortId,
}

pub fn sequential(a: SymbolicCost, b: SymbolicCost) -> SymbolicCost {
    normalize(SymbolicCost::SumCost {
        _0: boxed_cost_list_pair(a, b),
    })
}

pub fn iterate(bound: SymbolicCost, body: SymbolicCost) -> SymbolicCost {
    normalize(SymbolicCost::ProductCost {
        _0: boxed_cost_list_pair(bound, body),
    })
}

pub fn max_path(paths: &[SymbolicCost]) -> SymbolicCost {
    paths
        .iter()
        .fold(SymbolicCost::ConstantCost { _0: 0 }, |acc, candidate| {
            if dominates(candidate, &acc) {
                candidate.clone()
            } else if dominates(&acc, candidate) {
                acc
            } else {
                sequential(acc, candidate.clone())
            }
        })
}

pub fn normalize(cost: SymbolicCost) -> SymbolicCost {
    match cost {
        SymbolicCost::SumCost { _0: terms } => {
            reduce_sum(drop_zero_terms(boxed_terms_to_vec(&terms)))
        }
        SymbolicCost::ProductCost { _0: terms } => {
            reduce_product(drop_zero_terms(boxed_terms_to_vec(&terms)))
        }
        other => other,
    }
}

fn boxed_cost_list_pair(a: SymbolicCost, b: SymbolicCost) -> BoxedSymbolicCostList {
    BoxedSymbolicCostList {
        first: Box::new(a),
        second: Box::new(b),
        rest: Vec::new(),
    }
}

fn boxed_terms_to_vec(terms: &BoxedSymbolicCostList) -> Vec<SymbolicCost> {
    terms.iter().map(|term| term.as_ref().clone()).collect()
}

fn drop_zero_terms(terms: Vec<SymbolicCost>) -> Vec<SymbolicCost> {
    terms
        .into_iter()
        .filter(|t| !matches!(t, SymbolicCost::ConstantCost { _0: 0 }))
        .collect()
}

fn reduce_sum(mut terms: Vec<SymbolicCost>) -> SymbolicCost {
    terms = drop_dominated_in_sum(terms);
    match terms.len() {
        0 => SymbolicCost::ConstantCost { _0: 0 },
        1 => terms.into_iter().next().unwrap(),
        _ => SymbolicCost::SumCost {
            _0: boxed_cost_list_from_vec(terms),
        },
    }
}

fn reduce_product(terms: Vec<SymbolicCost>) -> SymbolicCost {
    match terms.len() {
        0 => SymbolicCost::ConstantCost { _0: 0 },
        1 => terms.into_iter().next().unwrap(),
        2 => {
            let mut iter = terms.into_iter();
            let a = iter.next().unwrap();
            let b = iter.next().unwrap();
            combine_binary_product(a, b)
        }
        _ => SymbolicCost::ProductCost {
            _0: boxed_cost_list_from_vec(terms),
        },
    }
}

fn boxed_cost_list_from_vec(terms: Vec<SymbolicCost>) -> BoxedSymbolicCostList {
    NonSingletonList::from_vec(terms.into_iter().map(Box::new).collect()).unwrap()
}

fn combine_binary_product(a: SymbolicCost, b: SymbolicCost) -> SymbolicCost {
    if let (SymbolicCost::LinearCost { _0: va }, SymbolicCost::LinearCost { _0: vb }) = (&a, &b) {
        if va == vb {
            return SymbolicCost::PolynomialCost {
                var: va.clone(),
                degree: DegreeAtLeastTwo::TWO,
            };
        }
    }
    SymbolicCost::ProductCost {
        _0: boxed_cost_list_pair(a, b),
    }
}

fn drop_dominated_in_sum(terms: Vec<SymbolicCost>) -> Vec<SymbolicCost> {
    let mut keep: Vec<SymbolicCost> = Vec::with_capacity(terms.len());
    for term in terms {
        let term_dominated = keep.iter().any(|k| dominates(k, &term));
        if term_dominated {
            continue;
        }
        keep.retain(|k| !dominates(&term, k));
        keep.push(term);
    }
    keep
}

pub fn dominates(a: &SymbolicCost, b: &SymbolicCost) -> bool {
    match a {
        SymbolicCost::UnknownCost { .. } => true,
        SymbolicCost::ConstantCost { .. } => matches!(b, SymbolicCost::ConstantCost { .. }),
        SymbolicCost::LinearCost { _0: va } => match b {
            SymbolicCost::ConstantCost { .. } | SymbolicCost::LogCost { .. } => true,
            SymbolicCost::LinearCost { _0: vb } => va == vb,
            SymbolicCost::PolynomialCost { var: _, degree: _ } => false,
            _ => false,
        },
        SymbolicCost::PolynomialCost {
            var: va,
            degree: ka,
        } => match b {
            SymbolicCost::ConstantCost { .. } | SymbolicCost::LogCost { .. } => true,
            SymbolicCost::LinearCost { _0: vb } => va == vb,
            SymbolicCost::PolynomialCost {
                var: vb,
                degree: kb,
            } => va == vb && ka.raw() >= kb.raw(),
            _ => false,
        },
        SymbolicCost::LogCost { _0: va } => match b {
            SymbolicCost::ConstantCost { .. } => true,
            SymbolicCost::LogCost { _0: vb } => va == vb,
            _ => false,
        },
        SymbolicCost::ProductCost { _0: terms } | SymbolicCost::SumCost { _0: terms } => {
            terms.iter().any(|child| dominates(child.as_ref(), b))
        }
    }
}

