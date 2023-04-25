use convert_case::Casing;
use discord_rich_presence::{activity::{self, Timestamps}, DiscordIpc, DiscordIpcClient};

use crate::{rank, util::current_unix_time, melee::{MeleeGameMode, stage::MeleeStage, character::MeleeCharacter}};
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
    pub stage: String,
    pub character: String,
    pub mode: String,
    pub timestamp: DiscordClientRequestTimestamp
}

impl Default for DiscordClientRequest {
    fn default() -> Self {
        DiscordClientRequest { req_type: DiscordClientRequestType::Clear, stage: "".to_string(), character: "".to_string(), mode: "".to_string(), timestamp: DiscordClientRequestTimestamp { mode: DiscordClientRequestTimestampMode::Start, timestamp: current_unix_time() } }
    }
}

impl DiscordClientRequest {
    pub fn clear() -> Self { Default::default() }
    pub fn queue() -> Self { DiscordClientRequest { req_type: DiscordClientRequestType::Queue, ..Default::default() } }
    pub fn game(stage: Option<MeleeStage>, character: Option<MeleeCharacter>, mode: MeleeGameMode, timestamp: DiscordClientRequestTimestamp) -> Self {
        DiscordClientRequest {
            req_type: DiscordClientRequestType::Game,
            stage: stage.and_then(|s| Some(s.to_string())).unwrap_or("questionmark".to_string()),
            character: character.and_then(|c| Some(c.to_string())).unwrap_or("questionmark".to_string()),
            mode: mode.to_string(),
            timestamp
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
    pub async fn queue(&mut self) {
        let rank_info = rank::get_rank_info("flcd-507").await.unwrap(); // TODO replace later

        self.client.set_activity(
            activity::Activity::new()
                .assets(
                    activity::Assets::new()
                        .large_image(&rank_info.name)
                        .large_text(format!("{} | {} ELO", rank_info.name, util::round(rank_info.elo, 2)).as_str())
                    )
                .timestamps(self.current_timestamp())
                .details("Ranked")
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
                .timestamps(if timestamp.mode == DiscordClientRequestTimestampMode::Start {Timestamps::new().start(timestamp.timestamp)} else {Timestamps::new().end(timestamp.timestamp)})
                .details(mode.as_str().to_case(convert_case::Case::Title).as_str())
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