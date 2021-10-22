# NOTE: Multi-stage Build

FROM nwtgck/rust-musl-builder:1.56.0 as build

# Copy to current directory and change the owner
COPY --chown=rust:rust . ./
# Build
RUN cargo build --release


FROM alpine:3.14.2
LABEL maintainer="Ryo Ota <nwtgck@nwtgck.org>"

# Copy executable
COPY --from=build /home/rust/src/target/x86_64-unknown-linux-musl/release/piping-server /usr/local/bin/piping-server
