use discord_rich_presence::{activity::{self, Timestamps, Button}, DiscordIpc, DiscordIpcClient};

use crate::{rank, util::current_unix_time, melee::{stage::{MeleeStage, OptionalMeleeStage}, character::{MeleeCharacter, OptionalMeleeCharacter}, MeleeScene, SlippiMenuScene}};
use crate::util;

#[derive(Debug, PartialEq, Clone)]
pub enum DiscordClientRequestType {
    Clear,
    Queue,
    Game,
}

#[derive(Debug, PartialEq, Clone)]
pub enum DiscordClientRequestTimestampMode {
    Start,
    Static, // like Start, but we never update even if the timestamp changes. Used for non-ingame actions. 
    End
}

#[derive(Debug, Clone)]
pub struct DiscordClientRequestTimestamp {
    pub mode: DiscordClientRequestTimestampMode,
    pub timestamp: i64
}

// we ignore this field
impl PartialEq for DiscordClientRequestTimestamp {
    fn eq(&self, o: &Self) -> bool {
        // if the game was in pause for too long, resynchronize by saying that this payload is not the same as the other
        // to respect the rate limit, we choose a relatively high amount of seconds
        self.mode == DiscordClientRequestTimestampMode::Static || self.timestamp.abs_diff(o.timestamp) < 15
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct DiscordClientRequest {
    pub req_type: DiscordClientRequestType,
    pub scene: Option<SlippiMenuScene>,
    pub stage: OptionalMeleeStage,
    pub character: OptionalMeleeCharacter,
    pub mode: String,
    pub timestamp: DiscordClientRequestTimestamp
}

impl Default for DiscordClientRequest {
    fn default() -> Self {
        DiscordClientRequest { req_type: DiscordClientRequestType::Clear, scene: None, stage: OptionalMeleeStage(None), character: OptionalMeleeCharacter(None), mode: "".to_string(), timestamp: DiscordClientRequestTimestamp { mode: DiscordClientRequestTimestampMode::Static, timestamp: current_unix_time() } }
    }
}

impl DiscordClientRequest {
    pub fn clear() -> Self { Default::default() }
    pub fn queue(scene: Option<SlippiMenuScene>, character: Option<MeleeCharacter>) -> Self {
        Self {
            req_type: DiscordClientRequestType::Queue,
            scene,
            character: OptionalMeleeCharacter(character),
            ..Default::default()
        }
    }
    pub fn game(stage: Option<MeleeStage>, character: Option<MeleeCharacter>, mode: MeleeScene, timestamp: DiscordClientRequestTimestamp) -> Self {
        Self {
            req_type: DiscordClientRequestType::Game,
            stage: OptionalMeleeStage(stage),
            character: OptionalMeleeCharacter(character),
            mode: mode.to_string(),
            timestamp,
            ..Default::default()
        }
    }
}

pub struct DiscordClient {
    client: DiscordIpcClient
}

impl DiscordClient {
    pub fn clear(&mut self) {
        self.client.clear_activity().unwrap();
    }
    pub async fn queue(&mut self, scene: Option<SlippiMenuScene>, character: OptionalMeleeCharacter) {
        let mut large_image = "".to_string();
        let mut large_text = "".to_string();
        if scene.unwrap_or(SlippiMenuScene::Direct) == SlippiMenuScene::Ranked {
            let rank_info = rank::get_rank_info("flcd-507").await.unwrap(); // TODO replace code later
            large_image = rank_info.name.to_lowercase().replace(" ", "_");
            large_text = format!("{} | {} ELO", rank_info.name, util::round(rank_info.elo, 2));
        }

        self.client.set_activity(
            activity::Activity::new()
                .assets(
                    activity::Assets::new()
                        .large_image(large_image.as_str())
                        .large_text(large_text.as_str())
                        .small_image(character.as_discord_resource().as_str())
                        .small_text(character.to_string().as_str())
                )
                .buttons(vec![Button::new("View Ranked Profile", format!("https://slippi.gg/user/{}", "flcd-507").as_str())])
                .timestamps(self.current_timestamp())
                .details(scene.and_then(|v| Some(v.to_string())).or(Some("".to_string())).unwrap().as_str())
                .state("In Queue")
        ).unwrap();
        
    }
    pub fn game(&mut self, stage: OptionalMeleeStage, character: OptionalMeleeCharacter, mode: String, timestamp: DiscordClientRequestTimestamp) {
        self.client.set_activity(
            activity::Activity::new()
                .assets(
                    activity::Assets::new()
                        .large_image(stage.as_discord_resource().as_str())
                        .large_text(stage.to_string().as_str())
                        .small_image(character.as_discord_resource().as_str())
                        .small_text(character.to_string().as_str())
                )
                .timestamps(if (timestamp.mode as u8) < (DiscordClientRequestTimestampMode::End as u8) { Timestamps::new().start(timestamp.timestamp) } else { Timestamps::new().end(timestamp.timestamp) })
                .details(mode.as_str())
                .state("In Game")
        ).unwrap();
    }
    pub fn close(&mut self) {
        self.client.close().unwrap();
    }

    fn current_timestamp(&self) -> Timestamps {
        Timestamps::new().start(util::current_unix_time())
    }
}

pub fn start_client() -> Result<DiscordClient, Box<dyn std::error::Error>> {
    let mut client = DiscordIpcClient::new("1096595344600604772")?;
    client.connect()?;

    Ok(DiscordClient { client })
}