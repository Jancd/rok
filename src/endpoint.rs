use std::future::Future;
use std::sync::Arc;

use crate::router::router::Router;
use crate::request::request::Request;
use crate::response::response::Response;

pub(crate) type DynEndpoint = dyn Endpoint;

#[crate::async_trait]
pub trait Endpoint: Send + Sync + 'static {
    /// Invoke the endpoint within the given context
    async fn call(&self, req: Request) -> Response;
}

#[crate::async_trait]
impl<F: Send + Sync + 'static, Fut, Res> Endpoint for F
    where
        F: Fn(Request) -> Fut,
        Fut: Future<Output=Res> + Send + 'static,
        Res: Into<Response> + 'static,
{
    async fn call(&self, req: Request) -> Response {
        let resp = self(req).await;
        resp.into()
    }
}

pub(crate) struct RouterEndpoint {
    router: Arc<Router>,
}

impl RouterEndpoint {
    pub(crate) fn new(router: Arc<Router>) -> RouterEndpoint {
        RouterEndpoint { router }
    }
}

#[crate::async_trait]
impl Endpoint for RouterEndpoint {
    async fn call(&self, req: Request) -> Response {
        self.router.route(req).await
    }
}

impl std::fmt::Debug for RouterEndpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RouterEndpoint{{ router: {:?} }}", self.router)
    }
}
