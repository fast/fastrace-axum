# fastrace-axum

[![Crates.io](https://img.shields.io/crates/v/fastrace-axum.svg?style=flat-square&logo=rust)](https://crates.io/crates/fastrace-axum)
[![Documentation](https://img.shields.io/docsrs/fastrace-axum?style=flat-square&logo=rust)](https://docs.rs/fastrace-axum/)
[![MSRV 1.80.0](https://img.shields.io/badge/MSRV-1.80.0-green?style=flat-square&logo=rust)](https://www.whatrustisit.com)
[![CI Status](https://img.shields.io/github/actions/workflow/status/fast/fastrace-axum/ci.yml?style=flat-square&logo=github)](https://github.com/fast/fastrace-axum/actions)
[![License](https://img.shields.io/crates/l/fastrace-axum?style=flat-square)](https://github.com/fast/fastrace-axum/blob/main/LICENSE)

`fastrace-axum` is a middleware library that connects [fastrace](https://crates.io/crates/fastrace), a distributed tracing library, with [axum](https://crates.io/crates/axum), a web framework for Rust. This integration enables seamless trace context propagation across microservice boundaries in axum-based applications.

## What is Context Propagation?

Context propagation is a fundamental concept in distributed tracing that enables the correlation of operations spanning multiple services. When a request moves from one service to another, trace context information needs to be passed along, ensuring that all operations are recorded as part of the same trace.

`fastrace-axum` implements the [W3C Trace Context](https://www.w3.org/TR/trace-context/) standard for propagating trace information between services. This ensures compatibility with other tracing systems that follow the same standard.

## Features

- 🔄 **Automatic Context Propagation**: Automatically extract trace context from HTTP requests.
- 🌉 **Seamless Integration**: Works seamlessly with the `fastrace` library for complete distributed tracing.
- 📊 **Full Compatibility**: Works with fastrace's collection and reporting capabilities.

## Usage

### Server Integration

Add `fastrace-axum` to your Cargo.toml:

```toml
[dependencies]
fastrace = "0.7"
fastrace-axum = "0.1"
```

Apply the `FastraceLayer` to your axum server:

```rust
use fastrace::collector::Config;
use fastrace::collector::ConsoleReporter;

#[tokio::main]
async fn main() {
    // Configurate fastrace reporter.
    fastrace::set_reporter(ConsoleReporter, Config::default());

    let app = axum::Router::new()
        .route("/ping", axum::routing::get(ping))
        // Add a the FastraceLayer to routes.
        // The layer extracts trace context from incoming requests.
        .layer(fastrace_axum::FastraceLayer);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[fastrace::trace] // Trace individual handlers.
async fn ping() -> &'static str {
    "pong"
}

```

### Client Usage with fastrace-reqwest

To propagate trace context from clients to your axum service:

```rust
use fastrace::prelude::*;
use reqwest::Client;

#[fastrace::trace]
async fn send_request() {
    let client = Client::new();
    let response = client
        .get("http://your-axum-service/endpoint")
        .headers(fastrace_reqwest::traceparent_headers()) // Adds traceparent header.
        .send()
        .await
        .unwrap();

    // Process response...
}
```

## Example

Check out the [examples directory](https://github.com/fast/fastrace-axum/tree/main/example) for a complete ping/pong service example that demonstrates both client and server tracing.

To run the example:

1. Start the server:
   ```
   cargo run --example server
   ```

3. In another terminal, run the client:
   ```
   cargo run --example client
   ```

Both applications will output trace information showing the request flow, including the propagated context.

## How It Works

1. When a request arrives, the middleware checks for a `traceparent` header.
2. If present, it extracts the trace context; otherwise, it creates a new random context.
3. A new root span is created for the request using the URI as the name.
4. The request handler is executed within this span, and any child spans are properly linked.
5. The trace is then collected by your configured fastrace reporter.

## License

This project is licensed under the [Apache-2.0](./LICENSE) license.
