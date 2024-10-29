# Build stage
FROM rust:1.75-slim-bullseye as builder

# Install required dependencies
RUN apt-get update && \
    apt-get install -y \
    pkg-config \
    libssl-dev \
    sqlite3 \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

# Create a new empty shell project
WORKDIR /usr/src/app
COPY . .

# Build the project with release optimizations
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y \
    ca-certificates \
    sqlite3 \
    libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the built binary from builder
COPY --from=builder /usr/src/app/target/release/pfp-checker /app/pfp-checker
# Copy migrations folder
COPY --from=builder /usr/src/app/migrations /app/migrations

# Create volume for persistent database storage
VOLUME ["/app/data"]

# Set environment variables
ENV DATABASE_URL=sqlite:/app/data/database.sqlite

# Run the binary
CMD ["./pfp-checker"]
