use chrono::{DateTime, Utc};

pub struct user_full {
    id: String,
    username: String,
    email: String,
    created_at: Datetime<Utc>,
    update_at: Datetime<Utc>,
}

pub struct user {
    id: String,
    username: String,
    email: String,
}
