# NOTE: Multi-stage Build
FROM rust:1.41.1 as build
LABEL maintainer="Ryo Ota <nwtgck@nwtgck.org>"
# (from: https://blog.rust-lang.org/2016/05/13/rustup.html)
RUN rustup target add x86_64-unknown-linux-musl
RUN rustup toolchain install nightly
# Copy project
COPY . /app
# Build
RUN cd /app && cargo +nightly build --release


FROM ubuntu:18.04
# Copy binary
COPY --from=build /app/target/release/piping-server /app/target/release/piping-server
