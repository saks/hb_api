use actix_service::{Service, Transform};
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    http::{header, HeaderValue, Uri},
};
use futures::future::{ok, Future, FutureResult};
use futures::Poll;

const MAX_AGE: &str = "max-age=60";
const NO_CACHE: &str = "no-cache";

// There are two step in middleware processing.
// 1. Middleware initialization, middleware factory get called with
//    next service in chain as parameter.
// 2. Middleware's call method get called with normal request.
pub struct PwaCacheHeaders;

// Middleware factory is `Transform` trait from actix-service crate
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S> for PwaCacheHeaders
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>>,
    S::Future: 'static,
    S::Error: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = S::Error;
    type InitError = ();
    type Transform = PwaCacheHeadersMiddleware<S>;
    type Future = FutureResult<Self::Transform, Self::InitError>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(PwaCacheHeadersMiddleware { service })
    }
}

pub struct PwaCacheHeadersMiddleware<S> {
    service: S,
}

impl<S> PwaCacheHeadersMiddleware<S> {
    fn is_skip_cache(uri: &Uri) -> bool {
        let path = uri.path();
        path.ends_with("manifest.json") || path.ends_with("service-worker.js")
    }

    fn is_cache_file(uri: &Uri) -> bool {
        !Self::is_skip_cache(uri)
    }

    fn header_value(req: &ServiceRequest) -> HeaderValue {
        let header_value = if Self::is_cache_file(req.uri()) {
            MAX_AGE
        } else {
            NO_CACHE
        };

        HeaderValue::from_static(header_value)
    }
}

impl<S, B> Service for PwaCacheHeadersMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>>,
    S::Future: 'static,
    S::Error: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = S::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        self.service.poll_ready()
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let header_value = Self::header_value(&req);

        Box::new(self.service.call(req).and_then(move |mut res| {
            res.headers_mut()
                .insert(header::CACHE_CONTROL, header_value);
            Ok(res)
        }))
    }
}
