#[derive(Clone, Debug, PartialEq)]
pub struct BranchSettings {
    pub url: String,
    pub branch_name: String,
    pub key: String,
    pub pull_request_interval_millis: u64,
    pub path_target: Option<String>,
}

impl BranchSettings {
    pub fn new(
        url: String,
        branch_name: String,
        pull_request_interval_millis: u64,
        path_target: Option<String>,
    ) -> Self {
        let key = Self::crete_branch_key(&url, &branch_name);

        Self {
            url,
            branch_name,
            key,
            pull_request_interval_millis,
            path_target,
        }
    }

    fn crete_branch_key(url: &str, branch_name: &str) -> String {
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
