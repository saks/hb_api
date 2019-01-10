use actix_web::{HttpResponse, Json, Query, Responder, Result as WebResult, Scope};
use actix_web_async_await::{await, compat};

use crate::apps::{middlewares::VerifyAuthToken, AppState, Request, State};

mod db;
mod forms;

use self::db::{CreateNewRecordMessage, GetRecordsMessage};
use self::forms::RecordForm as Form;
use super::index_params::Params;
use super::index_response::Data;
use crate::db::models::Record as RecordModel;

type ResponseData = Data<RecordModel>;

// use crate::apps::Response;
// use actix_web::{AsyncResponder};
// use futures::{future, future::Future};
// fn index((query_params, state, request): (Query<Params>, State, Request)) -> Response {
//     let token = crate::auth_token_from_request!(request);
//     let params = query_params.into_inner();
//
//     let validation_result: Result<Params, ResponseData> = params.validate();
//     match validation_result {
//         Ok(Params { page, per_page }) => {
//             let user_id = token.user_id;
//
//             let message = GetRecordsMessage {
//                 page,
//                 per_page,
//                 user_id,
//             };
//
//             state
//                 .db
//                 .send(message)
//                 .from_err()
//                 .and_then(|result| {
//                     result
//                         .map(|data| HttpResponse::Ok().json(data))
//                         .map_err(|e| e.into())
//                 })
//                 .responder()
//         }
//         Err(response_data) => Box::new(future::ok(HttpResponse::BadRequest().json(response_data))),
//     }
// }

async fn index((params, state, req): (Query<Params>, State, Request)) -> WebResult<impl Responder> {
    let token = crate::auth_token_from_async_request!(req);
    let params = params.into_inner();
    let validation_result: Result<Params, ResponseData> = params.validate();

    let params = match validation_result {
        Ok(params) => params,
        Err(response_data) => {
            return Ok(HttpResponse::BadRequest().json(response_data));
        }
    };

    let message = GetRecordsMessage {
        page: params.page,
        per_page: params.per_page,
        user_id: token.user_id,
    };

    let res = await!(state.db.send(message))?;

    Ok(HttpResponse::Ok().json(res?))
}

use crate::apps::Redis;
use actix_redis::{Command, Error as RedisError, RespValue};
use failure::Fallible;
use redis_async::resp_array;
async fn increment_tags(user_id: i32, tags: &Vec<String>, redis: Redis) -> Fallible<()> {
    let key = crate::config::user_tags_redis_key(user_id);

    let responses = tags
        .iter()
        .map(|tag| resp_array!["zincrby", &key, "1", tag])
        .map(Command)
        .map(|cmd| redis.send(cmd))
        .collect::<Vec<_>>();

    await!(futures::future::join_all(responses))?
        .into_iter()
        .collect::<Result<Vec<RespValue>, RedisError>>()?;

    Ok(())
}

async fn create((form, state, req): (Json<Form>, State, Request)) -> WebResult<impl Responder> {
    let token = crate::auth_token_from_async_request!(req);
    let data = form.into_inner().validate()?;

    await!(state.db.send(CreateNewRecordMessage::new(&data, &token)))??;
    await!(increment_tags(
        token.user_id,
        &data.tags,
        state.redis.clone()
    ))?;

    Ok(HttpResponse::Ok().json(""))
}

pub fn scope(scope: Scope<AppState>) -> Scope<AppState> {
    scope
        .middleware(VerifyAuthToken::default())
        .resource("/record-detail/", |r| {
            r.get().with(compat(index));
            r.post().with(compat(create));
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{assert_response_body_eq, db::builders::UserBuilder, tests};
    use actix_web::{client::ClientRequest, http::StatusCode, test::TestServer};

    fn setup() -> TestServer {
        tests::setup_env();
        setup_test_server()
    }

    fn setup_test_server() -> TestServer {
        use crate::apps::middlewares::VerifyAuthToken;

        TestServer::build_with_state(|| AppState::new()).start(|app| {
            app.middleware(VerifyAuthToken::default())
                .resource("/api/records/record-detail/", |r| {
                    r.get().with(compat(index))
                });
        })
    }

    #[test]
    fn auth_required_for_records_app() {
        let mut srv = setup();

        let request = ClientRequest::build()
            .uri(&srv.url("/api/records/record-detail/"))
            .finish()
            .unwrap();

        let response = srv.execute(request.send()).unwrap();

        assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    }

    #[test]
    fn empty_list_of_records() {
        let mut session = tests::DbSession::new();
        let mut srv = setup();

        let user = session.create_user(UserBuilder::default().tags(vec!["foo"]));
        let request = tests::authenticated_request(&user, srv.url("/api/records/record-detail/"));
        let response = srv.execute(request.send()).unwrap();

        assert_eq!(StatusCode::OK, response.status(), "wrong status code");
        assert_response_body_eq!(
            srv,
            response,
            r#"{"total":0,"results":[],"next":false,"previous":false}"#
        );
    }
}
