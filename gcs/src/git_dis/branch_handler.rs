use log::debug;
use memotree::valu3::prelude::*;
use std::{
    collections::HashMap,
    process::Command,
    sync::{Arc, RwLock},
};

use memotree::branch::Branch;

const EXT_JSON: &str = ".json";
const EXT_YML: &str = ".yml";
const EXT_YAML: &str = ".yaml";

pub enum Error {
    GitError((Option<i32>, String)),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::GitError((code, error)) => {
                write!(f, "Git error: code: {:?}, error: {}", code, error)
            }
        }
    }
}

enum Status {
    Added,
    Modified,
    Deleted,
    Moved,
    Copied,
}

impl std::fmt::Debug for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Status::Added => write!(f, "Added"),
            Status::Modified => write!(f, "Modified"),
            Status::Deleted => write!(f, "Deleted"),
            Status::Moved => write!(f, "Moved"),
            Status::Copied => write!(f, "Copied"),
        }
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Status::Added => write!(f, "Added"),
            Status::Modified => write!(f, "Modified"),
            Status::Deleted => write!(f, "Deleted"),
            Status::Moved => write!(f, "Moved"),
            Status::Copied => write!(f, "Copied"),
        }
    }
}

pub type ArcBranch = Arc<RwLock<Branch>>;

pub struct BranchHandler {
    clone_path: String,
    url: String,
    branch_name: String,
    branch: ArcBranch,
    ignore: Vec<String>,
    repo_path: String,
    current_commit_hash: String,
    pull_request_interval_millis: u64,
}

impl BranchHandler {
    pub fn new(
        data_path: String,
        url: String,
        branch_name: String,
        branch: ArcBranch,
        pull_request_interval_millis: u64,
    ) -> Self {
        let repo_name = url.split("/").last().unwrap().replace(".git", "");
        let repo_path = format!("{}/{}", data_path, repo_name);

        Self {
            clone_path: data_path,
            url,
            branch_name,
            branch,
            ignore: vec!["/.git/".to_string()],
            repo_path,
            current_commit_hash: "".to_string(),
            pull_request_interval_millis,
        }
    }

    /// Get the data from the repository instantly
    pub fn clone_and_get_data(&self) -> Result<HashMap<String, Value>, Error> {
        if !std::path::Path::new(&self.clone_path).exists() {
            std::fs::create_dir(&self.clone_path).expect("Failed to create repo directory");
        }

        self.clone()?;
        self.get_initial_data()
    }

    /// Start the listener to listen for changes in the repository
    pub fn listener(&mut self) -> Result<(), Error> {
        self.setup()?;

        loop {
            std::thread::sleep(std::time::Duration::from_millis(
                self.pull_request_interval_millis,
            ));
            self.update()?;
        }
    }

    fn setup(&mut self) -> Result<(), Error> {
        if !std::path::Path::new(&self.clone_path).exists() {
            std::fs::create_dir(&self.clone_path).expect("Failed to create repo directory");
        }

        self.clone()?;
        self.load_initial_data()?;
        self.current_commit_hash = self.get_commit_hash()?;

        debug!("Initial commit hash: {}", self.current_commit_hash);

        Ok(())
    }

    fn update(&mut self) -> Result<(), Error> {
        self.pull()?;

        let current_commit_hash = self.get_commit_hash()?;

        debug!("Current commit hash: {}", current_commit_hash);

        if self.current_commit_hash == current_commit_hash {
            debug!("No changes");
            return Ok(());
        }

        debug!("Changes detected");

        self.current_commit_hash = current_commit_hash;

        let output = self.diff_stat()?;

        debug!("Diff stat: {}", output);

        let mut chars = output.split('\0');

        while let Some(char) = chars.next() {
            if char.is_empty() {
                break;
            }

            let status = match char {
                "A" => Status::Added,
                "D" => Status::Deleted,
                _ => {
                    if char.starts_with("R") {
                        Status::Moved
                    } else if char.starts_with("C") {
                        Status::Copied
                    } else if char.starts_with("M") {
                        Status::Modified
                    } else {
                        continue;
                    }
                }
            };

            match chars.next() {
                Some(file) => {
                    let file = format!("{}/{}", self.repo_path, file);

                    if self.is_ignore(&file) || !self.is_valid_file(&file) {
                        continue;
                    }

                    debug!("File: {}, Status: {}", file, status);

                    match status {
                        Status::Added | Status::Modified | Status::Copied => {
                            let content = self.get_file_content(&file);
                            let value = match Value::payload_to_value(&content) {
                                Ok(value) => value,
                                Err(_) => Value::Undefined,
                            };

                            match self.branch.write() {
                                Ok(mut branch) => branch.insert(self.fix_key(&file), value),
                                Err(_) => continue,
                            };
                        }
                        Status::Deleted => {
                            match self.branch.write() {
                                Ok(mut branch) => match branch.remove(&self.fix_key(&file)) {
                                    Ok(_) => (),
                                    Err(_) => (),
                                },
                                Err(_) => continue,
                            };
                        }
                        Status::Moved => match chars.next() {
                            Some(new_file) => {
                                let new_file = format!("{}/{}", self.repo_path, new_file);
                                let content = self.get_file_content(&new_file);
                                let value = match Value::payload_to_value(&content) {
                                    Ok(value) => value,
                                    Err(_) => Value::Undefined,
                                };

                                match self.branch.write() {
                                    Ok(mut branch) => {
                                        branch.insert(self.fix_key(&new_file), value);
                                        branch.remove(&self.fix_key(&file)).unwrap();
                                    }
                                    Err(_) => continue,
                                };
                            }
                            None => break,
                        },
                    }
                }
                None => break,
            }
        }

        Ok(())
    }

