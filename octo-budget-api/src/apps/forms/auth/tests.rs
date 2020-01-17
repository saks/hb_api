use super::*;
#[test]
fn test_validate_password_ok() {
    let user = make_user_with_pass("foo");
    let result = Form::validate_password(&user, "foo");

    assert!(result.is_ok());
}

#[test]
fn test_validate_password_err() {
    let user = make_user_with_pass("foo");
    let result = Form::validate_password(&user, "bar");

    assert_eq!(ValidationErrors::bad_password(), result.unwrap_err());
}

fn make_user_with_pass(password: &'static str) -> AuthUser {
    use chrono::naive::NaiveDateTime;

    AuthUser {
        id: 123,
        username: "".to_string(),
        password: djangohashers::make_password(password),
        email: "".to_string(),
        is_active: true,
        is_superuser: true,
        first_name: "".to_string(),
        last_name: "".to_string(),
        is_staff: false,
        date_joined: NaiveDateTime::from_timestamp(0, 0),
        tags: Vec::new(),
    }
}

#[test]
fn test_validate_ok() {
    let form = Form {
        username: Some("foo".to_string()),
        password: Some("bar".to_string()),
    };

    let res = form.validate();
    assert!(res.is_ok());

    let params = res.unwrap();

    assert_eq!("foo", params.username);
    assert_eq!("bar", params.password);
}

#[test]
fn test_no_username() {
    let form = Form {
        username: None,
        password: Some("bar".to_string()),
    };

    let errors = form.validate().unwrap_err();

    assert_eq!(vec![ValidationError::MustPresent], errors.username);

    assert!(errors.password.is_empty());
    assert!(errors.non_field_errors.is_empty());
}

#[test]
fn test_no_password() {
    let form = Form {
        username: Some("foo".to_string()),
        password: None,
    };

    let errors = form.validate().unwrap_err();

    assert_eq!(vec![ValidationError::MustPresent], errors.password);

    assert!(errors.username.is_empty());
    assert!(errors.non_field_errors.is_empty());
}

#[test]
fn test_username_is_empty() {
    let form = Form {
        username: Some("".to_string()),
        password: Some("bar".to_string()),
    };

    let errors = form.validate().unwrap_err();

    assert_eq!(vec![ValidationError::CannotBeBlank], errors.username);

    assert!(errors.password.is_empty());
    assert!(errors.non_field_errors.is_empty());
}

#[test]
fn test_password_is_empty() {
    let form = Form {
        username: Some("foo".to_string()),
        password: Some("".to_string()),
    };

    let errors = form.validate().unwrap_err();

    assert_eq!(vec![ValidationError::CannotBeBlank], errors.password);

    assert!(errors.username.is_empty());
    assert!(errors.non_field_errors.is_empty());
}

#[test]
fn test_username_and_password_is_empty() {
    let form = Form {
        username: Some("".to_string()),
        password: Some("".to_string()),
    };

    let errors = form.validate().unwrap_err();

    assert_eq!(vec![ValidationError::CannotBeBlank], errors.password);

    assert_eq!(vec![ValidationError::CannotBeBlank], errors.username);

    assert!(errors.non_field_errors.is_empty());
}

#[test]
fn test_no_username_and_no_password() {
    let form = Form {
        username: None,
        password: None,
    };

    let errors = form.validate().unwrap_err();

    assert_eq!(vec![ValidationError::MustPresent], errors.password);
    assert_eq!(vec![ValidationError::MustPresent], errors.username);

    assert!(errors.non_field_errors.is_empty());
}
