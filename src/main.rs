mod game;
mod packets;
use anyhow::Error;
use game::Card;
use packets::*;
use serde_json::Value;

use yew::format::Text;
use yew::prelude::*;
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};
use yew::services::ConsoleService;

struct Model {
    ws: Option<WebSocketTask>,
    link: ComponentLink<Self>,
    connected: bool,
    host: bool,
    username: Option<String>,
    room_id: Option<String>,
    text: String,
    server_data: String,
    cards: Vec<String>,
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
            username: None,
            room_id: Some("c05554ae-b4ee-4976-ac05-97aaf3c98a24".to_string()),
            text: String::new(),
            server_data: String::new(),
            cards: Vec::new(),
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
                    self.connected = true;
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

                                if let Ok(packet) = p {
                                    packet.cards.iter().for_each(|card| {
                                        self.cards.push(format!("{}:{}", card.color, card.r#type))
                                    });

                                    self.server_data
                                        .push_str(&format!("[CARDS]: {:#?}\n", packet.cards));
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
                <div class="connect-screen" hidden={self.connected}>

                    // room id
                    <p><input type="text" value=self.room_id.clone() oninput=self.link.callback(|e: InputData| Msg::RoomIDInput(e.value))/></p><br/>

                    // connect button
                    <p><button disabled={self.room_id.is_none() ||self.room_id == Some("".to_string())} onclick=self.link.callback(|_| Msg::Connect)>{ "Connect" }</button></p><br/>

                    // username
                    <p><input type="text" value=self.username.clone() oninput=self.link.callback(|e: InputData| Msg::UsernameInput(e.value))/></p><br/>

                    <p><button disabled={self.username.is_none() || self.username == Some("".to_string())} onclick=self.link.callback(|_| Msg::Register)>{ "Register" }</button></p><br/>

                    // text showing whether we're connected or not
                    <p>{ "Connected: "}{ self.ws.is_some() } </p><br/>

                </div>

                <div class="host-screen" hidden={!self.host}>
                    <button onclick=self.link.callback(|_| Msg::StartGame)>{ "Start game" }</button>
                </div>

                <div class="cards-container">
                    { for self.cards.iter().map(|card| html! {<p style=format!("background-image: url(static/img/test.png); width: 100px; height: 100px;")></p>}) }
                </div>

                // input box for sending text
                <p><input type="text" value=self.text.clone() oninput=self.link.callback(|e: InputData| Msg::TextInput(e.value))/></p><br/>
                // button for sending text
                <p><button onclick=self.link.callback(|_| Msg::SendText)>{ "Send" }</button></p><br/>
                // text area for showing data from the server
                <p><textarea value=self.server_data.clone()></textarea></p><br/>
            </div>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
