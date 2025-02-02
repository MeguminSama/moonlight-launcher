use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DiscordBranch {
    Stable,
    Canary,
    PTB,
    Development,
}

pub enum DiscordPath {
    Filesystem(PathBuf),
    FlatpakId(String),
}

#[cfg(windows)]
pub fn get_discord(branch: DiscordBranch) -> Option<DiscordPath> {
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

    Some(DiscordPath::Filesystem(executable))
}

#[cfg(target_os = "linux")]
pub fn get_discord(branch: DiscordBranch) -> Option<DiscordPath> {
    use std::process::Command;

    let local_share = dirs::data_local_dir()?;

    // Try non-flatpak installs first.
    let (name, lower_name) = match branch {
        DiscordBranch::Stable => ("Discord", "stable"),
        DiscordBranch::PTB => ("DiscordPTB", "ptb"),
        DiscordBranch::Canary => ("DiscordCanary", "canary"),
        DiscordBranch::Development => ("DiscordDevelopment", "development"),
    };

    // On linux, the executable is at /home/user/.local/share/DiscordCanary/DiscordCanary
    let executable = local_share.join(name).join(name);

    if executable.is_file() {
        return Some(DiscordPath::Filesystem(executable));
    }

    // If that doesn't work, try $HOME/.dvm/branches
    let executable = dirs::home_dir()?.join(format!(".dvm/branches/{lower_name}/{name}/{name}"));
    if executable.is_file() {
        return Some(DiscordPath::Filesystem(executable));
    }

    let executable = PathBuf::from(format!("/usr/bin/discord-{lower_name}"));
    if executable.is_file() {
        return Some(DiscordPath::Filesystem(executable));
    }

    // FIXME: Flatpak Support

    // As a last resort, try checking if it's in PATH
    let command = if branch == DiscordBranch::Stable {
        "discord".to_string()
    } else {
        format!("discord-{lower_name}")
    };

    let command_output = Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {}", command))
        .output()
        .ok()?;

    if command_output.status.success() {
        let path = String::from_utf8(command_output.stdout).ok()?;
        let path = path.trim(); // Remove any trailing newline
        return Some(DiscordPath::Filesystem(PathBuf::from(path)));
    }

    let flatpak_name = match branch {
        DiscordBranch::Stable => "com.discordapp.Discord",
        DiscordBranch::PTB => "com.discordapp.DiscordPTB",
        DiscordBranch::Canary => "com.discordapp.DiscordCanary",
        DiscordBranch::Development => "com.discordapp.DiscordDevelopment",
    };

    let flatpak_dir = PathBuf::from(format!(
        "/var/lib/flatpak/app/{flatpak_name}/current/active/"
    ));

    if flatpak_dir.is_dir() {
        return Some(DiscordPath::FlatpakId(flatpak_name.to_string()));
    }

    None
}

#[cfg(target_os = "macos")]
pub fn get_discord(name: &str) -> Option<PathBuf> {
    todo!();
}
