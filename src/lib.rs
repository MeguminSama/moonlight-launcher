// For compiling the modloader DLL:
pub use electron_hook::*;

pub mod constants;
pub mod discord;
pub mod updater;

// Library for the binaries to use:
#[cfg(windows)]
pub mod windows;

#[cfg(windows)]
pub use windows::*;

use clap::Parser;
use discord::{DiscordBranch, DiscordPath};

#[derive(clap::Parser, Debug)]
struct Args {
    /// To use a local instance of the mod, pass the path to the mod entrypoint.
    ///
    /// e.g. `--local "C:\\Users\\megu\\moonlight-mod\\dist\\injector.js"`
    #[clap(short, long)]
    pub local: Option<String>,

    /// Which branch of moonlight to launch.
    ///
    /// If you're running moonlight-stable, the default will be `stable`.
    ///
    /// If you're running moonlight-ptb or moonlight-canary, this will be `nightly`.
    #[clap(long, value_enum)]
    pub branch: Option<MoonlightBranch>,

    /// Optional launch arguments to pass to the Discord executable
    ///
    /// e.g. `-- --start-minimized --enable-blink-features=MiddleClickAutoscroll`
    #[clap(allow_hyphen_values = true, last = true)]
    pub launch_args: Vec<String>,
}
#[derive(clap::ValueEnum, Debug, Clone)]
pub enum MoonlightBranch {
    Stable,
    Nightly,
}

pub async fn launch(
    instance_id: &str,
    branch: DiscordBranch,
    display_name: &str,
    moonlight_branch: MoonlightBranch,
) {
    let args = Args::parse();

    let moonlight_branch = match args.branch {
        Some(branch) => branch,
        None => moonlight_branch,
    };

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

    let library_path = constants::get_library_path();

    let assets_dir = constants::asset_cache_dir().unwrap();

    // If `--local` is provided, use a local build. Otherwise, download assets.
    let mod_entrypoint = if let Some(local_path) = args.local {
        local_path
    } else {
        // We can usually attempt to run Discord even if the downloads fail...
        // TODO: Make this more robust. Maybe specific error reasons so we can determine if it's safe to continue.
        let _ = updater::download_assets(moonlight_branch).await;

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
                &library_path,
                &asar_path,
                args.launch_args,
                false,
            )
            .unwrap();
        }
        #[cfg(target_os = "linux")]
        DiscordPath::FlatpakId(id) => {
            electron_hook::launch_flatpak(&id, &library_path, &asar_path, args.launch_args, false)
                .unwrap();
        }
        #[cfg(not(target_os = "linux"))]
        DiscordPath::FlatpakId(_) => {
            panic!("Flatpak is only supported on Linux");
        }
    }
}
