# 1: Build
FROM rust:1.50.0 as builder
WORKDIR /usr/src

# 1a: Prepare for static linking

# 1b: Download and compile Rust dependencies (and store as a separate Docker layer)
RUN USER=root cargo new --bin app
WORKDIR /usr/src/app
RUN USER=root cargo new --bin tor-operator && \
              cargo new --bin tor-operator-daemon
COPY Cargo.lock Cargo.toml ./
COPY tor-operator/Cargo.toml ./tor-operator/
COPY tor-operator-daemon/Cargo.toml ./tor-operator-daemon/
RUN cargo build --release && \
    rm -rf src/

# 1c: Build the binary using the actual source code
COPY tor-operator-daemon .
RUN cargo build --release

# 2: Copy the exe and extra files to an empty Docker image
FROM rust:1.50.0-slim-buster
COPY --from=builder /usr/src/app/target/release/tor-operator-daemon .
CMD ["./tor-operator-daemon"]
