use actix_service::{Service, Transform};
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    http::{header, HeaderValue, Uri},
};
use futures::future::{ok, Future, FutureResult};
use futures::Poll;

const MAX_AGE: &str = "max-age=31536000";
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

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::{call_service, init_service, TestRequest};
    use actix_web::{web, App, HttpResponse};

    macro_rules! assert_cache_header {
        ( $expected:expr, $req:expr ) => {{
            let header = $req
                .headers()
                .get("cache-control")
                .unwrap()
                .to_str()
                .unwrap();

            assert_eq!($expected, header);
        }};
    }

    #[test]
    fn test_wrap() {
        let mut app = init_service(
            App::new()
                .wrap(PwaCacheHeaders)
                .service(web::resource("/v1/something/").to(|| HttpResponse::Ok())),
        );

        let req = TestRequest::with_uri("/v1/something/").to_request();
        let res = call_service(&mut app, req);

        assert!(res.status().is_success());
        assert_cache_header!(MAX_AGE, res);
    }

    #[test]
    fn should_not_cache_manifest() {
        let mut app = init_service(
            App::new()
                .wrap(PwaCacheHeaders)
                .service(web::resource("/manifest.json").to(|| HttpResponse::Ok())),
        );

        let req = TestRequest::with_uri("/manifest.json").to_request();
        let res = call_service(&mut app, req);

        assert!(res.status().is_success());
        assert_cache_header!("no-cache", res);
    }

    #[test]
    fn should_not_cache_service_worker() {
        let mut app = init_service(
            App::new()
                .wrap(PwaCacheHeaders)
                .service(web::resource("/service-worker.js").to(|| HttpResponse::Ok())),
        );

        let req = TestRequest::with_uri("/service-worker.js").to_request();
        let res = call_service(&mut app, req);

        assert!(res.status().is_success());
        assert_cache_header!("no-cache", res);
    }
}
