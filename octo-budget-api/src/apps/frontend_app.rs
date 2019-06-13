use actix_web::{get, http::header, HttpResponse, Result};

#[get("/")]
pub fn index() -> Result<HttpResponse> {
    Ok(HttpResponse::PermanentRedirect()
        .header(header::LOCATION, "/public/index.html")
        .finish())
}
