use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Card {
    pub r#type: String,
    pub color: String,
}

impl Card {
    pub fn new(r#type: &str, color: &str) -> Card {
        Card {
            r#type: r#type.to_string(),
            color: color.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    pub username: String,
    pub card_count: usize,
    pub index: usize,
}

impl Player {
    pub fn new(username: String, card_count: usize, index: usize) -> Player {
        Player {
            username,
            card_count,
            index,
        }
    }
}
