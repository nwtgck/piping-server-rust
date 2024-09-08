# NOTE: Multi-stage Build

FROM rust:1.81.0 as build

ARG TARGETPLATFORM

ENV TINI_STATIC_VERSION 0.19.0

RUN apt update && apt install -y musl-tools

RUN case $TARGETPLATFORM in\
      linux/amd64)  rust_target="x86_64-unknown-linux-musl";\
                    tini_static_arch="amd64";;\
      linux/arm64)  rust_target="aarch64-unknown-linux-musl";\
                    tini_static_arch="arm64";;\
      *)            exit 1;;\
    esac &&\
    echo $rust_target > /tmp/rust_target.txt &&\
    # Install tini
    curl -fL https://github.com/krallin/tini/releases/download/v${TINI_STATIC_VERSION}/tini-static-${tini_static_arch} > /tmp/tini-static &&\
    chmod +x /tmp/tini-static &&\
    rustup target add $rust_target

# Copy to Cargo setting
COPY Cargo.toml Cargo.lock /app/
# Build empty project for better cache
RUN cd /app && \
    mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release --target=$(cat /tmp/rust_target.txt) --locked && rm -r src

COPY . /app/
# Build
RUN cd /app && cargo build --release --target=$(cat /tmp/rust_target.txt) --locked
RUN cp -a /app/target/$(cat /tmp/rust_target.txt)/ /tmp/build/

FROM scratch
LABEL maintainer="Ryo Ota <nwtgck@nwtgck.org>"

# Copy executables
COPY --from=build /tmp/tini-static /tini-static
COPY --from=build /tmp/build/release/piping-server /piping-server
# Run a server
ENTRYPOINT [ "/tini-static", "--", "/piping-server" ]
