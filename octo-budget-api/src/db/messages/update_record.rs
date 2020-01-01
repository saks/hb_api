use std::result;

use actix::{Handler, Message as ActixMessage};
use bigdecimal::BigDecimal;
use octo_budget_lib::auth_token::UserId;

use crate::apps::forms::record::FormData;
use crate::db::DbExecutor;
use crate::errors::Error;

pub struct Message {
    amount: BigDecimal,
    amount_currency: String,
    tags: Vec<String>,
    transaction_type: String,
    user_id: UserId,
    id: i32,
}

impl Message {
    pub fn new(id: i32, data: &FormData, user_id: UserId) -> Self {
        Self {
            amount: data.amount.clone(),
            amount_currency: data.amount_currency.clone(),
            tags: data.tags.clone(),
            transaction_type: data.transaction_type.clone(),
            user_id,
            id,
        }
    }
}

impl ActixMessage for Message {
    type Result = result::Result<(), Error>;
}

impl Handler<Message> for DbExecutor {
    type Result = <Message as ActixMessage>::Result;

    fn handle(&mut self, msg: Message, _: &mut Self::Context) -> Self::Result {
        use crate::db::schema::records_record::dsl::*;
        use diesel::prelude::*;

        let connection = &self.pool.get()?;
        let current_user_id: i32 = msg.user_id.into();

        let target = records_record
            .filter(user_id.eq(current_user_id))
            .filter(id.eq(msg.id));

        let result = diesel::update(target)
            .set((
                amount.eq(msg.amount),
                amount_currency.eq(msg.amount_currency),
                tags.eq(msg.tags),
                transaction_type.eq(msg.transaction_type),
            ))
            .execute(connection);

        match result {
            Ok(1) => Ok(()),
            Ok(0) => Err(Error::RecordNotUpdated("records_record", msg.id)),
            Ok(_) => Err(Error::UnknownMsg("More than one record updated")),
            Err(err) => Err(Error::UnknownDb(err)),
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::db::builders::UserBuilder;
//     use crate::{get_db_message_result, tests::DbSession};
//     use actix::{Arbiter, System};
//     use futures::{future, Future};
//
//     #[test]
//     fn no_record_updated() {
//         let message = Message {
//             amount: BigDecimal::from(10.0),
//             amount_currency: "CAD".into(),
//             tags: vec![],
//             transaction_type: "INC".into(),
//             user_id: 1.into(),
//             id: 1,
//         };
//
//         get_db_message_result!(message, |res: <Message as ActixMessage>::Result| {
//             let err: failure::Error = res.err().unwrap().into();
//             assert_eq!(
//                 "Cannot update records_record with id: `1'",
//                 format!("{}", err)
//             );
//         });
//     }
//
//     #[test]
//     fn happy_path() {
//         let session = DbSession::new();
//         let user = session.create_user(UserBuilder::default().password("dummy password"));
//         let records = session.create_records2(user.id, 1);
//
//         let message = Message {
//             amount: BigDecimal::from(10.0),
//             amount_currency: "CAD".into(),
//             tags: vec![],
//             transaction_type: "INC".into(),
//             user_id: user.id.into(),
//             id: records[0].id,
//         };
//
//         get_db_message_result!(message, |res: <Message as ActixMessage>::Result| {
//             assert_eq!((), res.unwrap());
//         });
//     }
//
//     #[test]
//     fn check_update_result() {
//         let mut session = DbSession::new();
//         let user = session.create_user(UserBuilder::default().password("dummy password"));
//         let record = session.create_record2(user.id);
//
//         let message = Message {
//             amount: BigDecimal::from(10.0),
//             amount_currency: "USD".into(),
//             tags: vec!["foo".to_string()],
//             transaction_type: "INC".into(),
//             user_id: user.id.into(),
//             id: record.id,
//         };
//
//         get_db_message_result!(message, |res: <Message as ActixMessage>::Result| {
//             assert_eq!((), res.unwrap());
//         });
//
//         use crate::db::messages::GetRecords;
//         let get_records = GetRecords {
//             user_id: user.id,
//             page: 1,
//             per_page: 1,
//         };
//         get_db_message_result!(
//             get_records,
//             move |res: <GetRecords as ActixMessage>::Result| {
//                 let data = res.unwrap();
//                 let updated_record = data.results.get(0).unwrap();
//
//                 assert_ne!(record.amount, updated_record.amount);
//                 assert_ne!(record.amount_currency, updated_record.amount_currency);
//                 assert_ne!(record.tags, updated_record.tags);
//                 assert_ne!(record.transaction_type, updated_record.transaction_type);
//             }
//         );
//     }
// }
