FROM rust:alpine

RUN apk add musl-dev zig
#    rustup target add x86_64-unknown-linux-musl \
RUN cargo install cargo-zigbuild
RUN rustup target add x86_64-unknown-linux-musl
#    export PATH="$PATH;/usr/local/cargo/bin/"

WORKDIR /app