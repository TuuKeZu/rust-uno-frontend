use crate::game::Card;
use serde::{Deserialize, Serialize};
use serde_json::Result;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterPacket {
    pub r#type: String,
    pub username: String,
}

impl RegisterPacket {
    pub fn new(username: &str) -> RegisterPacket {
        RegisterPacket {
            r#type: "REGISTER".to_string(),
            username: username.to_string(),
        }
    }

    pub fn try_parse(data: &str) -> Result<RegisterPacket> {
        serde_json::from_str(data)
    }

    pub fn to_json(data: RegisterPacket) -> String {
        serde_json::to_string(&data).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MessagePacket {
    pub r#type: String,
    pub content: String,
}

impl MessagePacket {
    pub fn new(content: &str) -> MessagePacket {
        MessagePacket {
            r#type: String::from("MESSAGE"),
            content: String::from(content),
        }
    }
    pub fn try_parse(data: &str) -> Result<MessagePacket> {
        serde_json::from_str(data)
    }

    pub fn to_json(data: MessagePacket) -> String {
        serde_json::to_string(&data).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StartPacket {
    pub r#type: String,
    pub options: String,
}

impl StartPacket {
    pub fn new(options: &str) -> StartPacket {
        StartPacket {
            r#type: "START-GAME".to_string(),
            options: options.to_string(),
        }
    }

    pub fn try_parse(data: &str) -> Result<StartPacket> {
        serde_json::from_str(data)
    }

    pub fn to_json(data: StartPacket) -> String {
        serde_json::to_string(&data).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PublicGamePacket {
    pub r#type: String,
    pub id: Uuid,
    pub username: String,
    pub cards: usize,
    pub current: Card,
}

impl PublicGamePacket {
    pub fn new(id: Uuid, username: &str, cards: usize, current: Card) -> PublicGamePacket {
        PublicGamePacket {
            r#type: String::from("STATUS-UPDATE-PUBLIC"),
            id,
            username: String::from(username),
            cards,
            current,
        }
    }

    pub fn to_json(data: PublicGamePacket) -> String {
        serde_json::to_string(&data).unwrap()
    }

    pub fn try_parse(data: &str) -> Result<PublicGamePacket> {
        serde_json::from_str(data)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PrivateGamePacket {
    pub r#type: String,
    pub cards: Vec<Card>,
    pub current: Card,
}

impl PrivateGamePacket {
    pub fn new(cards: Vec<Card>, current: Card) -> PrivateGamePacket {
        PrivateGamePacket {
            r#type: String::from("STATUS-UPDATE-PRIVATE"),
            cards,
            current,
        }
    }

    pub fn to_json(data: PrivateGamePacket) -> String {
        serde_json::to_string(&data).unwrap()
    }

    pub fn try_parse(data: &str) -> Result<PrivateGamePacket> {
        serde_json::from_str(data)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AllowedCardsPacket {
    pub r#type: String,
    pub cards: Vec<Card>,
}

impl AllowedCardsPacket {
    pub fn new(cards: Vec<Card>) -> AllowedCardsPacket {
        AllowedCardsPacket {
            r#type: String::from("ALLOWED-CARDS-UPDATE"),
            cards,
        }
    }

    pub fn to_json(data: AllowedCardsPacket) -> String {
        serde_json::to_string(&data).unwrap()
    }

    pub fn try_parse(data: &str) -> Result<AllowedCardsPacket> {
        serde_json::from_str(data)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DrawPacket {
    pub r#type: String,
    pub amount: u8,
}

impl DrawPacket {
    pub fn new(amount: u8) -> DrawPacket {
        DrawPacket {
            r#type: "DRAW-CARDS".to_string(),
            amount,
        }
    }

    pub fn to_json(data: DrawPacket) -> String {
        serde_json::to_string(&data).unwrap()
    }

    pub fn try_parse(data: &str) -> Result<DrawPacket> {
        serde_json::from_str(data)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlaceCardPacket {
    pub r#type: String,
    pub index: usize,
}

impl PlaceCardPacket {
    pub fn new(index: usize) -> PlaceCardPacket {
        PlaceCardPacket {
            r#type: "PLACE-CARD".to_string(),
            index: index,
        }
    }

    pub fn try_parse(data: &str) -> Result<PlaceCardPacket> {
        serde_json::from_str(data)
    }

    pub fn to_json(data: PlaceCardPacket) -> String {
        serde_json::to_string(&data).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EndTurnPacket {
    pub r#type: String,
}

impl EndTurnPacket {
    pub fn new() -> EndTurnPacket {
        EndTurnPacket {
            r#type: "END-TURN".to_string(),
        }
    }

    pub fn to_json(data: EndTurnPacket) -> String {
        serde_json::to_string(&data).unwrap()
    }

    pub fn try_parse(data: &str) -> Result<EndTurnPacket> {
        serde_json::from_str(data)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ColorSwitchPacket {
    pub r#type: String,
    pub color: String,
}

impl ColorSwitchPacket {
    pub fn new(color: String) -> ColorSwitchPacket {
        ColorSwitchPacket {
            r#type: String::from("COLOR-SWITCH"),
            color,
        }
    }
    pub fn try_parse(data: &str) -> Result<ColorSwitchPacket> {
        serde_json::from_str(data)
    }

    pub fn to_json(data: ColorSwitchPacket) -> String {
        serde_json::to_string(&data).unwrap()
    }
}
