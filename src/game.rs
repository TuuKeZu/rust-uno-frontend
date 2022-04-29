use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Card {
    pub r#type: String,
    pub color: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    pub username: String,
    pub card_count: u8,
}

impl Card {
    pub fn new(r#type: &str, color: &str) -> Card {
        Card {
            r#type: r#type.to_string(),
            color: color.to_string(),
        }
    }
}
