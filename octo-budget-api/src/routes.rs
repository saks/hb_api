use actix_web::web;

use crate::apps;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(apps::frontend_app::index)
        .service(
            web::scope("/public")
                .wrap(middlewares::pwa_cache_headers::PwaCacheHeaders)
                .service(actix_files::Files::new("/", "./reactapp/build")),
        )
        .service(web::scope("/auth/jwt").service(apps::AuthService))
        .service(web::scope("/api/tags").service(apps::TagsService))
        .service(web::scope("/api/user").service(apps::users_app::show))
        .service(web::scope("/api/records").service(apps::RecordsService))
        .service(web::scope("/api/budgets").service(apps::BudgetsService));
}
