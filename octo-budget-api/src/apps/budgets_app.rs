use actix_web::{get, web, web::Query, HttpResponse, Result};
use octo_budget_lib::auth_token::UserId;

use super::index_params::Params;
use crate::db::{queries::GetBudgets, ConnectionPool};

#[get("/budget-detail/")]
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
            HttpServiceFactory::register(index, config);
        }
    }
}
