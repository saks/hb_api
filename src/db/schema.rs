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
        tags -> Array<String>,
        created_at -> Timestamptz,
        amount_currency -> String,
        transaction_type -> String,
        amount -> Float,
        user_id -> Integer,
    }
}
