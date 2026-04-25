# GC-Forge — top-level Makefile.
#
# Targets are intentionally minimal at iteration 1; richer targets land
# alongside the features that need them (see ROADMAP.md / ITERATION-LOG.md).

SHELL := /bin/bash
.SHELLFLAGS := -eu -o pipefail -c

CARGO        ?= cargo
MVN          ?= mvn
DOCKER       ?= docker
IMAGE_TAG    ?= gc-forge-runner:dev-jdk21
HARNESS_JAR  := workload-harness/target/workload-harness.jar
OUT_DIR      ?= out

.PHONY: help
help: ## List available targets
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  \033[36m%-18s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

.PHONY: bootstrap
bootstrap: ## Fetch Rust deps and build the harness fat-jar
	$(CARGO) fetch
	$(MVN) -f workload-harness/pom.xml -q -DskipTests package

.PHONY: build
build: ## Build all Rust crates + the harness fat-jar
	$(CARGO) build --workspace
	$(MVN) -f workload-harness/pom.xml -q -DskipTests package

.PHONY: test
test: ## Run Rust + Java unit tests
	$(CARGO) test --workspace
	$(MVN) -f workload-harness/pom.xml -q test

.PHONY: lint
lint: ## fmt-check, clippy, cargo-deny
	$(CARGO) fmt --all -- --check
	$(CARGO) clippy --workspace --all-targets -- -D warnings
	$(CARGO) deny check

.PHONY: dod-gate
dod-gate: ## Run the iteration DoD gate (lint + test + harness verify)
	bash scripts/dod-gate.sh

.PHONY: docker-image
docker-image: $(HARNESS_JAR) ## Build the runner Docker image
	$(DOCKER) build -t $(IMAGE_TAG) .

$(HARNESS_JAR):
	$(MVN) -f workload-harness/pom.xml -q -DskipTests package

.PHONY: demo
demo: docker-image ## Run a 10 s allocation loop in the Docker runner; log lands in $(OUT_DIR)/gc.log
	mkdir -p $(OUT_DIR)
	$(DOCKER) run --rm \
		-v "$$PWD/$(OUT_DIR):/work" \
		--network=none \
		$(IMAGE_TAG)
	@echo
	@echo "GC log written to $(OUT_DIR)/gc.log:"
	@head -n 5 $(OUT_DIR)/gc.log || true

.PHONY: clean
clean: ## Remove all build artifacts
	$(CARGO) clean
	$(MVN) -f workload-harness/pom.xml -q clean
	rm -rf $(OUT_DIR)
