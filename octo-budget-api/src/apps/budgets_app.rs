use actix_http::Error;
use actix_web::{web::Query, HttpResponse, Result};
use futures::Future;
use futures03::{compat::Future01CompatExt as _, FutureExt as _, TryFutureExt as _};
use octo_budget_lib::auth_token::UserId;

use super::index_params::Params;
use crate::db::{messages::GetBudgets, Pg};

async fn index(params: Query<Params>, pg: Pg, user_id: UserId) -> Result<HttpResponse> {
    let params = params.into_inner().validate()?;

    let message = GetBudgets {
        page: params.page,
        per_page: params.per_page,
        user_id: user_id.into(),
    };

    let budgets = Box::new(pg.send(message)).compat().await??;

    Ok(HttpResponse::Ok().json(budgets))
}

pub mod service {
    use super::*;
    use actix_web::dev::HttpServiceFactory;

    pub struct Service;

    fn __index(
        params: Query<Params>,
        pg: Pg,
        user_id: UserId,
    ) -> impl Future<Item = HttpResponse, Error = Error> {
        index(params, pg, user_id).boxed().compat()
    }

    impl HttpServiceFactory for Service {
        fn register(self, config: &mut actix_web::dev::AppService) {
            use actix_web::{guard::Get, Resource};

            HttpServiceFactory::register(
                Resource::new("/budget-detail/")
                    .guard(Get())
                    .to_async(__index),
                config,
            );
        }
    }

}
