# Dag Language Support for VS Code

This directory contains an initial VS Code language-support prototype for `.dag` files.

It currently provides:

- recognition of `.dag` files as Dag;
- TextMate-based syntax highlighting;
- `//` comment toggling;
- bracket matching and auto-closing pairs.

## Current status

The grammar and language-configuration JSON files are currently maintained as a prototype.

A planned follow-up is to generate the syntax metadata from the compiler’s Dag syntax definitions, so language changes do not require manually updating editor-support files.