use chrono::{DateTime, Utc};

pub struct parfume {
    id : String,
    brands_id : String,
    nama: String,
    concentration : Option<String>,
    description : Option<String>,
    created_at : DateTime<Utc>,
    update_at : DateTime<Utc>
}

pub struct batch_parfume {
    id : String,
    parfume_id : String,
    quantity_ml : f32,
    purchase_price : f32,
    created_at : DateTime<Utc>,
    update_at : DateTime<Utc>
}


enum batch_status {
    available,
    empty,
    inactive
}

pub struct batch_parfume_bottle {
    id : String,
    batch_parfume_id : String,
    remaining_ml : String,
    status : batch_status,
}

pub struct decant {
    id : String,
    parfume_id : String,
    size_ml : i32,
    sell_price : f32,
    is_active : boolean,
    created_at : DateTime<Utc>,
    update_at : DateTime<Utc>,
}



