mod github_api;

use std::{
    fs::{self, File},
    path::PathBuf,
};

use walkdir::WalkDir;
use zip::write::ZipWriter;

#[derive(Debug, Default)]
pub struct DownloadFolderResult {
    pub warnings: Vec<String>,
    pub destination: PathBuf,
    pub archive_name: String,
}

pub fn download_folder_from_github_repo_url(
    url: &str,
    dst: &str,
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

    fs::create_dir(&root_output_path).map_err(|e| {
        format!(
            "Failed to create cache output subdir ({:?}): {}",
            root_output_path,
            e.to_string(),
        )
    })?;

    let mut result = DownloadFolderResult {
        warnings: Vec::new(),
        destination: PathBuf::from(dst),
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
            RepositoryContentKind::File(file) => match file.download_url {
                None => result
                    .warnings
                    .push(format!("File {} doesn't have download url", file.path)),
                Some(download_url) => {
                    let download_result = reqwest::blocking::get(download_url);
                    match download_result {
                        Err(e) => result.warnings.push(format!(
                            "Failed to download file {}: {}",
                            file.path,
                            e.to_string()
                        )),
                        Ok(dr) => {
                            let mut file_path = root_output_path.to_path_buf();
                            file_path.push(file.name.clone());
                            match dr.text() {
                                Err(e) => result.warnings.push(format!(
                                    "Downloaded corrupted file {}: {}",
                                    file.name, e
                                )),
                                Ok(text) => fs::write(file_path, text).map_err(|e| {
                                    format!("Failed to create file {}: {}", file.name, e,)
                                })?,
                            }
                        }
                    }
                }
            },
        }
    }

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
