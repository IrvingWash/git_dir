const DESTINATION: &str = "/home/irving_wash/Downloads";

fn main() -> Result<(), String> {
    let result = git_dir::download_folder_from_github_repo_url(
        "https://github.com/KhronosGroup/glTF-Sample-Assets/tree/main/Models/Fox",
        DESTINATION,
    );

    match result {
        Err(e) => return Err(format!("Failed: {}", e)),
        Ok(r) => {
            println!(
                "Successfully downloaded to {:?}/{}.zip",
                r.destination, r.archive_name,
            );

            if r.warnings.len() > 0 {
                println!("But got {} warnings:", r.warnings.len());

                for warning in r.warnings {
                    println!("{}", warning);
                }
            }
        }
    }
    return Ok(());
}
