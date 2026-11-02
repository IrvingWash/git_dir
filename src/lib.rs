mod github_api;

use std::{
    fs::{self, File},
    io,
    path::{Path, PathBuf},
};

use walkdir::WalkDir;
use zip::write::ZipWriter;

use crate::github_api::objects::RepositoryContentKind;

#[derive(Debug, Default)]
pub struct DownloadFolderResult {
    pub warnings: Vec<String>,
    pub destination: PathBuf,
    pub archive_name: String,
}

pub fn download_folder_from_github_repo_url(
    url: &str,
    dst: &PathBuf,
) -> Result<DownloadFolderResult, String> {
    let github_client = github_api::ApiClient::new()?;

    let repo_content = github_client.get_repository_content_from_url(url)?;

    let project_dirs = directories::ProjectDirs::from("", "irving_wash", "git_dir")
        .ok_or("Failed to get OS info on your home directory")?;

    if !project_dirs.cache_dir().exists() {
        fs::create_dir(project_dirs.cache_dir()).map_err(|err| {
            format!(
                "Failed to create cache dir ({:?}): {}",
                project_dirs.cache_dir(),
                err.to_string(),
            )
        })?;
    }

    let output_name = format!(
        "{}-{}-{}",
        repo_content.owner, repo_content.repo, repo_content.in_repo_path,
    )
    .replace("/", "_")
    .replace("\\", "_");

    let mut root_output_path = project_dirs.cache_dir().to_path_buf();
    root_output_path.push(&output_name);

    let mut result = DownloadFolderResult {
        warnings: Vec::new(),
        destination: PathBuf::from(dst),
        archive_name: format!("{}.zip", output_name),
    };

    download_contents_recursive(
        &github_client,
        repo_content.contents,
        &root_output_path,
        &mut result.warnings,
    )?;

    let mut zip_file_path = result.destination.to_path_buf();
    zip_file_path.push(&result.archive_name);
    zip_it(zip_file_path, &root_output_path)?;

    fs::remove_dir_all(&root_output_path).map_err(|err| {
        format!(
            "Failed to delete cache subdir {:?}: {}",
            root_output_path,
            err.to_string(),
        )
    })?;

    return Ok(result);
}

fn download_contents_recursive(
    github_api_client: &github_api::ApiClient,
    contents: Vec<RepositoryContentKind>,
    output_dir: &Path,
    warnings: &mut Vec<String>,
) -> Result<(), String> {
    fs::create_dir(output_dir).map_err(|e| format!("Failed to create {output_dir:?}: {e}"))?;

    for content in contents {
        match content {
            RepositoryContentKind::Directory(dir) => {
                let sub_content =
                    github_api_client.get_repository_content_from_url(&dir.html_url)?;

                let nested_output_dir = output_dir.join(&dir.name);

                download_contents_recursive(
                    github_api_client,
                    sub_content.contents,
                    &nested_output_dir,
                    warnings,
                )?;
            }
            RepositoryContentKind::File(file) => {
                let Some(download_url) = file.download_url else {
                    warnings.push(format!("File {} doesn't have a download URL", file.path));
                    continue;
                };

                let mut response = match reqwest::blocking::get(&download_url)
                    .and_then(|response| response.error_for_status())
                {
                    Ok(response) => response,
                    Err(e) => {
                        warnings.push(format!("Failed to download {}: {}", file.path, e));
                        continue;
                    }
                };

                let file_path = output_dir.join(&file.name);

                let mut output = File::create(&file_path)
                    .map_err(|e| format!("Failed to create {file_path:?}: {e}"))?;

                io::copy(&mut response, &mut output)
                    .map_err(|e| format!("Failed to write {file_path:?}: {e}"))?;
            }

            RepositoryContentKind::Symlink(symlink) => {
                warnings.push(format!("Skipping symlink: {}", symlink.name));
            }
            RepositoryContentKind::Submodule(submodule) => {
                warnings.push(format!("Skipping submodule: {}", submodule.name));
            }
        }
    }

    Ok(())
}

fn zip_it(destination: PathBuf, src_dir_path: &PathBuf) -> Result<(), String> {
    let zip_file = File::create(&destination)
        .map_err(|e| format!("Failed to create zip file ({:?}): {}", destination, e))?;
    let mut zip_writer = ZipWriter::new(zip_file);
    let zip_options = zip::write::SimpleFileOptions::default();

    let walk_dir = WalkDir::new(src_dir_path);
    for entry_result in walk_dir.into_iter() {
        let entry = match entry_result {
            Ok(entry) => entry,
            Err(e) => {
                return Err(
                    format!("Error while traversing directory {src_dir_path:?}: {e}").into(),
                );
            }
        };
        let path = entry.path();
        let path_stripped = path
            .strip_prefix(&src_dir_path)
            .map_err(|e| e.to_string())?;
        let path_as_string = path_stripped
            .to_str()
            .map(str::to_owned)
            .ok_or_else(|| format!("{:?} is a Non UTF-8 Path", path_stripped.display()))?;

        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            println!(
                "adding file {:?} as {:?} ...",
                path.display(),
                path_stripped.display()
            );
            zip_writer
                .start_file(path_as_string, zip_options)
                .map_err(|e| e.to_string())?;
            let mut f = File::open(path).map_err(|e| e.to_string())?;

            std::io::copy(&mut f, &mut zip_writer).map_err(|e| e.to_string())?;
        } else if !path_stripped.as_os_str().is_empty() {
            // Only if not root! Avoids path spec / warning
            // and mapname conversion failed error on unzip
            println!(
                "adding dir '{}' as '{}' ...",
                path.display(),
                path_stripped.display()
            );
            zip_writer
                .add_directory(path_as_string, zip_options)
                .map_err(|e| e.to_string())?;
        }
    }

    zip_writer
        .finish()
        .map_err(|e| format!("Filed to create zip file: {}", e))?;

    return Ok(());
}
