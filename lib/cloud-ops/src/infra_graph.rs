//! Infrastructure spec visualization helpers.

use crate::infra_spec::InfraSpec;
use crate::project_spec::{RotationHandler, SecretStatus};

/// Render an `InfraSpec` dependency graph in DOT format.
pub fn render_infra_spec_dot(spec: &InfraSpec) -> String {
    let mut lines = vec![
        "digraph InfraSpec {".to_string(),
        "  rankdir=LR;".to_string(),
        format!(
            "  env [shape=box, style=filled, fillcolor=lightblue, label=\"env:{}\\nproject:{}\\nregion:{}\\nzone:{}\"];",
            spec.environment, spec.config.project, spec.config.region, spec.config.zone
        ),
        format!(
            "  wif [shape=component, label=\"wif:{}:{}\"];",
            spec.wif.pool_id, spec.wif.provider_id
        ),
    ];

    for sa in spec.service_accounts {
        let sa_node = format!("sa_{}", sa.name.replace('-', "_"));
        lines.push(format!(
            "  {} [shape=oval, label=\"sa:{}\"];",
            sa_node, sa.name
        ));
        lines.push(format!("  env -> {} [label=\"provisions\"]; ", sa_node));
        lines.push(format!("  wif -> {} [label=\"impersonates\"]; ", sa_node));
    }

    for secret in spec
        .secrets
        .iter()
        .filter(|s| s.status == SecretStatus::Active)
    {
        let secret_id = format!("{}{}", spec.config.secrets_prefix, secret.secret_id);
        let secret_node = format!("secret_{}", secret.secret_id.replace('-', "_"));
        lines.push(format!(
            "  {} [shape=folder, label=\"secret:{}\"];",
            secret_node, secret_id
        ));
        for sa in spec.service_accounts {
            let sa_node = format!("sa_{}", sa.name.replace('-', "_"));
            lines.push(format!(
                "  {} -> {} [label=\"accesses\"]; ",
                sa_node, secret_node
            ));
        }

        match secret.rotation {
            RotationHandler::None => {}
            handler => {
                let rotation_node = format!("rotation_{}", secret.secret_id.replace('-', "_"));
                lines.push(format!(
                    "  {} [shape=note, label=\"rotation:{:?}\"];",
                    rotation_node, handler
                ));
                lines.push(format!(
                    "  {} -> {} [label=\"managed_by\"]; ",
                    secret_node, rotation_node
                ));
            }
        }
    }

    lines.push("}".to_string());
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra_spec::DEV_SPEC;

    #[test]
    fn render_infra_spec_dot_contains_core_nodes_and_edges() {
        let dot = render_infra_spec_dot(&DEV_SPEC);
        assert!(dot.contains("digraph InfraSpec"));
        assert!(dot.contains("env:dev"));
        assert!(dot.contains("wif:github-pool:github"));
        assert!(dot.contains("sa:gunbai-dev-secrets"));
        assert!(dot.contains("secret:dev-github-token"));
        assert!(dot.contains("accesses"));
        assert!(dot.contains("managed_by"));
    }
}
