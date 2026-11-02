use url::Url;

const GITHUB_DOMAIN: &str = "github.com";
const BASE_URL: &str = "https://api.github.com";

struct GithubDeconstructedUrl {
    owner: Option<String>,
    repo: Option<String>,
    branch: Option<String>,
    in_repo_path: Option<String>,
}

impl GithubDeconstructedUrl {
    pub fn new(url: &str) -> Result<Self, String> {
        let url =
            Url::parse(url).map_err(|err| format!("Failed to parse the url {}: {}", url, err))?;

        let is_github_url = match url.host() {
            None => false,
            Some(host) => host == url::Host::Domain(GITHUB_DOMAIN),
        };

        if !is_github_url {
            return Err(format!("The url is not a github url: {}", url));
        }

        let url_segments = url
            .path_segments()
            .map(|char| char.collect::<Vec<_>>())
            .ok_or(format!("Couldn't divide url {} into segments", url))?;

        let deconstructed = GithubDeconstructedUrl {
            // This sucks, but idk
            owner: url_segments.get(0).map(|segment| segment.to_string()),
            repo: url_segments.get(1).map(|segment| segment.to_string()),
            branch: url_segments.get(3).map(|segment| segment.to_string()),
            in_repo_path: url_segments.split_at_checked(4).map(|t| t.1.join("/")),
        };

        return Ok(deconstructed);
    }
}

pub fn get_repository_content_from_url(url: &str) -> Result<(), String> {
    let deconstructed_url = GithubDeconstructedUrl::new(url)?;

    if let (Some(owner), Some(repo), Some(branch), Some(in_repo_path)) = (
        &deconstructed_url.owner,
        &deconstructed_url.repo,
        &deconstructed_url.branch,
        &deconstructed_url.in_repo_path,
    ) {
        return get_repository_content(owner, repo, branch, in_repo_path);
    }

    return Err(format!(
        "Wrong url {}. It should look like this: https://github.com/<OWNER>/<REPO>[/tree/<BRANCH>]/<REST-PATH>",
        url
    ));
}

pub fn get_repository_content(
    owner: &str,
    repo: &str,
    branch: &str,
    in_repo_path: &str,
) -> Result<(), String> {
    let mut url =
        Url::parse(format!("{BASE_URL}/repos/{owner}/{repo}/contents/{in_repo_path}").as_str())
            .map_err(|err| err.to_string())?;
    url.set_query(Some(format!("ref={branch}").as_str()));

    let client = reqwest::blocking::Client::new();

    let result = client
        .get(url)
        .header(reqwest::header::AUTHORIZATION, "Bearer GITHUB_TOKEN")
        .send();

    println!("{:?}", result);

    return Ok(());
}
