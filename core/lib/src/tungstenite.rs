//! WebSocket handling based on tungstenite crate.

use http::Header;

use crate::request::{FromRequest, Request};
use crate::response::{Response, Responder};
use crate::data::Outcome;
use crate::{http, Data};
use crate::data::FromData;
use tokio::sync::oneshot;
use crate::http::hyper;

pub use tokio_tungstenite::{
    tungstenite::protocol::{self, WebSocketConfig},
    tungstenite::Error as WsError,
    WebSocketStream,
};

fn convert_key(key: &str) -> String {
    use sha1::Digest;
    let mut sha1 = sha1::Sha1::default();
    sha1.input(key);
    sha1.input(&b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11"[..]);
    base64::encode(&sha1.result())
}

pub struct Websocket {
    data: crate::data::Data
    // handler: Box<dyn Handler>
}

impl  Websocket {
    pub async fn upgrade(self) -> Result<WebSocketStream<http::hyper::Upgraded>, http::hyper::Error> {
        let socket = self.data.into_hyper_body().on_upgrade().await?;
        Ok(WebSocketStream::from_raw_socket(socket, protocol::Role::Server, None).await)
    }

    pub async fn spawn_worker(self) -> Result<(), ()> {
        // let (tx, rx) = oneshot::channel();
        tokio::spawn(async move {
            dbg!("Working");
            let upgraded_socket_result = self.upgrade()
                .await;

            dbg!(&upgraded_socket_result);

            match upgraded_socket_result {
                Ok(socket_stream) => {
                    use crate::futures::StreamExt;
                    let (write, read) = socket_stream.split();
                    read.take_while(|m| futures::future::ready(m.as_ref().unwrap().is_text()))
                        .forward(write)
                        .await
                        .expect("failed to forward message")
                },
                Err(_) => { return; }
            };
        });

        Ok(())
    }
}

#[crate::async_trait]
#[cfg(debug_assertions)]
impl FromData for Websocket {
    type Error = std::convert::Infallible;

    #[inline(always)]
    async fn from_data(_: &Request<'_>, data: Data) -> Outcome<Self, Self::Error> {
        Outcome::Success(Self { data })
    }
}

impl<'r> Responder<'r, 'static> for Websocket {
    fn respond_to(self, request: &'r Request<'_>) -> crate::response::Result<'static> {
        let get_header = |name| request.headers().get_one(name).unwrap_or_default();
        let upgrade_present = get_header("upgrade").eq_ignore_ascii_case("websocket");
        let correct_version_match = get_header("sec-websocket-version") == "13";
        let sec_websocket_accept = get_header("sec-websocket-key");
        
        if !upgrade_present {
            Err(crate::http::Status::UpgradeRequired)
        } else if !correct_version_match || sec_websocket_accept.len() == 0 {
            Err(crate::http::Status::BadRequest)
        } else {
            crate::futures::executor::block_on(async { self.spawn_worker().await });
            Response::build()
                .header(Header::new("Connection", "Upgrade"))
                .header(Header::new("Upgrade", "websocket"))
                .header(Header::new(
                    "sec-websocket-accept",
                    convert_key(sec_websocket_accept),
                ))
                .status(http::Status::SwitchingProtocols)
                .ok()
        }
    }
}
