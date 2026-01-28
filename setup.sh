#!/bin/bash

# setup.sh - Complete Soroban Project Setup Script
# This script sets up a complete Soroban development environment

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

print_banner() {
    echo -e "${PURPLE}"
    echo "╔══════════════════════════════════════════════════════╗"
    echo "║              Soroban Project Setup                   ║"
    echo "║         Stellar Smart Contract Development           ║"
    echo "╚══════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

check_command() {
    if command -v "$1" &> /dev/null; then
        return 0
    else
        return 1
    fi
}

install_rust() {
    print_step "Installing Rust 1.92.0..."
    rustup install 1.92.0
    rustup override set 1.92.0
    rustc --version
    print_status "Rust 1.92.0 installed and set"
}

setup_rust_targets() {
    print_step "Setting up Rust targets and components..."
    rustup target add wasm32-unknown-unknown
    rustup component add rustfmt clippy rust-src
    print_status "Rust targets and components configured"
}

install_soroban_cli() {
    print_step "Installing Soroban CLI v23.1.4..."
    
    if check_command soroban; then
        CURRENT_VERSION=$(soroban --version | head -n1 | awk '{print $2}')
        if [ "$CURRENT_VERSION" = "23.1.4" ]; then
            print_status "Soroban CLI v23.1.4 already installed"
            return
        else
            print_warning "Different Soroban CLI version detected ($CURRENT_VERSION). Replacing..."
        fi
    fi

    curl -L -o stellar-cli.tar.gz https://github.com/stellar/soroban-cli/releases/download/v23.1.4/stellar-cli-23.1.4-x86_64-unknown-linux-gnu.tar.gz
    tar -xzf stellar-cli.tar.gz
    sudo mv stellar /usr/local/bin/
    sudo ln -sf /usr/local/bin/stellar /usr/local/bin/soroban
    rm stellar-cli.tar.gz

    soroban --version
    print_status "Soroban CLI v23.1.4 installed successfully"
}

create_project_structure() {
    print_step "Creating project structure..."
    
    # Create directories
    mkdir -p contracts/medical_records/src
    mkdir -p scripts
    mkdir -p tests/{integration,unit}
    mkdir -p .github/workflows
    mkdir -p deployments
    
    print_status "Project structure created"
}

initialize_git() {
    print_step "Initializing Git repository..."
    if [ ! -d ".git" ]; then
        git init
        git add .
        git commit -m "Initial commit: Complete Soroban project setup"
        print_status "Git repository initialized"
    else
        print_status "Git repository already exists"
    fi
}

setup_soroban_config() {
    print_step "Setting up Soroban configuration..."
    
    # Generate default identity if it doesn't exist
    if ! soroban config identity show default &> /dev/null; then
        print_status "Generating default identity..."
        soroban config identity generate default
        IDENTITY_ADDRESS=$(soroban config identity address default)
        print_status "Default identity created: $IDENTITY_ADDRESS"
    else
        print_status "Default identity already exists"
    fi
    
    # Configure networks
    print_status "Configuring networks..."
    
    # Local network
    soroban config network add local \
        --rpc-url http://localhost:8000/soroban/rpc \
        --network-passphrase "Standalone Network ; February 2017" \
        2>/dev/null || print_status "Local network already configured"
    
    # Testnet
    soroban config network add testnet \
        --rpc-url https://soroban-testnet.stellar.org:443 \
        --network-passphrase "Test SDF Network ; September 2015" \
        2>/dev/null || print_status "Testnet already configured"
    
    # Futurenet
    soroban config network add futurenet \
        --rpc-url https://rpc-futurenet.stellar.org:443 \
        --network-passphrase "Test SDF Future Network ; October 2022" \
        2>/dev/null || print_status "Futurenet already configured"
    
    print_status "Networks configured successfully"
}

build_project() {
    print_step "Building project..."
    cargo build --all-targets
    print_status "Project built successfully"
}

run_tests() {
    print_step "Running tests..."
    cargo test --all
    print_status "All tests passed"
}

make_scripts_executable() {
    print_step "Making scripts executable..."
    chmod +x scripts/*.sh 2>/dev/null || true
    print_status "Scripts are now executable"
}

install_development_tools() {
    print_step "Installing additional development tools..."
    
    # Install cargo-watch for auto-rebuilding
    if ! check_command cargo-watch; then
        cargo install cargo-watch
        print_status "cargo-watch installed"
    fi
    
    # Install cargo-audit for security auditing
    if ! check_command cargo-audit; then
        cargo install cargo-audit
        print_status "cargo-audit installed"
    fi
    
    print_status "Development tools installed"
}

print_success_message() {
    echo -e "${GREEN}"
    echo "╔══════════════════════════════════════════════════════╗"
    echo "║                  Setup Complete! 🚀                 ║"
    echo "╚══════════════════════════════════════════════════════╝"
    echo -e "${NC}"
    
    echo -e "${BLUE}Next steps:${NC}"
    echo "1. Start local Stellar network:"
    echo "   soroban network start local"
    echo ""
    echo "2. Deploy your first contract:"
    echo "   ./scripts/deploy.sh medical_records local"
    echo ""
    echo "3. Interact with your contract:"
    echo "   ./scripts/interact.sh <CONTRACT_ID> local initialize"
    echo ""
    echo "4. Available make commands:"
    echo "   make help          - Show all available commands"
    echo "   make build         - Build all contracts"
    echo "   make test          - Run all tests"
    echo "   make deploy-local  - Deploy to local network"
    echo ""
    echo -e "${GREEN}Happy coding! 🎉${NC}"
}

print_error_message() {
    echo -e "${RED}"
    echo "╔══════════════════════════════════════════════════════╗"
    echo "║                  Setup Failed! ❌                   ║"
    echo "╚══════════════════════════════════════════════════════╝"
    echo -e "${NC}"
    echo "Please check the error messages above and try again."
    echo "For help, visit: https://soroban.stellar.org/docs"
}

# Main setup function
main() {
    print_banner
    
    print_status "Starting Soroban project setup..."
    
    # Check if we're in the right directory
    if [ ! -f "Cargo.toml" ]; then
        print_error "No Cargo.toml found. Please run this script from the project root."
        exit 1
    fi
    
    # Run setup steps
    install_rust
    setup_rust_targets
    install_soroban_cli
    create_project_structure
    setup_soroban_config
    make_scripts_executable
    build_project
    run_tests
    install_development_tools
    initialize_git
    
    print_success_message
}

# Error handling
trap 'print_error_message; exit 1' ERR

# Run main function
main "$@"
