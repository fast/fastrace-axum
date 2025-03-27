#![doc = include_str!("../README.md")]

use std::task::Context;
use std::task::Poll;

use fastrace::prelude::*;
use http::Request;
use tower_layer::Layer;
use tower_service::Service;

/// The standard [W3C Trace Context](https://www.w3.org/TR/trace-context/) header name for passing trace information.
///
/// This is the header key used to propagate trace context between services according to
/// the W3C Trace Context specification.
pub const TRACEPARENT_HEADER: &str = "traceparent";

/// Layer for intercepting and processing trace context in incoming requests.
///
/// This layer extracts tracing context from incoming requests and creates a new span
/// for each request. Add this to your axum server to automatically handle trace context
/// propagation.
#[derive(Clone)]
pub struct FastraceLayer;

impl<S> Layer<S> for FastraceLayer {
    type Service = FastraceService<S>;

    fn layer(&self, service: S) -> Self::Service {
        FastraceService { service }
    }
}

/// A service that handles trace context propagation.
///
/// This service extracts trace context from incoming requests and creates
/// spans to track the request processing. It wraps the inner service and augments
/// it with tracing capabilities.
#[derive(Clone)]
pub struct FastraceService<S> {
    service: S,
}

impl<S, Body> Service<Request<Body>> for FastraceService<S>
where S: Service<Request<Body>>
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = fastrace::future::InSpan<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let headers = req.headers();
        let parent = headers
            .get(TRACEPARENT_HEADER)
            .and_then(|traceparent| SpanContext::decode_w3c_traceparent(traceparent.to_str().ok()?))
            .unwrap_or(SpanContext::random());
        let root = Span::root(req.uri().to_string(), parent);
        self.service.call(req).in_span(root)
    }
}
