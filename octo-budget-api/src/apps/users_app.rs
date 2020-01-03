use actix_web::{get, HttpResponse, Responder, Result as WebResult};
use serde_json::json;

use octo_budget_lib::auth_token::UserId;

#[get("/{user_id}/")]
pub async fn show(_current_user_id: UserId) -> WebResult<impl Responder> {
    Ok(HttpResponse::Ok().json(json!({})))
}
