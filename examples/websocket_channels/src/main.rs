#[macro_use]
extern crate rocket;

use std::collections::HashMap;

use rocket::{Request, websocket::Message, futures::{FutureExt, SinkExt, StreamExt}};
use rocket::{
    futures::select,
    websocket::{HandlerFuture, WebSocketStream, Websocket},
    State,
};
use rocket_contrib::templates::Template;
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedSender},
    Mutex,
};
use uuid::Uuid;

type ChannelRegistry = Mutex<HashMap<String, HashMap<Uuid, UnboundedSender<Message>>>>;

#[get("/channel/<channel_name>", upgrade = "<ws>")]
async fn channel_handler(
    channel_name: String,
    mut ws: Websocket,
    channels: State<'_, ChannelRegistry>,
) -> Websocket {
    channels
        .lock()
        .await
        .entry(channel_name.clone())
        .or_default();

    ws.handle(
        |req: &'_ Request, stream: WebSocketStream| -> HandlerFuture<'_> {
            Box::pin(async move {
                let (broadcast_write, mut broadcast_read) = unbounded_channel();
                let (mut ws_write, mut ws_read) = stream.split();
                let user_id = Uuid::new_v4();
                

                let channels = req.guard::<State<'_, ChannelRegistry>>().await.expect("Could not acquire channel registry");
                channels
                        .lock()
                        .await
                        .get_mut(&channel_name.clone())
                        .and_then(|val| val.insert(user_id.clone(), broadcast_write));

                loop {
                    select! {
                        ws_message = ws_read.next().fuse() => {
                            if let Some(ws_message) = ws_message {
                                let ws_message = ws_message.unwrap();
                                channels
                                    .lock()
                                    .await
                                    .get(&channel_name)
                                    .and_then(|entry| {
                                        for (user, channel) in entry {
                                            let ws_message = ws_message.clone();
                                            if *user == user_id {
                                                continue;
                                            }
                                            channel.send(ws_message.clone()).expect("Could not send message to peer websocket");
                                        }

                                        Some(())
                                    });

                               
                                ws_write.send(ws_message).await.expect("Could not send message to websocket");
                            } else {
                                channels
                                    .lock()
                                    .await
                                    .get_mut(&channel_name.clone())
                                    .and_then(|entry| entry.remove(&user_id));
                                return;
                            }
                        },
                        broadcast_message = broadcast_read.next().fuse() => {
                            if let Some(broadcast_message) = broadcast_message {
                                ws_write.send(broadcast_message.into()).await.expect("Could not send message to websocket");
                            }
                        }
                    };
                }
            })
        },
    );
    ws
}

#[get("/")]
fn index() -> Template {
    let context: HashMap<String, String> = HashMap::new();
    Template::render("index", &context)
}

#[launch]
fn rocket() -> rocket::Rocket {
    let channels: ChannelRegistry = Mutex::new(HashMap::new());

    rocket::ignite()
        .mount("/", routes![index, channel_handler])
        .attach(Template::fairing())
        .manage(channels)
}

#[cfg(test)]
mod test {
    use super::rocket;

    #[rocket::async_test]
    async fn test_connection() {
        use rocket::futures::{SinkExt, StreamExt};

        let client = rocket::local::asynchronous::Client::tracked(rocket())
            .await
            .unwrap();
        let request = client.get("/channel/1");
        let ws_client = request._upgrade().await;
        let (mut tx, mut rx) = ws_client.split();
        tx.send("foo".into()).await.unwrap();
        assert_eq!(
            rx.next().await.unwrap().expect("Could not receive message"),
            "foo".into()
        );
    }

    #[rocket::async_test]
    async fn test_multi_connection() {
        use rocket::futures::{SinkExt, StreamExt};

        let client = rocket::local::asynchronous::Client::tracked(rocket())
            .await
            .unwrap();

        let request = client.get("/channel/1");
        let request2 = client.get("/channel/1");
        let ws_client = request._upgrade().await;
        let ws_client2 = request2._upgrade().await;

        let (mut tx, mut rx) = ws_client.split();
        let (mut tx2, mut rx2) = ws_client2.split();

        tx.send("foo".into()).await.unwrap();
        assert_eq!(
            rx.next().await.unwrap().expect("Could not receive message"),
            "foo".into()
        );

        assert_eq!(
            rx2.next()
                .await
                .unwrap()
                .expect("Could not receive message"),
            "foo".into()
        );

        tx2.send("bar".into()).await.unwrap();
        assert_eq!(
            rx.next().await.unwrap().expect("Could not receive message"),
            "bar".into()
        );

        assert_eq!(
            rx2.next()
                .await
                .unwrap()
                .expect("Could not receive message"),
            "bar".into()
        );
    }
}
