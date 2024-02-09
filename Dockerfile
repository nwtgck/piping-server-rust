# NOTE: Multi-stage Build

FROM rust:1.76.0 as build

# Install tini
ENV TINI_VERSION v0.19.0
ADD https://github.com/krallin/tini/releases/download/${TINI_VERSION}/tini-static /tmp/tini-static
RUN chmod +x /tmp/tini-static

RUN apt update && apt install -y musl-tools
RUN rustup target add x86_64-unknown-linux-musl

# Copy to Cargo setting
COPY Cargo.toml Cargo.lock /app/
# Build empty project for better cache
RUN cd /app && \
    mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release --target=x86_64-unknown-linux-musl --locked && rm -r src

COPY . /app/
# Build
RUN cd /app && cargo build --release --target=x86_64-unknown-linux-musl --locked

FROM scratch
LABEL maintainer="Ryo Ota <nwtgck@nwtgck.org>"

# Copy executables
COPY --from=build /tmp/tini-static /tini-static
COPY --from=build /app/target/x86_64-unknown-linux-musl/release/piping-server /piping-server
# Run a server
ENTRYPOINT [ "/tini-static", "--", "/piping-server" ]
