# Makefile for Soroban Smart Contract Development

.PHONY: help build test clean fmt lint deploy-local start-local stop-local install-deps check-deps shellcheck dist dev-deploy

# Default target
help:
	@echo "Available commands:"
	@echo "  build          - Build all contracts"
	@echo "  build-opt      - Build optimized contracts for deployment"
	@echo "  test           - Run all tests"
	@echo "  test-unit      - Run unit tests only"
	@echo "  test-integration - Run integration tests only"
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
	@echo "  setup          - Complete setup for new developers"

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

# Check if required dependencies are installed
check-deps:
	@echo "Checking dependencies..."
	@command -v rustc >/dev/null 2>&1 || { echo "Rust not installed. Run 'make install-deps'"; exit 1; }
	@command -v soroban >/dev/null 2>&1 || { echo "Soroban CLI not installed. Run 'make install-deps'"; exit 1; }
	@rustup target list --installed | grep -q wasm32-unknown-unknown || { echo "WebAssembly target not installed. Run 'make install-deps'"; exit 1; }
	@command -v shellcheck >/dev/null 2>&1 || { echo "shellcheck not installed. Run 'make install-deps'"; exit 1; }
	@echo "All dependencies are installed!"

# Build all contracts
build: check-deps
	@echo "Building all contracts..."
	cargo build --all-targets

# Build optimized contracts for deployment
build-opt: check-deps
	@echo "Building optimized contracts..."
	@for contract in contracts/*/; do \
		if [ -d "$$contract" ]; then \
			echo "Building contract: $$contract"; \
			cd "$$contract" && \
			cargo build --target wasm32-unknown-unknown --release && \
			cd - > /dev/null; \
		fi \
	done
	@echo "Contracts built successfully!"

# Run all tests
test: check-deps
	@echo "Running all tests..."
	cargo test --all

# Run unit tests only
test-unit: check-deps
	@echo "Running unit tests..."
	cargo test --lib

# Run integration tests only
test-integration: check-deps
	@echo "Running integration tests..."
	cargo test --test integration

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	@find . -name "*.wasm" -type f -delete
	@rm -rf dist/ 2>/dev/null || true
	@echo "Clean completed!"

# Format code
fmt:
	@echo "Formatting code..."
	cargo fmt --all

# Run linter
lint: check-deps
	@echo "Running clippy..."
	cargo clippy --all-targets --all-features -- -D warnings

# Lint shell scripts
shellcheck: check-deps
	@echo "Linting shell scripts..."
	shellcheck scripts/*.sh || { echo "Shellcheck found issues—fix them!"; exit 1; }
	@echo "Shell scripts linted successfully!"

# Run all checks
check: fmt lint test shellcheck
	@echo "All checks passed!"

# Build .wasm into dist/
dist: build-opt check-deps
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

# Start local Stellar network with passphrase validation
start-local: check-deps
	@echo "Starting local Stellar network..."
	# Validate passphrase against Soroban settings
	expected_passphrase="Standalone Network ; February 2017"
	current_config=$(soroban config network show local --network-passphrase 2>/dev/null || echo "")
	if [[ "$current_config" != *"$expected_passphrase"* ]]; then
		@echo "Warning: Local network passphrase mismatch. Reconfiguring..."
		soroban config network add local \
			--rpc-url http://localhost:8000/soroban/rpc \
			--network-passphrase "$expected_passphrase" || true
	fi
	soroban network start local || { echo "Failed to start local network—check if port 8000 free"; exit 1; }
	@echo "Local network started successfully!"

# Stop local Stellar network
stop-local:
	@echo "Stopping local Stellar network..."
	soroban network stop local || true
	@echo "Local network stopped!"

# Deploy contracts to local network
deploy-local: build-opt start-local
	@echo "Deploying contracts to local network..."
	@for contract in contracts/*/; do \
		if [ -d "$$contract" ]; then \
			contract_name=$$(basename "$$contract"); \
			echo "Deploying $$contract_name..."; \
			./scripts/deploy.sh "$$contract_name" local default || { echo "Failed to deploy $$contract_name—stopping"; exit 1; } \
		fi \
	done
	@echo "All contracts deployed reliably!"

# Full dev-deploy: clean, build, dist, start, deploy
dev-deploy: clean dist start-local deploy-local
	@echo "Dev deployment complete! All contracts built/deployed reliably. 🚀"

# Complete setup for new developers
setup: install-deps
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

# Development workflow shortcuts
dev-build: fmt lint build test shellcheck

# Docker support (optional)
docker-build:
	@echo "Building Docker image..."
	docker build -t soroban-project .

docker-run:
	@echo "Running in Docker container..."
	docker run -it --rm -v $(PWD):/workspace soroban-project

# Security audit
audit:
	@echo "Running security audit..."
	cargo audit

# Generate documentation
docs:
	@echo "Generating documentation..."
	cargo doc --no-deps --all-features --open

# Watch for changes and rebuild (requires cargo-watch)
watch:
	@echo "Watching for changes..."
	cargo watch -x "build --all-targets"

# Benchmark tests (if any)
bench:
	@echo "Running benchmarks..."
	cargo bench

# Profile build times
profile:
	@echo "Profiling build times..."
	cargo build --timings