table! {
    auth_user (id) {
        id -> Integer,
        email -> Text,
        username -> Text,
        password -> Text,
        is_active -> Bool,
    }
}

table! {
    records_record (id) {
        id -> Integer,
        created_at -> Timestamp,
        tags -> Array<Text>,
        amount_currency -> Text,
        transaction_type -> Text,
        // amount -> Float,
        user_id -> Integer,
    }
}
