# Multi-stage Dockerfile for Soroban development

# Build stage
FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    git \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Install Rust targets and components
RUN rustup target add wasm32-unknown-unknown && \
    rustup component add rustfmt clippy

# Install Soroban CLI
RUN cargo install --locked soroban-cli --features opt

# Set working directory
WORKDIR /workspace

# Copy manifests
COPY Cargo.toml rust-toolchain.toml ./
COPY contracts/ ./contracts/

# Build dependencies (this layer will be cached)
RUN cargo fetch

# Development stage
FROM builder as development

# Install additional development tools
RUN cargo install cargo-watch cargo-audit

# Copy source code
COPY . .

# Set up environment
ENV RUST_BACKTRACE=1
ENV CARGO_TERM_COLOR=always

# Expose port for local Stellar network
EXPOSE 8000

CMD ["/bin/bash"]

# Production build stage
FROM builder as production

# Copy source code
COPY . .

# Build optimized contracts
RUN cargo build --release --target wasm32-unknown-unknown

# Runtime stage
FROM debian:bookworm-slim as runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy Soroban CLI from builder
COPY --from=builder /usr/local/cargo/bin/soroban /usr/local/bin/

# Copy built contracts
COPY --from=production /workspace/contracts/*/target/wasm32-unknown-unknown/release/*.wasm /contracts/

# Copy scripts
COPY scripts/ /scripts/
RUN chmod +x /scripts/*.sh

WORKDIR /workspace

CMD ["/bin/bash"]