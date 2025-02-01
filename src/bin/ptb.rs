#![windows_subsystem = "windows"]

use moonlight_launcher::discord::DiscordBranch;

static INSTANCE_ID: &str = "moonlight-ptb";
static DISCORD_BRANCH: DiscordBranch = DiscordBranch::PTB;
static DISPLAY_NAME: &str = "moonlight PTB";

#[tokio::main]
async fn main() {
    moonlight_launcher::launch(INSTANCE_ID, DISCORD_BRANCH, DISPLAY_NAME).await;
}
