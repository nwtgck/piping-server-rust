# NOTE: Multi-stage Build

FROM ekidd/rust-musl-builder:1.51.0 as build

# Copy to current directory and change the owner
COPY --chown=rust:rust . ./
# Build
RUN cargo build --release

# Install tini
ENV TINI_VERSION v0.19.0
ADD https://github.com/krallin/tini/releases/download/${TINI_VERSION}/tini-static /tini-static
RUN sudo chmod +x /tini-static

FROM scratch
LABEL maintainer="Ryo Ota <nwtgck@nwtgck.org>"

# Copy executables
COPY --from=build /tini-static /tini-static
COPY --from=build /home/rust/src/target/x86_64-unknown-linux-musl/release/piping-server /piping-server
# Run a server
ENTRYPOINT [ "/tini-static", "--", "/piping-server" ]
