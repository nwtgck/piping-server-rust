# piping-server
[![CI](https://github.com/nwtgck/piping-server-rust/workflows/CI/badge.svg)](https://github.com/nwtgck/piping-server-rust/actions) [![CircleCI](https://circleci.com/gh/nwtgck/piping-server-rust.svg?style=shield)](https://circleci.com/gh/nwtgck/piping-server-rust) [![Docker Image Size (latest by date)](https://img.shields.io/docker/image-size/nwtgck/piping-server-rust)](https://hub.docker.com/r/nwtgck/piping-server-rust)

[![Deploy](https://www.herokucdn.com/deploy/button.svg)](https://heroku.com/deploy)

[Piping Server](https://github.com/nwtgck/piping-server) written in Rust

## Purpose
**Faster Piping Server than ever**  

* Faster is better
* Low memory cost
* Machine-friendly implementation

## Why Rust?
Safe, Fast and No garbage collection (GC)

## Run a server
You can choose some ways to run a server.

### Way 1: Docker
Run a Piping Server on <http://localhost:8181> by the following command.

```rs
docker run -p 8181:8080 nwtgck/piping-server-rust
```

### Way 2: Binary for Linux

```bash
# Download and extract
curl -L https://github.com/nwtgck/piping-server-rust/releases/download/v0.10.2/piping-server-x86_64-unknown-linux-musl.tar.gz | tar xzf -
# Run on 8181 port
./piping-server-x86_64-unknown-linux-musl/piping-server --http-port=8181
```

### Way 3: Binary for macOS (Intel)

```bash
# Download and extract
curl -L https://github.com/nwtgck/piping-server-rust/releases/download/v0.10.2/piping-server-x86_64-apple-darwin.tar.gz | tar xzf -
# Run on 8181 port
./piping-server-x86_64-apple-darwin/piping-server --http-port=8181
```

### Way 4: Binary for macOS (Apple Silicon)

```bash
# Download and extract
curl -L https://github.com/nwtgck/piping-server-rust/releases/download/v0.10.2/piping-server-aarch64-apple-darwin.tar.gz | tar xzf -
# Run on 8181 port
./piping-server-aarch64-apple-darwin/piping-server --http-port=8181
```

### Way 5: Windows

<https://github.com/nwtgck/piping-server-rust/releases/download/v0.10.2/piping-server-x86_64-pc-windows-gnu.zip>  

Executable files are available on [GitHub Release](https://github.com/nwtgck/piping-server-rust/releases).

### Way 6: Heroku

Click the button bellow to deploy.

[![Deploy](https://www.herokucdn.com/deploy/button.svg)](https://heroku.com/deploy)

### Way 7: Replit

Click <kbd>Fork</kbd> button in the link below and fork it.

<https://replit.com/@nwtgck/piping-rust>

### Way 8: Build and run by yourself
You can clone, build and run `piping-server` as follows.

```bash
# Clone
git clone https://github.com/nwtgck/piping-server-rust.git
cd piping-server-rust
# Build
cargo build --release
# Run on 8181 port
./target/release/piping-server --http-port=8181
```

### Server-side help

```txt
piping-server 0.11.0
Piping Server in Rust

USAGE:
    piping-server [OPTIONS]

OPTIONS:
        --http-port <HTTP_PORT>      HTTP port [default: 8080]
        --enable-https               Enable HTTPS
        --https-port <HTTPS_PORT>    HTTPS port
        --crt-path <CRT_PATH>        Certification path
        --key-path <KEY_PATH>        Private key path
        --bind-addr <BIND_ADDR>      Bind address [default: 0.0.0.0]
    -h, --help                       Print help information
    -V, --version                    Print version information
```
