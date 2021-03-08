# Rust gRPC Web

Standalone proxy service for gRPC-Web, written in Rust.

**[Specification](https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-WEB.md)**

> Note: client-side & bi-directional streaming is not currently supported in the specification.

For efforts related to Tonic integration, see [this PR](https://github.com/hyperium/tonic/pull/455).

## Structure

- **grpc-web**: logic & types
- **grpc-web-proxy**: hyper server

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