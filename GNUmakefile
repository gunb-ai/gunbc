# Bootstrap makefile - handwritten, not generated.
#
# This file provides the `make install` target to bootstrap the repo.
# After running `make install`, the generated Makefile provides all other targets.
#
# GNU Make checks for GNUmakefile before Makefile, so this file takes precedence.
# We include the generated Makefile to make all its targets available.
#
# To regenerate this file manually:
#   cargo run -p gunbc-dag --bin gunbc-bootstrap
#
# Or after initial bootstrap, use:
#   make bootstrap

# Include generated Makefile if it exists (provides all the real targets)
-include Makefile

# Bootstrap target - generates the Makefile and other artifacts
install:
	@echo "Bootstrapping gunbc..."
	@echo ""
	@echo "Step 1/2: Ensuring generated CLI entrypoints exist..."
	cargo run -p gunbc-dag --bin gunbc-codegen -- codegen
	@echo ""
	@echo "Step 2/2: Generating Makefile and .gitignore..."
	cargo run -p gunbc-dag --bin gunbc-bootstrap
	@echo ""
	@echo "Bootstrap complete. Run 'make help' to see available targets."

.PHONY: install
