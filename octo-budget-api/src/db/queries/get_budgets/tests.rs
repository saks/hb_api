use super::*;
use crate::db::builders::{BudgetBuilder, RecordBuilder, UserBuilder};
use crate::tests::DbSession;
use bigdecimal::ToPrimitive;

#[test]
fn test_empty_result() {
    let message = GetBudgets {
        page: 1,
        per_page: 10,
        user_id: 123,
    };
    let session = DbSession::new();

    let data = handle(&message, session.conn()).unwrap();

    assert_eq!(0, data.total);
    assert_eq!(false, data.next);
    assert_eq!(false, data.previous);
    assert!(data.results.is_empty());
}

#[test]
fn test_first_page_result() {
    let mut session = DbSession::new();
    let user = session.create_user(UserBuilder::default().password("dummy password"));

    for _ in 0..12 {
        session.create_budget(BudgetBuilder::default().user_id(user.id).finish());
    }

    let message = GetBudgets {
        page: 1,
        per_page: 10,
        user_id: user.id,
    };
    let data = handle(&message, session.conn()).unwrap();

    assert_eq!(12, data.total);
    assert_eq!(false, data.previous);
    assert_eq!(true, data.next);
    assert_eq!(10, data.results.len());
}

#[test]
fn test_second_page_result() {
    let mut session = DbSession::new();
    let user = session.create_user(UserBuilder::default().password("dummy password"));
    for _ in 0..12 {
        session.create_budget(BudgetBuilder::default().user_id(user.id).finish());
    }

    let message = GetBudgets {
        page: 2,
        per_page: 10,
        user_id: user.id,
    };
    let data = handle(&message, session.conn()).unwrap();

    assert_eq!(12, data.total);
    assert_eq!(true, data.previous);
    assert_eq!(false, data.next);
    assert_eq!(2, data.results.len());
}

#[test]
fn test_records_for_correct_user() {
    let mut session = DbSession::new();
    let user1 = session.create_user(
        UserBuilder::default()
            .username("user1")
            .password("dummy password"),
    );
    session.create_budget(BudgetBuilder::default().user_id(user1.id).finish());

    let user2 = session.create_user(
        UserBuilder::default()
            .username("user2")
            .password("dummy password"),
    );
    session.create_budget(BudgetBuilder::default().user_id(user2.id).finish());

    let message = GetBudgets {
        page: 1,
        per_page: 10,
        user_id: user1.id,
    };
    let data = handle(&message, session.conn()).unwrap();

    assert_eq!(1, data.total);
    assert_eq!(false, data.previous);
    assert_eq!(false, data.next);
    assert_eq!(1, data.results.len());
}

#[test]
fn amount_aggregation_with_other_tags_type() {
    let mut session = DbSession::new();
    let user = session.create_user(UserBuilder::default().password("dummy password"));
    let budget = BudgetBuilder::default().user_id(user.id).finish();

    let record = RecordBuilder::default()
        .user_id(user.id)
        .transaction_type("EXP");

    session.create_record(record.clone().amount(1.123).finish());

    let amount = budget_spent(&budget, session.conn()).unwrap();

    assert_eq!(1.12, amount.to_f64().unwrap());
}

#[test]
fn amount_aggregation_with_including_tags() {
    let mut session = DbSession::new();
    let user = session.create_user(UserBuilder::default().password("dummy password"));
    let budget = BudgetBuilder::default()
        .user_id(user.id)
        .tags_type("INCL")
        .tags(vec!["foo"])
        .finish();

    let record = RecordBuilder::default()
        .user_id(user.id)
        .transaction_type("EXP");

    for (amount, tag) in [(1.0, "foo"), (3.0, "foo"), (2.0, "bar")].iter() {
        session.create_record(record.clone().amount(*amount).tags(vec![tag]).finish());
    }

    let amount = budget_spent(&budget, session.conn()).unwrap();

    assert_eq!(BigDecimal::from(4), amount);
}

#[test]
fn amount_aggregation_with_excluding_tags() {
    let mut session = DbSession::new();
    let user = session.create_user(UserBuilder::default().password("dummy password"));
    let budget = BudgetBuilder::default()
        .user_id(user.id)
        .tags_type("EXCL")
        .tags(vec!["foo"])
        .finish();

    let record = RecordBuilder::default()
        .user_id(user.id)
        .transaction_type("EXP");

    let test_data = [(1.0, "foo"), (3.0, "foo"), (2.0, "bar"), (4.0, "bar")];
    for (amount, tag) in test_data.into_iter() {
        let rec = record.clone().amount(*amount).tags(vec![tag]).finish();
        session.create_record(rec);
    }

    let amount = budget_spent(&budget, session.conn()).unwrap();

    assert_eq!(BigDecimal::from(6), amount);
}
