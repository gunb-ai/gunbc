# Bootstrap makefile - handwritten, not generated.
#
# This file provides `make install` and `make help` before bootstrap,
# and delegates everything else to the generated Makefile after bootstrap.
#
# GNU Make checks for GNUmakefile before Makefile, so this takes precedence.
#
# To regenerate the generated Makefile:
#   cargo run -p gunbc-app --bin gunbc-bootstrap
# Or after initial bootstrap:
#   make bootstrap

# Include generated Makefile if it exists (provides all real targets).
# We intentionally override `help` below — the warning is cosmetic.
-include Makefile

.PHONY: install help
.DEFAULT_GOAL := help

# Bootstrap — generates the Makefile and other artifacts
install:
	@echo "Bootstrapping gunbc..."
	@echo ""
	@echo "Step 1/2: Ensuring generated CLI entrypoints exist..."
	cargo run -p gunbc-app --bin gunbc-codegen -- codegen
	@echo ""
	@echo "Step 2/2: Generating Makefile and .gitignore..."
	cargo run -p gunbc-app --bin gunbc-bootstrap
	@echo ""
	@echo "Bootstrap complete. Run 'make help' to see available targets."

# Curated help overview (overrides the generated verbose help)
help:
	@printf '\n'
	@printf '  \033[1mgunbc\033[0m — DSL-first workflow compiler\n'
	@printf '\n'
	@printf '  \033[1;36mGetting started:\033[0m\n'
	@printf '    make install        Bootstrap repo (generates Makefile + CLI entrypoints)\n'
	@printf '\n'
	@printf '  \033[1;36mDev workflow:\033[0m\n'
	@printf '    make test           Run tests (fast, <=S depth)\n'
	@printf '    make test-fix       Format + lint fix, then test\n'
	@printf '    make test-all       Run all tests (cargo test --workspace)\n'
	@printf '    make check          Type-check all targets\n'
	@printf '    make clippy         Run clippy linter\n'
	@printf '    make fmt            Format all code\n'
	@printf '\n'
	@printf '  \033[1;36mBuild:\033[0m\n'
	@printf '    make build          Full build: codegen -> testgen -> cargo build\n'
	@printf '    make codegen        Generate CLI entrypoints from DSL\n'
	@printf '    make testgen        Regenerate tests from DAG structures\n'
	@printf '    make clean          Clean build artifacts\n'
	@printf '\n'
	@printf '  \033[1;36mCI / verification:\033[0m\n'
	@printf '    make ci             Run CI via build DAG\n'
	@printf '    make verify         Verify generated artifacts match generators\n'
	@printf '    make verify-fix     Regenerate stale artifacts\n'
	@printf '\n'
	@printf '  \033[1;36mTools:\033[0m\n'
	@printf '    make bootstrap      Regenerate Makefile + .gitignore\n'
	@printf '    make deps           Manage workspace dependencies\n'
	@printf '    make pragma         Run pragma lint enforcement\n'
	@printf '    make gist           Create gist snapshot\n'
	@printf '    make design         Generate design docs\n'
	@printf '    make docgen         Generate documentation\n'
	@printf '    make infra          Infrastructure management\n'
	@printf '\n'
	@printf '  \033[1;36mConventions:\033[0m\n'
	@printf '    make <target>       Verify only (CI-safe, fails on issues)\n'
	@printf '    make <target>-fix   Auto-fix then verify (for dev)\n'
	@printf '    make <target>-dry   Dry-run (no side effects)\n'
	@printf '\n'
