table! {
    auth_user (id) {
        id -> Integer,
        email -> Text,
        username -> Text,
        password -> Text,
        is_active -> Bool,
    }
}
