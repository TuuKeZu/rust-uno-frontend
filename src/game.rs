use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct Card {
    pub r#type: String,
    pub color: String,
    pub owner: Option<Uuid>,
}

impl Card {
    pub fn new(r#type: &str, color: &str, owner: Uuid) -> Card {
        Card {
            r#type: r#type.to_string(),
            color: color.to_string(),
            owner: Some(owner),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    pub username: String,
    pub card_count: usize,
    pub index: usize,
    pub turn: bool,
    pub next: bool,
}

impl Player {
    pub fn new(username: String, card_count: usize, index: usize) -> Player {
        Player {
            username,
            card_count,
            index,
            turn: false,
            next: false,
        }
    }
}
