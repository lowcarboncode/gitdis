use std::{fs, sync::mpsc, thread};

use gitdis::{BranchSettings, Gitdis, GitdisSettings};
use quickleaf::Event;

use super::*;

const TEST_URL: &str = "https://github.com/lowcarboncode/gitdis-example-repository.git";

#[test]
fn test_branch_settings_get_repo_key() {
    let settings = BranchSettings {
        url: TEST_URL.to_string(),
        branch_name: "main".to_string(),
        pull_request_interval_millis: 1000,
    };

    let repo_key = settings.get_repo_key();
    assert_eq!(repo_key, "lowcarboncode/gitdis-example-repository/main");
}

#[test]
fn test_gitdis_add_repo() {
    let settings = GitdisSettings {
        total_branch_items: 100,
        local_clone_path: "data".to_string(),
    };

    let mut gitdis = Gitdis::from(settings);

    let settings = BranchSettings {
        url: TEST_URL.to_string(),
        branch_name: "main".to_string(),
        pull_request_interval_millis: 1000,
    };

    let result = gitdis.add_repo(settings.clone());
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

    gitdis
        .add_repo(BranchSettings {
            url: TEST_URL.to_string(),
            branch_name: "main".to_string(),
            pull_request_interval_millis: 1000,
        })
        .unwrap();

    let mut handler = gitdis
        .create_branch_handler(BranchSettings {
            url: TEST_URL.to_string(),
            branch_name: "main".to_string(),
            pull_request_interval_millis: 1000,
        })
        .unwrap();

    thread::spawn(move || {
        if let Err(e) = handler.listen() {
            eprintln!("Error: {:?}", e);
        }
    });

    for event in gitdis.receiver.iter() {
        if let Event::Insert(data) = event {
            fs::remove_dir_all("data").unwrap();
            println!("Data: {:?}", data);
            assert!(data.value.is_object());
            break;
        }
    }
}
