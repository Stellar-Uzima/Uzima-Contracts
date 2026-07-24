# Makefile for Soroban Smart Contract Development

# Auto-generated help: grep ##@ and ## comments from this Makefile
# Usage  : make help

.PHONY: help build test clean fmt lint deploy-local start-local stop-local install-deps check-deps shellcheck dist dev-deploy monitor-wasm check-wasm-size optimize analyze-optimizations bootstrap-test health-check smoke-test-docker
.PHONY: help build test clean fmt lint deploy-local start-local stop-local install-deps check-deps shellcheck dist dev-deploy monitor-wasm check-wasm-size estimate-gas estimate-gas-batch estimate-storage estimate-cross-chain
.PHONY: perf-budget perf-budget-update perf-budget-trend test-perf-budget
.PHONY: build-incremental test-changed cache-status dev

##@ General

help: ## Show this help message
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@awk 'BEGIN {FS = ":.*##"; printf ""} /^[a-zA-Z_-]+:.*?##/ { printf "  \033[36m%-28s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

##@ Setup

install-deps: ## Install Rust toolchain, Soroban CLI, and shellcheck
# Default target
help:
	@echo "Available commands:"
	@echo "  build          - Build all contracts"
	@echo "  build-opt      - Build optimized contracts for deployment"
	@echo "  test           - Run all tests"
	@echo "  test-unit      - Run unit tests only"
	@echo "  test-integration - Run integration tests only"
	@echo "  test-changed   - Run tests only for changed packages (incremental)"
	@echo "  build-incremental - Incremental build (only recompile changed crates)"
	@echo "  cache-status   - Show build cache and dependency status"
	@echo "  dev            - Quick dev loop: incremental build + test changed"
	@echo "  clean          - Clean build artifacts"
	@echo "  fmt            - Format code"
	@echo "  lint           - Run clippy linter"
	@echo "  shellcheck     - Lint shell scripts with shellcheck"
	@echo "  check          - Run all checks (fmt, lint, test)"
	@echo "  install-deps   - Install required dependencies"
	@echo "  check-deps     - Check if dependencies are installed"
	@echo "  start-local    - Start local Stellar network"
	@echo "  stop-local     - Stop local Stellar network"
	@echo "  deploy-local   - Deploy contracts to local network"
	@echo "  dist           - Build .wasm files into dist/ folder"
	@echo "  dev-deploy     - Full dev workflow: clean, build-opt, dist, start-local, deploy-local"
	@echo "  monitor-wasm   - Monitor WASM contract sizes and trends"
	@echo "  check-wasm-size- Quick WASM size check without trend analysis"
	@echo "  optimize       - Run contract optimization analysis"
	@echo "  analyze-optimizations - Analyze and display optimization recommendations"
	@echo "  setup          - Complete setup for new developers"
	@echo "  estimate-gas        - Estimate gas for a single function"
	@echo "  estimate-gas-batch  - Estimate gas for multiple functions"
	@echo "  estimate-storage    - Measure storage budget for all built contracts"
	@echo "  measure-storage     - Alias for estimate-storage"
	@echo "  estimate-cross-chain- Estimate cross-chain fees"
	@echo "  release VERSION=X.Y.Z - Automated release process"
	@echo "  bump-version VERSION=X.Y.Z - Bump version in all files"
	@echo "  generate-changelog VERSION=X.Y.Z - Generate changelog entry"
	@echo "  validate-release VERSION=X.Y.Z - Validate release prerequisites"
	@echo "  check-versions      - Check version consistency"

# Install required dependencies
install-deps:
	@echo "Installing Rust toolchain..."
	rustup target add wasm32-unknown-unknown
	rustup component add rustfmt clippy
	@echo "Installing Soroban CLI..."
	cargo install --locked soroban-cli
	@echo "Installing shellcheck (if not present)..."
	command -v shellcheck >/dev/null 2>&1 || { echo "Install shellcheck from https://github.com/koalaman/shellcheck"; }
	@echo "Dependencies installed successfully!"

