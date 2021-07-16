use actix_web::dev::{Service, Transform};
use futures::future::{ok, Either, Ready};
use std::task::{Context, Poll};

use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::http::{header, uri::PathAndQuery, Uri};
use actix_web::{error, Error, HttpResponse, Result};

const HTTPS_SCHEME: &str = "https";

pub struct ForceHttpsService<S> {
    service: S,
    is_enabled: bool,
}

impl<S> ForceHttpsService<S> {
    fn redirect_url(&self, req: &ServiceRequest) -> Option<Result<Uri>> {
        let connection_info = req.connection_info();

        if self.is_enabled && "http" == connection_info.scheme() {
            Some(Self::https_url(req.uri(), connection_info.host()))
        } else {
            None
        }
    }

    fn https_url(uri: &Uri, host: &str) -> Result<Uri> {
        let path_and_query = uri
            .path_and_query()
            .map(PathAndQuery::as_str)
            .unwrap_or_else(|| "");

        Uri::builder()
            .scheme(HTTPS_SCHEME)
            .authority(host)
            .path_and_query(path_and_query)
            .build()
            .map_err(|e| {
                log::error!(
                    "Failed to generate url: {:?}, err: {:?}, path_and_query: `{}', host: `{}'",
                    &uri,
                    e,
                    &path_and_query,
                    host
                );
                error::ErrorUnprocessableEntity("Failed to redirect to HTTPS")
            })
    }
}

impl<S, B> Service for ForceHttpsService<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = S::Error;
    type Future = Either<S::Future, Ready<Result<Self::Response, Self::Error>>>;

    fn poll_ready(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: S::Request) -> Self::Future {
        if let Some(uri_res) = self.redirect_url(&req) {
            let resp = match uri_res {
                Ok(uri) => HttpResponse::MovedPermanently()
                    .header(header::LOCATION, uri.to_string())
                    .finish()
                    .into_body(),
                Err(_) => HttpResponse::InternalServerError().finish().into_body(),
            };

            Either::Right(ok(req.into_response(resp)))
        } else {
            Either::Left(self.service.call(req))
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct ForceHttpsTransform {
    is_enabled: bool,
}

impl ForceHttpsTransform {
    pub fn new(is_enabled: bool) -> Self {
        Self { is_enabled }
    }
}

impl<S, B> Transform<S> for ForceHttpsTransform
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ForceHttpsService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(ForceHttpsService {
            service,
            is_enabled: self.is_enabled,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::{call_service, init_service, TestRequest};
    use actix_web::{web, App, HttpResponse};

    #[actix_rt::test]
    async fn test_wrap() {
        let mut app = init_service(
            App::new()
                .wrap(ForceHttpsTransform::new(false))
                .service(web::resource("/v1/something/").to(|| HttpResponse::Ok())),
        )
        .await;

        let req = TestRequest::with_uri("/v1/something/").to_request();
        let res = call_service(&mut app, req).await;
        assert!(res.status().is_success());
    }

    #[actix_rt::test]
    async fn test_redirect_http_request() {
        let mut app = init_service(
            App::new()
                .wrap(ForceHttpsTransform::new(true))
                .service(web::resource("/v1/something/").to(|| HttpResponse::Ok())),
        )
        .await;

        let req = TestRequest::with_uri("/v1/something/?a=1&a=2b=[]#/foo").to_request();
        let res = call_service(&mut app, req).await;

        assert_eq!(301, res.status());

        let location = res.headers().get("location").unwrap().to_str().unwrap();

        assert!(location.starts_with("https://"));

        // fragment part of the URL will be dropped (https://github.com/hyperium/http/issues/127)
        assert!(location.ends_with("/v1/something/?a=1&a=2b=[]"));
    }
}
