#[derive(Queryable, Serialize, Debug, Clone)]
pub struct AuthUser {
    pub id: i32,
    pub email: String,
    pub username: String,
    pub password: String,
    pub is_active: bool,
}
