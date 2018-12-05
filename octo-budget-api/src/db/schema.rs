// table! {
//     auth_group (id) {
//         id -> Int4,
//         name -> Varchar,
//     }
// }
//
// table! {
//     auth_group_permissions (id) {
//         id -> Int4,
//         group_id -> Int4,
//         permission_id -> Int4,
//     }
// }
//
// table! {
//     auth_permission (id) {
//         id -> Int4,
//         name -> Varchar,
//         content_type_id -> Int4,
//         codename -> Varchar,
//     }
// }

table! {
    auth_user (id) {
        date_joined -> Timestamptz,
        email -> Varchar,
        first_name -> Varchar,
        id -> Int4,
        is_active -> Bool,
        is_staff -> Bool,
        is_superuser -> Bool,
        last_name -> Varchar,
        password -> Varchar,
        username -> Varchar,

        // last_login -> Nullable<Timestamptz>,
        // tags -> Nullable<Array<Varchar>>,
    }
}

// table! {
//     auth_user_groups (id) {
//         id -> Int4,
//         user_id -> Int4,
//         group_id -> Int4,
//     }
// }
//
// table! {
//     auth_user_user_permissions (id) {
//         id -> Int4,
//         user_id -> Int4,
//         permission_id -> Int4,
//     }
// }

table! {
    budgets_budget (id) {
        amount -> Numeric,
        amount_currency -> Varchar,
        id -> Int4,
        name -> Varchar,
        start_date -> Date,
        tags -> Array<Text>,
        tags_type -> Varchar,
        user_id -> Int4,
    }
}

// table! {
//     django_admin_log (id) {
//         id -> Int4,
//         action_time -> Timestamptz,
//         object_id -> Nullable<Text>,
//         object_repr -> Varchar,
//         action_flag -> Int2,
//         change_message -> Text,
//         content_type_id -> Nullable<Int4>,
//         user_id -> Int4,
//     }
// }
//
// table! {
//     django_content_type (id) {
//         id -> Int4,
//         app_label -> Varchar,
//         model -> Varchar,
//     }
// }
//
// table! {
//     django_migrations (id) {
//         id -> Int4,
//         app -> Varchar,
//         name -> Varchar,
//         applied -> Timestamptz,
//     }
// }
//
// table! {
//     django_session (session_key) {
//         session_key -> Varchar,
//         session_data -> Text,
//         expire_date -> Timestamptz,
//     }

// }
table! {
    records_record (id) {
        amount -> Numeric,
        amount_currency -> Varchar,
        created_at -> Timestamptz,
        id -> Int4,
        tags -> Array<Text>,
        transaction_type -> Varchar,
        user_id -> Int4,

        // id -> Int4,
        // tags -> Array<Varchar>,
        // amount_currency -> Varchar,
        // amount -> Numeric,
        // transaction_type -> Varchar,
        // created_at -> Timestamptz,
        // user_id -> Int4,
    }
}

// joinable!(auth_group_permissions -> auth_group (group_id));
// joinable!(auth_group_permissions -> auth_permission (permission_id));
// joinable!(auth_permission -> django_content_type (content_type_id));
// joinable!(auth_user_groups -> auth_group (group_id));
// joinable!(auth_user_groups -> auth_user (user_id));
// joinable!(auth_user_user_permissions -> auth_permission (permission_id));
// joinable!(auth_user_user_permissions -> auth_user (user_id));
// joinable!(django_admin_log -> auth_user (user_id));
// joinable!(django_admin_log -> django_content_type (content_type_id));
joinable!(budgets_budget -> auth_user (user_id));
joinable!(records_record -> auth_user (user_id));

allow_tables_to_appear_in_same_query!(
    auth_user,
    records_record,
    budgets_budget,
    //     auth_group,
    //     auth_group_permissions,
    //     auth_permission,
    //     auth_user_groups,
    //     auth_user_user_permissions,
    //     django_admin_log,
    //     django_content_type,
    //     django_migrations,
    //     django_session,
);
