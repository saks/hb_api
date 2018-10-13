use chrono::NaiveDateTime;

#[derive(Queryable, Serialize, Debug, Clone, PartialEq)]
pub struct AuthUser {
    pub id: i32,
    pub email: String,
    pub username: String,
    pub password: String,
    pub is_active: bool,
}

#[derive(Queryable, Serialize, Debug, Clone, PartialEq)]
pub struct Record {
    pub id: i32,
    pub created_at: NaiveDateTime,
    pub tags: Vec<String>,
    // pub amount: f32,
    pub amount_currency: String,
    pub transaction_type: String,
    pub user_id: i32,
}
