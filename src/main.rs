mod game;
mod packets;
use std::collections::HashMap;

use anyhow::Error;
use game::{Card, Player};
use packets::*;
use serde_json::Value;

use uuid::Uuid;
use yew::format::Text;
use yew::prelude::*;
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};
use yew::services::ConsoleService;

struct Model {
    ws: Option<WebSocketTask>,
    link: ComponentLink<Self>,
    connected: bool,
    host: bool,
    active: bool,
    turn: bool,
    username: Option<String>,
    room_id: Option<String>,
    text: String,
    server_data: String,
    connections: HashMap<Uuid, Player>,
    cards: Vec<Card>,
    allowed_cards: Vec<Card>,
    current: Option<Card>,
}

enum Msg {
    Connect,
    Disconnected,
    Ignore,
    UsernameInput(String),
    RoomIDInput(String),
    TextInput(String),
    SendText,
    Register,
    StartGame,
    Received(Result<String, Error>),
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            ws: None,
            link,
            connected: false,
            host: false,
            active: false,
            turn: false,
            username: None,
            room_id: Some("c05554ae-b4ee-4976-ac05-97aaf3c98a24".to_string()),
            text: String::new(),
            server_data: String::new(),
            connections: HashMap::new(),
            cards: Vec::new(),
            allowed_cards: Vec::new(),
            current: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Connect => {
                ConsoleService::log("Connecting");

                let cbout = self.link.callback(Msg::Received);
                let cbnot = self.link.callback(|input| {
                    ConsoleService::log(&format!("Notification: {:?}", input));
                    match input {
                        WebSocketStatus::Closed | WebSocketStatus::Error => Msg::Disconnected,
                        _ => Msg::Ignore,
                    }
                });
                if self.ws.is_none() {
                    let task = WebSocketService::connect_text(
                        "ws://127.0.0.1:8090/c05554ae-b4ee-4976-ac05-97aaf3c98a24",
                        cbout,
                        cbnot,
                    );
                    self.ws = Some(task.unwrap());
                }
                true
            }
            Msg::Disconnected => {
                self.ws = None;
                true
            }
            Msg::Ignore => false,
            Msg::TextInput(e) => {
                self.text = e;
                true
            }
            Msg::UsernameInput(e) => {
                self.username = Some(e);
                true
            }
            Msg::RoomIDInput(e) => {
                self.room_id = Some(e);
                true
            }
            Msg::SendText => match self.ws {
                Some(ref mut task) => {
                    task.send::<Text>(Text::into(Ok(String::from(&self.text))));
                    true
                }
                None => false,
            },
            Msg::Register => match self.ws {
                Some(ref mut task) => {
                    task.send::<Text>(Text::into(Ok(RegisterPacket::to_json(
                        RegisterPacket::new(self.username.as_ref().unwrap()),
                    ))));
                    self.connected = true;
                    true
                }
                None => false,
            },
            Msg::StartGame => match self.ws {
                Some(ref mut task) => {
                    task.send::<Text>(Text::into(Ok(StartPacket::to_json(StartPacket::new(
                        "None",
                    )))));
                    true
                }
                None => false,
            },
            Msg::Received(Ok(s)) => {
                let json: serde_json::Result<Value> = serde_json::from_str(&s);

                if let Ok(data) = json {
                    if let Some(r#type) = data.get("type") {
                        match &r#type.to_string() as &str {
                            // Message event
                            "\"MESSAGE\"" => {
                                let p = MessagePacket::try_parse(&s);

                                if let Ok(packet) = p {
                                    if packet.content == "You are the host" {
                                        self.host = true;
                                    }

                                    self.server_data
                                        .push_str(&format!("[MESSAGE]: {}\n", packet.content));
                                }
                            }
                            "\"STATUS-UPDATE-PRIVATE\"" => {
                                let p = PrivateGamePacket::try_parse(&s);

                                if !self.active {
                                    self.active = true;
                                }

                                if let Ok(packet) = p {
                                    self.cards = packet.cards.clone();

                                    self.server_data
                                        .push_str(&format!("[CARDS]: {:#?}\n", packet));
                                }
                            }
                            "\"STATUS-UPDATE-PUBLIC\"" => {
                                ConsoleService::log(&format!("[PUBLIC] {:#?}", &s));
                                let p = PublicGamePacket::try_parse(&s);
                                let count = self.connections.len();
                                if let Ok(packet) = p {
                                    // Insert connection to the connection list
                                    if let std::collections::hash_map::Entry::Vacant(e) =
                                        self.connections.entry(packet.id)
                                    {
                                        e.insert(Player::new(packet.username, packet.cards, count));
                                    }

                                    self.current = Some(packet.current);
                                }
                            }
                            "\"ALLOWED-CARDS-UPDATE\"" => {
                                ConsoleService::log(&format!("[ALLOWED CARDS] {:#?}", &s));

                                let p = AllowedCardsPacket::try_parse(&s);
                                if let Ok(packet) = p {
                                    // Insert connection to the connection list
                                    self.allowed_cards = packet.cards;
                                    self.turn = true;
                                }
                            }
                            "\"ERROR\"" => {}
                            _ => ConsoleService::log("Unknown packet received"),
                        }

                        // ConsoleService::log(&format!("{:?}", s));
                        // self.server_data.push_str(&format!("{}\n", &s));
                    }
                }

