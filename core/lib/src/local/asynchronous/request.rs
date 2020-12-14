use std::borrow::Cow;

use tokio::io::{duplex, DuplexStream};
use tokio_tungstenite::{tungstenite::protocol, WebSocketStream};

use crate::http::{ext::IntoOwned, uri::Origin, Header, Method, Status};
use crate::websocket::AsyncReadWrite;
use crate::{Data, Request};

use super::{Client, LocalResponse};

impl AsyncReadWrite for DuplexStream {}

/// An `async` local request as returned by [`Client`](super::Client).
///
/// For details, see [the top-level documentation](../index.html#localrequest).
///
/// ## Example
///
/// The following snippet uses the available builder methods to construct and
/// dispatch a `POST` request to `/` with a JSON body:
///
/// ```rust
/// use rocket::local::asynchronous::{Client, LocalRequest};
/// use rocket::http::{ContentType, Cookie};
///
/// # rocket::async_test(async {
/// let client = Client::tracked(rocket::ignite()).await.expect("valid rocket");
/// let req = client.post("/")
///     .header(ContentType::JSON)
///     .remote("127.0.0.1:8000".parse().unwrap())
///     .cookie(Cookie::new("name", "value"))
///     .body(r#"{ "value": 42 }"#);
///
/// let response = req.dispatch().await;
/// # });
/// ```
pub struct LocalRequest<'c> {
    pub(super) client: &'c Client,
    pub(super) request: Request<'c>,
    data: Vec<u8>,
    uri: Cow<'c, str>,
}

impl<'c> LocalRequest<'c> {
    pub(crate) fn new(client: &'c Client, method: Method, uri: Cow<'c, str>) -> LocalRequest<'c> {
        // We try to validate the URI now so that the inner `Request` contains a
        // valid URI. If it doesn't, we set a dummy one.
        let origin = Origin::parse(&uri).unwrap_or_else(|_| Origin::dummy());
        let mut request = Request::new(client.rocket(), method, origin.into_owned());

        // Add any cookies we know about.
        if client.tracked {
            client._with_raw_cookies(|jar| {
                for cookie in jar.iter() {
                    request.cookies_mut().add_original(cookie.clone());
                }
            })
        }

        LocalRequest {
            client,
            request,
            uri,
            data: vec![],
        }
    }

    pub(crate) fn _request(&self) -> &Request<'c> {
        &self.request
    }

    pub(crate) fn _request_mut(&mut self) -> &mut Request<'c> {
        &mut self.request
    }

    pub(crate) fn _body_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }

    pub(crate) fn _into_inner(self) -> Request<'c> {
        self.request
    }

    // Performs the actual dispatch.
    async fn _dispatch(mut self) -> LocalResponse<'c> {
        // First, revalidate the URI, returning an error response (generated
        // from an error catcher) immediately if it's invalid. If it's valid,
        // then `request` already contains the correct URI.
        let rocket = self.client.rocket();
        if let Err(_) = Origin::parse(&self.uri) {
            error!("Malformed request URI: {}", self.uri);
            return LocalResponse::new(self.request, move |req| {
                rocket.handle_error(Status::BadRequest, req)
            })
            .await;
        }

        // Actually dispatch the request.
        let mut data = Data::local(self.data);
        let token = rocket
            .preprocess_request(&mut self.request, &mut data)
            .await;
        let response =
            LocalResponse::new(self.request, move |req| rocket.dispatch(token, req, data)).await;

        // If the client is tracking cookies, updates the internal cookie jar
        // with the changes reflected by `response`.
        if self.client.tracked {
            self.client._with_raw_cookies_mut(|jar| {
                let current_time = time::OffsetDateTime::now_utc();
                for cookie in response.cookies().iter() {
                    if let Some(expires) = cookie.expires() {
                        if expires <= current_time {
                            jar.force_remove(cookie);
                            continue;
                        }
                    }

                    jar.add_original(cookie.clone());
                }
            })
        }

        response
    }

    pub async fn _upgrade(mut self) -> WebSocketStream<DuplexStream> {
        let (client_socket, server_socket) = duplex(2048);
        self._request_mut().add_header(Header::new("upgrade", "websocket"));
        self._request_mut().add_header(Header::new("sec-websocket-version", "13"));
        self._request_mut().add_header(Header::new("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ=="));

        let rocket = self.client.rocket.clone();
        let response = self.dispatch().await;
        let upgrade = response._into_upgrade().expect("No upgrade available");
        let handler = upgrade.handler.unwrap();

        tokio::task::spawn(async move {
            let server = WebSocketStream::from_raw_socket(
                Box::new(server_socket) as Box<dyn AsyncReadWrite>,
                protocol::Role::Server,
                Default::default(),
            )
            .await;
            
            
            handler.handle(&Request::new(&rocket, Method::Get, Origin::dummy()), server).await;
        });

        WebSocketStream::from_raw_socket(client_socket, protocol::Role::Client, Default::default())
            .await
    }

    pub_request_impl!("# use rocket::local::asynchronous::Client;
        use rocket::local::asynchronous::LocalRequest;" async await);
}

impl<'c> Clone for LocalRequest<'c> {
    fn clone(&self) -> Self {
        LocalRequest {
            client: self.client,
            request: self.request.clone(),
            data: self.data.clone(),
            uri: self.uri.clone(),
        }
    }
}

impl std::fmt::Debug for LocalRequest<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self._request().fmt(f)
    }
}
