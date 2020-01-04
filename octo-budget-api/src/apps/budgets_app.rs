use actix_web::{web, web::Query, HttpResponse, Result};
use octo_budget_lib::auth_token::UserId;

use super::index_params::Params;
use crate::db::{messages::GetBudgets, ConnectionPool};

async fn index(
    user_id: UserId,
    params: Query<Params>,
    pool: web::Data<ConnectionPool>,
) -> Result<HttpResponse> {
    let params = params.into_inner().validate()?;

    let query = GetBudgets {
        page: params.page,
        per_page: params.per_page,
        user_id: user_id.into(),
    };

    let budgets = pool.execute(query).await?;

    Ok(HttpResponse::Ok().json(budgets))
}

pub mod service {
    use super::*;
    use actix_web::dev::HttpServiceFactory;

    pub struct Service;

    impl HttpServiceFactory for Service {
        fn register(self, config: &mut actix_web::dev::AppService) {
            use actix_web::{guard::Get, Resource};

            HttpServiceFactory::register(
                Resource::new("/budget-detail/").guard(Get()).to(index),
                config,
            );
        }
    }
}
