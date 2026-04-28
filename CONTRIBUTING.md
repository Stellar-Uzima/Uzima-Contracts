# Contributing to Uzima Contracts

Thank you for your interest in contributing to Uzima Contracts! This document provides guidelines and standards for contributing to the project.

## Development Workflow

1. **Fork the repository** and create a feature branch
2. **Follow coding standards** outlined in [CODING_STANDARDS.md](./docs/CODING_STANDARDS.md)
3. **Write tests** for new functionality
4. **Run linting and tests** before submitting
5. **Submit a pull request** with a clear description

## Code Quality Standards

### Naming Conventions
All code must follow the naming conventions defined in [CODING_STANDARDS.md](./docs/CODING_STANDARDS.md):

- **Functions**: `snake_case`
- **Types**: `PascalCase`  
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Modules**: `snake_case`
- **Error enums**: Always use `Error`, never `Err`

### Code Style
- Use Rust 2021 edition
- Follow Rustfmt formatting (run `cargo fmt`)
- Adhere to Clippy linting rules (run `cargo clippy`)

### Documentation
- Document all public APIs with `///` doc comments
- Include examples for complex functions
- Update relevant documentation when changing functionality

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

## Getting Help
- Review existing documentation in the `docs/` directory
- Check open issues for known problems
- Ask questions in pull request discussions

## License
By contributing, you agree that your contributions will be licensed under the project's MIT license.