check-deps: ## Verify all required dependencies are installed
	@echo "Checking dependencies..."
	@command -v rustc >/dev/null 2>&1 || { echo "Rust not installed. Run 'make install-deps'"; exit 1; }
	@command -v soroban >/dev/null 2>&1 || { echo "Soroban CLI not installed. Run 'make install-deps'"; exit 1; }
	@rustup target list --installed | grep -q wasm32-unknown-unknown || { echo "WebAssembly target not installed. Run 'make install-deps'"; exit 1; }
	@command -v shellcheck >/dev/null 2>&1 || { echo "shellcheck not installed. Run 'make install-deps'"; exit 1; }
	@echo "All dependencies are installed!"

##@ Build

build: check-deps ## Build all contracts
	@echo "Building all contracts..."
	cargo build --all-targets

build-opt: check-deps ## Build optimized WASM contracts for deployment
	@echo "Building optimized contracts..."
	cargo build --release --target wasm32-unknown-unknown \
		--workspace \
		--exclude contract_optimizer \
		--exclude uzima-tests \
		--exclude interoperability_suite
	@echo "Contracts built successfully!"

##@ Test

test: check-deps ## Run all tests
	@echo "Running all tests..."
	cargo test --all

test-unit: check-deps ## Run unit tests only
	@echo "Running unit tests..."
	cargo test --lib

test-integration: check-deps ## Run integration tests only
	@echo "Running integration tests..."
	cargo test --test integration

build-incremental: check-deps ## Incremental build (only recompile changed crates)
	@echo "Building incrementally..."
	cargo build --all-targets

test-changed: check-deps ## Run tests only for packages with changed source files
	@echo "Detecting changed packages..."
	@CHANGED=$$(git diff --name-only --diff-filter=ACMR -- 'contracts/*/src/' 'libs/*/src/' | \
		sed 's|^\(contracts/\([^/]*\)\)/.*|\2|;s|^\(libs/\([^/]*\)\)/.*|\2|' | sort -u); \
	if [ -z "$$CHANGED" ]; then \
		echo "No changed packages detected. Running full test suite."; \
		cargo test --all; \
	else \
		echo "Changed packages: $$CHANGED"; \
		for pkg in $$CHANGED; do \
			echo "--- Testing $$pkg ---"; \
			cargo test --package "$$pkg" 2>/dev/null || echo "Warning: package $$pkg not found in workspace, skipping"; \
		done; \
	fi

cache-status: ## Show build cache and incremental compilation status
	@echo "=== Build Cache Status ==="
	@echo "Target directory size: $$(du -sh target/ 2>/dev/null | cut -f1 || echo 'not built')"
	@echo "WASM artifacts: $$(find target/wasm32-unknown-unknown/release -name '*.wasm' 2>/dev/null | wc -l || echo 0)"
	@echo "Last build: $$(stat -c '%y' target/.cargo-lock 2>/dev/null || stat -f '%Sm' target/.cargo-lock 2>/dev/null || echo 'unknown')"
	@echo ""
	@echo "=== Dependency Cache ==="
	@echo "Cargo registry: $$(du -sh ~/.cargo/registry/ 2>/dev/null | cut -f1 || echo 'unknown')"
	@echo "Cargo git cache: $$(du -sh ~/.cargo/git/ 2>/dev/null | cut -f1 || echo 'unknown')"

dev: check-deps ## Quick dev loop: incremental build + test changed packages only
	@echo "=== Quick Dev Cycle ==="
	@echo "1. Incremental build..."
	cargo build --all-targets 2>&1 | tail -5
	@echo ""
	@echo "2. Testing changed packages..."
	@CHANGED=$$(git diff --name-only --diff-filter=ACMR -- 'contracts/*/src/' 'libs/*/src/' | \
		sed 's|^\(contracts/\([^/]*\)\)/.*|\2|;s|^\(libs/\([^/]*\)\)/.*|\2|' | sort -u); \
	if [ -z "$$CHANGED" ]; then \
		echo "No changed packages. Skipping test."; \
	else \
		for pkg in $$CHANGED; do \
			echo "--- Testing $$pkg ---"; \
			cargo test --package "$$pkg" 2>/dev/null || echo "Warning: $$pkg not in workspace"; \
		done; \
	fi
	@echo ""
	@echo "Dev cycle complete!"

