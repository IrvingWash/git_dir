use serde::Deserialize;

pub type RepositoryContent = Vec<RepositoryContentKind>;

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
#[allow(dead_code)]
pub enum RepositoryContentKind {
    #[serde(rename = "dir")]
    Directory(RepositoryContentDirectory),
    #[serde(rename = "file")]
    File(RepositoryContentFile),
    #[serde(rename = "symlink")]
    Symlink(RepositoryContentSymlink),
    #[serde(rename = "submodule")]
    Submodule(RepositoryContentSubmodule),
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct RepositoryContentDirectory {
    pub size: usize,
    pub name: String,
    pub path: String,
    pub content: Option<String>,
    pub sha: String,
    pub url: String,
    pub git_url: String,
    pub html_url: String,
    pub download_url: Option<String>,
    pub _links: RepositoryContentLinks,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct RepositoryContentFile {
    pub encoding: Option<String>,
    pub size: usize,
    pub name: String,
    pub path: String,
    pub content: Option<String>,
    pub sha: String,
    pub url: String,
    pub git_url: String,
    pub html_url: String,
    pub download_url: Option<String>,
    pub _links: RepositoryContentLinks,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct RepositoryContentSymlink {
    pub target: String,
    pub size: usize,
    pub name: String,
    pub path: String,
    pub sha: String,
    pub git_url: String,
    pub html_url: String,
    pub download_url: Option<String>,
    pub _links: RepositoryContentLinks,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct RepositoryContentSubmodule {
    pub submodule_git_url: String,
    pub size: usize,
    pub name: String,
    pub path: String,
    pub sha: String,
    pub url: String,
    pub git_url: Option<String>,
    pub html_url: Option<String>,
    pub download_url: Option<String>,
    pub _links: RepositoryContentLinks,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct RepositoryContentLinks {
    pub git: Option<String>,
    pub html: Option<String>,
    #[serde(rename = "self")]
    pub _self: String,
}
