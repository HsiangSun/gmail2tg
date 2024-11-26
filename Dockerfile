# Use a lightweight Rust image as the base image
FROM rust:latest AS builder

# Install required packages
RUN apt-get update && apt-get install -y pkg-config libssl-dev musl-tools libsqlite3-dev

# Add the x86_64-unknown-linux-musl target to Rust
RUN rustup target add x86_64-unknown-linux-musl

# Install the stable Rust toolchain for x86_64-unknown-linux-musl
RUN rustup toolchain install stable-x86_64-unknown-linux-musl

# Set the working directory in the container
WORKDIR /app

# Copy your Rust project into the container
COPY . .

# Build the Rust project in release mode for the x86_64-unknown-linux-musl target
RUN cargo build --target x86_64-unknown-linux-musl --release --features vendored

# Create the final Docker image
FROM alpine:3.14

# Set the working directory in the container
WORKDIR /app

# Copy the compiled binary and .env.toml file from the builder stage
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/gmail2tg /app/gmail2tg
#COPY --from=builder /app/.env.toml.toml /app/.env.toml.toml

# Start your Rust application
CMD ["/app/gmail2tg"]