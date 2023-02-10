# Use the latest Rust image as the base
FROM rust:latest

# Set the working directory
WORKDIR /app

# Copy the application code into the container
COPY . .

# Install the dependencies
RUN cargo install --path .

# Build the application
RUN cargo build --release

# Set the command to run when the container starts
CMD ["./target/release/public-sector-a11y_connect"]