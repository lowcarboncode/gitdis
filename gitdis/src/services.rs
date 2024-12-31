use crate::{branch_settings::BranchSettings, gitdis::GitdisSettings};
use std::sync::mpsc::{Receiver, Sender};

use super::gitdis::{Gitdis, GitdisError};
use log::debug;
use quickleaf::{valu3::prelude::*, Event};
use std::sync::{Arc, RwLock};

pub type ArcGitdisService = Arc<RwLock<GitdisService>>;

pub enum GitdisServiceError {
    RepoAlreadyExists,
    BranchNotFound,
    InternalError(String),
}

pub struct GitdisService {
    gitdis: Gitdis,
}

#[derive(ToValue, ToJson)]
pub struct ObjectBranchData {
    key: String,
    create_at: u128,
}

impl GitdisService {
    pub fn new(settings: GitdisSettings, sender: Sender<Event>, receiver: Receiver<Event>) -> Self {
        Self {
            gitdis: Gitdis::new(settings, sender, receiver),
        }
    }

    pub fn create_repo(
        &mut self,
        settings: BranchSettings,
    ) -> Result<ObjectBranchData, GitdisServiceError> {
        debug!("Creating new repo");

        match self.gitdis.add_repo(settings.clone()) {
            Ok(_) => {
                let repo_key = settings.repo_key.clone();
                let branch = self.gitdis.get_branch(&repo_key);

                match branch {
                    Some(branch) => {
                        let cache = branch.get_data().read().unwrap();

                        Ok(ObjectBranchData {
                            key: branch.get_settings().repo_key.clone(),
                            create_at: branch.get_create_at(),
                        })
                    }
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

    pub fn get_data(
        &self,
        branch_key: &str,
        object_key: &str,
    ) -> Result<Option<Value>, GitdisServiceError> {
        debug!("Getting data from branch {} {}", branch_key, object_key);

        match self.gitdis.get_branch_data(&branch_key) {
            Some(branch) => {
                let branch = branch.read().unwrap();

                match branch.get(&object_key) {
                    Some(value) => Ok(Some(value.clone())),
                    None => Ok(None),
                }
            }
            None => Err(GitdisServiceError::BranchNotFound),
        }
    }
}
