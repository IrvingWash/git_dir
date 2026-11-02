mod github_api;

use std::fs;

// RepositoryContentDirectory {
//     size: 0,
//     name: "glTF-Binary",
//     path: "Models/Fox/glTF-Binary",
//     content: None,
//     sha: "bd109c78e710180d6b4e509bab9138ddc821d224",
//     url: "https://api.github.com/repos/KhronosGroup/glTF-Sample-Assets/contents/Models/Fox/glTF-Binary?ref=main",
//     git_url: "https://api.github.com/repos/KhronosGroup/glTF-Sample-Assets/git/trees/bd109c78e710180d6b4e509bab9138ddc821d224",
//     html_url: "https://github.com/KhronosGroup/glTF-Sample-Assets/tree/main/Models/Fox/glTF-Binary",
//     download_url: None,
//     _links: RepositoryContentLinks {
//         git: Some(
//             "https://api.github.com/repos/KhronosGroup/glTF-Sample-Assets/git/trees/bd109c78e710180d6b4e509bab9138ddc821d224",
//         ),
//         html: Some(
//             "https://github.com/KhronosGroup/glTF-Sample-Assets/tree/main/Models/Fox/glTF-Binary",
//         ),
//         _self: "https://api.github.com/repos/KhronosGroup/glTF-Sample-Assets/contents/Models/Fox/glTF-Binary?ref=main",
//     },
// }
// RepositoryContentDirectory {
//     size: 0,
//     name: "glTF",
//     path: "Models/Fox/glTF",
//     content: None,
//     sha: "fdca4052bab994c22d14a0d443cf20ee3cd85557",
//     url: "https://api.github.com/repos/KhronosGroup/glTF-Sample-Assets/contents/Models/Fox/glTF?ref=main",
//     git_url: "https://api.github.com/repos/KhronosGroup/glTF-Sample-Assets/git/trees/fdca4052bab994c22d14a0d443cf20ee3cd85557",
//     html_url: "https://github.com/KhronosGroup/glTF-Sample-Assets/tree/main/Models/Fox/glTF",
//     download_url: None,
//     _links: RepositoryContentLinks {
//         git: Some(
//             "https://api.github.com/repos/KhronosGroup/glTF-Sample-Assets/git/trees/fdca4052bab994c22d14a0d443cf20ee3cd85557",
//         ),
//         html: Some(
//             "https://github.com/KhronosGroup/glTF-Sample-Assets/tree/main/Models/Fox/glTF",
//         ),
//         _self: "https://api.github.com/repos/KhronosGroup/glTF-Sample-Assets/contents/Models/Fox/glTF?ref=main",
//     },
// }
// RepositoryContentDirectory {
//     size: 0,
//     name: "screenshot",
//     path: "Models/Fox/screenshot",
//     content: None,
//     sha: "7b039bc65d7b62cdf53b88ebac93fae3b24424e0",
//     url: "https://api.github.com/repos/KhronosGroup/glTF-Sample-Assets/contents/Models/Fox/screenshot?ref=main",
//     git_url: "https://api.github.com/repos/KhronosGroup/glTF-Sample-Assets/git/trees/7b039bc65d7b62cdf53b88ebac93fae3b24424e0",
//     html_url: "https://github.com/KhronosGroup/glTF-Sample-Assets/tree/main/Models/Fox/screenshot",
//     download_url: None,
//     _links: RepositoryContentLinks {
//         git: Some(
//             "https://api.github.com/repos/KhronosGroup/glTF-Sample-Assets/git/trees/7b039bc65d7b62cdf53b88ebac93fae3b24424e0",
//         ),
//         html: Some(
//             "https://github.com/KhronosGroup/glTF-Sample-Assets/tree/main/Models/Fox/screenshot",
//         ),
//         _self: "https://api.github.com/repos/KhronosGroup/glTF-Sample-Assets/contents/Models/Fox/screenshot?ref=main",
//     },
// }

#[derive(Debug, Default)]
pub struct DownloadFolderResult {
    pub warnings: Vec<String>,
    pub destination: String,
    pub archive_name: String,
}

pub fn download_folder_from_github_repo_url(url: &str, dst: &str) -> Result<DownloadFolderResult, String> {
    let github_client = github_api::ApiClient::new()?;

    let repo_content = github_client.get_repository_content_from_url(url)?;

    let project_dirs = directories::ProjectDirs::from("", "irving_wash", "git_dir")
        .ok_or("Failed to get OS info on your home directory")?;

    if !project_dirs.cache_dir().exists() {
        fs::create_dir(project_dirs.cache_dir())
            .map_err(|err| format!(
                "Failed to create cache dir ({:?}): {}",
                project_dirs.cache_dir(),
                err.to_string(),
            ))?;
    }

    let output_name = format!(
        "{}-{}-{}",
        repo_content.owner,
        repo_content.repo,
        repo_content.in_repo_path,
    ).replace("/", "_").replace("\\", "_");

    let mut output_path = project_dirs.cache_dir().to_path_buf();
    output_path.push(&output_name);

    fs::create_dir(&output_path).map_err(
        |e| format!(
            "Failed to create cache output subdir ({:?}): {}",
            output_path,
            e.to_string(),
        ),
    )?;

    let mut result = DownloadFolderResult {
        warnings: Vec::new(),
        destination: dst.to_owned(),
        archive_name: format!("{}.zip", output_name),
    };

    for content in repo_content.contents {
        use github_api::objects::RepositoryContentKind;
        match content {
            RepositoryContentKind::Symlink(symlink) => {
                result
                    .warnings
                    .push(format!("Skipping symlink: {}", symlink.name));
            }
            RepositoryContentKind::Submodule(submodule) => {
                result
                    .warnings
                    .push(format!("Skipping submodule: {}", submodule.name));
            }
            RepositoryContentKind::Directory(dir) => {
                println!("{:#?}", dir);
            }
            RepositoryContentKind::File(file) => {
                match file.download_url {
                    None => result.warnings.push(format!("File {} doesn't have download url", file.path)),
                    Some(download_url) => {
                        let download_result = reqwest::blocking::get(download_url);
                        match download_result {
                            Err(e) => result.warnings.push(format!("Failed to download file {}: {}", file.path, e.to_string())),
                            Ok(dr) => {
                                let mut file_path = output_path.to_path_buf();
                                file_path.push(file.name.clone());
                                fs::write(file_path, dr.text().map_err(
                                    |e| format!("Contents of file {} seem to be corrupted {}", file.name, e),
                                )?).map_err(|e| format!("Failed to create file {}: {}", file.name, e))?;
                            },
                        }
                    }
                }
            }
        }
    }

    fs::remove_dir_all(&output_path).map_err(
        |err| format!(
            "Failed to delete cache subdir {:?}: {}",
            output_path,
            err.to_string(),
        )
    )?;

    return Ok(result);
}
