use super::WebSocketStream;

use crate::request::Request;
use super::HandlerFuture;

#[crate::async_trait]
pub trait Handler: Cloneable + Send + Sync + 'static {
    /// Called by Rocket when a `Request` with its associated `Data` should be
    /// handled by this handler.
    ///
    /// The variant of `Outcome` returned by the returned `Future` determines
    /// what Rocket does next. If the return value is a `Success(Response)`, the
    /// wrapped `Response` is used to respond to the client. If the return value
    /// is a `Failure(Status)`, the error catcher for `Status` is invoked to
    /// generate a response. Otherwise, if the return value is `Forward(Data)`,
    /// the next matching route is attempted. If there are no other matching
    /// routes, the `404` error catcher is invoked.
    async fn handle<'r, 's: 'r>(&'s self, request: &'r Request<'_>, stream: WebSocketStream) -> ();
}

#[crate::async_trait]
impl<F: Clone + Sync + Send + 'static> Handler for F
    where for<'x> F: FnOnce(&'x Request<'_>, WebSocketStream) -> HandlerFuture<'x>
{
    #[inline(always)]
    async fn handle<'r, 's: 'r>(&'s self, req: &'r Request<'_>, stream: WebSocketStream) -> () {
        self.clone()(req, stream).await
    }
}

mod private {
  pub trait Sealed {}
  impl<T: super::Handler + Clone> Sealed for T {}
}

/// Unfortunate but necessary hack to be able to clone a `Box<Handler>`.
///
/// This trait cannot be implemented by any type. Instead, all types that
/// implement `Clone` and `Handler` automatically implement `Cloneable`.
pub trait Cloneable: private::Sealed {
  #[doc(hidden)]
  fn clone_handler(&self) -> Box<dyn Handler>;
}

impl<T: Handler + Clone> Cloneable for T {
  fn clone_handler(&self) -> Box<dyn Handler> {
      Box::new(self.clone())
  }
}

impl Clone for Box<dyn Handler> {
  fn clone(&self) -> Box<dyn Handler> {
      self.clone_handler()
  }
}
