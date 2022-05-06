mod game;
mod packets;

use anyhow::Error;
use game::{Card, Player};
use packets::*;
use serde_json::Value;
use std::collections::HashMap;

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
    selecting: bool,
    hovering: bool,
    username: Option<String>,
    room_id: Option<String>,
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
    Register,
    StartGame,
    Received(Result<String, Error>),
    PlaceCard(Card),
    DrawCard,
    EndTurn,
    SwitchColor(String),
    HoverCard(bool),
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
            selecting: false,
            hovering: false,
            username: None,
            room_id: Some("c05554ae-b4ee-4976-ac05-97aaf3c98a24".to_string()),
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
            Msg::UsernameInput(e) => {
                self.username = Some(e);
                true
            }
            Msg::RoomIDInput(e) => {
                self.room_id = Some(e);
                true
            }
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
                                        .push_str(&format!("[MESSAGE] {:#?}", packet.content));
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
                                        .push_str(&format!("[CARDS] {:#?}", &packet.cards));

                                    self.current = Some(packet.current);
                                }
                            }
                            "\"STATUS-UPDATE-PUBLIC\"" => {
                                let p = PublicGamePacket::try_parse(&s);
                                let count = self.connections.len();

                                if let Ok(packet) = p {
                                    self.server_data
                                        .push_str(&format!("[CURRENT] {:#?}", packet.current));
                                    // Insert connection to the connection list
                                    if let std::collections::hash_map::Entry::Vacant(e) =
                                        self.connections.entry(packet.id)
                                    {
                                        e.insert(Player::new(packet.username, packet.cards, count));
                                    }

                                    self.connections.get_mut(&packet.id).unwrap().card_count =
                                        packet.cards;

                                    self.current = Some(packet.current);
                                }
                            }
                            "\"ALLOWED-CARDS-UPDATE\"" => {
                                let p = AllowedCardsPacket::try_parse(&s);
                                if let Ok(packet) = p {
                                    self.server_data
                                        .push_str(&format!("[ALLOWED] {:#?}", packet.cards));
                                    // Insert connection to the connection list
                                    self.allowed_cards = packet.cards;
                                    self.turn = true;
                                }
                            }
                            "\"End-TURN\"" => {
                                let p = EndTurnPacket::try_parse(&s);
                                if let Ok(packet) = p {
                                    ConsoleService::log("[MESSAGE] Your turn has ended.");
                                    self.turn = false;
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
                self.server_data.push_str(&format!("[ERROR] {:#?}", &s));
                true
            }
            Msg::PlaceCard(card) => {
                let index = self.cards.iter().position(|c| c == &card).unwrap();

                let p = PlaceCardPacket::new(index);

                if let Some(ref mut task) = self.ws {
                    task.send::<Text>(Text::into(Ok(PlaceCardPacket::to_json(p))));
                }

                if card.r#type == "Switch" || card.r#type == "DrawFour" {
                    self.selecting = true;
                }

                true
            }
            Msg::DrawCard => {
                ConsoleService::log("drawing a card");
                let p = DrawPacket::new(1);

                if let Some(ref mut task) = self.ws {
                    task.send::<Text>(Text::into(Ok(DrawPacket::to_json(p))));
                }

                true
            }
            Msg::EndTurn => {
                let p = EndTurnPacket::new();
                ConsoleService::log("Ending turn..");
                if let Some(ref mut task) = self.ws {
                    task.send::<Text>(Text::into(Ok(EndTurnPacket::to_json(p))));
                    self.allowed_cards.clear();
                }

                true
            }
            Msg::SwitchColor(color) => {
                let p = ColorSwitchPacket::new(color);

                if let Some(ref mut task) = self.ws {
                    task.send::<Text>(Text::into(Ok(ColorSwitchPacket::to_json(p))));
                    self.selecting = false;
                }

                true
            }
            Msg::HoverCard(active) => {
                self.hovering = active;
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
                    <input type="text" value=self.room_id.clone() oninput=self.link.callback(|e: InputData| Msg::RoomIDInput(e.value))/><br/>

                    // connect button
                    <button disabled={self.room_id.is_none() ||self.room_id == Some("".to_string())} onclick=self.link.callback(|_| Msg::Connect)>{ "Connect" }</button>

                    <h1 hidden={self.ws.is_none()}>{"Enter your username"}</h1>

                    // username
                    <input hidden={self.ws.is_none()} type="text" value=self.username.clone() oninput=self.link.callback(|e: InputData| Msg::UsernameInput(e.value))/>

                    <button hidden={self.ws.is_none()} disabled={self.username.is_none() || self.username == Some("".to_string())} onclick=self.link.callback(|_| Msg::Register)>{ "Register" }</button>

                    // text showing whether we're connected or not
                    <p class="connection-status">{ "Connected: "}{ self.ws.is_some() } </p>

                </div>

                <div class="waiting-screen" style={format!("display: {}", if self.connected && !self.active {"flex"} else {"none"})} >
                    <h1>{"Waiting for game to start"}</h1>

                    <p hidden={!self.host}>{"You are the host"}</p>
                    <button hidden={!self.host} onclick=self.link.callback(|_| Msg::StartGame)>{ "Start game" }</button>
                </div>

                <div class="cards-container"   style={format!("display: {}", if self.active {"flex"} else {"none"})} >
                    // <button onmouseover=self.link.callback(|_| Msg::HoverCard(true))  onmouseout=self.link.callback(|_| Msg::HoverCard(false)) class="card" id="allowed" style="background-image: url(http://localhost/uno-api/cards/Blue.Block.png);"></button>
                    // <button class="card" id="disallowed" style="background-image: url(static/img/Blue.DrawFour.svg);" disabled={true}></button>
                    { for self.cards.iter().to_owned().map(|card| {
                        let c = card.clone();
                        html! {
                            <button
                                class="card"
                                onclick=self.link.callback(move |_|  Msg::PlaceCard(c.clone()) )
                                onmouseover=self.link.callback(|_| Msg::HoverCard(true))  onmouseout=self.link.callback(|_| Msg::HoverCard(false))
                                style=format!("background-image: url(static/img/{}.{}.svg);", card.color, card.r#type)
                                id={(if self.allowed_cards.contains(card) {"allowed"} else {"disallowed"}).to_string()}
                                disabled={!self.allowed_cards.contains(card)} >
                            </button>
                            }
                        })
                    }
                    </div>
                    <h2 hidden={!self.connected} id="status-text"> { if self.turn && !self.selecting{"Your turn.".to_string()} else {"Waiting for the opponent".to_string()} }</h2>
                    <h1 style={ if self.hovering {"opacity: 100%;"} else {"opacity: 0;"}} id="place-card-text">{"Place a card."}</h1>
                // End turn button
                <button onclick=self.link.callback(|_| Msg::EndTurn) class="end-turn-button" style={format!("display: {}", if self.active {"flex"} else {"none"})} ><h1>{"End your turn"}</h1></button>

                <div class="deck-container" style={format!("display: {}", if self.active {"flex"} else {"none"})}  >
                    <button class="card" id="deck" onclick=self.link.callback(|_| Msg::DrawCard)><div class="logo"></div></button>
                    <button class="card" id="deck"><div class="logo"></div></button>
                    <button class="card" id="deck"><div class="logo"></div></button>
                    <button class="card" id="deck"><div class="logo"></div></button>
                    <div
                        class="card" id="placed-deck" style={
                            if self.current.is_some(){
                                format!("background-image: url(static/img/{}.{}.svg);",
                                self.current.clone().unwrap().color, self.current.clone().unwrap().r#type)
                            } else {
                                "static/img/uno.svg);".to_string()
                            }
                        } >
                    </div>
                    <h1 class="draw-card-text">{"Draw a card."}</h1>
                </div>

                { for self.connections.iter().map(|(_, v)| html! {
                    <div class="player-container" id={format!("player-{}", v.index + 1)}>
                        <div class="card-desing" ><div class="logo"></div></div>
                        <div class="card-desing" ><div class="logo"></div></div>
                        <div class="card-desing" ><div class="logo"></div></div>
                        <div class="card-desing" ><div class="logo"></div></div>
                        <div class="card-desing" ><div class="logo"></div></div>
                        <h1>{format!("{} : {}", &v.username, &v.card_count)}</h1>
                    </div>
                }) }

                <div class="color-selector" style={format!("display: {}", if self.selecting {"flex"} else {"none"})}>
                    <h1>{"Select the color you want to switch to"}</h1>
                    <button onclick=self.link.callback(|_| Msg::SwitchColor("Yellow".to_string())) class="card" id="color" style="background-image: url(static/img/Selector.Yellow.svg"></button>
                    <button onclick=self.link.callback(|_| Msg::SwitchColor("Red".to_string())) class="card" id="color" style="background-image: url(static/img/Selector.Red.svg"></button>
                    <button onclick=self.link.callback(|_| Msg::SwitchColor("Blue".to_string())) class="card" id="color" style="background-image: url(static/img/Selector.Blue.svg"></button>
                    <button onclick=self.link.callback(|_| Msg::SwitchColor("Green".to_string())) class="card" id="color" style="background-image: url(static/img/Selector.Green.svg"></button>
                </div>


                <div class="side-bar" >
                    // text area for showing data from the server
                    <textarea>{self.server_data.to_string()}</textarea>
                </div>

            </div>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
