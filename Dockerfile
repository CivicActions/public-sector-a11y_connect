## Dockerfile to Generate Rust Containers

# Use the official rust image as the base image
FROM rust:latest as builder

# Set the working directory
WORKDIR /usr/src

# Copy the Cargo.toml and Cargo.lock files to the container
COPY Cargo.* ./

# Copy the rest of the application's source code to the container
COPY src/ ./src/
COPY Rocket.toml .

# Install the dependencies and build the application
RUN cargo build --release

# Use the official alpine image as the base image
FROM alpine:latest

# Set the working directory
WORKDIR /usr/src

# Copy the binary from the builder stage
COPY --from=builder /usr/src/target/release/Rusty-A11y .

# Set the GOOGLE_CLOUD_KEY environment variable
ENV GOOGLE_CLOUD_KEY=<YOUR_GOOGLE_CLOUD_KEY>

# Expose the port that the application binds to
EXPOSE 8080

# Start the application
CMD ["/usr/src/Rusty-A11y", "--port=8080"]
