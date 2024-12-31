use crate::{branch_settings::BranchSettings, gitdis::GitdisSettings};
use std::sync::mpsc::{Receiver, Sender};

use super::gitdis::{Gitdis, GitdisError};
use log::debug;
use quickleaf::{valu3::prelude::*, Event};
use serde::Serialize;
use std::sync::{Arc, RwLock};

pub type ArcGitdisService = Arc<RwLock<GitdisService>>;

#[derive(Debug, Clone, PartialEq)]
pub enum GitdisServiceError {
    RepoAlreadyExists,
    BranchNotFound,
    InternalError(String),
}

pub struct GitdisService {
    pub gitdis: Gitdis,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct ServiceAddBranchResponse {
    pub url: String,
    pub branch_name: String,
    pub key: String,
    pub pull_request_interval_millis: u64,
    pub path_target: Option<String>,
    pub create_at: u128,
}

impl GitdisService {
    pub fn new(settings: GitdisSettings, sender: Sender<Event>, receiver: Receiver<Event>) -> Self {
        Self {
            gitdis: Gitdis::new(settings, sender, receiver),
        }
    }

    pub fn add_branch(
        &mut self,
        settings: BranchSettings,
    ) -> Result<ServiceAddBranchResponse, GitdisServiceError> {
        debug!("Creating new repo");

        match self.gitdis.add_branch(settings.clone()) {
            Ok(_) => {
                let branch_key = settings.key.clone();
                let branch = self.gitdis.get_branch(&branch_key);

                match branch {
                    Some(branch) => Ok(ServiceAddBranchResponse {
                        url: settings.url,
                        branch_name: settings.branch_name,
                        key: settings.key,
                        pull_request_interval_millis: settings.pull_request_interval_millis,
                        path_target: settings.path_target,
                        create_at: branch.get_create_at(),
                    }),
                    None => Err(GitdisServiceError::InternalError(
                        "Branch not found after adding repo".to_string(),
                    )),
                }
            }
            Err(err) => match err {
                GitdisError::RepoExists => Err(GitdisServiceError::RepoAlreadyExists),
                GitdisError::Sender(err) => Err(GitdisServiceError::InternalError(err.to_string())),
                GitdisError::BranchNotFound => Err(GitdisServiceError::BranchNotFound),
            },
        }
    }

    pub fn listen_branch(&mut self, branch_key: &str) -> Result<(), GitdisServiceError> {
        debug!("Listening to branch {}", branch_key);

        match self.gitdis.listen_branch(&branch_key) {
            Ok(_) => Ok(()),
            Err(err) => match err {
                GitdisError::BranchNotFound => Err(GitdisServiceError::BranchNotFound),
                GitdisError::Sender(err) => Err(GitdisServiceError::InternalError(err.to_string())),
                GitdisError::RepoExists => Err(GitdisServiceError::RepoAlreadyExists),
            },
        }
    }

    pub fn get_data(
        &self,
        branch_key: &str,
        object_key: &str,
    ) -> Result<Option<Value>, GitdisServiceError> {
        debug!("Getting data from branch {} {}", branch_key, object_key);

        match self.gitdis.get_branch_cache(&branch_key) {
            Some(branch) => match branch.read() {
                Ok(branch) => {
                    println!("{:#?}", branch);
                    match branch.get(&object_key) {
                        Some(value) => Ok(Some(value.clone())),
                        None => Ok(None),
                    }
                }
                Err(err) => {
                    return Err(GitdisServiceError::InternalError(err.to_string()));
                }
            },
            None => Err(GitdisServiceError::BranchNotFound),
        }
    }
}
