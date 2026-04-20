// AUTO-GENERATED from `src/v3/compiler/runtime_mirrors.dag`.
// Regenerate instead of hand-editing.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DagDifference {
    pub detail: String,
}

pub fn serialize_dag(dag: &Dag) -> Vec<u8> {
    let mut out = String::new();
    for declaration in dag.declarations() {
        out.push_str(&format!(
            "DECL {} {:?}\n",
            declaration.id.raw(),
            declaration
        ));
    }
    for behavior in dag.nodes() {
        out.push_str(&serialize_behavior(behavior));
    }
    for port in dag.ports() {
        out.push_str(&format!("PORT {} {:?}\n", port.id().raw(), port));
    }
    let mut diagnostics: Vec<_> = dag.diagnostics().iter().collect();
    diagnostics.sort_by_key(|(port, _)| port.raw());
    for (port, diagnostic) in diagnostics {
        out.push_str(&format!(
            "DIAG port={} {}\n",
            port.raw(),
            render_diagnostic(diagnostic)
        ));
    }
    out.into_bytes()
}

pub fn first_difference(lhs: &Dag, rhs: &Dag) -> Option<DagDifference> {
    let lhs_decls = lhs.declarations();
    let rhs_decls = rhs.declarations();
    if lhs_decls.len() != rhs_decls.len() {
        return Some(DagDifference {
            detail: format!(
                "declaration count mismatch: pass1={}, pass2={}",
                lhs_decls.len(),
                rhs_decls.len()
            ),
        });
    }
    for (left, right) in lhs_decls.iter().zip(rhs_decls.iter()) {
        if format!("{left:?}") != format!("{right:?}") {
            let name = left
                .name
                .as_deref()
                .or(right.name.as_deref())
                .unwrap_or("<anonymous>");
            return Some(DagDifference {
                detail: format!(
                    "declaration {} `{}` diverged: pass1=`{:?}`, pass2=`{:?}`",
                    left.id.raw(),
                    name,
                    left,
                    right
                ),
            });
        }
    }

    let lhs_nodes = lhs.nodes();
    let rhs_nodes = rhs.nodes();
    if lhs_nodes.len() != rhs_nodes.len() {
        return Some(DagDifference {
            detail: format!(
                "behavior count mismatch: pass1={}, pass2={}",
                lhs_nodes.len(),
                rhs_nodes.len()
            ),
        });
    }
    for (left, right) in lhs_nodes.iter().zip(rhs_nodes.iter()) {
        if format!("{left:?}") != format!("{right:?}") {
            return Some(DagDifference {
                detail: format!("behavior diverged: pass1=`{:?}`, pass2=`{:?}`", left, right),
            });
        }
    }

    let lhs_ports = lhs.ports();
    let rhs_ports = rhs.ports();
    if lhs_ports.len() != rhs_ports.len() {
        return Some(DagDifference {
            detail: format!(
                "port count mismatch: pass1={}, pass2={}",
                lhs_ports.len(),
                rhs_ports.len()
            ),
        });
    }
    for (left, right) in lhs_ports.iter().zip(rhs_ports.iter()) {
        if format!("{left:?}") != format!("{right:?}") {
            return Some(DagDifference {
                detail: format!("port diverged: pass1=`{:?}`, pass2=`{:?}`", left, right),
            });
        }
    }

    let mut lhs_diags: Vec<_> = lhs.diagnostics().iter().collect();
    let mut rhs_diags: Vec<_> = rhs.diagnostics().iter().collect();
    lhs_diags.sort_by_key(|(port, _)| port.raw());
    rhs_diags.sort_by_key(|(port, _)| port.raw());
    if lhs_diags.len() != rhs_diags.len() {
        return Some(DagDifference {
            detail: format!(
                "diagnostic count mismatch: pass1={}, pass2={}",
                lhs_diags.len(),
                rhs_diags.len()
            ),
        });
    }
    for ((left_port, left_diag), (right_port, right_diag)) in lhs_diags.iter().zip(rhs_diags.iter())
    {
        let left = format!(
            "DIAG port={} {}",
            left_port.raw(),
            render_diagnostic(left_diag)
        );
        let right = format!(
            "DIAG port={} {}",
            right_port.raw(),
            render_diagnostic(right_diag)
        );
        if left != right {
            return Some(DagDifference {
                detail: format!("diagnostic diverged: pass1=`{left}`, pass2=`{right}`"),
            });
        }
    }

    None
}

fn serialize_behavior(behavior: &Behavior) -> String {
    format!("BEHAV {:?}\n", behavior)
}

fn render_diagnostic(diagnostic: &Diagnostic) -> String {
    format!("{diagnostic:?}")
}
