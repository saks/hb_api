use actix_web::{
    get, post, put,
    web::{self, Json, Path, Query},
    HttpResponse, Result,
};
use octo_budget_lib::auth_token::UserId;

use super::forms::record::Form;
use super::index_params::Params;
use crate::db::{
    queries::{CreateRecord, FindRecord, GetRecords, UpdateRecord},
    ConnectionPool,
};
use crate::redis::{
    helpers::{decrement_tags, increment_tags},
    Redis,
};

#[get("/record-detail/")]
async fn index(
    user_id: UserId,
    params: Query<Params>,
    pool: web::Data<ConnectionPool>,
) -> Result<HttpResponse> {
    let params = params.into_inner().validate()?;

    let message = GetRecords {
        page: params.page,
        per_page: params.per_page,
        user_id: user_id.into(),
    };

    let records = pool.execute(message).await?;

    Ok(HttpResponse::Ok().json(records))
}

#[post("/record-detail/")]
async fn create(
    user_id: UserId,
    form: Json<Form>,
    pool: web::Data<ConnectionPool>,
    redis: web::Data<Redis>,
) -> Result<HttpResponse> {
    use serde_json::json;

    let data = form.into_inner().validate()?;
    let id = pool.execute(CreateRecord::new(&data, user_id)).await?;

    increment_tags(user_id, data.tags, &redis).await?;

    Ok(HttpResponse::Ok().json(json!({ "id": id })))
}

#[put("/record-detail/{id}/")]
async fn update(
    user_id: UserId,
    record_id: Path<i32>,
    form: Json<Form>,
    pool: web::Data<ConnectionPool>,
    redis: web::Data<Redis>,
) -> Result<HttpResponse> {
    let record_id = record_id.into_inner();
    let data = form.into_inner().validate()?;

    let record = pool.execute(FindRecord::new(record_id, user_id)).await?;
    pool.execute(UpdateRecord::new(record.id, &data, user_id))
        .await?;

    decrement_tags(user_id, record.tags, &redis).await?;
    increment_tags(user_id, data.tags, &redis).await?;

    Ok(HttpResponse::Ok().json(""))
}

pub mod service {
    use super::*;
    use actix_web::dev::HttpServiceFactory;

    pub struct Service;

    impl HttpServiceFactory for Service {
        fn register(self, config: &mut actix_web::dev::AppService) {
            HttpServiceFactory::register(index, config);
            HttpServiceFactory::register(create, config);
            HttpServiceFactory::register(update, config);
        }
    }
}

#[cfg(test)]
mod tests;
