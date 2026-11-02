mod github_api;

use github_api::GithubApiClient;

pub fn get_repo_contents_from_url(url: &str) -> Result<(), String> {
    let github_client = GithubApiClient::new()?;

    github_client.get_repository_content_from_url(url)?;

    return Ok(());
}

pub fn get_repo_contents(
    owner: &str,
    repo: &str,
    in_repo_path: &str,
    branch: Option<&str>,
) -> Result<(), String> {
    let github_client = GithubApiClient::new()?;

    github_client.get_repository_content(
        owner,
        repo,
        in_repo_path,
        branch,
    )?;

    return Ok(());
}
