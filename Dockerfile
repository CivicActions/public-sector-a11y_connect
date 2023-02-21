# Use the latest Rust image
FROM rust:latest

# Set the working directory
WORKDIR /app

# Copy the contents of the local directory to the working directory
COPY . .

# Set environment variables
ENV API_KEY=ChangeMe
ENV GOOGLE_PROJECT_ID=YourGoogleCloudProjectID
ENV GOOGLE_DATASET=YourDatasetName
ENV A11Y_URL=A11yWatchURL
ENV A11Y_JWT=A11yWatchAPIKey
ENV GOOGLE_APPLICATION_CREDENTIALS=YourGoogleCloudJsonCreds

# Install dependencies
RUN cargo install --path .

# Expose port 8000
EXPOSE 8000

# Build the application
RUN cargo build --release

# Set the command to run when the container starts
CMD ["./target/release/main"]
# CMD cargo run
