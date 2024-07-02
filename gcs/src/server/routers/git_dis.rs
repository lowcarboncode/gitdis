use axum::{extract::Path, http, Extension, Json};
use log::debug;
use memotree::valu3::prelude::*;
use serde::{Deserialize, Serialize};

use crate::git_dis::{
    git_dis::BranchSettings,
    services::{Error, ObjectBranchData},
};

use super::{ArcGitdisService, MessageError, Response};

#[derive(Deserialize, Serialize, Clone)]
pub struct CreateRepo {
    url: String,
    branch_name: Option<String>,
    pull_request_interval_millis: Option<u64>,
}

impl Into<BranchSettings> for CreateRepo {
    fn into(self) -> BranchSettings {
        BranchSettings {
            url: self.url,
            branch_name: self.branch_name.unwrap_or("main".to_string()),
            pull_request_interval_millis: self.pull_request_interval_millis.unwrap_or(3000),
        }
    }
}

fn resolve_errors(err: Error) -> Response<MessageError> {
    match err {
        Error::RepoAlreadyExists => Response {
            status: http::StatusCode::CONFLICT,
            data: MessageError::new("Repo already exists".to_string()),
        },
        Error::BranchNotFound => Response {
            status: http::StatusCode::NOT_FOUND,
            data: MessageError::new("Branch not found".to_string()),
        },
        Error::InternalError(err) => Response {
            status: http::StatusCode::INTERNAL_SERVER_ERROR,
            data: MessageError::new(err),
        },
    }
}

pub async fn create_repo(
    Extension(gitdis): Extension<ArcGitdisService>,
    Json(payload): Json<CreateRepo>,
) -> Result<Response<ObjectBranchData>, Response<MessageError>> {
    debug!("Creating new repo router");
    let mut services = gitdis.write().unwrap();

    match services.create_repo(payload.into()) {
        Ok(data) => Ok(Response {
            status: http::StatusCode::CREATED,
            data,
        }),
        Err(err) => Err(resolve_errors(err)),
    }
}

#[derive(Deserialize, Debug)]
pub struct ObjectParams {
    owner: String,
    repo: String,
    branch: String,
    object_key: String,
}

impl ObjectParams {
    fn get_branch_key(&self) -> String {
        format!(
            "{}/{}/{}",
            self.owner, self.repo, self.branch
        )
    }
}

pub async fn get_object(
    Extension(gitdis): Extension<ArcGitdisService>,
    Path(params): Path<ObjectParams>,
) -> Result<Response<Option<Value>>, Response<MessageError>> {
    let services: std::sync::RwLockReadGuard<crate::git_dis::services::GitdisServices> =
        gitdis.read().unwrap();

    let branch_key = params.get_branch_key();

    match services.get_data(&branch_key, &params.object_key) {
        Ok(value) => Ok(Response {
            status: http::StatusCode::OK,
            data: match value {
                Some(value) => Some(value),
                None => None,
            },
        }),
        Err(err) => Err(resolve_errors(err)),
    }
}