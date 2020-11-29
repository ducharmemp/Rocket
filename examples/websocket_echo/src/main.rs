#[macro_use]
extern crate rocket;

use rocket::http::ContentType;
use rocket::response::Content;
use rocket::tungstenite::Websocket;

#[get("/echo", data="<ws>")]
async fn echo(mut ws: Websocket) -> Websocket {
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
