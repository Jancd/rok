pub mod logs;
mod default_headers;
mod request_id;
mod with_state;

pub use default_headers::DefaultHeaders;
pub use request_id::RequestId;
pub use with_state::WithState;

use std::future::Future;
use std::sync::Arc;

use crate::request::request::Request;
use crate::response::response::Response;
use crate::endpoint::DynEndpoint;

#[crate::async_trait]
pub trait Middleware: 'static + Send + Sync {
    async fn handle<'a>(&'a self, req: Request, next: Next<'a>) -> Response;

    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}

#[crate::async_trait]
impl<F, Fut> Middleware for F
    where
        F: Fn(Request, Next<'_>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output=Response> + Send + 'static,
{
    async fn handle<'a>(&'a self, req: Request, next: Next<'a>) -> Response {
        (self)(req, next).await
    }
}

#[allow(missing_debug_implementations)]
pub struct Next<'a> {
    pub(crate) endpoint: &'a DynEndpoint,
    pub(crate) next_middleware: &'a [Arc<dyn Middleware>],
}

impl<'a> Next<'a> {
    pub async fn run(mut self, req: Request) -> Response {
        if let Some((current, next)) = self.next_middleware.split_first() {
            self.next_middleware = next;
            current.handle(req, self).await
        } else {
            (self.endpoint).call(req).await
        }
    }
}
