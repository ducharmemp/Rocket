#[macro_use]
extern crate rocket;

use rocket::http::ContentType;
use rocket::response::Content;
use rocket::websocket::{HandlerFuture, WebSocketStream, Websocket};
use rocket::Request;

#[get("/echo", upgrade = "<ws>")]
async fn echo(mut ws: Websocket) -> Websocket {
    fn websocket_handler<'r>(_req: &'r Request, stream: WebSocketStream) -> HandlerFuture<'r> {
        Box::pin(async move {
            use rocket::futures::StreamExt;
            let (write, read) = stream.split();
            read.take_while(|m| futures::future::ready(m.is_ok()))
                .forward(write)
                .await;
        })
    }

    ws.handle(websocket_handler);
    ws
}

#[get("/")]
fn index() -> Content<&'static str> {
    Content(
        ContentType::HTML,
        r#"<!DOCTYPE html>
<html lang="en">
    <head>

        <title>WebSocket Echo Server</title>
    </head>
    <body>
        <h1>Echo Server</h1>
        <p id="status">Connecting...</p>
        <input type="text" id="text" />
        <button type="button" id="send">Send</button>
        <div id="lines"></div>
        <script type="text/javascript">
            const lines = document.getElementById('lines');
            const text = document.getElementById('text');
            const status = document.getElementById('status');
            const ws = new WebSocket('ws://' + location.host + '/echo');

            ws.onopen = function() {
                status.innerText = 'Connected :)';
            };

            ws.onclose = function() {
                status.innerText = 'Disconnected :(';
                lines.innerHTML = '';
            };

            ws.onmessage = function(msg) {
                const line = document.createElement('p');
                line.innerText = msg.data;
                lines.prepend(line);
            };

            send.onclick = function() {
                ws.send(text.value);
                text.value = '';
            };
        </script>
    </body>
</html>"#,
    )
}

#[launch]
fn rocket() -> rocket::Rocket {
    rocket::ignite().mount("/", routes![index, echo])
}

#[cfg(test)]
mod test {
    use super::rocket;

    #[rocket::async_test]
    async fn test_connection() {
        use rocket::futures::{StreamExt, SinkExt};

        let client = rocket::local::asynchronous::Client::tracked(rocket()).await.unwrap();
        let request = client.get("/echo");
        let ws_client = request._upgrade().await;
        let (mut tx, mut rx) = ws_client.split();
        tx.send("foo".into()).await.unwrap();
        assert_eq!(rx.next().await.unwrap().expect("Could not receive message"), "foo".into());
    }

    #[rocket::async_test]
    async fn test_send_twice_read_once() {
        use rocket::futures::{StreamExt, SinkExt};

        let client = rocket::local::asynchronous::Client::tracked(rocket()).await.unwrap();
        let request = client.get("/echo");
        let ws_client = request._upgrade().await;
        let (mut writer, mut reader) = ws_client.split();
        writer.send("Foo".into()).await.unwrap();
        writer.send("Bar".into()).await.unwrap();
        assert_eq!(reader.next().await.unwrap().unwrap(), "Foo".into());
    }
}