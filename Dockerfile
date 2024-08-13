# Use Rust official image for backend
FROM rust:latest

# Set the working directory
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock
COPY Cargo.toml Cargo.lock ./

# Build the dependencies
RUN cargo build --release

# Copy the source code
COPY . .

# Build the application
RUN cargo install --path .

# Expose the application on port 8080
EXPOSE 8080

# Run the application
CMD ["my-blog-backend"]
