// use std::convert::Into;

use actix::{Handler, Message};
use actix_web::middleware::Logger;
use actix_web::{App, AsyncResponder, FutureResponse, HttpRequest, HttpResponse, Query, State};
use diesel::prelude::*;
use failure::Error;
use std::result;
// use actix_web::{App, AsyncResponder, FutureResponse, HttpResponse, Query, State};
use futures::{future, future::Future};

use apps::middlewares::auth_by_token::VerifyAuthToken;
use apps::AppState;
use auth_token::AuthToken;

#[derive(Deserialize, Debug, Default, Clone)]
struct Params {
    #[serde(default)]
    page: i64,
}

use db::pagination::*;
use db::{models::Record as RecordModel, schema::records_record, DbExecutor};
pub type GetRecordsResult = result::Result<Vec<RecordModel>, Error>;
struct GetRecordsMessage {
    user_id: i32,
    page: i64,
}

impl Message for GetRecordsMessage {
    type Result = GetRecordsResult;
}

impl Handler<GetRecordsMessage> for DbExecutor {
    type Result = GetRecordsResult;

    fn handle(&mut self, msg: GetRecordsMessage, _: &mut Self::Context) -> Self::Result {
        let connection = &self.0.get()?;
        let page = msg.page;
        let user_id = msg.user_id;
        let per_page = Some(10);

        let mut query = records_record::table
            .select(records_record::all_columns)
            .filter(records_record::user_id.eq(user_id))
            .order(records_record::created_at.desc())
            .paginate(page);

        if let Some(per_page) = per_page {
            use std::cmp::min;
            query = query.per_page(min(per_page, 25));
        }

        let results = query.load::<(RecordModel, i64)>(&*connection)?;

        // let (records, total_pages) =
        //     query.load_and_count_pages::<(Vec<RecordModel>, i64)>(&*connection)?;
        println!("Results {:?}", results);

        let results: Vec<RecordModel> = records_record::table
            .select(records_record::all_columns)
            .filter(records_record::user_id.eq(user_id))
            .order(records_record::created_at.desc())
            .limit(10)
            .get_results(&*connection)
            .unwrap();

        Ok(results)
    }
}

fn index(
    (query_params, state, request): (Query<Params>, State<AppState>, HttpRequest<AppState>),
) -> FutureResponse<HttpResponse> {
    let params = query_params.into_inner();

    let token: AuthToken = match request.extensions_mut().remove() {
        Some(token) => token,
        None => {
            return Box::new(future::ok(HttpResponse::Unauthorized().finish()));
        }
    };

    let get_records_message = GetRecordsMessage {
        page: params.page,
        user_id: token.data.user_id,
    };

    state
        .db
        .send(get_records_message)
        .from_err()
        .and_then(|r| {
            println!("res: {:?}", r);
            //
            Ok(HttpResponse::Ok().json("TODO"))
        })
        .responder()

    // println!("user_id: {:?}", user_id);
    // Box::new(future::ok(HttpResponse::Ok().json("TODO")))
}

pub fn build() -> App<AppState> {
    App::with_state(AppState::new())
        .prefix("/api/records/record-detail")
        .middleware(Logger::default())
        .middleware(VerifyAuthToken::new())
        .resource("/", |r| r.get().with(index))
}

#[cfg(test)]
mod test {
    use super::*;
    use actix_web::client::ClientRequest;
    use actix_web::http::StatusCode;
    use actix_web::test::TestServer;
    use dotenv::dotenv;

    fn setup() {
        dotenv().ok().expect("Failed to parse .env file");
    }

    fn make_token(hours_from_now: i64, secret_str: &str) -> String {
        use frank_jwt::{encode, Algorithm};
        use time::{now_utc, Duration};

        let exp = (now_utc() + Duration::hours(hours_from_now))
            .to_timespec()
            .sec;
        let header = json!({ "exp": exp });
        let payload = json!({ "user_id": 123 });
        let secret = secret_str.to_string();

        encode(header, &secret, &payload, Algorithm::HS256).expect("failed to encode token")
    }

    #[test]
    fn test_auth_required_for_records_app() {
        setup();

        let mut srv = TestServer::with_factory(build);

        let request = ClientRequest::build()
            .uri(&srv.url("/api/records/record-detail/"))
            .finish()
            .unwrap();

        let response = srv.execute(request.send()).unwrap();

        assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    }

    #[test]
    fn test_auth_success_for_records_app() {
        setup();

        let mut srv = TestServer::with_factory(build);
        let token = make_token(12, "foo");

        let request = ClientRequest::build()
            .header("Authorization", token)
            .uri(&srv.url("/api/records/record-detail/"))
            .finish()
            .unwrap();

        let response = srv.execute(request.send()).unwrap();

        assert_eq!(StatusCode::OK, response.status());
        // TODO: check body
    }
}
