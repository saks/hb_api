use actix_service::{Service, Transform};
use futures::future::{ok, Either, FutureResult};

use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::http::{header, uri::PathAndQuery, Uri};
use actix_web::{error, HttpResponse, Result};

const HTTPS_SCHEME: &str = "https";
const ENV_VAR_NAME: &str = "FORCE_HTTPS";

#[derive(Default, Clone, Copy)]
pub struct ForceHttps;

impl ForceHttps {
    fn is_enabled() -> bool {
        std::env::var(ENV_VAR_NAME).is_ok()
    }
}

impl<S> Transform<S> for ForceHttps
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse>,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse;
    type Error = S::Error;
    type InitError = ();
    type Transform = ForceSslService<S>;
    type Future = FutureResult<Self::Transform, Self::InitError>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(ForceSslService {
            service,
            is_enabled: Self::is_enabled(),
        })
    }
}

pub struct ForceSslService<S> {
    service: S,
    is_enabled: bool,
}

impl<S> ForceSslService<S> {
    fn redirect_url(&self, req: &ServiceRequest) -> Option<Result<Uri>> {
        let connection_info = req.connection_info();

        if self.is_enabled || HTTPS_SCHEME == connection_info.scheme() {
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
            .scheme("https")
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

impl<S> Service for ForceSslService<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse>,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse;
    type Error = S::Error;
    type Future = Either<S::Future, FutureResult<Self::Response, Self::Error>>;

    fn poll_ready(&mut self) -> futures::Poll<(), Self::Error> {
        self.service.poll_ready()
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        if let Some(uri_res) = self.redirect_url(&req) {
            let resp = match uri_res {
                Ok(uri) => HttpResponse::MovedPermanently()
                    .header(header::LOCATION, uri.to_string())
                    .finish()
                    .into_body(),
                Err(_) => HttpResponse::InternalServerError().finish().into_body(),
            };

            Either::B(ok(req.into_response(resp)))
        } else {
            Either::A(self.service.call(req))
        }
    }
}
