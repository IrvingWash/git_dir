fn main() {
    let err = git_dir::get_repo_contents_from_url("https://github.com/KhronosGroup/glTF-Sample-Assets/tree/main/Models/Fox");

    if err.is_err() {
        print!("{}", err.unwrap_err());
    }
}
