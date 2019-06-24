FROM rust:1.35

LABEL maintainer="Ryo Ota <nwtgck@gmail.com>"

COPY . /app

# Move to /app
WORKDIR /app

# Build
RUN cargo build --release

# Run a server
ENTRYPOINT [ "/app/target/release/piping-server" ]
