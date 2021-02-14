#[cfg(feature = "tls")]
use std::path::Path;
use std::sync::Arc;

use hyper::http;
use hyper::server::conn::Http;
use hyper::service::service_fn;
use lazy_static::lazy_static;
use tokio::net::{ToSocketAddrs};
use tokio::net::TcpListener;

use crate::errors::errors::Error;
use crate::middleware::{Middleware, WithState};
use crate::request::request::Request;
use crate::response::response::Response;
use crate::router::router::Router;
use crate::{
    endpoint::{Endpoint, RouterEndpoint},
};

lazy_static! {
    pub static ref SERVER_ID: String = format!("rok {}", env!("CARGO_PKG_VERSION"));
}

pub struct App {
    router: Router,
}

impl App {
    pub fn new() -> App {
        App {
            router: Router::new(),
        }
    }

    pub fn with_state<T>(state: T) -> App
        where
            T: Send + Sync + 'static + Clone,
    {
        let mut app = App::new();

        app.middleware(WithState::new(state));
        app
    }

    pub fn merge(
        &mut self,
        prefix: impl AsRef<str>,
        router: Router,
    ) -> Result<(), crate::errors::errors::Error> {
        self.router.merge(prefix, router)
    }

    pub fn register(&mut self, method: http::Method, path: impl AsRef<str>, ep: impl Endpoint) {
        self.router.register(method, path, ep)
    }

    pub fn options(&mut self, path: impl AsRef<str>, ep: impl Endpoint) {
        self.register(http::Method::OPTIONS, path, ep)
    }

    pub fn get(&mut self, path: impl AsRef<str>, ep: impl Endpoint) {
        self.register(http::Method::GET, path, ep)
    }

    pub fn head(&mut self, path: impl AsRef<str>, ep: impl Endpoint) {
        self.register(http::Method::HEAD, path, ep)
    }

    pub fn post(&mut self, path: impl AsRef<str>, ep: impl Endpoint) {
        self.register(http::Method::POST, path, ep)
    }

    pub fn put(&mut self, path: impl AsRef<str>, ep: impl Endpoint) {
        self.register(http::Method::PUT, path, ep)
    }

    pub fn delete(&mut self, path: impl AsRef<str>, ep: impl Endpoint) {
        self.register(http::Method::DELETE, path, ep)
    }

    pub fn trace(&mut self, path: impl AsRef<str>, ep: impl Endpoint) {
        self.register(http::Method::TRACE, path, ep)
    }

    pub fn connect(&mut self, path: impl AsRef<str>, ep: impl Endpoint) {
        self.register(http::Method::CONNECT, path, ep)
    }

    pub fn patch(&mut self, path: impl AsRef<str>, ep: impl Endpoint) {
        self.register(http::Method::PATCH, path, ep)
    }

    pub fn middleware(&mut self, m: impl Middleware) -> &mut Self {
        self.router.middleware(m);
        self
    }

    pub fn handle_not_found(&mut self, ep: impl Endpoint) -> &mut Self {
        self.router.set_not_found_handler(ep);
        self
    }

    pub async fn respond(self, req: impl Into<Request>) -> Response {
        let req = req.into();
        let App { router } = self;

        let router = Arc::new(router.finalize());

        let endpoint = RouterEndpoint::new(router);
        endpoint.call(req).await
    }

    pub async fn run(self, addr: impl ToSocketAddrs) -> Result<(), Error> {
        let App { router } = self;

        let router = Arc::new(router.finalize());

        let server = Http::new();

        let listener = TcpListener::bind(addr).await.unwrap();
        while let Ok((socket, remote_addr)) = listener.accept().await {
            let server = server.clone();
            let router = router.clone();

            tokio::spawn(async move {
                let router = router.clone();

                let ret = server.serve_connection(
                    socket,
                    service_fn(|req| {
                        let router = router.clone();
                        let req = Request::new(req, Some(remote_addr));

                        async move {
                            let endpoint = RouterEndpoint::new(router);
                            let resp = endpoint.call(req).await;
                            Ok::<_, Error>(resp.into())
                        }
                    }),
                );

                if let Err(e) = ret.await {
                    tracing::error!("serve_connection error: {:?}", e);
                }
            });
        }

        Ok(())
    }

    #[cfg(feature = "tls")]
    pub async fn run_with_tls(
        self,
        addr: impl ToSocketAddrs,
        cert: impl AsRef<Path>,
        key: impl AsRef<Path>,
    ) -> Result<(), Error> {
        let App { router } = self;

        let router = Arc::new(router.finalize());

        let server = Http::new();

        let tls_acceptor = crate::tls::new_tls_acceptor(cert, key)?;

        let listener = TcpListener::bind(addr).await.unwrap();
        while let Ok((socket, remote_addr)) = listener.accept().await {
            let tls_acceptor = tls_acceptor.clone();
            let server = server.clone();
            let router = router.clone();

            tokio::spawn(async move {
                let tls_acceptor = tls_acceptor.clone();
                let router = router.clone();

                match tls_acceptor.accept(socket).await {
                    Ok(stream) => {
                        let ret = server.serve_connection(
                            stream,
                            service_fn(|req| {
                                let router = router.clone();
                                let req = Request::new(req, Some(remote_addr));

                                async move {
                                    let endpoint = RouterEndpoint::new(router);
                                    let resp = endpoint.call(req).await;
                                    Ok::<_, Error>(resp.into())
                                }
                            }),
                        );

                        if let Err(e) = ret.await {
                            tracing::error!("serve_connection error: {:?}", e);
                        }
                    }
                    Err(err) => {
                        tracing::error!("tls accept failed, {:?}", err);
                    }
                }
            });
        }

        Ok(())
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

pub fn server_id() -> &'static str {
    &SERVER_ID
}

