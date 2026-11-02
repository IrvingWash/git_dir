mod github_api;

#[derive(Debug, Default)]
pub struct DownloadFolderResult {
    warnings: Vec<String>,
}

pub fn download_folder_from_repo_url(url: &str) -> Result<DownloadFolderResult, String> {
    let mut result = DownloadFolderResult::default();

    let github_client = github_api::ApiClient::new()?;

    let repo_content = github_client.get_repository_content_from_url(url)?;

    for content in repo_content {
        use github_api::objects::RepositoryContentKind;
        match content {
            RepositoryContentKind::Symlink(content) => {
                result
                    .warnings
                    .push(format!("Skipping symlink: {}", content.name));
            }
            RepositoryContentKind::Submodule(submodule) => {
                result
                    .warnings
                    .push(format!("Skipping submodule: {}", submodule.name));
            }
            RepositoryContentKind::Directory(_) => {}
            RepositoryContentKind::File(_) => {}
        }
    }

    return Ok(result);
}
