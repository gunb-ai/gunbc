# H9 Design: Render Workflows as DAGs to GitHub Actions

## Problem

CI generation is currently provider-specific and not derived from the shared workflow graph model.

## Decision

Use GitHub Actions as the first additional CI provider generated from `WorkflowSpec`.

## Proposed Mapping

- `WorkflowSpec` -> one workflow YAML file.
- `WorkflowTarget` -> one job or step group.
- DAG dependencies -> `needs` graph.
- resource/credential requirements -> `permissions`, `secrets`, and env bindings.

## Invariants

- No implicit secret injection; all secrets must be declared in workflow metadata.
- Job graph must remain acyclic and match source DAG dependencies.
- Provider renderer cannot mutate semantic meaning of the workflow model.

## Migration Plan

1. Add CI-neutral renderer interface.
2. Implement GitHub Actions renderer.
3. Add schema validation for generated YAML.
4. Add parity checks between CLI execution graph and CI graph.

## Follow-up Implementation Tasks

- `H9.1` Define CI renderer interface and shared metadata.
- `H9.2` Implement GitHub Actions YAML renderer.
- `H9.3` Add static validation (`actionlint`/schema check).
- `H9.4` Add graph-parity tests with `WorkflowSpec` dependencies.
- `H9.5` Add fixtures for secret and permission handling.
