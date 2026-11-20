# Dag Language Support for VS Code

This directory contains an initial VS Code language-support prototype for `.dag` files.

It currently provides:

- recognition of `.dag` files as Dag;
- TextMate-based syntax highlighting;
- `//` comment toggling;
- bracket matching and auto-closing pairs.

## Grammar generation

The TextMate grammar is generated from the DAG lexer definitions. Regenerate it
from the repository root:

```bash
target/release/gunbc run \
  --source-root dag \
  --source-root src/v2 \
  --entry dag/gunbc/instruments/vscode_dag_syntax_generate.dag \
  --function main
```

The runner requires a capped `gunbc-user-work.slice`.

## Tests

With Node.js and npm installed:

```bash
cd editor/vscode-dag
npm ci
npm test
```

The tests cover lexer keywords, identifier boundaries, operators, comments,
strings, escapes, and nested interpolation.

## Type names

The lexer classifies type names as identifiers. The generated grammar uses
`variable.other.dag` for these names and removes the prototype’s uppercase-name
type-coloring heuristic.