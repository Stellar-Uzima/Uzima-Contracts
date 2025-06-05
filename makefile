# Makefile for Soroban Smart Contract Development

.PHONY: help build test clean fmt lint deploy-local start-local stop-local install-deps check-deps

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
	@echo "  check          - Run all checks (fmt, lint, test)"
	@echo "  install-deps   - Install required dependencies"
	@echo "  check-deps     - Check if dependencies are installed"
	@echo "  start-local    - Start local Stellar network"
	@echo "  stop-local     - Stop local Stellar network"
	@echo "  deploy-local   - Deploy contracts to local network"
	@echo "  setup          - Complete setup for new developers"

# Install required dependencies
install-deps:
	@echo "Installing Rust toolchain..."
	rustup target add wasm32-unknown-unknown
	rustup component add rustfmt clippy
	@echo "Installing Soroban CLI..."
	cargo install --locked soroban-cli --features opt
	@echo "Dependencies installed successfully!"

# Check if required dependencies are installed
check-deps:
	@echo "Checking dependencies..."
	@command -v rustc >/dev/null 2>&1 || { echo "Rust not installed. Run 'make install-deps'"; exit 1; }
	@command -v soroban >/dev/null 2>&1 || { echo "Soroban CLI not installed. Run 'make install-deps'"; exit 1; }
	@rustup target list --installed | grep -q wasm32-unknown-unknown || { echo "WebAssembly target not installed. Run 'make install-deps'"; exit 1; }
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
	@echo "Clean completed!"

# Format code
fmt:
	@echo "Formatting code..."
	cargo fmt --all

# Run linter
lint: check-deps
	@echo "Running clippy..."
	cargo clippy --all-targets --all-features -- -D warnings

# Run all checks
check: fmt lint test
	@echo "All checks passed!"

# Start local Stellar network
start-local: check-deps
	@echo "Starting local Stellar network..."
	soroban network start local
	@echo "Configuring local network..."
	soroban config network add local \
		--rpc-url http://localhost:8000/soroban/rpc \
		--network-passphrase "Standalone Network ; February 2017" || true
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
			./scripts/deploy.sh "$$contract_name" local default || echo "Failed to deploy $$contract_name"; \
		fi \
	done

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
	@echo "2. Deploy contracts: make deploy-local"
	@echo "3. Happy coding! 🎉"

# Development workflow shortcuts
dev-build: fmt lint build test

dev-deploy: clean build-opt deploy-local

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