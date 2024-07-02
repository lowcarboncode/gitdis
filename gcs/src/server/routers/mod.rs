mod extras;
mod git_dis;
use axum::{
    body::Body,
    http::{self, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Extension, Router,
};
use extras::health_check;
use git_dis::{create_repo, get_object};
use serde::Serialize;


use crate::git_dis::services::ArcGitdisService;

#[derive(Serialize)]
pub struct MessageError {
    message: String,
}

pub struct Response<T>
where
    T: Serialize,
{
    status: StatusCode,
    data: T,
}

impl<T> IntoResponse for Response<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::http::Response<Body> {
        let body = serde_json::to_string(&self.data).unwrap();
        http::Response::builder()
            .status(self.status)
            .header("Content-Type", "application/json")
            .body(body.into())
            .unwrap()
    }
}

impl MessageError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

pub fn routes(service: ArcGitdisService) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/repos", post(create_repo))
        .route(
            "/repos/:owner/:repo/:branch/*object_key",
            get(get_object),
        )
        .layer(Extension(service))
}
