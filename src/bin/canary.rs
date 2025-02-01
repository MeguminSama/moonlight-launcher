#![windows_subsystem = "windows"]

use moonlight_launcher::discord::DiscordBranch;

static INSTANCE_ID: &str = "moonlight-canary";
static DISCORD_BRANCH: DiscordBranch = DiscordBranch::Canary;
static DISPLAY_NAME: &str = "moonlight Canary";

#[tokio::main]
async fn main() {
    moonlight_launcher::launch(INSTANCE_ID, DISCORD_BRANCH, DISPLAY_NAME).await;
}
