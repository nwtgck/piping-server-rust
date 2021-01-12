# NOTE: Multi-stage Build

FROM ekidd/rust-musl-builder:1.49.0 as build

# Copy to current directory and change the owner
COPY --chown=rust:rust . ./
# Build
RUN cargo build --release


FROM scratch
LABEL maintainer="Ryo Ota <nwtgck@nwtgck.org>"

# Copy executable
COPY --from=build /home/rust/src/target/x86_64-unknown-linux-musl/release/piping-server /piping-server
# Run a server
ENTRYPOINT [ "/piping-server" ]
