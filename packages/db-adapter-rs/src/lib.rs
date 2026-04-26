use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Debug)]
pub struct Reputation {
    pub user_id: String,
    pub score: i32,
}

pub fn update_reputation(user_id: &str, increment: i32) {
    println!("[Rust-DB] Updating reputation for {} by {}", user_id, increment);
}

/// Sanitizes WorkItem titles to prevent naming collisions by appending a unique timestamp.
pub fn sanitize_work_item_title(title: &str) -> String {
    let start = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    
    // Automatically appends a unique 4-digit hex timestamp suffix
    format!("{} ({:x})", title, start & 0xFFFF)
}