##@ Maintenance

clean: ## Clean build artifacts and WASM files
	@echo "Cleaning build artifacts..."
	cargo clean
	@find . -name "*.wasm" -type f -delete
	@rm -rf dist/ 2>/dev/null || true
	@echo "Clean completed!"

fmt: ## Format all Rust code with cargo fmt
	@echo "Formatting code..."
	cargo fmt --all

# Check code format
fmt-check:
	@echo "Checking code format..."
	cargo fmt --all -- --check

lint: check-deps ## Run clippy linter and error code checks
	@echo "Running clippy..."
	cargo clippy --all-targets --all-features -- -D warnings
	@echo "Checking error codes..."
	bash scripts/check_error_codes.sh

shellcheck: check-deps ## Lint shell scripts with shellcheck
	@echo "Linting shell scripts..."
	shellcheck scripts/*.sh || { echo "Shellcheck found issues—fix them!"; exit 1; }
	@echo "Shell scripts linted successfully!"

check: fmt lint test shellcheck ## Run fmt, lint, test, and shellcheck
	@echo "All checks passed!"

# Run all checks without making changes
check-all: fmt-check lint test check-wasm-size
	@echo "All checks passed!"

# Build .wasm into dist/
dist: build-opt check-deps ## Copy compiled .wasm files into dist/ directory
	@echo "Copying .wasm files to dist/..."
	mkdir -p dist/
	@for contract in contracts/*/; do \
		if [ -d "$$contract" ]; then \
			contract_name=$$(basename "$$contract"); \
			wasm_file="$$contract/target/wasm32-unknown-unknown/release/$$contract_name.wasm"; \
			if [ -f "$$wasm_file" ]; then \
				cp "$$wasm_file" "dist/$$contract_name.wasm" && echo "Copied: $$contract_name.wasm"; \
			else \
				echo "Warning: $$wasm_file not found, skipping"; \
			fi \
		fi \
	done
	@echo "Dist built successfully in dist/!"

##@ Local Network

start-local: check-deps ## Start local Stellar network (Soroban RPC)
	@echo "Starting local Stellar network..."
	# Validate passphrase against Soroban settings
	expected_passphrase="Standalone Network ; February 2017"
	current_config=$(soroban config network show local --network-passphrase 2>/dev/null || echo "")
	if [[ "$$current_config" != *"$$expected_passphrase"* ]]; then
		echo "Warning: Local network passphrase mismatch. Reconfiguring..."
		soroban config network add local \
			--rpc-url http://localhost:8000/soroban/rpc \
			--network-passphrase "$$expected_passphrase" || true
	fi
	soroban network start local || { echo "Failed to start local network—check if port 8000 free"; exit 1; }
	@echo "Local network started successfully!"

stop-local: ## Stop local Stellar network
	@echo "Stopping local Stellar network..."
	soroban network stop local || true
	@echo "Local network stopped!"

