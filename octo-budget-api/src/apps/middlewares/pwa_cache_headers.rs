use actix_web::middleware::{Middleware, Response};
use actix_web::{
    http::{header, Uri},
    HttpRequest, HttpResponse, Result,
};

#[derive(Default)]
pub struct PwaCacheHeaders;

impl PwaCacheHeaders {
    fn is_skip_cache(uri: &Uri) -> bool {
        let path = uri.path();
        path.ends_with("manifest.json") || path.ends_with("service-worker.js")
    }

    fn is_cache_file(uri: &Uri) -> bool {
        !Self::is_skip_cache(uri)
    }
}

impl<S> Middleware<S> for PwaCacheHeaders {
    fn response(&self, req: &HttpRequest<S>, mut resp: HttpResponse) -> Result<Response> {
        let header_value = if Self::is_cache_file(req.uri()) {
            "max-age=60"
        } else {
            "no-cache"
        };

        resp.headers_mut().insert(
            header::CACHE_CONTROL,
            header::HeaderValue::from_static(header_value),
        );

        Ok(Response::Done(resp))
    }
}
