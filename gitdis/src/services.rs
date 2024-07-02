use super::gitdis::{BranchSettings, Gitdis, GitdisError};
use log::debug;
use memotree::valu3::prelude::*;
use memotree::valu3::prelude::{ToValueBehavior, ToJsonBehavior};
use std::sync::{mpsc::Sender, Arc, RwLock};

pub type ArcGitdisService = Arc<RwLock<GitdisService>>;

pub enum GitdisServiceError {
    RepoAlreadyExists,
    BranchNotFound,
    InternalError(String),
}

pub struct GitdisService {
    sender: std::sync::mpsc::Sender<BranchSettings>,
    pub gitdis: Gitdis,
}

#[derive(ToValue, ToJson)]
pub struct ObjectBranchData {
    key: String,
    create_at: u128,
}

impl GitdisService {
    pub fn new(sender: Sender<BranchSettings>, gitdis: Gitdis) -> Self {
        Self { sender, gitdis }
    }

    pub fn create_repo(
        &mut self,
        settings: BranchSettings,
    ) -> Result<ObjectBranchData, GitdisServiceError> {
        debug!("Creating new repo");

        match self.gitdis.add_repo(settings.clone()) {
            Ok(_) => {
                let repo_key = settings.get_repo_key();
                let object = self.gitdis.get_object(&repo_key);

                match self.sender.send(settings) {
                    Ok(_) => match object {
                        Some(object) => Ok(ObjectBranchData {
                            key: repo_key,
                            create_at: object.get_create_at(),
                        }),
                        None => Err(GitdisServiceError::InternalError(
                            "Failed to get object".to_string(),
                        )),
                    },
                    Err(err) => Err(GitdisServiceError::InternalError(err.to_string())),
                }
            }
            Err(err) => match err {
                GitdisError::RepoExists => Err(GitdisServiceError::RepoAlreadyExists),
                GitdisError::Sender(err) => Err(GitdisServiceError::InternalError(err.to_string())),
            },
        }
    }

    pub fn get_data(
        &self,
        branch_key: &str,
        object_key: &str,
    ) -> Result<Option<Value>, GitdisServiceError> {
        debug!("Getting data from branch {} {}", branch_key, object_key);

        match self.gitdis.get_branch(&branch_key) {
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
