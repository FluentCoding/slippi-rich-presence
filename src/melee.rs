use strum_macros::Display;
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;

use crate::{discord::{DiscordClientRequest, DiscordClientRequestTimestamp, DiscordClientRequestTimestampMode}, util::{current_unix_time, sleep}, melee::{stage::MeleeStage, character::MeleeCharacter}};

use self::{dolphin_mem::DolphinMemory, game::MeleeGameVariant};

mod dolphin_mem;
mod game;
pub mod stage;
pub mod character;

pub struct MeleeClient {
    mem: DolphinMemory,
    last_payload: Option<DiscordClientRequest>
}

#[derive(Display)]
pub enum MeleeGameMode {
    VsMode,
    UnclePunch,
    TrainingMode,
    SlippiOnline
}

impl MeleeClient {
    pub fn new() -> Self {
        MeleeClient { mem: DolphinMemory::new(), last_payload: None }
    }

    fn get_game_variant(&mut self) -> Option<MeleeGameVariant> {
        const GAME_ID_ADDR: u32 = 0x80000000;
        const GAME_ID_LEN: usize = 0x06;

        let game_id = self.mem.read_string::<GAME_ID_LEN>(GAME_ID_ADDR);
        if game_id.is_none() {
            return None;
        }
        return match game_id.unwrap().as_str() {
            "GALE01" => Some(MeleeGameVariant::Vanilla),
            "GTME01" => Some(MeleeGameVariant::UnclePunch),
            _ => None
        }
    }

    fn get_gamemode(&mut self) -> Option<MeleeGameMode> {
        const MAJOR_SCENE: u32 = 0x80479D30;
        const MINOR_SCENE: u32 = MAJOR_SCENE + 0x03;
        let scene_tuple = (self.mem.read::<u8>(MAJOR_SCENE).unwrap_or(0), self.mem.read::<u8>(MINOR_SCENE).unwrap_or(0));

        match scene_tuple {
            (2, 2) => Some(MeleeGameMode::VsMode),
            (43, 1) => Some(MeleeGameMode::UnclePunch),
            (28, 2) => Some(MeleeGameMode::TrainingMode),
            (8, 2) => Some(MeleeGameMode::SlippiOnline),
            _ => None
        }
    }

    fn get_stage(&mut self) -> Option<MeleeStage> {
        const STAGE_ADDRESS: u32 = 0x8049E6C8 + 0x88 + 0x03;

        let res = self.mem.read::<u8>(STAGE_ADDRESS);
        if res.is_some() {
            return MeleeStage::try_from(res.unwrap()).ok();
        }
        return None;
    }

    fn get_character(&mut self, player_id: usize) -> Option<MeleeCharacter> {
        const PLAYER_BLOCKS: [u32; 4] = [0x80453080, 0x80453F10, 0x80454DA0, 0x80455C30];
        
        let res = self.mem.read::<u8>(PLAYER_BLOCKS[player_id] + 0x07);
        if res.is_some() {
            return MeleeCharacter::try_from(res.unwrap()).ok();
        }
        return None;
    }

    pub fn run(&mut self, stop_signal: CancellationToken, discord_send: Sender<DiscordClientRequest>) {
        macro_rules! send_discord_msg {
            ($req:expr) => {
                discord_send.blocking_send($req);
                self.last_payload = Some($req);
            };
        }

        loop {
            if stop_signal.is_cancelled() {
                return;
            }
            if !self.mem.has_process() {
                println!("{}", self.mem.find_process());
            } else {
                self.mem.check_process_running();
            }

            
            self.get_game_variant();
            let gamemode = self.get_gamemode();
            if gamemode.is_some() {
                let game_time = self.mem.read::<u32>(0x8046B6C8).and_then(|v| Some(v));
                let request = DiscordClientRequest::game(
                    self.get_stage(),
                    self.get_character(0),
                    gamemode.unwrap(),
                    DiscordClientRequestTimestamp {
                        mode: DiscordClientRequestTimestampMode::Start,
                        // determine when the match started to have a more precise timestamp
                        timestamp: current_unix_time() - (game_time.unwrap_or(0)) as i64
                    }
                );
                
                if self.last_payload.is_none() || self.last_payload.as_ref().unwrap() != &request {
                    send_discord_msg!(request.clone());
                }
            } else if self.last_payload.is_some() {
                discord_send.blocking_send(DiscordClientRequest::clear());
                self.last_payload = None;
            }

            sleep(1000);
        }
    }
}