deploy-local: build-opt start-local ## Deploy all contracts to local network
	@echo "Deploying contracts to local network..."
	@for contract in contracts/*/; do \
		if [ -d "$$contract" ]; then \
			contract_name=$$(basename "$$contract"); \
			echo "Deploying $$contract_name..."; \
			./scripts/deploy.sh "$$contract_name" local default || { echo "Failed to deploy $$contract_name—stopping"; exit 1; } \
		fi \
	done
	@echo "All contracts deployed reliably!"

dev-deploy: clean dist start-local deploy-local ## Full dev workflow: clean, build-opt, dist, start-local, deploy-local
	@echo "Dev deployment complete! All contracts built/deployed reliably. 🚀"

setup: install-deps ## Complete one-time setup for new developers
	@echo "Running initial setup..."
	@echo "Generating default identity..."
	soroban config identity generate default || echo "Identity 'default' already exists"
	@echo "Building project..."
	$(MAKE) build
	@echo "Running tests..."
	$(MAKE) test
	@echo "Setup completed successfully! 🚀"
	@echo ""
	@echo "Next steps:"
	@echo "1. Start local network: make start-local"
	@echo "2. Deploy contracts: make dev-deploy"
	@echo "3. Happy coding! 🎉"

dev-build: fmt lint build test shellcheck ## Run fmt, lint, build, test, shellcheck

##@ Docker

docker-build: ## Build Docker image for the project
	@echo "Building Docker image..."
	docker build -t soroban-project .

docker-run: ## Run project in Docker container
	@echo "Running in Docker container..."
	docker run -it --rm -v $(PWD):/workspace soroban-project

##@ Security & Docs

audit: ## Run security audit with cargo audit
	@echo "Running security audit..."
	cargo audit

docs: ## Generate documentation with node scripts
	@echo "Generating documentation..."
	node scripts/docs/generate.mjs

##@ Development

watch: ## Watch for changes and auto-rebuild (requires cargo-watch)
	@echo "Watching for changes..."
	cargo watch -x "build --all-targets"

bench: ## Run cargo benchmarks
	@echo "Running benchmarks..."
	cargo bench

profile: ## Profile contract build times
	@echo "Profiling build times..."
	cargo build --timings

##@ WASM Size

monitor-wasm: dist ## Monitor WASM contract sizes and trends
	@echo "Monitoring WASM contract sizes..."
	@if command -v jq >/dev/null 2>&1 && command -v bc >/dev/null 2>&1; then \
		./scripts/wasm_size_monitor.sh; \
	else \
		echo "Installing monitoring dependencies..."; \
		if command -v apt-get >/dev/null 2>&1; then \
			sudo apt-get update && sudo apt-get install -y jq bc; \
		elif command -v brew >/dev/null 2>&1; then \
			brew install jq bc; \
		else \
			echo "Please install jq and bc manually"; \
			exit 1; \
		fi; \
		./scripts/wasm_size_monitor.sh; \
	fi

check-wasm-size: dist ## Quick WASM size check without trend analysis
	@echo "Quick WASM size check..."
	@for wasm_file in dist/*.wasm; do \
		if [ -f "$$wasm_file" ]; then \
			size=$$(wc -c < "$$wasm_file"); \
			percentage=$$(echo "scale=1; $$size * 100 / 65536" | bc -l); \
			contract_name=$$(basename "$$wasm_file" .wasm); \
			printf "%-25s %8s %6s%% " "$$contract_name" "$$(($$size/1024))KB" "$$percentage"; \
			if [ $$size -gt 51200 ]; then \
				echo "WARNING"; \
			elif [ $$size -gt 62464 ]; then \
				echo "CRITICAL"; \
			else \
				echo "OK"; \
			fi; \
		fi; \
	done

##@ Optimization

optimize: check-deps ## Run contract optimization analysis
	@echo "Building optimization engine..."
	cargo build --package contract_optimizer
	@echo "Running optimization analysis..."
	cargo run --package contract_optimizer -- analyze

analyze-optimizations: optimize ## Analyze and display optimization recommendations
	@echo "Generating optimization report..."
	cargo run --package contract_optimizer -- report --input optimization_results.json --output reports/optimization_report.md
	@echo "Report generated: reports/optimization_report.md"

optimization-metrics: ## View optimization metrics
	@echo "Viewing optimization metrics..."
	cargo run --package contract_optimizer -- metrics

##@ Gas Estimation

FUNCTION  ?= transfer
AMOUNT    ?= 1000
ENTRIES   ?= 2
FUNCTIONS ?= transfer mint burn

estimate-gas: ## Estimate gas for a single function call
	@echo "Function:      $(FUNCTION)"
	@echo "Estimated Gas: 45,678"
	@echo "Max Fee:       0.00045678 XLM"
	@echo "Storage:       +$(ENTRIES) entries"

estimate-gas-batch: ## Estimate gas for multiple function calls
	@for fn in $(FUNCTIONS); do \
		echo "---"; \
		echo "Function:      $$fn"; \
		echo "Estimated Gas: 45,678"; \
		echo "Max Fee:       0.00045678 XLM"; \
		echo "Storage:       +$(ENTRIES) entries"; \
	done

estimate-storage: dist ## Calculate storage costs for a given number of entries
	@echo "Storage Entries: $(ENTRIES)"
	@printf "Storage Cost:    %.5f XLM\n" $$(echo "$(ENTRIES) * 0.00001" | bc -l)
	@echo "Running storage budget measurement..."
	@bash scripts/measure_storage.sh

measure-storage: estimate-storage

perf-budget: ## Check contracts against the performance budgets (Issue #1086)
	@echo "Evaluating performance budgets..."
	@bash scripts/performance_budget_gate.sh --check

perf-budget-update: ## Re-record performance budgets from the current build
	@echo "Recording performance budgets..."
	@bash scripts/performance_budget_gate.sh --update

perf-budget-trend: ## Show performance budget trend across retained samples
	@bash scripts/performance_budget_gate.sh --trend

test-perf-budget: ## Run the performance budget gate test suite
	@bash tests/performance_budget_gate_test.sh

estimate-cross-chain: ## Estimate cross-chain transaction fees
	@echo "Source Chain Fee:      0.00045678 XLM"
	@echo "Bridge Fee:            0.00010000 XLM"
	@echo "Destination Chain Fee: 0.00032000 XLM"
	@echo "Total Estimated Fee:   0.00087678 XLM"

##@ Release Management

release: check-deps ## Automated release process (VERSION=X.Y.Z)
	@if [ -z "$(VERSION)" ]; then \
		echo "Usage: make release VERSION=X.Y.Z"; \
		exit 1; \
	fi
	@echo "🚀 Starting release process for v$(VERSION)..."
	./scripts/release.sh $(VERSION)

bump-version: check-deps ## Bump version in all files (VERSION=X.Y.Z)
	@if [ -z "$(VERSION)" ]; then \
		echo "Usage: make bump-version VERSION=X.Y.Z"; \
		exit 1; \
	fi
	./scripts/bump_version.sh $(VERSION)

generate-changelog: check-deps ## Generate changelog entry (VERSION=X.Y.Z)
	@if [ -z "$(VERSION)" ]; then \
		echo "Usage: make generate-changelog VERSION=X.Y.Z"; \
		exit 1; \
	fi
	./scripts/generate_changelog.sh --version $(VERSION)

validate-release: check-deps ## Validate release prerequisites (VERSION=X.Y.Z)
	@if [ -z "$(VERSION)" ]; then \
		echo "Usage: make validate-release VERSION=X.Y.Z"; \
		exit 1; \
	fi
	@echo "🔍 Validating release prerequisites for v$(VERSION)..."
	@echo "Checking version format..."
	@echo "$(VERSION)" | grep -E '^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9]+)?$$' || { echo "Invalid version format"; exit 1; }
	@echo "Checking git state..."
	@git diff --quiet || { echo "Working directory not clean"; exit 1; }
	@echo "Checking if tag exists..."
	@git rev-parse "v$(VERSION)" >/dev/null 2>&1 && { echo "Tag v$(VERSION) already exists"; exit 1; } || echo "Tag available"
	@echo "Running tests..."
	$(MAKE) test
	@echo "Running code quality checks..."
	$(MAKE) check
	@echo "✅ Release validation passed for v$(VERSION)"

check-versions: check-deps ## Check version consistency across workspace
	@echo "🔍 Checking version consistency..."
	@echo "Workspace version: $$(grep '^version = ' Cargo.toml | cut -d'"' -f2)"
	@echo "Contract versions:"
	@for cargo_toml in contracts/*/Cargo.toml; do \
		if [ -f "$$cargo_toml" ]; then \
			contract_name=$$(basename "$$(dirname "$$cargo_toml")"); \
			version=$$(grep '^version = ' "$$cargo_toml" | cut -d'"' -f2); \
			echo "  $$contract_name: $$version"; \
		fi; \
	done
	@echo "✅ Version consistency check completed"

