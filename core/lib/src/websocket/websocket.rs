//! WebSocket handling based on tungstenite crate.
use http::Header;
use rocket_http::hyper::OnUpgrade;

use crate::data::FromData;
use crate::data::Outcome;
use crate::response::{Responder, Response, Result};
use crate::{http, Data, Request};

use super::handler::Handler;
use super::AsyncReadWrite;
use super::HandlerFuture;

pub use tokio_tungstenite::{
    tungstenite::protocol::{self, WebSocketConfig},
    tungstenite::Error as WsError,
    WebSocketStream,
};

fn convert_key(key: &str) -> String {
    let mut sha1 = sha1::Sha1::default();
    sha1.update(key.as_bytes());
    sha1.update(&b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11"[..]);
    base64::encode(&sha1.digest().bytes())
}

pub struct Websocket {
    upgrade: OnUpgrade,
    pub(crate) handler: Option<Box<dyn Handler>>,
}

impl<'w> Websocket {
    pub(crate) async fn run(self, request: &Request<'_>) {
        let upgraded = self.upgrade.await.unwrap();
        let socket = WebSocketStream::from_raw_socket(
            Box::new(upgraded) as Box<dyn AsyncReadWrite>,
            protocol::Role::Server,
            None,
        )
        .await;
        match self.handler {
            Some(handler) => {
                handler.handle(request, socket).await;
            }
            None => {
                panic!("No handler registered");
            }
        }
    }

    pub fn handle<T: Clone + Send + Sync + 'static>(&mut self, handler: T) where T: for<'r> FnOnce(&'r Request<'_>, WebSocketStream<Box<dyn AsyncReadWrite>>) -> HandlerFuture<'r> {
        self.handler = Some(Box::new(handler));
    }
}

#[crate::async_trait]
impl FromData for Websocket {
    type Error = ();

    async fn from_data(_s: &Request<'_>, data: Data) -> Outcome<Self, ()> {
        let upgrade = data.into_hyper_body().on_upgrade();
        Outcome::Success(Self {
            upgrade,
            handler: None,
        })
    }
}

impl<'r> Responder<'r, 'static> for Websocket {
    fn respond_to(self, request: &'r Request<'_>) -> Result<'static> {
        let get_header = |name| request.headers().get_one(name).unwrap_or_default();
        let upgrade_present = get_header("upgrade").eq_ignore_ascii_case("websocket");
        let correct_version_match = get_header("sec-websocket-version") == "13";
        let sec_websocket_accept = get_header("sec-websocket-key");

        if !upgrade_present {
            return Err(http::Status::UpgradeRequired);
        } else if !correct_version_match || sec_websocket_accept.len() == 0 {
            return Err(http::Status::BadRequest);
        }

        Response::build()
            .header(Header::new("Connection", "Upgrade"))
            .header(Header::new("Upgrade", "websocket"))
            .header(Header::new(
                "sec-websocket-accept",
                convert_key(sec_websocket_accept),
            ))
            .set_upgrade(Some(self))
            .status(http::Status::SwitchingProtocols)
            .ok()
    }
}
