FROM rust:c9c623bcf8dd

WORKDIR /app

COPY . /app

RUN cargo build
