# Contributing to Uzima Contracts

Thank you for your interest in contributing to Uzima Contracts! This document provides guidelines and standards for contributing to the project.

## Development Workflow

1. **Fork the repository** and create a feature branch
2. **Follow coding standards** outlined in [CODING_STANDARDS.md](./docs/CODING_STANDARDS.md)
3. **Write tests** for new functionality
4. **Review the contract review checklist** in [docs/contract-review-checklist.md](./docs/contract-review-checklist.md)
5. **Run linting and tests** before submitting
6. **Submit a pull request** with a clear description

## Code Quality Standards

### Naming Conventions
All code must follow the naming conventions defined in [CODING_STANDARDS.md](./docs/CODING_STANDARDS.md) and [CONTRACT_NAMING_CONVENTIONS.md](./docs/CONTRACT_NAMING_CONVENTIONS.md):

- **Functions**: `snake_case`
- **Types**: `PascalCase`  
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Modules**: `snake_case`
- **Error enums**: Always use `Error`, never `Err`
- **File names**: `snake_case` (e.g., `lib.rs`, `errors.rs`, `types.rs`, `events.rs`, `test.rs`)
- **Contract directories**: `snake_case`

Run the naming check script before submitting PRs:
```bash
bash scripts/check-naming.sh
```

### Code Style
- Use Rust 2021 edition
- Follow Rustfmt formatting (run `cargo fmt`)
- Adhere to Clippy linting rules (run `cargo clippy`)

### Documentation
- Document all public APIs with `///` doc comments
- Include examples for complex functions
- Update relevant documentation when changing functionality

### Secret Scanning
Never commit secrets (Stellar seeds, RPC keys, API tokens, private keys). The
repo enforces this with [`gitleaks`](https://github.com/gitleaks/gitleaks) both
locally and in CI. Install the pre-commit hook once:
```bash
pip install pre-commit   # or: brew install pre-commit
pre-commit install       # scans staged changes on every commit
```
CI also runs a secret scan on every push and PR and **blocks merges** on any
finding. See the [Secret Management Policy](./docs/SECRET_MANAGEMENT.md) for the
full policy, allowlist rules, and what to do if a scan flags something.

## Testing Requirements

### Unit Tests
- Write tests for all new functionality
- Test edge cases and error conditions
- Mock external dependencies where appropriate

### Integration Tests
- Test contract interactions
- Verify cross-contract calls work correctly
- Ensure upgrade paths are tested

## Pull Request Process

1. **Ensure code compiles** without warnings
2. **Run all tests** and verify they pass
3. **Update documentation** if needed
4. **Describe changes** in the PR description
5. **Link related issues** if applicable

### PR Review Checklist
- [ ] Code follows naming conventions
- [ ] Tests are included and pass
- [ ] Documentation is updated
- [ ] No new Clippy warnings
- [ ] Code is properly formatted
- [ ] No secrets committed (gitleaks pre-commit hook passes)
- [ ] Contract review checklist items have been considered for correctness, safety, and testing

## Contract Review Checklist
Use the shared contract review checklist for all smart contract and contract-related pull requests: [docs/contract-review-checklist.md](./docs/contract-review-checklist.md)

## Development Setup

### Prerequisites
- Rust toolchain (stable)
- Soroban CLI
- Cargo make (optional)

### Local Development
```bash
# Clone the repository
git clone <repository-url>
cd Uzima-Contracts

# Install dependencies
cargo build

# Run tests
cargo test

# Run linter
cargo clippy -- -D warnings

# Format code
cargo fmt
```

### CI/CD Pipeline
The project uses GitHub Actions for CI/CD. The pipeline includes:
- Build verification
- Linting (Clippy)
- Testing
- Formatting check
- Secret scanning (gitleaks)

## Getting Help
- Review existing documentation in the `docs/` directory
- Check open issues for known problems
- Ask questions in pull request discussions

## License
By contributing, you agree that your contributions will be licensed under the project's MIT license.