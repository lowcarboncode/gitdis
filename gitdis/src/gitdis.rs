use std::{
    collections::HashMap,
    sync::{mpsc::SendError, Arc, RwLock},
};

use branch_handler::BranchHandler;
use log::debug;

use crate::cache::{ArcCache, CacheRepo};

use super::branch_handler;

#[derive(Debug)]
pub enum GitdisError {
    RepoExists,
    Sender(SendError<BranchSettings>),
}

#[derive(Clone, Debug)]
pub struct BranchSettings {
    pub url: String,
    pub branch_name: String,
    pub pull_request_interval_millis: u64,
}

impl BranchSettings {
    pub fn get_repo_key(&self) -> String {
        let url = self.url.clone();
        let url = url.split('/').collect::<Vec<&str>>();
        let repo_name = url[url.len() - 1].split('.').collect::<Vec<&str>>()[0];
        let repo_owner = {
            let repo_owner = url[url.len() - 2];

            if repo_owner.contains(":") {
                repo_owner.split(':').collect::<Vec<&str>>()[1]
            } else {
                repo_owner
            }
        };

        format!("{}/{}/{}", repo_owner, repo_name, self.branch_name)
    }
}

pub struct GitdisSettings {
    pub total_branch_items: usize,
    pub local_clone_path: String,
}

#[derive(Clone)]
pub struct ObjectBranch {
    data: ArcCache,
    create_at: u128,
}

impl ObjectBranch {
    pub fn new(total_branch_items: usize) -> Self {
        let create_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        debug!("Creating new branch with {} items", total_branch_items);

        ObjectBranch {
            data: Arc::new(RwLock::new(CacheRepo::new(total_branch_items))),
            create_at,
        }
    }

    pub fn get_data(&self) -> ArcCache {
        self.data.clone()
    }

    pub fn get_create_at(&self) -> u128 {
        self.create_at
    }
}

pub struct Gitdis {
    pub settings: GitdisSettings,
    branches: HashMap<String, ObjectBranch>,
}

impl Gitdis {
    pub fn update_settings(&mut self, settings: GitdisSettings) {
        self.settings = settings;
    }

    pub fn get_object(&self, repo_key: &str) -> Option<ObjectBranch> {
        match self.branches.get(repo_key) {
            Some(branch) => Some(branch.clone()),
            None => None,
        }
    }

    pub fn get_branch(&self, repo_key: &str) -> Option<ArcCache> {
        debug!("Getting branch: {}", repo_key);
        debug!("Branches: {:?}", self.branches.keys());

        match self.branches.get(repo_key) {
            Some(branch) => Some(branch.get_data()),
            None => None,
        }
    }

    pub fn add_repo(&mut self, settings: BranchSettings) -> Result<(), GitdisError> {
        debug!("Adding new repo");

        let repo_key = settings.get_repo_key();

        debug!("Repo key: {}", repo_key);

        if self.branches.contains_key(&repo_key) {
            debug!("Repo already exists");
            return Err(GitdisError::RepoExists);
        }

        let key = settings.get_repo_key();

        self.branches.insert(
            key.clone(),
            ObjectBranch::new(self.settings.total_branch_items),
        );

        debug!("Added new repo: {}", key);

        Ok(())
    }

    pub fn spawn_branch_listener(&self, settings: BranchSettings) -> tokio::task::JoinHandle<()> {
        let branch = self.get_branch(&settings.get_repo_key()).unwrap();
        let local_clone_path = self.settings.local_clone_path.clone();

        debug!("Starting listener for branch: {}", settings.get_repo_key());

        tokio::spawn(async move {
            let mut branch = BranchHandler::new(
                local_clone_path.clone(),
                settings.url,
                settings.branch_name,
                branch,
                settings.pull_request_interval_millis,
            );

            match branch.listener() {
                Ok(_) => (),
                Err(e) => panic!("Error: {}", e),
            }
        })
    }
}

impl From<GitdisSettings> for Gitdis {
    fn from(settings: GitdisSettings) -> Self {
        Self {
            settings,
            branches: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_settings_get_repo_key() {
        let settings = BranchSettings {
            url: "https://github.com/user/repo.git".to_string(),
            branch_name: "main".to_string(),
            pull_request_interval_millis: 1000,
        };

        let repo_key = settings.get_repo_key();
        assert_eq!(repo_key, "user/repo/main");
    }
}
