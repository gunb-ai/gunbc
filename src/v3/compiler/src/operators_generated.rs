// AUTO-GENERATED from `src/v3/compiler/operators.dag` via
// `emit_rust_module`. Regenerate instead of hand-editing.

pub fn from_symbol(p0: &str) -> Option<OperatorKind> {
    if p0 == "+" {
        Some(OperatorKind::Arithmetic(ArithmeticOp::Add))
    } else if p0 == "-" {
        Some(OperatorKind::Arithmetic(ArithmeticOp::Sub))
    } else if p0 == "*" {
        Some(OperatorKind::Arithmetic(ArithmeticOp::Mul))
    } else if p0 == "/" {
        Some(OperatorKind::Arithmetic(ArithmeticOp::Div))
    } else if p0 == "==" {
        Some(OperatorKind::Comparison(ComparisonOp::Eq))
    } else if p0 == "!=" {
        Some(OperatorKind::Comparison(ComparisonOp::Ne))
    } else if p0 == "<" {
        Some(OperatorKind::Comparison(ComparisonOp::Lt))
    } else if p0 == "<=" {
        Some(OperatorKind::Comparison(ComparisonOp::Le))
    } else if p0 == ">" {
        Some(OperatorKind::Comparison(ComparisonOp::Gt))
    } else if p0 == ">=" {
        Some(OperatorKind::Comparison(ComparisonOp::Ge))
    } else if p0 == "&&" {
        Some(OperatorKind::Logical(LogicalOp::And))
    } else if p0 == "||" {
        Some(OperatorKind::Logical(LogicalOp::Or))
    } else {
        None
    }
}

pub fn symbol(p0: OperatorKind) -> String {
    match p0 {
        OperatorKind::Arithmetic(ArithmeticOp::Add) => String::from("+"),
        OperatorKind::Arithmetic(ArithmeticOp::Sub) => String::from("-"),
        OperatorKind::Arithmetic(ArithmeticOp::Mul) => String::from("*"),
        OperatorKind::Arithmetic(ArithmeticOp::Div) => String::from("/"),
        OperatorKind::Comparison(ComparisonOp::Eq) => String::from("=="),
        OperatorKind::Comparison(ComparisonOp::Ne) => String::from("!="),
        OperatorKind::Comparison(ComparisonOp::Lt) => String::from("<"),
        OperatorKind::Comparison(ComparisonOp::Le) => String::from("<="),
        OperatorKind::Comparison(ComparisonOp::Gt) => String::from(">"),
        OperatorKind::Comparison(ComparisonOp::Ge) => String::from(">="),
        OperatorKind::Logical(LogicalOp::And) => String::from("&&"),
        OperatorKind::Logical(LogicalOp::Or) => String::from("||"),
    }
}

pub fn algebra_field_name(p0: OperatorKind) -> String {
    match p0 {
        OperatorKind::Arithmetic(ArithmeticOp::Add) => String::from("add"),
        OperatorKind::Arithmetic(ArithmeticOp::Sub) => String::from("sub"),
        OperatorKind::Arithmetic(ArithmeticOp::Mul) => String::from("mul"),
        OperatorKind::Arithmetic(ArithmeticOp::Div) => String::from("div"),
        OperatorKind::Comparison(ComparisonOp::Eq) => String::from("eq"),
        OperatorKind::Comparison(ComparisonOp::Ne) => String::from("ne"),
        OperatorKind::Comparison(ComparisonOp::Lt) => String::from("lt"),
        OperatorKind::Comparison(ComparisonOp::Le) => String::from("le"),
        OperatorKind::Comparison(ComparisonOp::Gt) => String::from("gt"),
        OperatorKind::Comparison(ComparisonOp::Ge) => String::from("ge"),
        OperatorKind::Logical(LogicalOp::And) => String::from("meet"),
        OperatorKind::Logical(LogicalOp::Or) => String::from("join"),
    }
}
