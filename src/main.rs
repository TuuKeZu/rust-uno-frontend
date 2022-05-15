mod game;
mod packets;

use anyhow::Error;
use game::{Card, EndStatus, GameStatistics, Player};
use packets::*;
use std::collections::HashMap;
use std::time::SystemTime;
use time::OffsetDateTime;

use uuid::Uuid;
use yew::format::Text;
use yew::prelude::*;
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};
use yew::services::ConsoleService;

struct Model {
    ws: Option<WebSocketTask>,
    link: ComponentLink<Self>,

    connected: bool,
    registered: bool,
    host: bool,
    active: bool,
    ended: bool,
    turn: bool,
    next: bool,
    selecting: bool,
    hovering: bool,

    username: Option<String>,
    room_id: Option<String>,
    server_data: String,
    chat: Vec<ServerMessage>,
    connections: HashMap<Uuid, Player>,
    connection_count: usize,
    cards: Vec<Card>,
    allowed_cards: Vec<Card>,
    current: Option<Card>,

    end_status: Option<EndStatus>,
}
enum Msg {
    Connect,
    Disconnected,
    Connected,
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
    Error(String),
}

enum ServerMessage {
    Join(String),
    Leave(String),
    Message(String, String),
    Error(String),
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            ws: None,
            link,
            connected: false,
            registered: false,
            host: false,
            active: false,
            ended: false,
            turn: false,
            next: false,
            selecting: false,
            hovering: false,
            username: None,
            room_id: Some("c05554ae-b4ee-4976-ac05-97aaf3c98a24".to_string()),
            server_data: String::new(),
            chat: Vec::new(),
            connections: HashMap::new(),
            connection_count: 1,
            cards: Vec::new(),
            allowed_cards: Vec::new(),
            current: None,
            end_status: None,
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
                        WebSocketStatus::Closed => Msg::Disconnected,
                        WebSocketStatus::Error => {
                            Msg::Error("Failed to connect to servers".to_string())
                        }
                        _ => Msg::Connected,
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
                self.connected = false;
                true
            }
            Msg::Connected => {
                self.connected = true;
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
                    task.send::<Text>(Text::into(Ok(to_json(PacketType::Register(
                        self.username.clone().unwrap_or("player".to_string()),
                    )))));
                    true
                }
                None => false,
            },
            Msg::StartGame => match self.ws {
                Some(ref mut task) => {
                    task.send::<Text>(Text::into(Ok(to_json(PacketType::StartGame(
                        "None".to_string(),
                    )))));
                    true
                }
                None => false,
            },
            Msg::Received(Ok(s)) => {
                let json: Result<PacketType, serde_json::Error> = serde_json::from_str(&s);

                if let Ok(packet) = json {
                    match packet {
                        PacketType::Register(_) => {}
                        PacketType::GameData(self_id, _self_username, connections) => {
                            self.registered = true;

                            connections.iter().for_each(|(id, username)| {
                                if id != &self_id {
                                    // Insert connection to the connection list
                                    if let std::collections::hash_map::Entry::Vacant(e) =
                                        self.connections.entry(*id)
                                    {
                                        e.insert(Player::new(
                                            username.clone(),
                                            0,
                                            self.connection_count,
                                        ));
                                    }
                                }
                            });
                        }
                        PacketType::Connect(id, username) => {
                            self.connection_count += 1;

                            self.chat.push(ServerMessage::Join(username.clone()));

                            // Insert connection to the connection list
                            if let std::collections::hash_map::Entry::Vacant(e) =
                                self.connections.entry(id)
                            {
                                e.insert(Player::new(username, 0, self.connection_count));
                            }
                        }
                        PacketType::Disconnect(id, username) => {
                            self.connection_count -= 1;

                            self.chat.push(ServerMessage::Leave(username));

                            self.connections.remove(&id);
                        }
                        PacketType::Message(content) => {
                            if content == "You are the host" {
                                self.host = true;
                            }

                            self.chat
                                .push(ServerMessage::Message("Unknown".to_string(), content));
                        }
                        PacketType::StartGame(_) => {} // will never be received by client
                        PacketType::StatusUpdatePublic(id, _username, card_count, current) => {
                            self.server_data
                                .push_str(&format!("[CURRENT] {:#?}", current));

                            self.connections.get_mut(&id).unwrap().card_count = card_count;
                            self.active = true;
                            self.current = Some(current);
                        }
                        PacketType::StatusUpdatePrivate(cards, current) => {
                            self.server_data.push_str(&format!("[CARDS] {:#?}", &cards));

                            self.cards = cards;
                            self.current = Some(current);
                        }
                        PacketType::AllowedCardsUpdate(cards) => {
                            self.server_data
                                .push_str(&format!("[ALLOWED] {:#?}", cards));

                            self.allowed_cards = cards;
                            self.turn = true;
                        }
                        PacketType::DrawCard(_) => {} // will never be received by client
                        PacketType::PlaceCard(_) => {} // will never be received by client
                        PacketType::EndTurn => {
                            ConsoleService::log("[MESSAGE] Your turn has ended.");
                            self.allowed_cards.clear();
                            self.turn = false;
                        }
                        PacketType::ColorSwitch(_) => todo!(),
                        PacketType::TurnUpdate(id, next) => {
                            self.connections.iter_mut().for_each(|p| {
                                p.1.turn = &id == p.0;
                                p.1.next = &next == p.0;
                            });

                            if !self.connections.contains_key(&next) {
                                self.next = true;
                            } else {
                                self.next = false;
                            }
                        }
                        PacketType::Error(_code, body) => {
                            self.chat.push(ServerMessage::Error(body));
                        }
                        PacketType::WinUpdate(id, username, placements, statistics) => {
                            self.end_status = Some(EndStatus {
                                winner_id: id,
                                winner: username,
                                placements,
                                statistics,
                            });
                            self.ended = true;
                        }
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

                if let Some(ref mut task) = self.ws {
                    task.send::<Text>(Text::into(Ok(to_json(PacketType::PlaceCard(index)))));
                }

                if card.r#type == "Switch" || card.r#type == "DrawFour" {
                    self.selecting = true;
                }

                true
            }
            Msg::DrawCard => {
                ConsoleService::log("drawing a card");

                if let Some(ref mut task) = self.ws {
                    task.send::<Text>(Text::into(Ok(to_json(PacketType::DrawCard(1)))));
                }

                true
            }
            Msg::EndTurn => {
                ConsoleService::log("Ending turn..");
                if let Some(ref mut task) = self.ws {
                    task.send::<Text>(Text::into(Ok(to_json(PacketType::EndTurn))));
                }

                true
            }
            Msg::SwitchColor(color) => {
                if let Some(ref mut task) = self.ws {
                    task.send::<Text>(Text::into(Ok(to_json(PacketType::ColorSwitch(color)))));
                    self.selecting = false;
                }

                true
            }
            Msg::HoverCard(active) => {
                self.hovering = active;
                true
            }
            Msg::Error(e) => {
                self.chat.push(ServerMessage::Error(e.clone()));
                ConsoleService::log(&e);
                false
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
                <div class="connect-screen" style={format!("display: {}", if !self.registered {"flex"} else {"none"})}>
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

                <div class="waiting-screen" style={format!("display: {}", if self.registered && !self.active {"flex"} else {"none"})} >
                    <h1>{"Waiting for game to start"}</h1>

                    <p hidden={!self.host}>{"You are the host"}</p>
                    <button hidden={!self.host} disabled={self.connection_count <= 1} onclick=self.link.callback(|_| Msg::StartGame)>{ "Start game" }</button>
                </div>

                <div class="cards-container" style={format!("display: {}", if self.active {"flex"} else {"none"})} >
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
                    <h2 hidden={!self.active} id="status-text"> { if self.turn && !self.selecting{"Your turn.".to_string()} else {"Waiting for the opponent".to_string()} }</h2>
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

                <div class="player-list">
                    <div class="player-object" id="player-self" style={"order: -1;"}>
                        <div class="player-detail"></div>
                        <h2>{self.cards.len()}</h2>
                        <h1 style={if self.turn {"color: var(--green)"} else {"color: white"}}>
                        {format!("{} [You]", self.username.clone().unwrap_or_else(|| "unset".to_string()))}
                        </h1>
                        {if self.next {html! {<h3>{"[Next]"}</h3>}} else if self.turn {html!{<h4>{"[Turn]"}</h4>}} else {html!{<h3></h3>}}}
                    </div>

                    {
                        for self.connections.iter().map(|(_id, player)| {

                            html! {
                                <div class="player-object" id="player-self" style={format!("order: {};", player.index)}>
                                    <div class="player-detail"></div>
                                    <h2>{player.card_count}</h2>
                                    <h1
                                    style={if player.turn {"color: var(--green)"} else {"color: white"}}
                                    >
                                    {&player.username}
                                    </h1>
                                    {if player.next {html! {<h3>{"[Next]"}</h3>}} else if player.turn {html!{<h4>{"[Turn]"}</h4>}} else {html!{<h3></h3>}}}
                                </div>
                            }
                        })
                    }
                </div>

                <div class="color-selector" style={format!("display: {}", if self.selecting {"flex"} else {"none"})}>
                    <h1>{"Select the color you want to switch to"}</h1>
                    <button onclick=self.link.callback(|_| Msg::SwitchColor("Yellow".to_string())) class="card" id="color" style="background-image: url(static/img/Selector.Yellow.svg"></button>
                    <button onclick=self.link.callback(|_| Msg::SwitchColor("Red".to_string())) class="card" id="color" style="background-image: url(static/img/Selector.Red.svg"></button>
                    <button onclick=self.link.callback(|_| Msg::SwitchColor("Blue".to_string())) class="card" id="color" style="background-image: url(static/img/Selector.Blue.svg"></button>
                    <button onclick=self.link.callback(|_| Msg::SwitchColor("Green".to_string())) class="card" id="color" style="background-image: url(static/img/Selector.Green.svg"></button>
                </div>

                {
                    if self.end_status.is_some() {
                        let status = self.end_status.as_ref().unwrap();

                        // Get the game duration as seconds and format it to {min:s} format
                        let duration = (status.statistics.end_time.unwrap().duration_since(status.statistics.start_time.unwrap())).unwrap().as_secs() as f64;
                        let (minutes, seconds) = ((duration / 60.0).floor(), (((duration / 60.0) - (duration / 60.0).floor()) * 60.0).round());

                        html! {
                            <div class="win-screen" style={format!("display: {}", if self.ended {"flex"} else {"none"})} >
                                <h2>{"Game Ended"}</h2>

                                <ul><a>{"Game lasted "}</a><a class="highlight">{format!("{}min {}s", minutes, seconds)}</a></ul>
                                <ul><a class="highlight">{status.statistics.player_count}</a><a>{" players took part"}</a></ul>
                                <ul><a class="highlight">{status.statistics.spectator_count}</a><a>{" spectators took part"}</a></ul>
                                <ul><a class="highlight">{status.statistics.cards_drawn}</a><a>{" cards were drawn"}</a></ul>
                                <ul><a class="highlight">{status.statistics.cards_placed}</a><a>{" cards were placed"}</a></ul>

                                <h1>{format!("{} won", status.winner)}</h1>
                                {
                                    for status.placements.iter().map(|username| {
                                        let index = status.placements.iter().position(|u| u == username).unwrap() + 2;
                                        html!{
                                            <h3>{format!("{}. {}", index, username)}</h3>
                                        }
                                    })
                                }
                            </div>
                        }
                    }
                    else {
                        html! {}
                    }
                }

                <div class="side-bar" >
                    // text area for showing data from the server
                    <textarea>{self.server_data.to_string()}</textarea>
                </div>

                <div class="chat">
                {
                    for self.chat.iter().map(|message| {

                        match message {
                            ServerMessage::Join(username) => {
                                html! {
                                    <div class="chat-object">
                                        <h3 id="connection">{"[Connected]"}</h3>
                                        <h2>{username}</h2>
                                    </div>
                                }
                            },
                            ServerMessage::Leave(username) => {
                                html! {
                                    <div class="chat-object">
                                    <h3 id="connection">{"[Disconnected]"}</h3>
                                    <h2>{username}</h2>
                                </div>
                                }
                            },
                            ServerMessage::Message(username, content) => {
                                html! {
                                    <div class="chat-object">
                                        <h3 id="message">{"[Message]"}</h3>
                                        <h2>{username}</h2>
                                        <h1>{content}</h1>
                                    </div>
                                }
                            },
                            ServerMessage::Error(body) => {
                                html! {
                                    <div class="chat-object">
                                        <h3 id="error">{"[Error]"}</h3>
                                        <h2>{"Server"}</h2>
                                        <h1>{body}</h1>
                                    </div>
                                }
                            },
                        }
                    })
                }
                </div>

            </div>

        }
    }
}

pub fn to_json(data: PacketType) -> String {
    serde_json::to_string(&data).unwrap()
}

fn main() {
    yew::start_app::<Model>();
}
