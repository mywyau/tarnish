# Use the official Rust image as the base
FROM rust:latest

# Set the working directory inside the container
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock to the working directory
COPY Cargo.toml Cargo.lock ./

# Copy the source code and other necessary files to the working directory
COPY src ./src
COPY migrations ./migrations

# Copy the .env file to the container
COPY .env .env

# Build the project in release mode
RUN cargo build --release

# Install the compiled binary
RUN cargo install --path .

# Verify that the binary exists (this is for debugging purposes)
RUN ls /usr/local/cargo/bin

# Expose the port that the application will run on
EXPOSE 8080

# Run the installed binary using the project name
CMD ["tarnish"]
