use super::{Request, Scope};
use actix_web::fs::{self, NamedFile};
use actix_web::http::header;
use actix_web::middleware;
use failure::Fallible;
use std::path::PathBuf;
use crate::apps::middlewares::PwaCacheHeaders;

pub fn index(_: &Request) -> Fallible<NamedFile> {
    let path: PathBuf = PathBuf::from("reactapp/build/index.html");
    Ok(NamedFile::open(path)?)
}

pub fn scope(scope: Scope) -> Scope {
    scope
        .middleware(PwaCacheHeaders::default())
        .handler(
            "/",
            fs::StaticFiles::new("./reactapp/build")
                .unwrap()
                .show_files_listing(),
        )
}
