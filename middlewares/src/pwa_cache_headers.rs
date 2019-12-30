use actix_service::{Service, Transform};
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    http::{header, HeaderValue, Uri},
    Error,
};
use futures::future::{ok, Future, Ready};
use std::{
    cell::RefCell,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

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
impl<S: 'static, B> Transform<S> for PwaCacheHeaders
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    S::Error: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = S::Error;
    type InitError = ();
    type Transform = PwaCacheHeadersMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(PwaCacheHeadersMiddleware {
            service: Rc::new(RefCell::new(service)),
        })
    }
}

pub struct PwaCacheHeadersMiddleware<S> {
    service: Rc<RefCell<S>>,
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
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    S::Error: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let mut svc = self.service.clone();
        let header_value = Self::header_value(&req);

        Box::pin(async move {
            let mut res = svc.call(req).await?;

            res.headers_mut()
                .insert(header::CACHE_CONTROL, header_value);

            Ok(res)
        })
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

    #[actix_rt::test]
    async fn test_wrap() {
        let mut app = init_service(
            App::new()
                .wrap(PwaCacheHeaders)
                .service(web::resource("/v1/something/").to(|| HttpResponse::Ok())),
        )
        .await;

        let req = TestRequest::with_uri("/v1/something/").to_request();
        let res = call_service(&mut app, req).await;

        assert!(res.status().is_success());
        assert_cache_header!(MAX_AGE, res);
    }

    #[actix_rt::test]
    async fn should_not_cache_manifest() {
        let mut app = init_service(
            App::new()
                .wrap(PwaCacheHeaders)
                .service(web::resource("/manifest.json").to(|| HttpResponse::Ok())),
        )
        .await;

        let req = TestRequest::with_uri("/manifest.json").to_request();
        let res = call_service(&mut app, req).await;

        assert!(res.status().is_success());
        assert_cache_header!("no-cache", res);
    }

    #[actix_rt::test]
    async fn should_not_cache_service_worker() {
        let mut app = init_service(
            App::new()
                .wrap(PwaCacheHeaders)
                .service(web::resource("/service-worker.js").to(|| HttpResponse::Ok())),
        )
        .await;

        let req = TestRequest::with_uri("/service-worker.js").to_request();
        let res = call_service(&mut app, req).await;

        assert!(res.status().is_success());
        assert_cache_header!("no-cache", res);
    }
}
