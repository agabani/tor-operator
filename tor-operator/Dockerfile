# 1: Build
FROM rust:1.51.0 as builder
RUN groupadd --gid 1000 appuser && \
    useradd --uid 1000 --gid 1000 --shell /bin/sh appuser
USER 1000:1000
WORKDIR /home/appuser

# 1a: Prepare for static linking

# 1b: Download and compile Rust dependencies (and store as a separate Docker layer)
RUN cargo new --bin app
WORKDIR /home/appuser/app
RUN cargo new --bin tor-operator && \
    cargo new --bin tor-operator-daemon
COPY --chown=appuser Cargo.lock Cargo.toml ./
COPY --chown=appuser tor-operator/Cargo.toml ./tor-operator/
COPY --chown=appuser tor-operator-daemon/Cargo.toml ./tor-operator-daemon/
RUN cargo build --release && \
    rm -rf src/

# 1c: Build the binary using the actual source code
COPY --chown=appuser tor-operator .
COPY --chown=appuser tor-operator-daemon .
RUN cargo build --release

# 2: Copy the exe and extra files to an empty Docker image
FROM rust:1.51.0-slim-buster
RUN groupadd --gid 1000 appuser && \
    useradd --uid 1000 --gid 1000 --shell /bin/sh appuser
USER 1000:1000
COPY --chown=appuser --from=builder /home/appuser/app/target/release/tor-operator /home/appuser/tor-operator
CMD ["./home/appuser/tor-operator"]
