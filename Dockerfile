# Use the official Rust image as the base
FROM rust:latest

# Set the working directory inside the container
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock to the working directory
COPY Cargo.toml Cargo.lock ./

# Copy the source code and other necessary files to the working directory
COPY src ./src
COPY migrations ./migrations

# Build the dependencies (this step will cache the dependencies)
RUN cargo build --release

# Copy the compiled output from the build stage
RUN cargo install --path .

# Expose the port that the application will run on
EXPOSE 8080

# Command to run the application
CMD ["my-blog-backend"]
