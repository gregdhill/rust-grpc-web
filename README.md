# Rust gRPC Web (WIP)

Proxy service for gRPC-Web, written in Rust.

**[Specification](https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-WEB.md)**

## Structure

- **grpc-web**: logic & types
- **grpc-web-proxy**: hyper server

## Features

- [x] Unary Support
- [x] Server Streaming Support
- [ ] Client Streaming Support
- [ ] Bi-Directional Streaming Support

In a future release I aim to support an embedded service for Tonic just as `grpcweb` for Go wraps its handlers.

## Tutorial

Generate the JS helloworld client.

```shell
make helloworld
```

Run the helloworld gRPC server (built on Tonic).

```shell
cargo run --bin helloworld-server
```

Run the gRPC-Web proxy server.

```shell
cargo run --bin grpc-web-proxy
```

Open [index.html](./examples/helloworld/js/index.html) in a browser.