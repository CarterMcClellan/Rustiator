# Use a Rust image to build the application
FROM rust:bookworm as builder

# Copy your manifests
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

# This build step is to cache your dependencies
RUN mkdir ./src && echo 'fn main() { println!("Dummy!"); }' > ./src/main.rs && touch ./src/lib.rs
RUN cargo build --release
RUN rm src/*.rs

# Copy Deps
COPY ./src ./src
COPY ./client ./client
RUN touch ./src/main.rs
RUN touch ./src/lib.rs

# Build for release
RUN cargo build --release

# Use a smaller image to run the application
FROM debian:bookworm

# Copy the build artifact from the build stage
COPY --from=builder /target/release/server .
COPY ./client ./client

# Run the binary
CMD ["./server"]
