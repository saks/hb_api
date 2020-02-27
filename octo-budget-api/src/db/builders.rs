use crate::db::models::*;
use bigdecimal::BigDecimal;
use chrono::offset::Local;

#[derive(Debug, Clone, Default)]
pub struct UserBuilder {
    pub email: String,
    pub password: String,
    pub username: String,
    pub tags: Vec<String>,
}

impl UserBuilder {
    pub fn username(mut self, username: &str) -> Self {
        self.username = username.into();
        self
    }

    pub fn password(mut self, password: &str) -> Self {
        self.password = password.into();
        self
    }

    pub fn tags(mut self, tags: Vec<&str>) -> Self {
        self.tags = tags.into_iter().map(|t| t.to_string()).collect();
        self
    }

    pub fn finish(self) -> AuthUser {
        AuthUser {
            id: 1,
            username: self.username,
            password: self.password,
            is_superuser: false,
            is_active: true,
            is_staff: false,
            email: self.email,
            first_name: String::new(),
            last_name: String::new(),
            date_joined: Local::now().naive_local(),
            tags: self.tags,
        }
    }
}

#[derive(Default, Clone)]
pub struct RecordBuilder {
    pub amount: BigDecimal,
    pub amount_currency: String,
    pub id: i32,
    pub tags: Vec<String>,
    pub transaction_type: String,
    pub comment: String,
    pub user_id: i32,
}

impl RecordBuilder {
    pub fn tags(mut self, tags: Vec<&str>) -> Self {
        self.tags = tags.into_iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn transaction_type(mut self, transaction_type: &str) -> Self {
        self.transaction_type = transaction_type.to_string();
        self
    }

    pub fn user_id(mut self, user_id: i32) -> Self {
        self.user_id = user_id;
        self
    }

    pub fn amount(mut self, amount: f64) -> Self {
        self.amount = BigDecimal::from(amount);
        self
    }

    pub fn finish(self) -> Record {
        Record {
            id: self.id,
            amount: self.amount,
            amount_currency: self.amount_currency,
            created_at: Local::now().naive_local(),
            tags: self.tags,
            transaction_type: self.transaction_type,
            user_id: self.user_id,
            comment: Some(self.comment),
        }
    }
}

#[derive(Default)]
pub struct BudgetBuilder {
    pub amount: BigDecimal,
    pub amount_currency: String,
    pub id: i32,
    pub name: String,
    pub tags: Vec<String>,
    pub tags_type: String,
    pub user_id: i32,
}

impl BudgetBuilder {
    pub fn tags_type(mut self, tags_type: &str) -> Self {
        self.tags_type = tags_type.to_string();
        self
    }

    pub fn tags(mut self, tags: Vec<&str>) -> Self {
        self.tags = tags.into_iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn user_id(mut self, user_id: i32) -> Self {
        self.user_id = user_id;
        self
    }

    pub fn finish(self) -> Budget {
        use chrono::naive::NaiveDate;

        Budget {
            amount: self.amount,
            amount_currency: self.amount_currency,
            id: self.id,
            name: self.name,
            tags: self.tags,
            tags_type: self.tags_type,
            user_id: self.user_id,
            start_date: NaiveDate::from_ymd(2015, 3, 14),
        }
    }
}
