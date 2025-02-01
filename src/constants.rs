#[cfg(windows)]
pub static LIBRARY: &str = "moonlight_launcher.dll";

#[cfg(not(windows))]
pub static LIBRARY: &str = "moonlight_launcher.so";

pub static MOD_ENTRYPOINT: &str = "injector.js";
pub static RELEASE_URL: &str =
    "https://api.github.com/repos/moonlight-mod/moonlight/releases/latest";
// TODO: Add fallback URL
// pub static RELEASE_URL_FALLBACK: &str = "PROVIDE A FALLBACK URL FOR WHEN GITHUB IS RATELIMITED";
pub static RELEASE_INFO_FILE: &str = "release.json";
pub static RELEASE_ASSETS: &[&str] = &["dist.tar.gz"];
