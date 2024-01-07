# Use the official Rust image as a base
FROM rust:slim as builder

# Argument to invalidate cache
ARG CACHEBUST=1

# Install protobuf compiler
RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*

# Create a new empty shell project
WORKDIR /usr/src/app

# Copy the entire workspace definition files
COPY ./Cargo.toml .

# Copy all workspace members
COPY ./core_db_entities ./core_db_entities
COPY ./core_operations ./core_operations
COPY ./protos ./protos
COPY ./graphql ./graphql

# Build the graphql and core_operations binary
RUN cargo build --release --package graphql

# Build the core_operations binary
RUN cargo build --release --package core_operations

# Start a new stage from a minimal Debian image to reduce the final size
FROM debian:bookworm-slim as graphql-runner

# Set environment variables for graphql-runner
ENV OAUTH_DOMAIN=https://dev-sudattas.eu.auth0.com/ \
    OAUTH_AUDIANCE=https://dev-sudattas.eu.auth0.com/api/v2/ \
    RUST_LOG=info \
    GRPC_URL=grpc://172.31.32.193:50051 

# Copy the graphql binary from the builder stage
COPY --from=builder /usr/src/app/target/release/graphql /usr/local/bin/graphql

# Set the command to run the graphql binary
CMD ["graphql"]

# Start another stage for the core_operations binary
FROM debian:bookworm-slim as core-operations-runner

# Set environment variables for core-operations-runner
ENV GRPC_SERVER=0.0.0.0:50051

# Copy the core_operations binary from the builder stage
COPY --from=builder /usr/src/app/target/release/core_operations /usr/local/bin/core_operations

# Set the command to run the core_operations binary
CMD ["core_operations"]
