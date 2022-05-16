use crate::game::{Card, GameStatistics};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
pub enum PacketType {
    Register(String),                                          // username
    GameData(Uuid, String, Vec<(Uuid, String)>), // self_id, self_username, Vec<(id, username)>
    Connect(Uuid, String),                       // id, username
    Disconnect(Uuid, String),                    // id, username
    Message(String, String),                     // sender, content
    StartGame(String),                           // option
    StatusUpdatePublic(Uuid, String, usize, Card), // id, username, card-count, current
    StatusUpdatePrivate(Vec<Card>, Card),        // cards, current
    AllowedCardsUpdate(Vec<Card>),               // allowed-cards
    DrawCard(u8),                                // amount
    PlaceCard(usize),                            // index
    EndTurn,                                     //
    ColorSwitch(String),                         // color
    TurnUpdate(Uuid, Uuid),                      // current, next
    WinUpdate(Uuid, String, VecDeque<String>, GameStatistics), // id, username, placements, statistics
    Error(u64, String),                                        // error-code, body
}
