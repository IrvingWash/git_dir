fn main() -> Result<(), String> {
    let result = git_dir::download_folder_from_repo_url(
        "https://github.com/KhronosGroup/glTF-Sample-Assets/tree/main/Models/Fox",
    );

    if result.is_err() {
        return Err(format!("Failed: {}", result.unwrap_err()));
    }

    return Ok(());
}