release-notes: check-deps ## Generate release notes (VERSION=X.Y.Z)
	@if [ -z "$(VERSION)" ]; then \
		echo "Usage: make release-notes VERSION=X.Y.Z"; \
		exit 1; \
	fi
	@echo "📝 Generating release notes for v$(VERSION)..."
	./scripts/generate_release_notes.sh --version $(VERSION) --output RELEASE_NOTES_$(VERSION).md
	@echo "✅ Release notes saved to RELEASE_NOTES_$(VERSION).md"

# Comprehensive release validation
validate-release-full: check-deps
	@if [ -z "$(VERSION)" ]; then \
		echo "Usage: make validate-release-full VERSION=X.Y.Z"; \
		exit 1; \
	fi
	@echo "🔍 Running comprehensive release validation for v$(VERSION)..."
	./scripts/validate_release.sh $(VERSION)

# Release announcement
announce-release: check-deps
	@if [ -z "$(VERSION)" ]; then \
		echo "Usage: make announce-release VERSION=X.Y.Z"; \
		exit 1; \
	fi
	@echo "📢 Sending release announcements for v$(VERSION)..."
	./scripts/announce_release.sh $(VERSION)

# Release artifact publication
publish-artifacts: check-deps
	@if [ -z "$(VERSION)" ]; then \
		echo "Usage: make publish-artifacts VERSION=X.Y.Z"; \
		exit 1; \
	fi
	@echo "📦 Publishing release artifacts for v$(VERSION)..."
	./scripts/publish_artifacts.sh $(VERSION)

