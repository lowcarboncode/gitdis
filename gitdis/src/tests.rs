use std::{fs, sync::mpsc};

use branch_settings::BranchSettings;
use gitdis::{Gitdis, GitdisSettings};
use quickleaf::Event;

use super::*;

const TEST_URL: &str = "https://github.com/lowcarboncode/gitdis-example-repository.git";

#[test]
fn test_branch_settings_get_repo_key() {
    let branch_settings = BranchSettings::new(TEST_URL.to_string(), "main".to_string(), 1000);

    let repo_key = branch_settings.repo_key;
    assert_eq!(repo_key, "lowcarboncode/gitdis-example-repository/main");
}

#[test]
fn test_gitdis_add_repo() {
    let settings = GitdisSettings {
        total_branch_items: 100,
        local_clone_path: "data".to_string(),
    };

    let mut gitdis = Gitdis::from(settings);

    let branch_settings = BranchSettings::new(TEST_URL.to_string(), "main".to_string(), 1000);

    let result = gitdis.add_repo(branch_settings.clone());
    assert_eq!(result, Ok(()));
}

#[tokio::test]
async fn test_gitdis_spawn_branch_listener() {
    let settings = GitdisSettings {
        total_branch_items: 100,
        local_clone_path: "data".to_string(),
    };

    let (sender, receiver) = mpsc::channel();

    let mut gitdis = Gitdis::new(settings, sender, receiver);

    let branch_settings = BranchSettings::new(TEST_URL.to_string(), "main".to_string(), 1000);

    gitdis.add_repo(branch_settings.clone()).unwrap();

    gitdis
        .listen_branch(&branch_settings.repo_key, None)
        .unwrap();

    for event in gitdis.receiver.iter() {
        if let Event::Insert(data) = event {
            // fs::remove_dir_all("data").unwrap();
            assert!(data.value.is_object());
            break;
        }
    }
}

#[tokio::test]
async fn test_gitdis_yml() {
    let settings = GitdisSettings {
        total_branch_items: 100,
        local_clone_path: "data".to_string(),
    };

    let (sender, receiver) = mpsc::channel();

    let mut gitdis = Gitdis::new(settings, sender, receiver);

    let branch_settings = BranchSettings::new(TEST_URL.to_string(), "main".to_string(), 1000);

    gitdis.add_repo(branch_settings.clone()).unwrap();

    gitdis
        .listen_branch(&branch_settings.repo_key, Some("application1/settings.yml"))
        .unwrap();

    let mut count = 0;
    for event in gitdis.receiver.iter() {
        if let Event::Insert(_) = event {
            count += 1;
            if count == 1 {
                break;
            }
        }
    }

    match gitdis.get_branch_data(&branch_settings.repo_key) {
        Some(branch) => {
            let branch = branch.read().unwrap();
            let data = branch.get("application1/settings").unwrap();

            assert!(data.is_object());
            assert_eq!(data.get("foo").unwrap().as_bool(), Some(&true));
            assert_eq!(data.get("bar").unwrap().as_bool(), Some(&false));
        }
        None => panic!("Branch not found"),
    }
}
