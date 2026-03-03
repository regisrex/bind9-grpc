use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Instant;
use tonic::{Request, Response, Status};
use tower::{Layer, Service};
use tracing::{debug, error, info, instrument};

#[derive(Debug, Clone)]
pub struct LoggingInterceptor<S> {
    inner: S,
}

impl<S> LoggingInterceptor<S> {
    pub fn new(service: S) -> Self {
        LoggingInterceptor { inner: service }
    }
}

#[derive(Debug, Clone)]
pub struct LoggingLayer;

impl LoggingLayer {
    pub fn new() -> Self {
        LoggingLayer
    }
}

impl<S> Layer<S> for LoggingLayer {
    type Service = LoggingInterceptor<S>;

    fn layer(&self, service: S) -> Self::Service {
        LoggingInterceptor::new(service)
    }
}

impl<S, ReqBody> Service<tonic::codegen::http::Request<ReqBody>> for LoggingInterceptor<S>
where
    S: Service<
            tonic::codegen::http::Request<ReqBody>,
            Response = tonic::codegen::http::Response<tonic::body::BoxBody>,
        > + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync + 'static>> + std::fmt::Display,
    ReqBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: tonic::codegen::http::Request<ReqBody>) -> Self::Future {
        let start_time = Instant::now();

        // Extract request information for logging
        let method = req.method().clone();
        let uri = req.uri().clone();
        let version = req.version();
        let headers = req.headers().clone();

        // Log incoming request
        info!(
            method = %method,
            uri = %uri,
            version = ?version,
            "Incoming gRPC request"
        );

        // Log headers in debug mode
        debug!(headers = ?headers, "Request headers");

        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Execute the request
            let response = inner.call(req).await;

            let duration = start_time.elapsed();

            match &response {
                Ok(res) => {
                    let status_code = res.status();
                    info!(
                        method = %method,
                        uri = %uri,
                        status = %status_code,
                        duration_ms = duration.as_millis(),
                        "gRPC request completed successfully"
                    );

                    // Log response headers in debug mode
                    debug!(headers = ?res.headers(), "Response headers");
                }
                Err(err) => {
                    error!(
                        method = %method,
                        uri = %uri,
                        duration_ms = duration.as_millis(),
                        error = %err,
                        "gRPC request failed"
                    );
                }
            }

            response
        })
    }
}

// Additional interceptor for gRPC-specific logging
#[derive(Debug)]
pub struct GrpcLoggingInterceptor;

impl GrpcLoggingInterceptor {
    pub fn new() -> Self {
        GrpcLoggingInterceptor
    }

    pub fn intercept<T>(&self, request: Request<T>) -> Result<Request<T>, Status> {
        let metadata = request.metadata();

        // Log metadata in debug mode
        debug!(metadata = ?metadata, "Request metadata");

        Ok(request)
    }

    pub fn intercept_response<T>(
        &self,
        response: Result<Response<T>, Status>,
    ) -> Result<Response<T>, Status> {
        match &response {
            Ok(res) => {
                let metadata = res.metadata();

                info!("gRPC method completed successfully");
                debug!(metadata = ?metadata, "Response metadata");
            }
            Err(status) => {
                let code = status.code();
                let message = status.message();

                error!(
                    code = ?code,
                    message = message,
                    "gRPC method failed"
                );
            }
        }

        response
    }
}

// Helper macros for logging in service implementations
#[macro_export]
macro_rules! log_grpc_request {
    ($service:literal, $method:literal, $request:expr) => {
        tracing::info!(
            service = $service,
            method = $method,
            "Processing gRPC request"
        );
        tracing::debug!(request = ?$request, "Request details");
    };
}

#[macro_export]
macro_rules! log_grpc_response {
    ($service:literal, $method:literal, $response:expr) => {
        tracing::info!(
            service = $service,
            method = $method,
            "gRPC request completed"
        );
        tracing::debug!(response = ?$response, "Response details");
    };
}

#[macro_export]
macro_rules! log_grpc_error {
    ($service:literal, $method:literal, $error:expr) => {
        tracing::error!(
            service = $service,
            method = $method,
            error = %$error,
            "gRPC request failed"
        );
    };
}
