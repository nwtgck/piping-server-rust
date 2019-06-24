# NOTE: Multi-stage Build

FROM rust:1.35 as build

COPY . /app

# Move to /app
WORKDIR /app

# (from: https://blog.rust-lang.org/2016/05/13/rustup.html)
RUN rustup target add x86_64-unknown-linux-musl

# Build
RUN cargo build --release


FROM ubuntu:18.04
LABEL maintainer="Ryo Ota <nwtgck@gmail.com>"

COPY --from=build /app/target/release/piping-server /app/target/release/piping-server

# Run a server
ENTRYPOINT [ "/app/target/release/piping-server" ]
