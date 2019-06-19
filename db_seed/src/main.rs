#[macro_use]
extern crate derive_more;

#[derive(FromStr, Display, From)]
struct Username(String);

#[derive(FromStr, Display, From)]
struct Email(String);

#[derive(FromStr, From, Display)]
struct Password(String);

fn read_user_input() -> (Username, Email, Password) {
    use djangohashers::make_password;
    use read_input::shortcut;
    use std::env;

    let username = env::var("ADMIN_DEFAULT_USERNAME")
        .map(From::from)
        .unwrap_or_else(|_| {
            print!("Username: ");
            shortcut::simple_input()
        });

    let email = env::var("ADMIN_DEFAULT_EMAIL")
        .map(From::from)
        .unwrap_or_else(|_| {
            print!("Email address: ");
            shortcut::simple_input()
        });

    let password: String = env::var("ADMIN_DEFAULT_PASSWORD").unwrap_or_else(|_| {
        print!("Password: ");
        shortcut::simple_input()
    });

    let password = make_password(&password).into();

    (username, email, password)
}

fn insert(admin_username: Username, admin_email: Email, new_pass: Password) -> Result<(), String> {
    use chrono::offset::Local;
    use diesel::*;
    use models::schema::auth_user::dsl::*;

    let database_url =
        std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL must be set".to_string())?;
    let conn = pg::PgConnection::establish(&database_url)
        .map_err(|e| format!("Error connecting to {}: {:?}", database_url, e))?;

    let now = Local::now().naive_local();

    let n = insert_into(auth_user)
        .values((
            username.eq(admin_username.to_string()),
            password.eq(new_pass.to_string()),
            is_superuser.eq(true),
            is_active.eq(true),
            is_staff.eq(true),
            email.eq(admin_email.to_string()),
            first_name.eq("Admin"),
            last_name.eq("Admin"),
            date_joined.eq(now),
            tags.eq(Vec::<String>::new()),
        ))
        .execute(&conn)
        .map_err(|e| format!("Failed to insert user: {:?}", e))?;

    assert_eq!(1, n, "Expected to insert exactly one user");

    Ok(())
}

fn main() -> Result<(), String> {
    let (username, email, password) = read_user_input();

    insert(username, email, password)?;

    Ok(())
}
