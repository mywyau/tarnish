# Stage 1: Build the Rust application using musl for static linking
FROM rust:latest AS builder

# Set the working directory inside the container
WORKDIR /app

# Install dependencies required for PostgreSQL and musl target
RUN apt-get update && apt-get install -y libpq-dev musl-tools

# Add the musl target for static linking
RUN rustup target add x86_64-unknown-linux-musl

# Copy only Cargo.toml and Cargo.lock first to leverage caching of dependencies
COPY Cargo.toml Cargo.lock ./

# Fetch dependencies (will be cached unless Cargo.toml or Cargo.lock changes)
RUN cargo fetch

# Copy the rest of the source code
COPY . .

# Build the Rust project in release mode with musl target for static linking
RUN cargo build --release --target x86_64-unknown-linux-musl

# Stage 2: Create a lightweight image using 'scratch' for a statically linked binary
FROM scratch

# Set up a non-root user for security (optional, can be removed in 'scratch')
USER 1000

# Copy the statically compiled binary from the build stage
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/tarnish /usr/local/bin/tarnish

# Expose the port your Rust app is running on
EXPOSE 8080

# Run the binary
CMD ["/usr/local/bin/tarnish"]
