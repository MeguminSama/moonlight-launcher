use std::collections::HashMap;
use tinyjson::JsonValue;
use tokio::task::JoinSet;

use crate::constants;

struct GithubRelease {
    pub tag_name: String,
    pub name: String,
}

struct GithubReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
}

pub async fn download_assets() -> Option<()> {
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

    // Get the latest release manifest from GitHub. If it fails, try the fallback.
    println!("[moonlight launcher] Checking for updates...");
    let mut response = ureq::get(constants::RELEASE_URL).call().ok()?;
    // TODO: Add fallback URL
    // if response.status() != 200 {
    //     println!("[moonlight launcher] GitHub ratelimited... Trying fallback...");
    //     response = ureq::get(constants::RELEASE_URL_FALLBACK).call().ok()?;
    // }
    let body = response.body_mut().read_to_string().ok()?;

    let json: JsonValue = body.parse().ok()?;
    let object: &HashMap<_, _> = json.get()?;

    let tag_name: &String = object.get("tag_name")?.get()?;
    let name: &String = object.get("name")?.get()?;

    // If the latest release is the same as our current one, don't bother downloading.
    if let Some(release) = current_version {
        if release.name == *name && release.tag_name == *tag_name {
            return Some(());
        }
    }

    println!("[moonlight launcher] An update is available... Downloading...");

    // Loop over the assets and find the ones we want.
    let assets: &Vec<_> = object.get("assets")?.get()?;
    let assets: Vec<_> = assets
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
		}}"
    );

    std::fs::write(&release_file, release_json).ok()?;

    Some(())
}
