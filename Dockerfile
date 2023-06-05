FROM rust:latest as build

# 1a: Prepare toolchain

# 1b: Download and compile Rust dependencies using fake source code and store as a separate Docker layer
WORKDIR /root

COPY --chown=0 .docker/main.rs src/
COPY --chown=0 Cargo.toml Cargo.lock ./

RUN cargo build --release

# 1c: Build the application using the real source code
COPY --chown=0 . .

RUN cargo build --release

# 2: Copy the excutable and extra files to an empty Docker image
FROM gcr.io/distroless/cc

COPY --chown=0 --from=build /root/target/release/tor-operator /usr/bin/tor-operator

ENTRYPOINT [ "tor-operator" ]
CMD [ "--help" ]
