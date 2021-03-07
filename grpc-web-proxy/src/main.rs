use clap::Clap;
use grpc_web::Error;
use hyper::service::{make_service_fn, service_fn};
use hyper::{http::HeaderValue, Server};
use std::net::SocketAddr;

use proxy::{HttpConfig, Proxy};

mod proxy;

/// Simple gRPC-Web proxy, built in Rust.
#[derive(Clap)]
#[clap(version = "0.1", author = "Gregory Hill <gregorydhill@outlook.com>")]
struct Opts {
    /// Address to forward grpc requests to.
    #[clap(long, default_value = "http://[::1]:50052")]
    grpc_addr: String,

    /// Address to bind this proxy server to.
    #[clap(long, default_value = "[::1]:8080")]
    host_addr: String,

    /// Comma separated list of allowed origins.
    #[clap(long, default_value = "*")]
    allowed_cors_domains: HeaderValue,

    /// Comma separated list of allowed headers.
    #[clap(long, default_value = "*")]
    allowed_cors_headers: HeaderValue,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let opts: Opts = Opts::parse();

    let proxy = Proxy::new(
        opts.grpc_addr,
        HttpConfig {
            allowed_cors_domains: opts.allowed_cors_domains,
            allowed_cors_headers: opts.allowed_cors_headers,
        },
    )
    .await
    .expect("Unable to start proxy");

    let addr: SocketAddr = opts.host_addr.parse().expect("Invalid host_addr");

    let make_svc = make_service_fn(move |_| {
        let proxy = proxy.clone();

        async move {
            Ok::<_, Error>(service_fn(move |req| {
                let mut proxy = proxy.clone();
                async move {
                    let result = proxy.handle_http_request(req).await;
                    if let Err(ref err) = result {
                        eprintln!("{:?}", err);
                    }
                    result
                }
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
