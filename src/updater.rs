use std::collections::HashMap;
use std::fmt;

use tinyjson::JsonValue;
use tokio::task::JoinSet;

use crate::{constants, MoonlightBranch};

struct GithubRelease {
    pub tag_name: String,
    pub name: String,
}

struct GithubReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
}

#[derive(Debug)]
pub enum UpdateError {
    /// The release metadata could not be fetched.
    Fetch(String),
    /// An asset download failed.
    Download { name: String, error: String },
    /// An asset was downloaded but could not be written to disk.
    Write { name: String, error: String },
    /// A tarball was downloaded but could not be extracted.
    Unpack { name: String, error: String },
}

impl fmt::Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UpdateError::Fetch(e) => write!(f, "Failed to fetch release info: {e}"),
            UpdateError::Download { name, error } => {
                write!(f, "Failed to download {name}: {error}")
            }
            UpdateError::Write { name, error } => write!(f, "Failed to write {name}: {error}"),
            UpdateError::Unpack { name, error } => write!(f, "Failed to extract {name}: {error}"),
        }
    }
}

impl std::error::Error for UpdateError {}

static NIGHTLY_REF_URL: &str = "https://moonlight-mod.github.io/moonlight/ref";
static NIGHTLY_DOWNLOAD_URL: &str = "https://moonlight-mod.github.io/moonlight/dist.tar.gz";

/// Fetch a single URL, returning the response body.
fn fetch_url(url: &str) -> Result<Vec<u8>, String> {
    let mut response = ureq::get(url).call().map_err(|e| e.to_string())?;
    response.body_mut().read_to_vec().map_err(|e| e.to_string())
}

fn fetch_nightly_release() -> Result<(GithubRelease, Vec<GithubReleaseAsset>), String> {
    // Fetch the ref file which contains the build hash and tag
    let body = fetch_url(NIGHTLY_REF_URL)?;
    let body = String::from_utf8(body).map_err(|e| format!("ref file is not valid UTF-8: {e}"))?;
    let mut lines = body.lines();

    // First line is the build hash, second line is the tag (refs/heads/develop)
    let build_hash = lines
        .next()
        .ok_or("ref file is empty")?
        .to_string();
    let tag = lines
        .next()
        .ok_or("ref file is missing the tag line")?
        .to_string();

    // Create a release and asset for the nightly build
    let release = GithubRelease {
        tag_name: build_hash,
        name: tag,
    };

    let asset = GithubReleaseAsset {
        name: "dist.tar.gz".to_string(),
        browser_download_url: NIGHTLY_DOWNLOAD_URL.to_string(),
    };

    Ok((release, vec![asset]))
}

fn fetch_stable_release() -> Result<(GithubRelease, Vec<GithubReleaseAsset>), String> {
    // Get the latest release manifest from GitHub
    let body = fetch_url(constants::RELEASE_URL)?;
    let body = String::from_utf8(body).map_err(|e| format!("release JSON is not valid UTF-8: {e}"))?;
    let json: JsonValue = body
        .parse()
        .map_err(|e| format!("failed to parse release JSON: {e}"))?;
    let object: &HashMap<_, _> = json.get().ok_or("release JSON is not an object")?;

    let tag_name: String = object
        .get("tag_name")
        .ok_or("release JSON is missing tag_name")?
        .get::<String>()
        .ok_or("tag_name is not a string")?
        .clone();
    let name: String = object
        .get("name")
        .ok_or("release JSON is missing name")?
        .get::<String>()
        .ok_or("name is not a string")?
        .clone();

    // Get the assets
    let assets: &Vec<_> = object
        .get("assets")
        .ok_or("release JSON is missing assets")?
        .get()
        .ok_or("assets is not an array")?;
    let assets: Vec<GithubReleaseAsset> = assets
        .iter()
        .filter_map(|asset| {
            let asset: &HashMap<_, _> = asset.get()?;
            let name: &String = asset.get("name")?.get()?;
            let browser_download_url: &String = asset.get("browser_download_url")?.get()?;

            if constants::RELEASE_ASSETS.contains(&name.as_str()) {
                Some(GithubReleaseAsset {
                    name: name.clone(),
                    browser_download_url: browser_download_url.clone(),
                })
            } else {
                None
            }
        })
        .collect();

    Ok((GithubRelease { tag_name, name }, assets))
}

/// Read the version of the last successful update from `release.json`, if any.
fn read_cached_version(release_file: &std::path::Path) -> Option<GithubRelease> {
    let data = std::fs::read_to_string(release_file).ok()?;
    let json: JsonValue = data.parse().ok()?;
    let object: &HashMap<_, _> = json.get()?;

    let tag_name: &String = object.get("tag_name")?.get()?;
    let name: &String = object.get("name")?.get()?;

    Some(GithubRelease {
        tag_name: tag_name.clone(),
        name: name.clone(),
    })
}

pub async fn download_assets(moonlight_branch: MoonlightBranch) -> Result<(), UpdateError> {
    let assets_dir = constants::asset_cache_dir().unwrap();
    let release_file = assets_dir.join(constants::RELEASE_INFO_FILE);

    let current_version = read_cached_version(&release_file);

    println!("[moonlight launcher] Checking for updates...");

    // Fetch the appropriate release based on the branch
    let (release, assets) = tokio::task::spawn_blocking(move || match moonlight_branch {
        MoonlightBranch::Stable => fetch_stable_release(),
        MoonlightBranch::Nightly => fetch_nightly_release(),
    })
    .await
    .map_err(|e| UpdateError::Fetch(format!("release fetch task panicked: {e}")))?
    .map_err(UpdateError::Fetch)?;

    // If the latest release is the same as our current one, don't bother downloading.
    if let Some(current) = current_version {
        if current.name == release.name && current.tag_name == release.tag_name {
            return Ok(());
        }
    }

    println!("[moonlight launcher] An update is available... Downloading...");

    // Spawn all the download tasks simultaneously
    let mut tasks = JoinSet::new();
    for asset in assets {
        let name = asset.name;
        let url = asset.browser_download_url.clone();

        tasks.spawn(async move {
            let body = tokio::task::spawn_blocking(move || fetch_url(&url))
                .await
                .map_err(|e| (name.clone(), format!("task panicked: {e}")))?
                .map_err(|e| (name.clone(), e))?;

            Ok::<_, (String, String)>((name, body))
        });
    }

    // Wait for each task to finish and write them to disk.
    while let Some(joined) = tasks.join_next().await {
        let (name, body) = joined
            .map_err(|e| UpdateError::Download {
                name: "unknown".into(),
                error: format!("task panicked: {e}"),
            })?
            .map_err(|(name, error)| UpdateError::Download { name, error })?;

        let path = assets_dir.join(&name);

        if name.ends_with(".tar.gz") {
            let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(body.as_slice()));
            archive
                .unpack(&assets_dir)
                .map_err(|e| UpdateError::Unpack { name, error: e.to_string() })?;
        } else {
            std::fs::write(&path, body).map_err(|e| UpdateError::Write {
                name,
                error: e.to_string(),
            })?;
        }
    }

    // Write the new release.json to disk after everything succeeds.
    let release_json = format!(
        "{{\n\
        	\"tag_name\": \"{tag_name}\",\n\
        	\"name\": \"{name}\"\n\
		}}",
        tag_name = release.tag_name,
        name = release.name
    );

    std::fs::write(&release_file, release_json).map_err(|e| UpdateError::Write {
        name: constants::RELEASE_INFO_FILE.into(),
        error: e.to_string(),
    })?;

    Ok(())
}
