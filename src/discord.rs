use std::path::PathBuf;

#[derive(Clone, Copy)]
pub enum DiscordBranch {
    Stable,
    Canary,
    PTB,
    Development,
}

#[cfg(windows)]
pub fn get_discord(branch: DiscordBranch) -> Option<PathBuf> {
    use crate::windows::get_latest_executable;
    let local_appdata = dirs::data_local_dir()?;

    let name = match branch {
        DiscordBranch::Stable => "Discord",
        DiscordBranch::PTB => "DiscordPTB",
        DiscordBranch::Canary => "DiscordCanary",
        DiscordBranch::Development => "DiscordDevelopment",
    };

    let dir = local_appdata.join(name);

    if !dir.join("Update.exe").exists() {
        return None;
    }

    let executable = get_latest_executable(&dir).ok()?;

    Some(executable)
}

#[cfg(target_os = "linux")]
pub fn get_discord(branch: DiscordBranch) -> Option<PathBuf> {
    let local_share = dirs::data_local_dir()?;

    // Try non-flatpak installs first.
    let (name, dvm_branch) = match branch {
        DiscordBranch::Stable => ("Discord", "stable"),
        DiscordBranch::PTB => ("DiscordPTB", "ptb"),
        DiscordBranch::Canary => ("DiscordCanary", "canary"),
        DiscordBranch::Development => ("DiscordDevelopment", "development"),
    };

    // On linux, the executable is at /home/user/.local/share/DiscordCanary/DiscordCanary
    let executable = local_share.join(name).join(name);

    if executable.is_file() {
        return Some(executable);
    }

    // If that doesn't work, try $HOME/.dvm/branches

    let executable = dirs::home_dir()?.join(format!(".dvm/branches/{dvm_branch}/{name}/{name}"));

    if executable.is_file() {
        return Some(executable);
    }

    // FIXME: Flatpak Support

    None
}

#[cfg(target_os = "macos")]
pub fn get_discord(name: &str) -> Option<PathBuf> {
    todo!();
}
