# NOTE: Multi-stage Build

FROM rust:1.45.2 as build

# (from: https://blog.rust-lang.org/2016/05/13/rustup.html)
RUN rustup target add x86_64-unknown-linux-musl
COPY . /app
# Move to /app
WORKDIR /app
# Build
RUN cargo build --release


FROM ubuntu:18.04
LABEL maintainer="Ryo Ota <nwtgck@nwtgck.org>"

ENV TINI_VERSION "v0.19.0"
ADD https://github.com/krallin/tini/releases/download/${TINI_VERSION}/tini /usr/local/bin/tini
RUN chmod +x /usr/local/bin/tini

COPY --from=build /app/target/release/piping-server /app/target/release/piping-server

# Run a server
ENTRYPOINT [ "/usr/local/bin/tini", "--", "/app/target/release/piping-server" ]
