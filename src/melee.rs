use strum::{IntoEnumIterator};
use strum_macros::{Display, EnumIter};
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;

use crate::{discord::{DiscordClientRequest, DiscordClientRequestTimestamp, DiscordClientRequestTimestampMode}, util::{current_unix_time, sleep}, melee::{stage::MeleeStage, character::MeleeCharacter}};

use self::{dolphin_mem::DolphinMemory, game::MeleeGameVariant};

mod dolphin_mem;
mod game;
pub mod stage;
pub mod character;

// reference: https://github.com/akaneia/m-ex/blob/master/MexTK/include/match.h#L11-L14
#[derive(PartialEq, EnumIter, Clone, Copy)]
enum TimerMode {
    Countup = 0x03,
    Countdown = 0x02,
    Hidden = 0x01,
    Frozen = 0x00,
}

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

    fn timer_mode(&mut self) -> TimerMode {
        const MATCH_INIT: u32 = 0x8046DB68; // first byte, reference: https://github.com/akaneia/m-ex/blob/master/MexTK/include/match.h#L136
        let req = self.mem.read::<u8>(MATCH_INIT);
        if req.is_none() {
            return TimerMode::Countup;
        }

        let data = req.unwrap();
        for timer_mode in TimerMode::iter() {
            let val = timer_mode as u8;
            if data & (val as u8) == (val as u8) {
                return timer_mode;
            }
        }
        return TimerMode::Countup; // should never reach but countup is the default
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

        let req = self.mem.read::<u8>(STAGE_ADDRESS);
        if req.is_some() {
            return MeleeStage::try_from(req.unwrap()).ok();
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

            
            // self.get_game_variant();
            let gamemode = self.get_gamemode();
            if gamemode.is_some() {
                let game_time = self.mem.read::<u32>(0x8046B6C8).and_then(|v| Some(v)).unwrap_or(0) as i64;
                let timestamp = DiscordClientRequestTimestamp {
                    mode: if self.timer_mode() == TimerMode::Countdown { DiscordClientRequestTimestampMode::End } else { DiscordClientRequestTimestampMode::Start },
                    timestamp: if self.timer_mode() == TimerMode::Countdown { current_unix_time() + game_time } else { current_unix_time() - game_time }
                };
                let request = DiscordClientRequest::game(
                    self.get_stage(),
                    self.get_character(0),
                    gamemode.unwrap(),
                    timestamp
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