//! CI rendering from `WorkflowSpec` graphs.
//!
//! This module provides a workflow-model bridge to CI YAML generation.

use crate::makegen::registry::WorkflowSpec;
use gunbc_ir::CargoInvocation;
use gunbc_ir::transport::ci::{CiRenderer, GitHubActionsProvider, GitLabCiProvider, RenderConfig};
use gunbc_ir::{Dag, Edge, Node, Port};
use std::collections::BTreeSet;

/// Build a DAG from workflow specs where each workflow is one node and each
/// dependency is one directed edge.
pub fn workflow_specs_to_dag(specs: &[WorkflowSpec]) -> Dag<()> {
    let mut dag = Dag::new();

    let mut sorted = specs.iter().collect::<Vec<_>>();
    sorted.sort_by(|a, b| a.name.cmp(&b.name));

    for spec in &sorted {
        let inputs = spec
            .deps
            .iter()
            .map(|dep| Port::scalar(dep.as_str(), "Bool"))
            .collect::<Vec<_>>();
        dag.add_node(Node::opaque(
            spec.name.as_str(),
            inputs,
            vec![Port::scalar("ok", "Bool")],
            (),
        ));
    }

    let names = sorted
        .iter()
        .map(|spec| spec.name.clone())
        .collect::<BTreeSet<_>>();

    for spec in &sorted {
        for dep in &spec.deps {
            if names.contains(dep) {
                dag.add_edge(Edge::new(
                    dep.as_str(),
                    "ok",
                    spec.name.as_str(),
                    dep.as_str(),
                ));
            }
        }
    }

    dag
}

/// Render a GitHub Actions workflow YAML from workflow specs.
pub fn render_github_actions_from_workflow_specs(
    workflow_name: &str,
    specs: &[WorkflowSpec],
) -> String {
    let dag = workflow_specs_to_dag(specs);
    let mut config = RenderConfig::new(workflow_name, CargoInvocation::composed("ci", "dag"));
    for secret in workflow_live_secrets(specs) {
        config = config.with_env(
            secret.as_str(),
            format!("${{{{ secrets.{secret} }}}}").as_str(),
        );
    }
    GitHubActionsProvider.render(&dag, &config)
}

/// Render a GitLab CI workflow YAML from workflow specs.
pub fn render_gitlab_ci_from_workflow_specs(workflow_name: &str, specs: &[WorkflowSpec]) -> String {
    let dag = workflow_specs_to_dag(specs);
    let mut config = RenderConfig::new(workflow_name, CargoInvocation::composed("ci", "dag"));
    for secret in workflow_live_secrets(specs) {
        config = config.with_env(secret.as_str(), format!("${secret}").as_str());
    }
    GitLabCiProvider::default().render(&dag, &config)
}

fn workflow_live_secrets(specs: &[WorkflowSpec]) -> Vec<String> {
    let mut secrets = BTreeSet::new();
    for spec in specs {
        for secret in &spec.live_secrets {
            secrets.insert(secret.clone());
        }
    }
    secrets.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::makegen::registry::WorkflowKind;
    use gunbc_ir::transport::ci::{dag_to_shared_steps, SharedStep};
    use std::collections::BTreeMap;

    fn sample_specs() -> Vec<WorkflowSpec> {
        vec![
            WorkflowSpec {
                name: "build".to_string(),
                description: "Build".to_string(),
                kind: WorkflowKind::Core,
                entrypoints: Vec::new(),
                deps: Vec::new(),
                resources: Vec::new(),
                live_secrets: vec!["GCP_PROJECT_ID".to_string()],
            },
            WorkflowSpec {
                name: "test".to_string(),
                description: "Test".to_string(),
                kind: WorkflowKind::Core,
                entrypoints: Vec::new(),
                deps: vec!["build".to_string()],
                resources: Vec::new(),
                live_secrets: vec!["GCP_PROJECT_ID".to_string(), "API_TOKEN".to_string()],
            },
            WorkflowSpec {
                name: "lint".to_string(),
                description: "Lint".to_string(),
                kind: WorkflowKind::Meta,
                entrypoints: Vec::new(),
                deps: vec!["build".to_string()],
                resources: Vec::new(),
                live_secrets: Vec::new(),
            },
        ]
    }

    #[test]
    fn workflow_specs_to_dag_preserves_dependency_edges() {
        let dag = workflow_specs_to_dag(&sample_specs());
        let edges = dag
            .edges
            .iter()
            .map(|e| (e.from_node.0.clone(), e.to_node.0.clone()))
            .collect::<BTreeSet<_>>();
        assert!(edges.contains(&("build".to_string(), "test".to_string())));
        assert!(edges.contains(&("build".to_string(), "lint".to_string())));
    }

    #[test]
    fn shared_steps_dependencies_match_workflow_specs() {
        let specs = sample_specs();
        let dag = workflow_specs_to_dag(&specs);
        let config = RenderConfig::new("ci", CargoInvocation::composed("ci", "dag"));
        let steps = dag_to_shared_steps(&dag, &config);

        let expected = specs
            .iter()
            .map(|spec| {
                let deps = spec.deps.iter().cloned().collect::<BTreeSet<_>>();
                (spec.name.clone(), deps)
            })
            .collect::<BTreeMap<_, _>>();

        let mut actual: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for step in steps {
            if let SharedStep::DagStep {
                node_id,
                depends_on,
                ..
            } = step
            {
                actual.insert(
                    node_id.0.clone(),
                    depends_on.into_iter().map(|id| id.0).collect(),
                );
            }
        }

        assert_eq!(actual, expected);
    }

    #[test]
    fn github_render_includes_dependency_and_secret_bindings() {
        let yaml = render_github_actions_from_workflow_specs("ci", &sample_specs());
        assert!(yaml.contains("id: test"));
        assert!(yaml.contains("steps.build.outputs"));
        assert!(yaml.contains("GCP_PROJECT_ID: ${{ secrets.GCP_PROJECT_ID }}"));
        assert!(yaml.contains("API_TOKEN: ${{ secrets.API_TOKEN }}"));
    }

    #[test]
    fn gitlab_render_includes_needs_and_secret_variables() {
        let yaml = render_gitlab_ci_from_workflow_specs("ci", &sample_specs());
        assert!(yaml.contains("test:"));
        assert!(yaml.contains("needs:"));
        assert!(yaml.contains("- build"));
        assert!(yaml.contains("GCP_PROJECT_ID: \"$GCP_PROJECT_ID\""));
        assert!(yaml.contains("API_TOKEN: \"$API_TOKEN\""));
    }
}
