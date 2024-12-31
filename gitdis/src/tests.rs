use std::sync::{mpsc, Arc, RwLock};

use branch_settings::BranchSettings;
use gitdis::{Gitdis, GitdisSettings};
use prelude::{GitdisService, ServiceAddBranchResponse};
use quickleaf::{
    prelude::{NumberBehavior, StringBehavior},
    Event,
};

use super::*;

const TEST_URL: &str = "https://github.com/lowcarboncode/gitdis-example-repository.git";

#[test]
fn test_branch_settings_get_branch_key() {
    let branch_settings = BranchSettings::new(TEST_URL.to_string(), "main".to_string(), 1000, None);

    let branch_key = branch_settings.key;
    assert_eq!(branch_key, "lowcarboncode/gitdis-example-repository/main");
}

#[test]
fn test_gitdis_add_repo() {
    let settings = GitdisSettings {
        total_branch_items: 100,
        local_clone_path: "data".to_string(),
    };

    let mut gitdis = Gitdis::from(settings);

    let branch_settings = BranchSettings::new(TEST_URL.to_string(), "main".to_string(), 1000, None);

    let result = gitdis.add_branch(branch_settings.clone());
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

    let branch_settings = BranchSettings::new(TEST_URL.to_string(), "main".to_string(), 1000, None);

    gitdis.add_branch(branch_settings.clone()).unwrap();

    gitdis.listen_branch(&branch_settings.key).unwrap();

    for event in gitdis.receiver.iter() {
        if let Event::Insert(data) = event {
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

    let branch_settings = BranchSettings::new(
        TEST_URL.to_string(),
        "main".to_string(),
        1000,
        Some("default/settings.yml".to_string()),
    );

    gitdis.add_branch(branch_settings.clone()).unwrap();

    gitdis.listen_branch(&branch_settings.key).unwrap();

    let mut count = 0;
    for event in gitdis.receiver.iter() {
        if let Event::Insert(_) = event {
            count += 1;
            if count == 1 {
                break;
            }
        }
    }

    match gitdis.get_branch_cache(&branch_settings.key) {
        Some(branch) => {
            let branch = branch.read().unwrap();
            let data = branch.get("default.settings").unwrap();

            assert!(data.is_object());

            let services = data.get("services").unwrap().as_object().unwrap();

            assert_eq!(services.get("foo").unwrap().as_bool(), Some(&true));
            assert_eq!(services.get("bar").unwrap().to_u64(), Some(123u64));
            assert_eq!(
                services.get("bax").unwrap().as_string(),
                "hello".to_string()
            );
        }
        None => panic!("Branch not found"),
    }
}

#[tokio::test]
async fn test_gitdis_json() {
    let settings = GitdisSettings {
        total_branch_items: 100,
        local_clone_path: "data".to_string(),
    };

    let (sender, receiver) = mpsc::channel();

    let mut gitdis = Gitdis::new(settings, sender, receiver);

    let branch_settings = BranchSettings::new(
        TEST_URL.to_string(),
        "main".to_string(),
        1000,
        Some("default/settings.json".to_string()),
    );

    gitdis.add_branch(branch_settings.clone()).unwrap();

    gitdis.listen_branch(&branch_settings.key).unwrap();

    let mut count = 0;
    for event in gitdis.receiver.iter() {
        if let Event::Insert(_) = event {
            count += 1;
            if count == 1 {
                break;
            }
        }
    }

    match gitdis.get_branch_cache(&branch_settings.key) {
        Some(branch) => {
            let branch = branch.read().unwrap();
            let data = branch.get("default.settings").unwrap();

            assert!(data.is_object());

            let services = data.get("services").unwrap().as_object().unwrap();

            assert_eq!(services.get("foo").unwrap().as_bool(), Some(&true));
            assert_eq!(services.get("bar").unwrap().to_u64(), Some(123u64));
            assert_eq!(
                services.get("bax").unwrap().as_string(),
                "hello".to_string()
            );
        }
        None => panic!("Branch not found"),
    }
}

#[tokio::test]
async fn test_gitdis_listen_events() {
    let settings = GitdisSettings {
        total_branch_items: 100,
        local_clone_path: "data".to_string(),
    };

    let (sender, receiver) = mpsc::channel();

    let mut gitdis = Gitdis::new(settings, sender, receiver);

    let branch_settings = BranchSettings::new(
        TEST_URL.to_string(),
        "main".to_string(),
        1000,
        Some("default/settings.json".to_string()),
    );

    gitdis.add_branch(branch_settings.clone()).unwrap();

    gitdis.listen_branch(&branch_settings.key).unwrap();

    let count = Arc::new(RwLock::new(0));

    let count_clone = count.clone();

    listen_events!(gitdis, move |event| {
        if let Event::Insert(_) = event {
            let mut count = count_clone.write().unwrap();
            *count += 1;
        }
    });

    while *count.read().unwrap() < 1 {
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    assert_eq!(*count.read().unwrap(), 1);
}

#[tokio::test]
async fn test_gitdis_services_add_branch() {
    let settings = GitdisSettings {
        total_branch_items: 100,
        local_clone_path: "data".to_string(),
    };

    let (sender, receiver) = mpsc::channel();

    let mut service = GitdisService::new(settings, sender, receiver);

    let branch_settings = BranchSettings::new(TEST_URL.to_string(), "main".to_string(), 1000, None);

    let result = service.add_branch(branch_settings.clone());

    assert_eq!(
        result.clone(),
        Ok(ServiceAddBranchResponse {
            url: branch_settings.url,
            branch_name: branch_settings.branch_name,
            key: branch_settings.key,
            pull_request_interval_millis: branch_settings.pull_request_interval_millis,
            path_target: branch_settings.path_target,
            create_at: result.unwrap().create_at,
        })
    );
}

#[tokio::test]
async fn test_gitdis_services_get_data() {
    let settings = GitdisSettings {
        total_branch_items: 100,
        local_clone_path: "data".to_string(),
    };

    let (sender, receiver) = mpsc::channel();

    let mut service = GitdisService::new(settings, sender, receiver);

    let branch_settings = BranchSettings::new(TEST_URL.to_string(), "main".to_string(), 1000, None);

    service.add_branch(branch_settings.clone()).unwrap();

    service.listen_branch(&branch_settings.key).unwrap();

    service
        .gitdis
        .await_branch_unsafe(&branch_settings.key, 1)
        .await
        .unwrap();

    let result = service.get_data(&branch_settings.key, "service.context.data");

    println!("{:#?}", result);
}
