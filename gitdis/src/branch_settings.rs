#[derive(Clone, Debug, PartialEq)]
pub struct BranchSettings {
    pub url: String,
    pub branch_name: String,
    pub repo_key: String,
    pub pull_request_interval_millis: u64,
}

impl BranchSettings {
    pub fn new(url: String, branch_name: String, pull_request_interval_millis: u64) -> Self {
        let repo_key = Self::crete_repo_key(&url, &branch_name);

        Self {
            url,
            branch_name,
            repo_key,
            pull_request_interval_millis,
        }
    }

    fn crete_repo_key(url: &str, branch_name: &str) -> String {
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

        format!("{}/{}/{}", repo_owner, repo_name, branch_name)
    }
}
