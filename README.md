# piping-server
[![CI](https://github.com/nwtgck/piping-server-rust/workflows/CI/badge.svg)](https://github.com/nwtgck/piping-server-rust/actions) [![CircleCI](https://circleci.com/gh/nwtgck/piping-server-rust.svg?style=shield)](https://circleci.com/gh/nwtgck/piping-server-rust) [![Docker Image Size (latest by date)](https://img.shields.io/docker/image-size/nwtgck/piping-server-rust)](https://hub.docker.com/r/nwtgck/piping-server-rust) [![Gitpod ready-to-code](https://img.shields.io/badge/Gitpod-ready--to--code-blue?logo=gitpod)](https://gitpod.io/#https://github.com/nwtgck/piping-server-rust) [![Gitpod ready-to-code](https://img.shields.io/badge/Gitpod-ready--to--code-blue?logo=gitpod)](https://gitpod.io/#https://github.com/nwtgck/piping-server-rust)

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

### Way 1: Binary

Executable files are available on [GitHub Release](https://github.com/nwtgck/piping-server-rust/releases) for Linux and macOS. You can download it and run it.

The executable file for Linux is portable because it is statically linked.

### Way 2: Docker
Run a Piping Server on <http://localhost:8181> by the following command.

```rs
docker run -p 8181:8080 --init nwtgck/piping-server-rust
```

### Way 3: Cargo
You can clone, build and run this project as follows.

```bash
git clone https://github.com/nwtgck/piping-server-rust.git
cd piping-server-rust
cargo run --release
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
