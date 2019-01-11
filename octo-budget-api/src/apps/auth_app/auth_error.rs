use failure_derive::Fail;
use serde::{Serialize, Serializer};

#[derive(Fail, Debug, Clone, Copy, PartialEq)]
pub enum AuthError {
    #[fail(display = "Unable to log in with provided credentials.")]
    AuthFailed,
    #[fail(display = "This field may not be blank.")]
    CannotBeBlank,
    #[fail(display = "This field is required.")]
    MustPresent,
}

impl Serialize for AuthError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(&self.to_string())
    }
}
