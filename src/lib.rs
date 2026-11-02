mod github_api;

pub fn get_repo_contents_from_url(url: &str) -> Result<(), String> {
    github_api::get_repository_content_from_url(url)?;

    return Ok(());
}
