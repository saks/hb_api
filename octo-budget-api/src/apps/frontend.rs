use super::{Request, Scope};
use actix_web::{
    fs::{self, NamedFile},
    http::Method,
};
use failure::Fallible;
use std::path::PathBuf;

pub fn index(_: &Request) -> Fallible<NamedFile> {
    let path: PathBuf = PathBuf::from("reactapp/build/index.html");
    Ok(NamedFile::open(path)?)
}

pub fn scope(scope: Scope) -> Scope {
    scope
        .resource("/", |r| r.method(Method::GET).f(index))
        .handler(
            "/static",
            fs::StaticFiles::new("./reactapp/build")
                .unwrap()
                .show_files_listing(),
        )
}
