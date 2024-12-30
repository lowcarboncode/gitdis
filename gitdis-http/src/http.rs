use crate::routers::routes;
use gitdis::prelude::GitdisService;
use log::debug;

pub struct HttpServer {
    port: String,
    service: GitdisService,
}

impl HttpServer {
    pub fn new(port: String, service: GitdisService) -> Self {
        Self { port, service }
    }

    pub async fn listen(&self) {
        let port = self.port.clone();
        let routes = routes(self.service.clone());

        let address = format!("0.0.0.0:{}", port);
        let listener = tokio::net::TcpListener::bind(address).await.unwrap();

        debug!("Starting gitdis http server");
        axum::serve(listener, routes).await.unwrap();
    }
}
