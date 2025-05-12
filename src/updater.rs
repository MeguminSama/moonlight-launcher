use std::collections::HashMap;
use tinyjson::JsonValue;
use tokio::task::JoinSet;

use crate::{constants, MoonlightBranch};

async fn fetch_nightly_release() -> Option<(GithubRelease, Vec<GithubReleaseAsset>)> {
    // Fetch the ref file which contains the build hash and tag
    let mut response = ureq::get(NIGHTLY_REF_URL).call().ok()?;
    let body = response.body_mut().read_to_string().ok()?;
    let mut lines = body.lines();

    // First line is the build hash, second line is the tag (refs/heads/develop)
    let build_hash = lines.next()?.to_string();
    let tag = lines.next()?.to_string();

    // Create a release and asset for the nightly build
    let release = GithubRelease {
        tag_name: build_hash,
        name: tag,
    };

    let asset = GithubReleaseAsset {
        name: "dist.tar.gz".to_string(),
        browser_download_url: NIGHTLY_DOWNLOAD_URL.to_string(),
    };

    Some((release, vec![asset]))
}

async fn fetch_stable_release() -> Option<(GithubRelease, Vec<GithubReleaseAsset>)> {
    // Get the latest release manifest from GitHub
    let mut response = ureq::get(constants::RELEASE_URL).call().ok()?;
    let body = response.body_mut().read_to_string().ok()?;
    let json: JsonValue = body.parse().ok()?;
    let object: &HashMap<_, _> = json.get()?;

    let tag_name: String = object.get("tag_name")?.get::<String>()?.clone();
    let name: String = object.get("name")?.get::<String>()?.clone();

    // Get the assets
    let assets: &Vec<_> = object.get("assets")?.get()?;
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

    Some((GithubRelease { tag_name, name }, assets))
}

struct GithubRelease {
    pub tag_name: String,
    pub name: String,
}

struct GithubReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
}

static NIGHTLY_REF_URL: &str = "https://moonlight-mod.github.io/moonlight/ref";
static NIGHTLY_DOWNLOAD_URL: &str = "https://moonlight-mod.github.io/moonlight/dist.tar.gz";

pub async fn download_assets(moonlight_branch: MoonlightBranch) -> Option<()> {
    let assets_dir = constants::asset_cache_dir().unwrap();
    let release_file = assets_dir.join(constants::RELEASE_INFO_FILE);

    // Get the current release.json if it exists.
    let current_version = if release_file.exists() {
        let data = std::fs::read_to_string(&release_file).ok()?;
        let json: JsonValue = data.parse().ok()?;
        let object: &HashMap<_, _> = json.get()?;

        let tag_name: &String = object.get("tag_name")?.get()?;
        let name: &String = object.get("name")?.get()?;

        Some(GithubRelease {
            tag_name: tag_name.clone(),
            name: name.clone(),
        })
    } else {
        None
    };

    println!("[moonlight launcher] Checking for updates...");

    // Fetch the appropriate release based on the branch
    let (release, assets) = match moonlight_branch {
        MoonlightBranch::Stable => fetch_stable_release().await?,
        MoonlightBranch::Nightly => fetch_nightly_release().await?,
    };

    // If the latest release is the same as our current one, don't bother downloading.
    if let Some(current) = current_version {
        if current.name == release.name && current.tag_name == release.tag_name {
            return Some(());
        }
    }

    println!("[moonlight launcher] An update is available... Downloading...");

    // Spawn all the download tasks simultaneously.
    // TODO: Make this more robust. What if one fails but the rest succeed? We want to try re-downloading it.
    let mut tasks = JoinSet::new();
    for asset in assets {
        let url = asset.browser_download_url.clone();

        tasks.spawn(async move {
            let mut response = ureq::get(&url).call().ok()?;
            let body = response.body_mut().read_to_vec().ok()?;
            Some((asset.name, body))
        });
    }

    // Wait for each task to finish and write them to disk.
    while let Some(resp) = tasks.join_next().await {
        let (name, body) = resp.ok()??;
        let path = assets_dir.join(&name);

        if name.ends_with(".tar.gz") {
            let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(body.as_slice()));
            archive.unpack(&assets_dir).ok()?;
        } else {
            std::fs::write(&path, body).ok()?;
        }
    }

    // Write the new release.json to disk.
    let release_json = format!(
        "{{\n\
        	\"tag_name\": \"{tag_name}\",\n\
        	\"name\": \"{name}\"\n\
		}}",
        tag_name = release.tag_name,
        name = release.name
    );

    std::fs::write(&release_file, release_json).ok()?;

    Some(())
}
