use actix_web::middleware::{Middleware, Started};
use actix_web::{
    error,
    http::{header, uri::PathAndQuery, Uri},
    HttpRequest, HttpResponse, Result,
};

const HTTPS_SCHEME: &str = "https";
const ENV_VAR_NAME: &str = "FORCE_HTTPS";

#[derive(Default)]
pub struct ForceHttps;

impl ForceHttps {
    fn is_enabled() -> bool {
        std::env::var(ENV_VAR_NAME).is_ok()
    }

    fn https_url(uri: &Uri, host: &str) -> Result<Uri, actix_web::Error> {
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
                error::ErrorUnprocessableEntity("Failed to record to HTTPS")
            })
    }
}

impl<S> Middleware<S> for ForceHttps {
    fn start(&self, req: &HttpRequest<S>) -> Result<Started> {
        let connection_info = req.connection_info();

        if !Self::is_enabled() || HTTPS_SCHEME == connection_info.scheme() {
            return Ok(Started::Done);
        }

        Self::https_url(req.uri(), connection_info.host()).map(|url| {
            let response = HttpResponse::MovedPermanently()
                .header(header::LOCATION, url.to_string())
                .finish();
            Started::Response(response)
        })
    }
}
