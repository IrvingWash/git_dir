use clap::Parser;
use std::path::PathBuf;

#[derive(clap::Parser, Debug)]
#[command(about, long_about = None)]
struct Args {
    #[arg(short, long)]
    source: String,
    #[arg(short, long)]
    destination: String,
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let args = Args::parse();

    let expanded_dst = expand_tilde(&args.destination).ok_or(format!(
        "Failed to expand desintation path ({})",
        args.destination,
    ))?;

    let result = git_dir::download_folder_from_github_repo_url(&args.source, &expanded_dst).await;

    match result {
        Err(e) => return Err(format!("Failed: {}", e)),
        Ok(r) => {
            println!(
                "Successfully downloaded to {:?}/{}.zip",
                r.destination, r.archive_name,
            );

            if !r.warnings.is_empty() {
                println!("But got {} warnings:", r.warnings.len());

                for warning in r.warnings {
                    println!("{}", warning);
                }
            }
        }
    }
    return Ok(());
}

fn expand_tilde(path: &str) -> Option<PathBuf> {
    let base_dirs = directories::BaseDirs::new()?;

    if path == "~" {
        Some(base_dirs.home_dir().to_path_buf())
    } else if let Some(rest) = path.strip_prefix("~/") {
        Some(base_dirs.home_dir().join(rest))
    } else {
        Some(PathBuf::from(path))
    }
}
