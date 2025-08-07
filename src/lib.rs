#![doc = include_str!("../README.md")]

use std::future::Future;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

use axum::extract::MatchedPath;
use axum::response::Response;
use fastrace::future::InSpan;
use fastrace::local::LocalSpan;
use fastrace::prelude::*;
use opentelemetry_semantic_conventions::trace::HTTP_REQUEST_METHOD;
use opentelemetry_semantic_conventions::trace::HTTP_RESPONSE_STATUS_CODE;
use opentelemetry_semantic_conventions::trace::HTTP_ROUTE;
use opentelemetry_semantic_conventions::trace::URL_PATH;
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

use axum::extract::Request;

impl<S> Service<Request> for FastraceService<S>
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = InSpan<InspectHttpResponse<S::Future>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let headers = req.headers();
        let parent = headers
            .get(TRACEPARENT_HEADER)
            .and_then(|traceparent| SpanContext::decode_w3c_traceparent(traceparent.to_str().ok()?))
            .unwrap_or(SpanContext::random());

        let span_name = get_request_span_name(&req);
        let root = Span::root(span_name, parent);

        root.add_properties(|| {
            [
                (HTTP_REQUEST_METHOD, req.method().to_string()),
                (URL_PATH, req.uri().path().to_string()),
            ]
        });
        if let Some(route) = req.extensions().get::<MatchedPath>() {
            root.add_property(|| (HTTP_ROUTE, route.as_str().to_string()));
        }

        let fut = self.service.call(req);
        let fut = InspectHttpResponse { inner: fut };
        fut.in_span(root)
    }
}

#[pin_project::pin_project]
pub struct InspectHttpResponse<F> {
    #[pin]
    inner: F,
}

impl<F, E> Future for InspectHttpResponse<F>
where F: Future<Output = Result<Response, E>>
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let poll = this.inner.poll(cx);

        if let Poll::Ready(Ok(response)) = &poll {
            LocalSpan::add_property(|| {
                (
                    HTTP_RESPONSE_STATUS_CODE,
                    response.status().as_u16().to_string(),
                )
            });
        }

        poll
    }
}

// See [OpenTelemetry semantic conventions](https://opentelemetry.io/docs/specs/semconv/http/http-spans/#name)
fn get_request_span_name(req: &Request) -> String {
    let method = req.method().as_str();
    if let Some(target) = req.extensions().get::<MatchedPath>() {
        format!("{} {}", method, target.as_str())
    } else {
        method.to_string()
    }
}
