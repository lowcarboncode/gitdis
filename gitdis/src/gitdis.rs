use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::{
    collections::HashMap,
    sync::{
        mpsc::{self, SendError},
        Arc, RwLock,
    },
};

use branch_handler::BranchHandler;
use log::debug;
use quickleaf::{Cache, Event};

use crate::cache::ArcCache;

use super::branch_handler;

#[derive(Debug, PartialEq)]
pub enum GitdisError {
    RepoExists,
    Sender(SendError<BranchSettings>),
    BranchNotFound,
    RepoListener,
}

#[derive(Clone, Debug, PartialEq)]
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
pub struct CacheBranch {
    cache: ArcCache,
    create_at: u128,
}

impl CacheBranch {
    pub fn new(total_cache_items: usize, sender: Sender<Event>) -> Self {
        let create_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        debug!("Creating new cache with {} items", total_cache_items);

        CacheBranch {
            cache: Arc::new(RwLock::new(Cache::with_sender(total_cache_items, sender))),
            create_at,
        }
    }

    pub fn get_data(&self) -> ArcCache {
        self.cache.clone()
    }

    pub fn get_create_at(&self) -> u128 {
        self.create_at
    }
}

pub struct Gitdis {
    pub settings: GitdisSettings,
    branches: HashMap<String, CacheBranch>,
    sender: Sender<Event>,
    pub receiver: Receiver<Event>,
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

    pub fn get_object_branch(&self, repo_key: &str) -> Option<CacheBranch> {
        match self.branches.get(repo_key) {
            Some(cache) => Some(cache.clone()),
            None => None,
        }
    }

    pub fn get_data_branch(&self, repo_key: &str) -> Option<ArcCache> {
        debug!("Getting branch: {}", repo_key);
        debug!("Branches: {:?}", self.branches.keys());

        match self.branches.get(repo_key) {
            Some(cache) => Some(cache.get_data()),
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
            CacheBranch::new(self.settings.total_branch_items, self.sender.clone()),
        );

        debug!("Added new repo: {}", key);

        Ok(())
    }

    pub fn create_branch_handler(
        &self,
        settings: BranchSettings,
    ) -> Result<BranchHandler, GitdisError> {
        let cache = match self.get_data_branch(&settings.get_repo_key()) {
            Some(cache) => cache,
            None => {
                return Err(GitdisError::BranchNotFound);
            }
        };

        Ok(BranchHandler::new(
            self.settings.local_clone_path.clone(),
            settings.url,
            settings.branch_name,
            cache,
            settings.pull_request_interval_millis,
        ))
    }

    pub fn repo_listen(
        &self,
        settings: BranchSettings,
    ) -> Result<thread::JoinHandle<()>, GitdisError> {
        let mut handler = self.create_branch_handler(settings)?;

        Ok(thread::spawn(move || {
            if let Err(e) = handler.listen() {
                eprintln!("Error: {:?}", e);
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

impl From<GitdisSettings> for Gitdis {
    fn from(settings: GitdisSettings) -> Self {
        let (sender, receiver) = mpsc::channel();

        Self::new(settings, sender, receiver)
    }
}
