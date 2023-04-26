use std::fmt::Display;

use num_enum::TryFromPrimitive;
use strum::{IntoEnumIterator};
use strum_macros::{Display, EnumIter};
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;

use crate::{discord::{DiscordClientRequest, DiscordClientRequestTimestamp, DiscordClientRequestTimestampMode}, util::{current_unix_time, sleep}, melee::{stage::MeleeStage, character::MeleeCharacter}};

use self::{dolphin_mem::DolphinMemory};

mod dolphin_mem;
pub mod stage;
pub mod character;

macro_rules! R13 {($offset:expr) => { 0x804db6a0 - $offset }}
const CSSDT_BUF_ADDR: u32 = 0x80005614; // reference: https://github.com/project-slippi/slippi-ssbm-asm/blob/0be644aff85986eae17e96f4c98b3342ab087d05/Online/Online.s#L31

// reference: https://github.com/akaneia/m-ex/blob/master/MexTK/include/match.h#L11-L14
#[derive(PartialEq, EnumIter, Clone, Copy)]
enum TimerMode {
    Countup = 3,
    Countdown = 2,
    Hidden = 1,
    Frozen = 0,
}

#[derive(TryFromPrimitive, Display)]
#[repr(u8)]
enum MatchmakingMode {
    Idle = 0,
    Initializing = 1,
    Matchmaking = 2,
    OpponentConnecting = 3,
    ConnectionSuccess = 4,
    ErrorEncountered = 5
}

#[derive(Debug, TryFromPrimitive, Display, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum SlippiMenuScene {
    Ranked = 0,
    Unranked = 1,
    Direct = 2,
    Teams = 3
}

pub struct MeleeClient {
    mem: DolphinMemory,
    last_payload: Option<DiscordClientRequest>
}

#[derive(PartialEq)]
pub enum MeleeScene {
    VsMode,
    UnclePunch,
    TrainingMode,
    SlippiOnline,
    SlippiCss
}

impl Display for MeleeScene {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::VsMode => write!(f, "Vs. Mode"),
            Self::UnclePunch => write!(f, "UnclePunch"),
            Self::TrainingMode => write!(f, "Training Mode"),
            Self::SlippiOnline => write!(f, "Slippi Online"),
            Self::SlippiCss => write!(f, "Character Select Screen"),
        }
    }
}

impl MeleeClient {
    pub fn new() -> Self {
        MeleeClient { mem: DolphinMemory::new(), last_payload: None }
    }

    fn get_player_port(&mut self) -> Option<u8> {self.mem.read::<u8>(R13!(0x5108)) }
    fn timer_mode(&mut self) -> TimerMode {
        const MATCH_INIT: u32 = 0x8046DB68; // first byte, reference: https://github.com/akaneia/m-ex/blob/master/MexTK/include/match.h#L136
        self.mem.read::<u8>(MATCH_INIT).and_then(|v| {
            for timer_mode in TimerMode::iter() {
                let val = timer_mode as u8;
                if v & val == val {
                    return Some(timer_mode);
                }
            }
            return None;
        }).unwrap_or(TimerMode::Countup)
    }
    fn game_time(&mut self) -> i64 { self.mem.read::<u32>(0x8046B6C8).and_then(|v| Some(v)).unwrap_or(0) as i64 }
    fn matchmaking_type(&mut self) -> Option<MatchmakingMode> { self.mem.read::<u8>(CSSDT_BUF_ADDR).and_then(|v| MatchmakingMode::try_from(v).ok()) }
    fn slippi_online_scene(&mut self) -> Option<SlippiMenuScene> { self.mem.read::<u8>(R13!(0x5060)).and_then(|v| SlippiMenuScene::try_from(v).ok()) }
    /*fn game_variant(&mut self) -> Option<MeleeGameVariant> {
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
    }*/
    fn get_melee_scene(&mut self) -> Option<MeleeScene> {
        const MAJOR_SCENE: u32 = 0x80479D30;
        const MINOR_SCENE: u32 = MAJOR_SCENE + 0x03;
        let scene_tuple = (self.mem.read::<u8>(MAJOR_SCENE).unwrap_or(0), self.mem.read::<u8>(MINOR_SCENE).unwrap_or(0));

        match scene_tuple {
            (2, 2) => Some(MeleeScene::VsMode),
            (43, 1) => Some(MeleeScene::UnclePunch),
            (28, 2) => Some(MeleeScene::TrainingMode),
            (8, 2) => Some(MeleeScene::SlippiOnline),
            (8, 0) => Some(MeleeScene::SlippiCss),
            _ => None
        }
    }
    fn get_stage(&mut self) -> Option<MeleeStage> {
        self.mem.read::<u8>( 0x8049E6C8 + 0x88 + 0x03).and_then(|v| MeleeStage::try_from(v).ok())
    }
    fn get_character(&mut self, player_id: u8) -> Option<MeleeCharacter> {
        const PLAYER_BLOCKS: [u32; 4] = [0x80453080, 0x80453F10, 0x80454DA0, 0x80455C30];
        self.mem.read::<u8>(PLAYER_BLOCKS[player_id as usize] + 0x07).and_then(|v| MeleeCharacter::try_from(v).ok())
    }

    pub fn run(&mut self, stop_signal: CancellationToken, discord_send: Sender<DiscordClientRequest>) {
        macro_rules! send_discord_msg {
            ($req:expr) => {
                if self.last_payload.is_none() || self.last_payload.as_ref().unwrap() != &$req {
                    discord_send.blocking_send($req);
                    self.last_payload = Some($req);
                }
            };
        }

        loop {
            if stop_signal.is_cancelled() {
                return;
            }
            if !self.mem.has_process() {
                println!("{}", if self.mem.find_process() { "Found" } else { "Searching process..." });
            } else {
                self.mem.check_process_running();
            }

            // self.get_game_variant();
            let gamemode_opt = self.get_melee_scene();
            if gamemode_opt.is_some() {
                let gamemode = gamemode_opt.unwrap();
                // Check if we are queueing a game
                if gamemode == MeleeScene::SlippiCss {
                    // println!("{:?}", self.get_player_port());
                    /*self.matchmaking_type().and_then(|v| {
                        println!("{}", v);
                        None::<MatchmakingMode>
                    });*/
                } else {
                    let game_time = self.game_time();
                    let timestamp = DiscordClientRequestTimestamp {
                        mode: if self.timer_mode() == TimerMode::Countdown { DiscordClientRequestTimestampMode::End } else { DiscordClientRequestTimestampMode::Start },
                        timestamp: if self.timer_mode() == TimerMode::Countdown { current_unix_time() + game_time } else { current_unix_time() - game_time }
                    };
                    let player_index = match gamemode {
                        MeleeScene::VsMode => self.get_player_port().unwrap_or(0u8),
                        _ => 0u8 // default to port 1, mostly the case in single player modes like training mode/unclepunch
                    };
                    let request = DiscordClientRequest::game(
                        self.get_stage(),
                        self.get_character(player_index),
                        gamemode,
                        timestamp
                    );
                    
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