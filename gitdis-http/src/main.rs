mod http;
mod routers;

use gitdis::prelude::*;
use http::HttpServer;
use log::debug;
use std::sync::{Arc, RwLock};

#[tokio::main]
async fn main() -> std::io::Result<()> {
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

    let gitdis = Gitdis::from(GitdisSettings {
        total_branch_items: 100,
        local_clone_path,
    });

    let service = GitdisService::new(gitdis);

    let server = HttpServer::new(http_port, service);
    server.listen();

    Ok(())
}
