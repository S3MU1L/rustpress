use actix_web::{
    Error,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
};
use futures_util::future::{Ready, ok};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Security headers middleware
pub struct SecurityHeaders;

impl<S, B> Transform<S, ServiceRequest> for SecurityHeaders
where
    S: Service<
            ServiceRequest,
            Response = ServiceResponse<B>,
            Error = Error,
        >,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = SecurityHeadersMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(SecurityHeadersMiddleware { service })
    }
}

pub struct SecurityHeadersMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for SecurityHeadersMiddleware<S>
where
    S: Service<
            ServiceRequest,
            Response = ServiceResponse<B>,
            Error = Error,
        >,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<
        Box<dyn Future<Output = Result<Self::Response, Self::Error>>>,
    >;

    fn poll_ready(
        &self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;

            let headers = res.headers_mut();

            // Prevent clickjacking
            headers.insert(
                actix_web::http::header::HeaderName::from_static(
                    "x-frame-options",
                ),
                actix_web::http::header::HeaderValue::from_static(
                    "DENY",
                ),
            );

            // Prevent MIME sniffing
            headers.insert(
                actix_web::http::header::HeaderName::from_static(
                    "x-content-type-options",
                ),
                actix_web::http::header::HeaderValue::from_static(
                    "nosniff",
                ),
            );

            // Enable XSS protection
            headers.insert(
                actix_web::http::header::HeaderName::from_static(
                    "x-xss-protection",
                ),
                actix_web::http::header::HeaderValue::from_static(
                    "1; mode=block",
                ),
            );

            // Strict Transport Security (HSTS)
            headers.insert(
                actix_web::http::header::HeaderName::from_static(
                    "strict-transport-security",
                ),
                actix_web::http::header::HeaderValue::from_static(
                    "max-age=31536000; includeSubDomains",
                ),
            );

            // Content Security Policy
            headers.insert(
                actix_web::http::header::HeaderName::from_static("content-security-policy"),
                actix_web::http::header::HeaderValue::from_static(
                    "default-src 'self'; script-src 'self' 'unsafe-inline' https://esm.sh https://unpkg.com; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; connect-src 'self' https://esm.sh; frame-ancestors 'none'"
                ),
            );

            // Referrer Policy
            headers.insert(
                actix_web::http::header::HeaderName::from_static(
                    "referrer-policy",
                ),
                actix_web::http::header::HeaderValue::from_static(
                    "strict-origin-when-cross-origin",
                ),
            );

            Ok(res)
        })
    }
}
