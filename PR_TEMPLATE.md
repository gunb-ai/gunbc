# SHELL-TYPED-INVOCATION: cut emit-host verdict narration over to typed printf argv

## Objective

Migrate `tools.emit_host_gate.gate_failure_exit` from shell-based text construction to proper typed command execution.

Today it:
1. Escapes a value for Bash double quotes
2. Constructs a textual `printf` shell program 
3. Seals that text as retained foreign shell
4. Runs it through `shell.Exec.Run`
5. Interprets `narrated.success`

Replace that route with the existing typed command authority:
```text
verdicts
  → extdeps.tools.gnu_coreutils.printf_command
  → ArgvCommand { program, arguments }
  → shell.Exec.RunArgv
  → exit_code
```

## Changes

### Files to modify:
- `dag/gunbc/instruments/emit_host_gate.dag`
- `dag/gunbc/shell_bash_runner.dag` 
- `dag/test/claim/emit_host_gate_verdicts_test.dag`
- `dag/test/claim/shell_bash_runner_witness_test.dag`

### Key changes:
1. Replace imports from `gunbc.retained_shell_script` and `gunbc.shell_bash_runner` with typed command imports
2. Add new function `emit_host_verdict_narration_command` that uses `printf_command`
3. Update `gate_failure_exit` to use `shell.Exec.RunArgv` instead of `shell.Exec.Run`
4. Delete dead helper functions (`bash_escape_for_double_quotes`, `shell_printf_line`)
5. Update tests and witnesses

## Benefits
- Eliminates shell text construction that violates single authority principle
- Uses existing typed command authority instead of retained foreign shell
- Maintains exact same behavior for all failure cases
- Deletes dead code and reduces redundancy
- Follows project's fail-closed safety principles