use super::gitdis::{BranchSettings, Gitdis, GitdisError};
use log::debug;
use quickleaf::valu3::prelude::*;
use std::sync::{Arc, RwLock};

#[derive(Debug, PartialEq)]
pub enum GitdisServiceError {
    RepoAlreadyExists,
    BranchNotFound,
    InternalError(String),
    RepoNotCreated,
}

#[derive(Clone)]
pub struct GitdisService {
    pub gitdis: Arc<RwLock<Gitdis>>,
}

#[derive(ToValue, ToJson)]
pub struct BranchInfo {
    key: String,
    create_at: u128,
}

impl GitdisService {
    pub fn new(gitdis: Arc<RwLock<Gitdis>>) -> Self {
        Self { gitdis }
    }

    pub fn add_repo(&mut self, settings: BranchSettings) -> Result<BranchInfo, GitdisServiceError> {
        debug!("Creating new repo");

        let mut gitdis = match self.gitdis.write() {
            Ok(gitdis) => gitdis,
            Err(_) => {
                return Err(GitdisServiceError::InternalError(
                    "Error writing gitdis".to_string(),
                ))
            }
        };

        match gitdis.add_repo(settings.clone()) {
            Ok(_) => {
                let repo_key = settings.get_repo_key();
                let object = gitdis.get_object_branch(&repo_key);

                match object {
                    Some(object) => Ok(BranchInfo {
                        key: repo_key,
                        create_at: object.get_create_at(),
                    }),
                    None => Err(GitdisServiceError::RepoNotCreated),
                }
            }
            Err(err) => match err {
                GitdisError::RepoExists => Err(GitdisServiceError::RepoAlreadyExists),
                GitdisError::Sender(err) => Err(GitdisServiceError::InternalError(err.to_string())),
                GitdisError::BranchNotFound => Err(GitdisServiceError::BranchNotFound),
                GitdisError::RepoListener => Err(GitdisServiceError::InternalError(
                    "Error creating repo listener".to_string(),
                )),
            },
        }
    }

    // pub fn get_data(
    //     &self,
    //     branch_key: &str,
    //     object_key: &str,
    // ) -> Result<Option<Value>, GitdisServiceError> {
    //     debug!("Getting data from branch {} {}", branch_key, object_key);

    //     match self.gitdis.get_data_branch(&branch_key) {
    //         Some(branch) => {
    //             let branch = branch.read().unwrap();

    //             match branch.get(&object_key) {
    //                 Some(value) => Ok(Some(value.clone())),
    //                 None => Ok(None),
    //             }
    //         }
    //         None => Err(GitdisServiceError::BranchNotFound),
    //     }
    // }
}
