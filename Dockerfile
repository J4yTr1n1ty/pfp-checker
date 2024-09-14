# Stage 1: Build the Rust project
FROM rust:latest AS builder

# Install dependencies required for sqlx
RUN apt-get update && apt-get install -y \
    libpq-dev \
    libssl-dev \
    pkg-config \
    openssl

# Create a new empty shell project
RUN USER=root cargo new --bin pfp-checker
WORKDIR /pfp-checker

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Build dependencies (this is done to cache dependencies separately from the app code)
RUN cargo build --release

# Copy the rest of the source code
COPY . .

# If you are using sqlx offline mode (recommended for production), run:
RUN cargo sqlx prepare -- --lib
RUN cargo build --release

# Stage 2: Create a smaller image with only the compiled binary
FROM debian:buster-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libpq-dev \
    libssl1.1 \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /pfp-checker/target/release/pfp-checker /usr/local/bin/pfp-checker

# Set the binary as the entry point
ENTRYPOINT ["pfp-checker"]
