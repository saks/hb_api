use ::middlewares::pwa_cache_headers::PwaCacheHeaders;
use actix_files as fs;
use actix_service::NewService;
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    get,
    http::header,
    web, Error, HttpRequest, HttpResponse, Result, Scope,
};
use actix_web_async_compat::async_compat;
use futures::Future;

#[get("/")]
#[async_compat]
async fn index(_req: HttpRequest) -> Result<HttpResponse> {
    Ok(HttpResponse::PermanentRedirect()
        .header(header::LOCATION, "/public/index.html")
        .finish())
}

pub fn static_files_scope() -> Scope<
    impl NewService<Request = ServiceRequest, Response = ServiceResponse, Error = Error, InitError = ()>,
> {
    web::scope("/public")
        .wrap(PwaCacheHeaders)
        .service(fs::Files::new("/", "./reactapp/build"))
}
