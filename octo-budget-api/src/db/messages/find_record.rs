use crate::errors::Error;
use actix::{Handler, Message as ActixMessage};

use crate::db::{models::Record, DbExecutor};

pub struct Message {
    user_id: i32,
    id: i32,
}

impl Message {
    pub fn new(id: i32, user_id: i32) -> Self {
        Message { id, user_id }
    }
}

impl ActixMessage for Message {
    type Result = Result<Record, Error>;
}

impl Handler<Message> for DbExecutor {
    type Result = <Message as ActixMessage>::Result;

    fn handle(&mut self, msg: Message, _: &mut Self::Context) -> Self::Result {
        use crate::db::schema::records_record::dsl::*;
        use diesel::prelude::*;

        let connection = &self.pool.get()?;

        records_record
            .filter(user_id.eq(msg.user_id))
            .filter(id.eq(msg.id))
            .first(connection)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => Error::UserNotFound(msg.user_id),
                err => Error::UnknownDb(err),
            })
    }
}
