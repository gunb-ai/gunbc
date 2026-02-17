# Workflow Contract Fixtures

These JSON files are golden fixtures consumed by
`tests/workflow_contracts.rs`.

Each fixture locks two contracts for one workflow module:

1. **Module snapshot contract** from `daglang modules dsl --format json`
   - `module`
   - `path` (normalized relative to `dsl/`)
   - `items`
   - `dependencies`
2. **Expand contract** from `daglang expand dsl/<path>`
   - `expand_contract.status` (`success`, `typecheck_error`, `lower_error`, or `error`)
   - `expand_contract.error_contains` sentinel substring for non-success cases
3. **Obligations contract** from `daglang obligations dsl/<path> --format json`
   - `obligations_contract.status`
   - `obligations_contract.error_contains`
   - `obligations_contract.expected_json` for successful workflows

Why this exists:
- Gives deterministic, machine-checkable workflow contracts.
- Prevents silent drift in module topology/dependency expectations.
- Captures expected compile/lower readiness per workflow as roadmap work advances.
- Locks derived obligation counters for workflows that successfully lower.

When updating fixtures:
1. Run `cargo test -p daglang-cli --test workflow_contracts -- --nocapture`.
2. Update fixture JSON intentionally (never by accident).
3. Re-run:
   - `cargo test -p daglang-cli --test workflow_contracts`
   - `cargo test -p daglang-cli --test cli_commands`
   - `cargo test -p daglang-cli --test compile_commands`
