#![feature(prelude_import)]
#[prelude_import]
use std::prelude::v1::*;
#[macro_use]
extern crate std;
#[macro_use]
extern crate rocket;
use rocket::http::ContentType;
use rocket::response::Content;
use rocket::websocket::{HandlerFuture, WebSocketStream, Websocket};
use rocket::Request;
async fn echo(mut ws: Websocket) -> Websocket {
    fn websocket_handler<'r>(_req: &'r Request, stream: WebSocketStream) -> HandlerFuture<'r> {
        Box::pin(async move {
            use rocket::futures::StreamExt;
            let (write, read) = stream.split();
            read.take_while(|m| {
                futures::future::ready(m.as_ref().map(|v| v.is_text()).unwrap_or_default())
            })
            .forward(write)
            .await
            .unwrap();
        })
    }
    ws.handle(websocket_handler);
    ws
}
#[doc(hidden)]
#[allow(non_camel_case_types)]
/// Rocket code generated proxy structure.
struct echo {}
/// Rocket code generated proxy static conversion implementation.
impl From<echo> for rocket::StaticRouteInfo {
    fn from(_: echo) -> rocket::StaticRouteInfo {
        fn monomorphized_function<'_b>(
            __req: &'_b rocket::request::Request,
            __data: rocket::data::Data,
        ) -> rocket::handler::HandlerFuture<'_b> {
            ::std::boxed::Box::pin(async move {
                let __transform =
                    <Websocket as rocket::data::FromTransformedData>::transform(__req, __data)
                        .await;
                #[allow(unreachable_patterns, unreachable_code)]
                let __outcome = match __transform {
                    rocket::data::Transform::Owned(rocket::outcome::Outcome::Success(__v)) => {
                        rocket::data::Transform::Owned(rocket::outcome::Outcome::Success(__v))
                    }
                    rocket::data::Transform::Borrowed(rocket::outcome::Outcome::Success(
                        ref __v,
                    )) => rocket::data::Transform::Borrowed(rocket::outcome::Outcome::Success(
                        ::std::borrow::Borrow::borrow(__v),
                    )),
                    rocket::data::Transform::Borrowed(__o) => {
                        rocket::data::Transform::Borrowed(__o.map(|_| {
                            {
                                {
                                    ::core::panicking::panic_fmt(::core::fmt::Arguments::new_v1(
                                        &["internal error: entered unreachable code: "],
                                        &match (
                                            &"Borrowed(Success(..)) case handled in previous block",
                                        ) {
                                            (arg0,) => [::core::fmt::ArgumentV1::new(
                                                arg0,
                                                ::core::fmt::Display::fmt,
                                            )],
                                        },
                                    ))
                                }
                            }
                        }))
                    }
                    rocket::data::Transform::Owned(__o) => rocket::data::Transform::Owned(__o),
                };
                #[allow(non_snake_case, unreachable_patterns, unreachable_code)]
                let __rocket_param_ws: Websocket =
                    match <Websocket as rocket::data::FromTransformedData>::from_data(
                        __req, __outcome,
                    )
                    .await
                    {
                        rocket::outcome::Outcome::Success(__d) => __d,
                        rocket::outcome::Outcome::Forward(__d) => {
                            return rocket::outcome::Outcome::Forward(__d)
                        }
                        rocket::outcome::Outcome::Failure((__c, _)) => {
                            return rocket::outcome::Outcome::Failure(__c)
                        }
                    };
                let ___responder = echo(__rocket_param_ws).await;
                rocket::handler::Outcome::from(__req, ___responder)
            })
        }
        rocket::StaticRouteInfo {
            name: "echo",
            method: ::rocket::http::Method::Get,
            path: "/echo",
            handler: monomorphized_function,
            format: ::std::option::Option::None,
            rank: ::std::option::Option::None,
        }
    }
}
/// Rocket code generated proxy conversion implementation.
impl From<echo> for rocket::Route {
    #[inline]
    fn from(_: echo) -> rocket::Route {
        rocket::StaticRouteInfo::from(echo {}).into()
    }
}
#[doc(hidden)]
pub use rocket_uri_macro_echo2876018103858631756 as rocket_uri_macro_echo;
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
#[doc(hidden)]
#[allow(non_camel_case_types)]
/// Rocket code generated proxy structure.
struct index {}
/// Rocket code generated proxy static conversion implementation.
impl From<index> for rocket::StaticRouteInfo {
    fn from(_: index) -> rocket::StaticRouteInfo {
        fn monomorphized_function<'_b>(
            __req: &'_b rocket::request::Request,
            __data: rocket::data::Data,
        ) -> rocket::handler::HandlerFuture<'_b> {
            ::std::boxed::Box::pin(async move {
                let ___responder = index();
                rocket::handler::Outcome::from(__req, ___responder)
            })
        }
        rocket::StaticRouteInfo {
            name: "index",
            method: ::rocket::http::Method::Get,
            path: "/",
            handler: monomorphized_function,
            format: ::std::option::Option::None,
            rank: ::std::option::Option::None,
        }
    }
}
/// Rocket code generated proxy conversion implementation.
impl From<index> for rocket::Route {
    #[inline]
    fn from(_: index) -> rocket::Route {
        rocket::StaticRouteInfo::from(index {}).into()
    }
}
#[doc(hidden)]
pub use rocket_uri_macro_index9699653849120046244 as rocket_uri_macro_index;
#[allow(dead_code)]
fn rocket() -> rocket::Rocket {
    rocket::ignite().mount("/", {
        let ___vec: ::std::vec::Vec<::rocket::Route> = <[_]>::into_vec(box [
            {
                let ___struct = index {};
                let ___item: ::rocket::Route = ___struct.into();
                ___item
            },
            {
                let ___struct = echo {};
                let ___item: ::rocket::Route = ___struct.into();
                ___item
            },
        ]);
        ___vec
    })
}
fn main() {
    ::rocket::async_main(async move {
        let _ = {
            let ___rocket: rocket::Rocket = {
                rocket::ignite().mount("/", {
                    let ___vec: ::std::vec::Vec<::rocket::Route> = <[_]>::into_vec(box [
                        {
                            let ___struct = index {};
                            let ___item: ::rocket::Route = ___struct.into();
                            ___item
                        },
                        {
                            let ___struct = echo {};
                            let ___item: ::rocket::Route = ___struct.into();
                            ___item
                        },
                    ]);
                    ___vec
                })
            };
            let ___rocket: ::rocket::Rocket = ___rocket;
            ___rocket
        }
        .launch()
        .await;
    })
}
