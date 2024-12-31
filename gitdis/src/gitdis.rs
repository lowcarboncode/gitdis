use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::{
    collections::HashMap,
    sync::mpsc::{self, SendError},
};

use branch_handler::BranchHandler;
use log::debug;
use quickleaf::Event;

use crate::branch_settings::BranchSettings;
use crate::cache::ArcCache;
use crate::cache_branch::CacheBranch;

use super::branch_handler;

#[derive(Debug, PartialEq)]
pub enum GitdisError {
    RepoExists,
    Sender(SendError<BranchSettings>),
    BranchNotFound,
}

#[derive(Clone)]
pub struct GitdisSettings {
    pub total_branch_items: usize,
    pub local_clone_path: String,
}

pub struct Gitdis {
    pub settings: GitdisSettings,
    branches: HashMap<String, Branch>,
    sender: Sender<Event>,
    pub receiver: Receiver<Event>,
}

#[derive(Clone)]
pub struct Branch {
    settings: BranchSettings,
    cache: CacheBranch,
}

impl Branch {
    pub fn new(settings: BranchSettings, cache: CacheBranch) -> Self {
        Self { settings, cache }
    }

    pub fn get_data(&self) -> &ArcCache {
        self.cache.get_data()
    }

    pub fn get_create_at(&self) -> u128 {
        self.cache.get_create_at()
    }

    pub fn get_settings(&self) -> &BranchSettings {
        &self.settings
    }
}

impl Gitdis {
    pub fn new(settings: GitdisSettings, sender: Sender<Event>, receiver: Receiver<Event>) -> Self {
        Self {
            settings,
            branches: HashMap::new(),
            sender,
            receiver,
        }
    }

    pub fn update_settings(&mut self, settings: GitdisSettings) {
        self.settings = settings;
    }

    pub fn get_branch(&self, repo_key: &str) -> Option<Branch> {
        match self.branches.get(repo_key) {
            Some(cache) => Some(cache.clone()),
            None => None,
        }
    }

    pub fn get_branch_data(&self, repo_key: &str) -> Option<ArcCache> {
        debug!("Getting branch: {}", repo_key);
        debug!("Branches: {:?}", self.branches.keys());

        match self.branches.get(repo_key) {
            Some(branch) => Some(branch.cache.get_data().clone()),
            None => None,
        }
    }

    pub fn add_repo(&mut self, settings: BranchSettings) -> Result<(), GitdisError> {
        debug!("Adding new repo");

        let repo_key = settings.repo_key.clone();

        debug!("Repo key: {}", repo_key);

        if self.branches.contains_key(&repo_key) {
            debug!("Repo already exists");
            return Err(GitdisError::RepoExists);
        }

        let cache = CacheBranch::new(self.settings.total_branch_items, self.sender.clone());

        debug!("Added new repo: {}", repo_key);

        self.branches.insert(repo_key, Branch { settings, cache });

        Ok(())
    }

    pub fn listen_all_branches(&self) {
        for (repo_key, _) in self.branches.iter() {
            if let Err(err) = self.listen_branch(repo_key) {
                log::error!("Error listening branch: {:?}", err);
            }
        }
    }

    pub fn listen_branch(&self, repo_key: &str) -> Result<thread::JoinHandle<()>, GitdisError> {
        let branch: Branch = match self.get_branch(&repo_key) {
            Some(cache) => cache,
            None => {
                return Err(GitdisError::BranchNotFound);
            }
        };

        let mut handler = BranchHandler::new(
            self.settings.local_clone_path.clone(),
            branch.settings.url,
            branch.settings.branch_name,
            branch.cache.get_data().clone(),
            branch.settings.pull_request_interval_millis,
            branch.settings.path_target,
        );

        Ok(thread::spawn(move || {
            if let Err(e) = handler.listen() {
                log::error!("Error listening branch: {:?}", e);
            }
        }))
    }

    pub fn listen_events<Callback>(&self, callback: Callback)
    where
        Callback: Fn(Event) + Send + 'static,
    {
        for event in self.receiver.iter() {
            match event {
                Event::Insert(data) => {
                    debug!("Inserting data: {:?}", data);
                    callback(Event::Insert(data));
                }
                Event::Remove(data) => {
                    debug!("Removing data: {:?}", data);
                    callback(Event::Remove(data));
                }
                Event::Clear => {
                    debug!("Clearing data");
                    callback(Event::Clear);
                }
            }
        }
    }
}

#[macro_export]
macro_rules! listen_events {
    ($gitdis:ident, $callback:expr) => {
        std::thread::spawn(move || {
            $gitdis.listen_events($callback);
        });
    };
}

impl From<GitdisSettings> for Gitdis {
    fn from(settings: GitdisSettings) -> Self {
        let (sender, receiver) = mpsc::channel();

        Self::new(settings, sender, receiver)
    }
}
