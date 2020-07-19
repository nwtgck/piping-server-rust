# NOTE: Multi-stage Build

FROM rust:1.45.0 as build

# (from: https://blog.rust-lang.org/2016/05/13/rustup.html)
RUN rustup target add x86_64-unknown-linux-musl

# (base: https://techno-tanoc.github.io/posts/rust-multistage-build/)
COPY Cargo.toml /app/Cargo.toml
COPY Cargo.lock /app/Cargo.lock
RUN mkdir /app/src
RUN echo "fn main() {}" > /app/src/main.rs
RUN cd /app && cargo build
RUN rm -r /app/src

COPY . /app

# Move to /app
WORKDIR /app

# Build
RUN cargo build --release


FROM ubuntu:18.04
LABEL maintainer="Ryo Ota <nwtgck@nwtgck.org>"

COPY --from=build /app/target/release/piping-server /app/target/release/piping-server

# Run a server
ENTRYPOINT [ "/app/target/release/piping-server" ]
