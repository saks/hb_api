use super::{Request, Scope};
use crate::apps::middlewares::PwaCacheHeaders;
use actix_web::fs;
use actix_web::http::header;
use actix_web::HttpResponse;
use actix_web::{Responder, Result as WebResult};

pub fn index(_: &Request) -> WebResult<impl Responder> {
    Ok(HttpResponse::PermanentRedirect()
        .header(header::LOCATION, "/public/index.html")
        .finish())
}

pub fn scope(scope: Scope) -> Scope {
    scope.middleware(PwaCacheHeaders::default()).handler(
        "/",
        fs::StaticFiles::new("./reactapp/build")
            .unwrap()
            .show_files_listing(),
    )
}
