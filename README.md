# piping-server
[![CircleCI](https://circleci.com/gh/nwtgck/piping-server-rust.svg?style=shield)](https://circleci.com/gh/nwtgck/piping-server-rust)　[![](https://images.microbadger.com/badges/image/nwtgck/piping-server-rust.svg)](https://microbadger.com/images/nwtgck/piping-server-rust "Get your own image badge on microbadger.com")

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
You can choose Cargo or Docker to run a server.

### Cargo
```rs
cargo run --release
```

### Docker
Run a Piping Server on <http://localhost:8181> by the following command.

```rs
docker run -p 8181:8080 nwtgck/piping-server-rust
```

### Server-side help

```txt
Piping Server in Rust

USAGE:
    piping-server [FLAGS] [OPTIONS]

FLAGS:
        --enable-https    Enable HTTPS
    -h, --help            Prints help information
    -V, --version         Prints version information

OPTIONS:
        --crt-path <crt-path>        Certification path
        --http-port <http-port>      HTTP port [default: 8080]
        --https-port <https-port>    HTTPS port
        --key-path <key-path>        Private key path
```
