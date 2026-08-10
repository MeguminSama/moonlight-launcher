// For compiling the modloader DLL:
pub use electron_hook::*;

pub mod app_version;
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

fn show_error(title: &str, message: &str) {
    #[cfg(not(windows))]
    {
        use dialog::DialogBox as _;
        let _ = dialog::Message::new(message.to_string())
            .title(title.to_string())
            .show();
    }

    #[cfg(windows)]
    messagebox(title, message, MessageBoxIcon::Error);
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

        show_error(&title, &message);

        return;
    };

    let library_path = constants::get_library_path();

    let Some(assets_dir) = constants::asset_cache_dir() else {
        show_error(
            &format!("Failed to initialize {display_name}"),
            "moonlight couldn't determine your data directory.",
        );
        return;
    };

    // If `--local` is provided, use a local build. Otherwise, download assets.
    let mod_entrypoint = if let Some(local_path) = args.local {
        local_path
    } else {
        let entrypoint = assets_dir.join(constants::MOD_ENTRYPOINT);

        // We can usually attempt to run Discord even if the downloads fail, so
        // fall back to the previous installation when one exists.
        if let Err(e) = updater::download_assets(moonlight_branch).await {
            eprintln!("[moonlight launcher] Failed to update moonlight: {e}");

            if !entrypoint.exists() {
                let title = format!("Failed to update {display_name}");
                let message = format!(
                    "moonlight couldn't download the latest version.\n\
					You can try reinstalling {display_name} and try again.\n\n\
					Error: {e}"
                );

                show_error(&title, &message);

                return;
            }

            println!("[moonlight launcher] Using previous installation.");
        }

        entrypoint
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

    let asar = match electron_hook::asar::Asar::new()
        .with_id(instance_id)
        .with_mod_entrypoint(&mod_entrypoint)
        .with_template(include_str!("./require.js"))
        .with_wm_class(&format!("moonlight"))
        .create()
    {
        Ok(asar) => asar,
        Err(e) => {
            show_error(
                &format!("Failed to launch {display_name}"),
                &format!("moonlight couldn't create its launcher files.\n\nError: {e}"),
            );
            return;
        }
    };

    let asar_path = asar.to_string_lossy().to_string();

    match discord_dir {
        DiscordPath::Filesystem(discord_dir) => {
            let discord_dir = discord_dir.to_string_lossy().to_string();

            if let Err(e) = electron_hook::launch(
                &discord_dir,
                &library_path,
                &asar_path,
                args.launch_args,
                false,
            ) {
                show_error(
                    &format!("Failed to launch {display_name}"),
                    &format!("moonlight couldn't start Discord.\n\nError: {e}"),
                );
            }
        }
        #[cfg(target_os = "linux")]
        DiscordPath::FlatpakId(id) => {
            if let Err(e) = electron_hook::launch_flatpak(
                &id,
                &library_path,
                &asar_path,
                args.launch_args,
                false,
            ) {
                show_error(
                    &format!("Failed to launch {display_name}"),
                    &format!("moonlight couldn't start Discord.\n\nError: {e}"),
                );
            }
        }
        #[cfg(not(target_os = "linux"))]
        DiscordPath::FlatpakId(_) => {
            panic!("Flatpak is only supported on Linux");
        }
    }
}
