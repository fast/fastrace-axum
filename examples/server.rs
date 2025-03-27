use fastrace::collector::Config;
use fastrace::collector::ConsoleReporter;
use tracing_subscriber::layer::SubscriberExt;

#[tokio::main]
async fn main() {
    tracing::subscriber::set_global_default(
        tracing_subscriber::Registry::default().with(fastrace_tracing::FastraceCompatLayer::new()),
    )
    .unwrap();
    logforth::stderr().apply();
    fastrace::set_reporter(ConsoleReporter, Config::default());

    let app = axum::Router::new()
        .route("/ping", axum::routing::get(ping))
        .layer(fastrace_axum::FastraceLayer);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[fastrace::trace]
async fn ping() -> &'static str {
    "pong"
}