                true
            }
            Msg::Received(Err(s)) => {
                self.server_data.push_str(&format!(
                    "Error when reading from server: {}\n",
                    &s.to_string()
                ));
                true
            }
        }
    }

    fn change(&mut self, _prop: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            // <div></div>
            <div class="container">
                // login screen element
                <div class="connect-screen" style={format!("display: {}", if !self.connected {"flex"} else {"none"})}>
                    <h1>{"Enter Room ID"}</h1>
                    // room id
                    <input type="text" value=self.room_id.clone() oninput=self.link.callback(|e: InputData| Msg::RoomIDInput(e.value))/>    <br/>

                    // connect button
                    <button disabled={self.room_id.is_none() ||self.room_id == Some("".to_string())} onclick=self.link.callback(|_| Msg::Connect)>{ "Connect" }</button>

                    <h1 hidden={self.ws.is_none()}>{"Enter your username"}</h1>
                    // username
                    <input hidden={self.ws.is_none()} type="text" value=self.username.clone() oninput=self.link.callback(|e: InputData| Msg::UsernameInput(e.value))/>

                    <button hidden={self.ws.is_none()} disabled={self.username.is_none() || self.username == Some("".to_string())} onclick=self.link.callback(|_| Msg::Register)>{ "Register" }</button>

                    // text showing whether we're connected or not
                    <p class="connection-status">{ "Connected: "}{ self.ws.is_some() } </p>

                </div>
                /*
                <div class="player-container" id="player-1">
                    <div class="card-desing" ></div>
                    <div class="card-desing" ></div>
                    <div class="card-desing" ></div>
                    <div class="card-desing" ></div>
                    <div class="card-desing" ></div>
                    <h1>{"test-1"}</h1>
                </div>
                */
                <div class="waiting-screen" style={format!("display: {}", if self.connected && !self.active {"flex"} else {"none"})} >
                    <h1>{"Waiting for game to start"}</h1>

                    <p hidden={!self.host}>{"You are the host"}</p>
                    <button hidden={!self.host} onclick=self.link.callback(|_| Msg::StartGame)>{ "Start game" }</button>
                </div>

                <div class="cards-container"  /*  style={format!("display: {}", if self.active {"flex"} else {"none"})} */>
                    // <div class="card" id="allowed" style="background-image: url(http://localhost/uno-api/cards/Blue.Block.png);"></div>
                    // <div class="card" id="disallowed" style="background-image: url(http://localhost/uno-api/cards/Red.Reverse.png);"></div>
                    { for self.cards.iter().map(|card| html! {<div class="card" style=format!("background-image: url(http://localhost/uno-api/cards/{}.{}.png);", card.color, card.r#type) id={format!("{}", if self.allowed_cards.contains(card) {"allowed"} else {"disallowed"})}></div>}) }
                    <h1 class="place-card-text">{"Place a card."}</h1>
                    <h2  class="status-text">{if self.turn {"Your turn.".to_string()} else {"Waiting for the opponent".to_string()}}</h2>
                </div>

                <div class="deck-container" style={format!("display: {}", if self.active {"flex"} else {"none"})} >
                    <div class="card" id="deck"></div>
                    <div class="card" id="deck"></div>
                    <div class="card" id="deck"></div>
                    <div class="card" id="deck"></div>
                    <div class="card" id="placed-deck" style={if self.current.is_some() { format!("background-image: url(http://localhost/uno-api/cards/{}.{}.png);", self.current.clone().unwrap().color, self.current.clone().unwrap().r#type)} else {"http://localhost/uno-api/cards/uno.png);".to_string()}}></div>
                    <h1 class="draw-card-text">{"Draw a card."}</h1>
                </div>


                { for self.connections.iter().map(|(_, v)| html! {
                    <div class="player-container" id={format!("player-{}", v.index + 1)}>
                        <div class="card-desing" ></div>
                        <div class="card-desing" ></div>
                        <div class="card-desing" ></div>
                        <div class="card-desing" ></div>
                        <div class="card-desing" ></div>
                        <h1>{format!("{} : {}", &v.username, &v.card_count)}</h1>
                    </div>
                }) }


                <div class="side-bar" >
                    // input box for sending text
                    <input type="text" value=self.text.clone() oninput=self.link.callback(|e: InputData| Msg::TextInput(e.value))/><br/>
                    // button for sending text
                    <p><button onclick=self.link.callback(|_| Msg::SendText)>{ "Send" }</button></p><br/>
                    // text area for showing data from the server
                    <p><textarea value=self.server_data.clone()></textarea></p><br/>

                </div>
            </div>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
