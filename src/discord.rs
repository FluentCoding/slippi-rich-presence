use discord_rich_presence::{activity::{self, Timestamps}, DiscordIpc, DiscordIpcClient};

use crate::{rank, util::current_unix_time, melee::{stage::MeleeStage, character::MeleeCharacter, MeleeScene, SlippiMenuScene}};
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
    End
}

#[derive(Debug, Clone)]
pub struct DiscordClientRequestTimestamp {
    pub mode: DiscordClientRequestTimestampMode,
    pub timestamp: i64
}

// we ignore this field
impl PartialEq for DiscordClientRequestTimestamp {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct DiscordClientRequest {
    pub req_type: DiscordClientRequestType,
    pub scene: Option<SlippiMenuScene>,
    pub stage: String,
    pub character: String,
    pub mode: String,
    pub timestamp: DiscordClientRequestTimestamp
}

impl Default for DiscordClientRequest {
    fn default() -> Self {
        DiscordClientRequest { req_type: DiscordClientRequestType::Clear, scene: None, stage: "".to_string(), character: "".to_string(), mode: "".to_string(), timestamp: DiscordClientRequestTimestamp { mode: DiscordClientRequestTimestampMode::Start, timestamp: current_unix_time() } }
    }
}

impl DiscordClientRequest {
    pub fn clear() -> Self { Default::default() }
    pub fn queue(scene: Option<SlippiMenuScene>, character: Option<MeleeCharacter>) -> Self {
        Self {
            req_type: DiscordClientRequestType::Queue,
            scene,
            character: Self::character_transformer(character),
            ..Default::default()
        }
    }
    pub fn game(stage: Option<MeleeStage>, character: Option<MeleeCharacter>, mode: MeleeScene, timestamp: DiscordClientRequestTimestamp) -> Self {
        Self {
            req_type: DiscordClientRequestType::Game,
            stage: Self::stage_transformer(stage),
            character: Self::character_transformer(character),
            mode: mode.to_string(),
            timestamp,
            ..Default::default()
        }
    }
    fn stage_transformer(stage: Option<MeleeStage>) -> String { Self::default_questionmark(stage.and_then(|s| Some(format!("stage{}", s as u8)))) }
    fn character_transformer(character: Option<MeleeCharacter>) -> String { Self::default_questionmark(character.and_then(|c| Some(format!("char{}", c as u8)))) }
    fn default_questionmark(opt: Option<String>) -> String { opt.unwrap_or("questionmark".to_string()) }
}

pub struct DiscordClient {
    client: DiscordIpcClient
}

impl DiscordClient {
    pub fn clear(&mut self) {
        self.client.clear_activity().unwrap();
    }
    pub async fn queue(&mut self, scene: Option<SlippiMenuScene>, character: String) {
        let mut large_image = "".to_string();
        let mut large_text = "".to_string();
        if scene.unwrap_or(SlippiMenuScene::Direct) == SlippiMenuScene::Ranked {
            let rank_info = rank::get_rank_info("flcd-507").await.unwrap();
            large_image = rank_info.name.clone(); // TODO replace code later
            large_text = format!("{} | {} ELO", rank_info.name, util::round(rank_info.elo, 2));
        }

        self.client.set_activity(
            activity::Activity::new()
                .assets(
                    activity::Assets::new()
                        .large_image(large_image.as_str())
                        .large_text(large_text.as_str())
                        .small_image(character.as_str())
                    )
                .timestamps(self.current_timestamp())
                .details(scene.and_then(|v| Some(v.to_string())).or(Some("".to_string())).unwrap().as_str())
                .state("In Queue")
        ).unwrap();
        
    }
    pub fn game(&mut self, stage: String, character: String, mode: String, timestamp: DiscordClientRequestTimestamp) {
        self.client.set_activity(
            activity::Activity::new()
                .assets(
                    activity::Assets::new()
                        .large_image(stage.as_str())
                        .small_image(character.as_str())
                    )
                .timestamps(if timestamp.mode == DiscordClientRequestTimestampMode::Start { Timestamps::new().start(timestamp.timestamp) } else { Timestamps::new().end(timestamp.timestamp) })
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