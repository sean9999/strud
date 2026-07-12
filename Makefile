CARGO ?= cargo
ARGS ?=
DEMO_DIR ?= demo-diary

.PHONY: help build check run test clippy fmt clean install demo demo-clean

help: ## Show available targets
	@awk 'BEGIN {FS = ":.*##"} /^[a-zA-Z_-]+:.*##/ {printf "  %-12s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

build: ## Compile the binary (debug)
	$(CARGO) build

check: ## Type-check without producing a binary
	$(CARGO) check

run: ## Run the CLI, e.g. `make run ARGS="list --last 7"`
	$(CARGO) run -- $(ARGS)

test: ## Run the test suite
	$(CARGO) test

clippy: ## Lint with clippy
	$(CARGO) clippy -- -D warnings

fmt: ## Format the source
	$(CARGO) fmt

clean: ## Remove build artifacts
	$(CARGO) clean

install: ## Install the `strud` binary to ~/.cargo/bin
	$(CARGO) install --path .

demo: ## Scaffold a throwaway diary at $(DEMO_DIR)
	$(CARGO) run -- init $(DEMO_DIR)
	@echo "Try: make run ARGS=\"new --dir $(DEMO_DIR)\""

demo-clean: ## Remove the demo diary
	rm -rf $(DEMO_DIR)