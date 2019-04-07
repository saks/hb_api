use actix_web::middleware::{Middleware, Started};
use actix_web::{
    error,
    http::{header, uri::PathAndQuery, Uri},
    HttpRequest, HttpResponse, Result,
};

#[derive(Default)]
pub struct ForceHttps; // <- Our middleware

/// Middleware implementation, middlewares are generic over application state,
/// so you can access state with `HttpRequest::state()` method.
impl<S> Middleware<S> for ForceHttps {
    /// Method is called when request is ready. It may return
    /// future, which should resolve before next middleware get called.
    fn start(&self, req: &HttpRequest<S>) -> Result<Started> {
        let connection_info = req.connection_info();

        match connection_info.scheme() {
            "https" => Ok(Started::Done),
            _ => {
                let uri = req.uri();
                let path_and_query = uri
                    .path_and_query()
                    .map(PathAndQuery::as_str)
                    .unwrap_or_else(|| "");

                Uri::builder()
                    .scheme("https")
                    .authority(connection_info.host())
                    .path_and_query(path_and_query)
                    .build()
                    .map_err(|e| {
                        log::error!(
                            "Failed to generate url: {:?}, err: {:?}, path_and_query: `{}', host: `{}'",
                            &uri,
                            e,
                            &path_and_query,
                            connection_info.host()
                        );
                        error::ErrorUnprocessableEntity("Failed to record to HTTPS")
                    })
                    .map(|url| {
                        let response = HttpResponse::MovedPermanently()
                            .header(header::LOCATION, url.to_string())
                            .finish();
                        Started::Response(response)
                    })
            }
        }
    }
}
