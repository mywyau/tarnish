# Use the official Rust image as the base
FROM rust:latest

# Set the working directory inside the container
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock to the working directory
COPY Cargo.toml Cargo.lock ./

# Copy the source code and other necessary files to the working directory
COPY src ./src
COPY migrations ./migrations

# Build the project in release mode
RUN cargo build --release

# Install the compiled binary to /usr/local/cargo/bin
RUN cargo install --path .

# Expose the port that the application will run on
EXPOSE 8080

# Run the installed binary; specify the full path if needed
CMD ["/usr/local/cargo/bin/my-blog-backend"]
