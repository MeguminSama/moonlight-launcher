// For compiling the modloader DLL:
pub use electron_hook::*;

pub mod constants;
pub mod discord;

// Library for the binaries to use:
#[cfg(windows)]
pub mod windows;

#[cfg(windows)]
pub use windows::*;

use clap::Parser;
use discord::{DiscordBranch, DiscordPath};
use std::collections::HashMap;
use tinyjson::JsonValue;
use tokio::task::JoinSet;

struct GithubRelease {
    pub tag_name: String,
    pub name: String,
}

struct GithubReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
}

#[derive(clap::Parser, Debug)]
struct Args {
    /// To use a local instance of the mod, pass the path to the mod entrypoint.
    ///
    /// e.g. `--local "C:\\Users\\megu\\moonlight-mod\\dist\\injector.js"`
    #[clap(short, long)]
    pub local: Option<String>,

    /// Optional launch arguments to pass to the Discord executable
    ///
    /// e.g. `-- --start-minimized --enable-blink-features=MiddleClickAutoscroll`
    #[clap(allow_hyphen_values = true, last = true)]
    pub launch_args: Vec<String>,
}

pub async fn launch(instance_id: &str, branch: DiscordBranch, display_name: &str) {
    let args = Args::parse();

    let Some(discord_dir) = discord::get_discord(branch) else {
        let title = format!("No {display_name} installation found!");
        let message = format!(
            "moonlight couldn't find your Discord installation.\n\
			Try reinstalling {display_name} and try again."
        );

        #[cfg(not(windows))]
        {
            use dialog::DialogBox as _;
            let _ = dialog::Message::new(message).title(title).show();
        }

        #[cfg(windows)]
        messagebox(&title, &message, MessageBoxIcon::Error);

        return;
    };

    // TODO: This is windows-specific logic. On linux, finding the library can be much more complicated.
    #[cfg(unix)]
    let library_name = format!("lib{}", constants::LIBRARY);
    #[cfg(windows)]
    let library_name = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join(constants::LIBRARY)
        .to_string_lossy()
        .to_string();

    let assets_dir = asset_cache_dir().unwrap();

    // If `--local` is provided, use a local build. Otherwise, download assets.
    let mod_entrypoint = if let Some(local_path) = args.local {
        local_path
    } else {
        // We can usually attempt to run Discord even if the downloads fail...
        // TODO: Make this more robust. Maybe specific error reasons so we can determine if it's safe to continue.
        let _ = download_assets().await;

        assets_dir
            .join(constants::MOD_ENTRYPOINT)
            .to_string_lossy()
            .replace("\\", "\\\\")
            .to_string()
    };

    let branch_name = match branch {
        DiscordBranch::Stable => "stable",
        DiscordBranch::PTB => "ptb",
        DiscordBranch::Canary => "canary",
        DiscordBranch::Development => "development",
    };

    let asar = electron_hook::asar::Asar::new()
        .with_id(instance_id)
        .with_mod_entrypoint(&mod_entrypoint)
        .with_template(include_str!("./require.js"))
        .with_wm_class(&format!("moonlight-{branch_name}"))
        .create()
        .unwrap();

    let asar_path = asar.to_string_lossy().to_string();

    match discord_dir {
        DiscordPath::Filesystem(discord_dir) => {
            let discord_dir = discord_dir.to_string_lossy().to_string();

            electron_hook::launch(
                &discord_dir,
                &library_name,
                &asar_path,
                args.launch_args,
                false,
            )
            .unwrap();
        }
        #[cfg(target_os = "linux")]
        DiscordPath::FlatpakId(id) => {
            electron_hook::launch_flatpak(&id, &library_name, &asar_path, args.launch_args, false)
                .unwrap();
        }
        #[cfg(not(target_os = "linux"))]
        DiscordPath::FlatpakId(_) => {
            panic!("Flatpak is only supported on Linux");
        }
    }
}

async fn download_assets() -> Option<()> {
    let assets_dir = asset_cache_dir().unwrap();
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

fn asset_cache_dir() -> Option<std::path::PathBuf> {
    let local_appdata = dirs::data_local_dir()?;

    let dir = local_appdata.join("moonlight-launcher").join("cache");

    if !dir.exists() {
        std::fs::create_dir_all(&dir).ok()?;
    }

    Some(dir)
}
