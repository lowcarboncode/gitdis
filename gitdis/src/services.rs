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

    //// Get the key and property from a string
    ///
    /// # Example
    /// ```
    /// let (key, property) = get_key_and_property("service.context.data(resources.1.tags.2)");
    /// assert_eq!(key, "service.context.data");
    /// assert_eq!(property, "resources.1.tags.2");
    /// ```
    fn get_key_and_property<'a>(key: &'a str) -> (&'a str, Option<&'a str>) {
        let bytes = key.as_bytes();

        if let Some(start) = bytes.iter().position(|&c| c == b'(') {
            if let Some(end) = bytes.iter().rposition(|&c| c == b')') {
                // Use slices directly on the byte indices
                return (&key[..start], Some(&key[start + 1..end]));
            }
        }

        (key, None)
    }

    pub fn get_data(
        &self,
        branch_key: &str,
        link: &str,
    ) -> Result<Option<Value>, GitdisServiceError> {
        debug!("Getting data from branch {} {}", branch_key, link);

        let (namespace, prop_path) = Self::get_key_and_property(link);

        match self.gitdis.get_branch_cache(&branch_key) {
            Some(cache) => match cache.read() {
                Ok(cache) => {
                    let value = cache.get(namespace).cloned();

                    if value.is_none() {
                        return Ok(None);
                    }

                    if prop_path.is_none() {
                        return Ok(value);
                    }

                    return match value.unwrap() {
                        Value::Object(obj) => Ok(obj.get(prop_path.unwrap()).cloned()),
                        _ => Ok(None),
                    };
                }
                Err(err) => {
                    return Err(GitdisServiceError::InternalError(err.to_string()));
                }
            },
            None => Err(GitdisServiceError::BranchNotFound),
        }
    }
}
