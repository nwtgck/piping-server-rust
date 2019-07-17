# piping-server
[![CircleCI](https://circleci.com/gh/nwtgck/piping-server-rust.svg?style=shield)](https://circleci.com/gh/nwtgck/piping-server-rust)　[![](https://images.microbadger.com/badges/image/nwtgck/piping-server-rust.svg)](https://microbadger.com/images/nwtgck/piping-server-rust "Get your own image badge on microbadger.com")

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
Run a Piping server on <http://localhost:8181> by the following command.

```rs
 docker run -p 8181:8080 --init nwtgck/piping-server-rust
```

### Server-side help

```txt
Piping Server in Rust

USAGE:
    piping-server [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --http-port <http-port>    Image width [default: 8080]
```
