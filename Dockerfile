# Stage 1: Build the application
FROM rust:latest as builder

# Set the working directory inside the container
WORKDIR /app

# Copy Cargo.toml and Cargo.lock to the working directory
COPY Cargo.toml Cargo.lock ./

# Create a dummy src directory to pre-fetch dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build the dependencies to cache this layer
RUN cargo build --release

# Remove the dummy source code
RUN rm -rf src

# Copy the actual source code to the container
COPY src ./src
COPY migrations ./migrations

# Copy the .env file to the container
COPY .env .env

# Build the project in release mode
RUN cargo build --release

# Stage 2: Create a smaller image with the compiled binary
FROM debian:buster-slim

# Install necessary runtime dependencies for Diesel and PostgreSQL
RUN apt-get update && apt-get install -y libpq-dev

# Set the working directory inside the container
WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/tarnish .

# Copy the .env file to the container (if needed)
COPY --from=builder /app/.env .env

# Expose the port that the application will run on
EXPOSE 8080

# Run the compiled binary
CMD ["./tarnish"]
