use std::{collections::VecDeque, time::SystemTime};

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameStatistics {
    pub start_time: Option<SystemTime>,
    pub end_time: Option<SystemTime>,
    pub player_count: usize,
    pub spectator_count: usize,
    pub cards_placed: usize,
    pub cards_drawn: usize,
}

impl GameStatistics {
    pub fn new() -> GameStatistics {
        GameStatistics {
            start_time: None,
            end_time: None,
            player_count: 0,
            spectator_count: 0,
            cards_placed: 0,
            cards_drawn: 0,
        }
    }
}

impl Default for GameStatistics {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EndStatus {
    pub winner_id: Uuid,
    pub winner: String,
    pub placements: VecDeque<String>,
    pub statistics: GameStatistics,
}