    fn fix_key(&self, key: &str) -> String {
        key.replace(&format!("{}/", &self.repo_path), "").split(".").next().unwrap().to_string()
    }

    fn get_initial_data(&self) -> Result<HashMap<String, Value>, Error> {
        let files = self.list_all_files(&self.clone_path);
        let mut data = HashMap::new();

        for file in files {
            let content = self.get_file_content(&file);
            let value = match Value::payload_to_value(&content) {
                Ok(value) => value,
                Err(_) => Value::Undefined,
            };

            data.insert(self.fix_key(&file), value);
        }

        Ok(data)
    }

    fn load_initial_data(&mut self) -> Result<(), Error> {
        let data = self.get_initial_data()?;

        match self.branch.write() {
            Ok(mut branch) => {
                for (key, value) in data {
                    branch.insert(self.fix_key(&key), value);
                }
            }
            Err(_) => (),
        }

        Ok(())
    }

    fn get_file_content(&self, path: &str) -> String {
        debug!("Reading file: {}", path);
        std::fs::read_to_string(path).unwrap()
    }

    fn is_valid_file(&self, path: &str) -> bool {
        path.ends_with(EXT_JSON) || path.ends_with(EXT_YML) || path.ends_with(EXT_YAML)
    }

    fn is_ignore(&self, key: &str) -> bool {
        for ignore in &self.ignore {
            if key.contains(ignore) {
                return true;
            }
        }

        false
    }

    fn list_all_files(&self, path: &str) -> Vec<String> {
        let mut files = Vec::new();

        for entry in std::fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let path_str = path.to_str().unwrap();

            if self.is_ignore(path_str) {
                continue;
            }

            if path.is_dir() {
                files.append(&mut self.list_all_files(path_str));
            } else if self.is_valid_file(path_str) {
                let full_path = path_str.to_string();

                files.push(full_path);
            }
        }

        files
    }

    fn clone(&self) -> Result<(), Error> {
        debug!("Cloning repository");

        if std::path::Path::new(&self.repo_path).exists() {
            return self.pull();
        }

        let output = Command::new("git")
            .arg("clone")
            .arg("--branch")
            .arg(&self.branch_name)
            .arg(&self.url)
            .current_dir(&self.clone_path)
            .output()
            .expect("Failed to execute git clone");

        if !output.status.success() {
            let code = output.status.code();
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(Error::GitError((code, error.to_string())));
        }

        Ok(())
    }

    fn pull(&self) -> Result<(), Error> {
        debug!("Pulling changes");

        let output = Command::new("git")
            .arg("pull")
            .current_dir(&self.repo_path)
            .output()
            .expect("Failed to execute git pull");

        if !output.status.success() {
            let code = output.status.code();
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(Error::GitError((code, error.to_string())));
        }

        Ok(())
    }

    fn diff_stat(&mut self) -> Result<String, Error> {
        debug!("Getting diff stat");

        let output = Command::new("git")
            .arg("diff")
            .arg("-z")
            .arg("--name-status")
            .arg("HEAD^")
            .arg("HEAD")
            .current_dir(&self.repo_path)
            .output()
            .expect("Failed to execute git diff --stat");

        if !output.status.success() {
            let code = output.status.code();
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(Error::GitError((code, error.to_string())));
        }

        let output = String::from_utf8_lossy(&output.stdout).to_string();

        Ok(output)
    }

    fn get_commit_hash(&mut self) -> Result<String, Error> {
        let output = Command::new("git")
            .arg("rev-parse")
            .arg("HEAD")
            .current_dir(&self.repo_path)
            .output()
            .expect("Failed to execute git rev-parse HEAD");

        if !output.status.success() {
            let code = output.status.code();
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(Error::GitError((code, error.to_string())));
        }

        let output = String::from_utf8_lossy(&output.stdout).to_string();

        Ok(output)
    }
}
