use tokio::io::{AsyncRead, AsyncWrite};
use tokio_tungstenite::WebSocketStream as _WebSocketStream;
use futures::future::BoxFuture;

use crate::http::hyper::Upgraded;

mod websocket;
mod handler;
mod from_message;

pub use websocket::{Websocket};
pub use tokio_tungstenite::tungstenite::Message;

pub type WebSocketStream = _WebSocketStream<Box<dyn AsyncReadWrite>>;
pub type HandlerFuture<'r> = BoxFuture<'r, ()>;
pub trait AsyncReadWrite: AsyncRead + AsyncWrite + Unpin + Send {}

impl AsyncReadWrite for Upgraded {}