mod http;
mod routers;

#[cfg(debug_assertions)]
use dotenv::dotenv;
use gitdis::prelude::*;
use http::HttpServer;
use log::debug;
use std::sync::{Arc, RwLock};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    #[cfg(debug_assertions)]
    dotenv().ok();

    debug!("Starting gitdis");

    env_logger::init();

    let http_port = std::env::var("GITDIS_HTTP_PORT").unwrap_or("3000".to_string());
    let local_clone_path = std::env::var("GITDIS_LOCAL_CLONE_PATH").unwrap_or("data".to_string());

    if !std::path::Path::new(&local_clone_path).exists() {
        std::fs::create_dir(&local_clone_path).expect("Failed to create repo directory");
    }

    debug!(
        "Starting gitdis with local clone path: {}",
        local_clone_path
    );

    let (sender, receiver) = std::sync::mpsc::channel();

    let gitdis = Gitdis::from(GitdisSettings {
        total_branch_items: 100,
        local_clone_path,
    });

    let service = Arc::new(RwLock::new(GitdisService::new(sender, gitdis)));

    let server = HttpServer::new(http_port, service.clone());

    tokio::spawn(async move {
        server.listen().await;
    });

    for settings in receiver.iter() {
        let service = service.read().unwrap();
        service.gitdis.spawn_branch_listener(settings.clone());
    }

    Ok(())
}