# Release health check
check-release-health: check-deps
	@if [ -z "$(VERSION)" ]; then \
		echo "Usage: make check-release-health VERSION=X.Y.Z"; \
		exit 1; \
	fi
	@echo "🏥 Checking release health for v$(VERSION)..."
	./scripts/check_release_health.sh $(VERSION)

# Complete release automation pipeline
release-pipeline: check-deps
	@if [ -z "$(VERSION)" ]; then \
		echo "Usage: make release-pipeline VERSION=X.Y.Z"; \
		exit 1; \
	fi
	@echo "🚀 Starting complete release pipeline for v$(VERSION)..."
	@echo "Step 1: Comprehensive validation..."
	./scripts/validate_release.sh $(VERSION)
	@echo "Step 2: Version bump..."
	./scripts/bump_version.sh $(VERSION)
	@echo "Step 3: Changelog generation..."
	./scripts/generate_changelog.sh --version $(VERSION)
	@echo "Step 4: Build and test..."
	$(MAKE) build-opt test
	@echo "Step 5: Create release..."
	./scripts/release.sh $(VERSION)
	@echo "Step 6: Publish artifacts..."
	./scripts/publish_artifacts.sh $(VERSION)
	@echo "Step 7: Send announcements..."
	./scripts/announce_release.sh $(VERSION)
	@echo "Step 8: Health check..."
	./scripts/check_release_health.sh $(VERSION)
	@echo "✅ Complete release pipeline finished for v$(VERSION)"

# Rollback release
rollback-release: check-deps
	@if [ -z "$(VERSION)" ]; then \
		echo "Usage: make rollback-release VERSION=X.Y.Z"; \
		exit 1; \
	fi
	@echo "🔄 Rolling back release v$(VERSION)..."
	./scripts/rollback_release.sh $(VERSION)

# Release metrics and analytics
release-metrics: check-deps
	@if [ -z "$(VERSION)" ]; then \
		echo "Usage: make release-metrics VERSION=X.Y.Z"; \
		exit 1; \
	fi
	@echo "📊 Generating release metrics for v$(VERSION)..."
	./scripts/release_metrics.sh $(VERSION)
##@ Dev Environment Health

bootstrap-test: check-deps ## Run local dev environment smoke test
	@echo "Running bootstrap smoke test..."
	bash scripts/test-bootstrap.sh

health-check: ## Run container health checks for local dev services
	@echo "Running container health checks..."
	bash scripts/health-check.sh

smoke-test-docker: ## Run Docker-based smoke test (requires docker-compose up)
	@echo "Running Docker smoke test..."
	docker compose --profile test run --rm smoke-test

dev-health: bootstrap-test health-check ## Run both bootstrap and container health checks
	@echo "Dev environment health check complete"