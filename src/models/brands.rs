use chrono::{DateTime, Utc};

pub struct brands_full {
    owner_id: String,
    id: String,
    name: String,
    created_at: DateTime<Utc>,
    update_at: DateTime<Utc>,
}

pub struct brands {
    owner_id: String,
    id: String,
    name: String,
}
