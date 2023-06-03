FROM rust:latest as build

# 1a: Prepare toolchain
RUN apt update && \
    apt install -y musl-tools musl-dev && \
    rustup target add x86_64-unknown-linux-musl

# 1b: Download and compile Rust dependencies using fake source code and store as a separate Docker layer
WORKDIR /root

COPY --chown=0 .docker/main.rs src/
COPY --chown=0 Cargo.toml Cargo.lock ./

RUN cargo build --target x86_64-unknown-linux-musl --release

# 1c: Build the application using the real source code
COPY --chown=0 . .

RUN cargo build --target x86_64-unknown-linux-musl --release

# 2: Copy the excutable and extra files to an empty Docker image
FROM scratch

COPY --chown=0 --from=build /root/target/x86_64-unknown-linux-musl/release/tor-operator /usr/bin/tor-operator

ENTRYPOINT [ "tor-operator" ]
CMD [ "--help" ]
