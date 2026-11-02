use url::Url;

const GITHUB_DOMAIN: &str = "github.com";
const BASE_URL: &str = "https://api.github.com";

pub struct GithubApiClient {
    client: reqwest::blocking::Client,
}

impl GithubApiClient {
    pub fn new() -> Result<Self, String> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(reqwest::header::USER_AGENT, reqwest::header::HeaderValue::from_static("git_dir"));
        headers.insert("X-GitHub-Api-Version", reqwest::header::HeaderValue::from_static("2026-03-10"));
        headers.insert(reqwest::header::ACCEPT, reqwest::header::HeaderValue::from_static("application/json"));

        let client = reqwest::blocking::Client::builder()
                .default_headers(headers)
                .build()
                .map_err(|err| err.to_string())?;

        return Ok(Self {
            client,
        });
    }

    pub fn get_repository_content_from_url(&self, url: &str) -> Result<(), String> {
        let deconstructed_url = GithubDeconstructedUrl::new(url)?;

        if let (Some(owner), Some(repo), Some(in_repo_path)) = (
            &deconstructed_url.owner,
            &deconstructed_url.repo,
            &deconstructed_url.in_repo_path,
        ) {
            return self.get_repository_content(owner, repo, in_repo_path, deconstructed_url.branch.as_deref());
        }

        return Err(format!(
            "Wrong url {}. It should look like this: https://github.com/<OWNER>/<REPO>[/tree/<BRANCH>]/<REST-PATH>",
            url
        ));
    }

    pub fn get_repository_content(
        &self,
        owner: &str,
        repo: &str,
        in_repo_path: &str,
        branch: Option<&str>,
    ) -> Result<(), String> {
        let mut url =
            Url::parse(format!("{BASE_URL}/repos/{owner}/{repo}/contents/{in_repo_path}").as_str())
                .map_err(|err| err.to_string())?;
        if let Some(branch) = branch {
            url.set_query(Some(format!("ref={branch}").as_str()));
        }

        let result = self.client
            .get(url)
            .send().map_err(|err| err.to_string())?
            .error_for_status()
            .map_err(|err| err.to_string())?;

        let json = result.text().map_err(|err| err.to_string())?;

        return Ok(());
    }
}

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
