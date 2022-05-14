use crate::game::Card;
use serde::{Deserialize, Serialize};
use serde_json::Result;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
pub enum PacketType {
    Register(String),                              // username
    GameData(Uuid, String, Vec<(Uuid, String)>),   // self_id, self_username, Vec<(id, username)>
    Connect(Uuid, String),                         // id, username
    Disconnect(Uuid, String),                      // id, username
    Message(String),                               // content
    StartGame(String),                             // option
    StatusUpdatePublic(Uuid, String, usize, Card), // id, username, card-count, current
    StatusUpdatePrivate(Vec<Card>, Card),          // cards, current
    AllowedCardsUpdate(Vec<Card>),                 // allowed-cards
    DrawCard(u8),                                  // amount
    PlaceCard(usize),                              // index
    EndTurn,                                       //
    ColorSwitch(String),                           // color
    TurnUpdate(Uuid, Uuid),                        // current, next
    Error(u64, String),                            // error-code, body
}

/*
#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterPacket {
    pub r#type: PacketType,
    pub username: String,
}

impl RegisterPacket {
    pub fn new(username: &str) -> RegisterPacket {
        RegisterPacket {
            r#type: PacketType::Register,
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
*/

/*
#[derive(Serialize, Deserialize, Debug)]
pub struct GameDataPacket {
    pub r#type: PacketType,
    pub self_id: Uuid,
    pub self_username: String,
    pub connections: Vec<(Uuid, String)>,
}

impl GameDataPacket {
    pub fn new(
        self_id: Uuid,
        self_username: &str,
        connections: Vec<(Uuid, String)>,
    ) -> GameDataPacket {
        GameDataPacket {
            r#type: PacketType::GameData,
            self_id,
            self_username: String::from(self_username),
            connections,
        }
    }

    pub fn try_parse(data: &str) -> Result<GameDataPacket> {
        serde_json::from_str(data)
    }

    pub fn to_json(data: GameDataPacket) -> String {
        serde_json::to_string(&data).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConnectPacket {
    pub r#type: PacketType,
    pub id: Uuid,
    pub username: String,
}

impl ConnectPacket {
    pub fn new(id: Uuid, username: &str) -> ConnectPacket {
        ConnectPacket {
            r#type: PacketType::Connect,
            id,
            username: String::from(username),
        }
    }
    pub fn try_parse(data: &str) -> Result<ConnectPacket> {
        serde_json::from_str(data)
    }

    pub fn to_json(data: ConnectPacket) -> String {
        serde_json::to_string(&data).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DisconnectPacket {
    pub r#type: PacketType,
    pub id: Uuid,
    pub username: String,
}

impl DisconnectPacket {
    pub fn new(id: Uuid, username: &str) -> DisconnectPacket {
        DisconnectPacket {
            r#type: PacketType::Disconnect,
            id,
            username: String::from(username),
        }
    }
    pub fn try_parse(data: &str) -> Result<DisconnectPacket> {
        serde_json::from_str(data)
    }

    pub fn to_json(data: DisconnectPacket) -> String {
        serde_json::to_string(&data).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MessagePacket {
    pub r#type: PacketType,
    pub content: String,
}

impl MessagePacket {
    pub fn new(content: &str) -> MessagePacket {
        MessagePacket {
            r#type: PacketType::Message,
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
    pub r#type: PacketType,
    pub options: String,
}

impl StartPacket {
    pub fn new(options: &str) -> StartPacket {
        StartPacket {
            r#type: PacketType::StartGame,
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
    pub r#type: PacketType,
    pub id: Uuid,
    pub username: String,
    pub cards: usize,
    pub current: Card,
}

impl PublicGamePacket {
    pub fn new(id: Uuid, username: &str, cards: usize, current: Card) -> PublicGamePacket {
        PublicGamePacket {
            r#type: PacketType::StatusUpdatePublic,
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
    pub r#type: PacketType,
    pub cards: Vec<Card>,
    pub current: Card,
}

impl PrivateGamePacket {
    pub fn new(cards: Vec<Card>, current: Card) -> PrivateGamePacket {
        PrivateGamePacket {
            r#type: PacketType::StatusUpdatePrivate,
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
    pub r#type: PacketType,
    pub cards: Vec<Card>,
}

impl AllowedCardsPacket {
    pub fn new(cards: Vec<Card>) -> AllowedCardsPacket {
        AllowedCardsPacket {
            r#type: PacketType::AllowedCardsUpdate,
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
    pub r#type: PacketType,
    pub amount: u8,
}

impl DrawPacket {
    pub fn new(amount: u8) -> DrawPacket {
        DrawPacket {
            r#type: PacketType::DrawCard,
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
    pub r#type: PacketType,
    pub index: usize,
}

impl PlaceCardPacket {
    pub fn new(index: usize) -> PlaceCardPacket {
        PlaceCardPacket {
            r#type: PacketType::PlaceCard,
            index,
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
    pub r#type: PacketType,
}

impl EndTurnPacket {
    pub fn new() -> EndTurnPacket {
        EndTurnPacket {
            r#type: PacketType::EndTurn,
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
    pub r#type: PacketType,
    pub color: String,
}

impl ColorSwitchPacket {
    pub fn new(color: String) -> ColorSwitchPacket {
        ColorSwitchPacket {
            r#type: PacketType::ColorSwitch,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct TurnUpdatePacket {
    pub r#type: PacketType,
    pub id: Uuid,
    pub next: Uuid,
}

impl TurnUpdatePacket {
    pub fn new(id: Uuid, next: Uuid) -> TurnUpdatePacket {
        TurnUpdatePacket {
            r#type: PacketType::TurnUpdate,
            id,
            next,
        }
    }
    pub fn try_parse(data: &str) -> Result<TurnUpdatePacket> {
        serde_json::from_str(data)
    }

    pub fn to_json(data: TurnUpdatePacket) -> String {
        serde_json::to_string(&data).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HTMLError {
    pub r#type: PacketType,
    pub status_code: u64,
    pub body: String,
}

impl HTMLError {
    pub fn to_json(err: HTMLError) -> String {
        serde_json::to_string(&err).unwrap()
    }

    pub fn try_parse(data: &str) -> Result<HTMLError> {
        serde_json::from_str(data)
    }

    pub fn new(status_code: u64, body: &str) -> HTMLError {
        HTMLError {
            r#type: PacketType::Error,
            status_code,
            body: String::from(body),
        }
    }
}
*/
