use fastrace::collector::Config;
use fastrace::collector::ConsoleReporter;
use fastrace::prelude::*;
use fastrace_reqwest::traceparent_headers;
use reqwest::Client;

#[tokio::main]
async fn main() {
    // Initialize fastrace with the console reporter.
    fastrace::set_reporter(ConsoleReporter, Config::default());

    {
        // Create a root span for the client operation.
        let root = Span::root("client".to_string(), SpanContext::random());
        let _g = root.set_local_parent();

        send_request().await;
    }

    // Flush any remaining traces before the program exits.
    fastrace::flush();
}

/// Send an HTTP request to the server with trace context propagation.
/// The traceparent_headers() function adds the trace context to the request headers.
#[fastrace::trace]
async fn send_request() {
    let client = Client::new();
    let response = client
        .get("http://localhost:8080/ping")
        .headers(traceparent_headers())
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());
    println!("{:?}", response.text().await.unwrap());
}
