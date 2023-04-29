use std::{fmt::Display};

use num_enum::TryFromPrimitive;
use strum::{IntoEnumIterator};
use strum_macros::{Display, EnumIter};
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;

use crate::{discord::{DiscordClientRequest, DiscordClientRequestType, DiscordClientRequestTimestamp, DiscordClientRequestTimestampMode}, util::{current_unix_time, sleep}, melee::{stage::MeleeStage, character::MeleeCharacter}, config::{CONFIG}};

use self::{dolphin_mem::{DolphinMemory, MSRBAccess}, msrb::MSRBOffset};

mod dolphin_mem;
mod msrb;
pub mod stage;
pub mod character;

macro_rules! R13 {($offset:expr) => { 0x804db6a0 - $offset }}

// reference: https://github.com/akaneia/m-ex/blob/master/MexTK/include/match.h#L11-L14
#[derive(PartialEq, EnumIter, Clone, Copy)]
enum TimerMode {
    Countup = 3,
    Countdown = 2,
    Hidden = 1,
    Frozen = 0,
}

#[derive(TryFromPrimitive, Display, Debug)]
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
    last_payload: DiscordClientRequest
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
            Self::UnclePunch => write!(f, "UnclePunch Training Mode"),
            Self::TrainingMode => write!(f, "Training Mode"),
            Self::SlippiOnline => write!(f, "Slippi Online"),
            Self::SlippiCss => write!(f, "Character Select Screen"),
        }
    }
}

impl MeleeClient {
    pub fn new() -> Self {
        MeleeClient { mem: DolphinMemory::new(), last_payload: DiscordClientRequest::clear() }
    }

    // fn osb_data_ptr(&mut self) -> Option<u32> { self.pointer_indirection(, amount)}

    // Fetching functions
    fn get_player_port(&mut self) -> Option<u8> { self.mem.read::<u8>(R13!(0x5108)) }
    fn get_slippi_player_port(&mut self) -> Option<u8> { self.mem.read_msrb(MSRBOffset::MsrbLocalPlayerIndex) }
    fn get_player_connect_code(&mut self, port: u8) -> Option<String> {
        const PLAYER_CONNECTCODE_OFFSETS: [MSRBOffset; 4] = [MSRBOffset::MsrbP1ConnectCode, MSRBOffset::MsrbP2ConnectCode, MSRBOffset::MsrbP3ConnectCode, MSRBOffset::MsrbP4ConnectCode];
        self.mem.read_msrb_string_shift_jis::<10>(PLAYER_CONNECTCODE_OFFSETS[port as usize])
    }
    fn get_character_selection(&mut self, port: u8) -> Option<MeleeCharacter> {
        // 0x04 = character, 0x05 = skin (reference: https://github.com/bkacjios/m-overlay/blob/master/source/modules/games/GALE01-2.lua#L199-L202)
        const PLAYER_SELECTION_BLOCKS: [u32; 4] = [0x8043208B, 0x80432093, 0x8043209B, 0x804320A3];
        self.mem.read::<u8>(PLAYER_SELECTION_BLOCKS[port as usize] + 0x04).and_then(|v| MeleeCharacter::try_from(v).ok())
    }
    fn timer_mode(&mut self) -> TimerMode {
        const MATCH_INIT: u32 = 0x8046DB68; // first byte, reference: https://github.com/akaneia/m-ex/blob/master/MexTK/include/match.h#L136
        self.mem.read::<u8>(MATCH_INIT).and_then(|v| {
            for timer_mode in TimerMode::iter() {
                let val = timer_mode as u8;
                if v & val == val {
                    return Some(timer_mode);
                }
            }
            None
        }).unwrap_or(TimerMode::Countup)
    }
    fn game_time(&mut self) -> i64 { self.mem.read::<u32>(0x8046B6C8).and_then(|v| Some(v)).unwrap_or(0) as i64 }
    fn matchmaking_type(&mut self) -> Option<MatchmakingMode> {
        self.mem.read_msrb::<u8>(MSRBOffset::MsrbConnectionState).and_then(|v| MatchmakingMode::try_from(v).ok())
    }
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
    fn get_connect_code(&mut self) {
        // reference: https://github.com/project-slippi/slippi-ssbm-asm/blob/9c36ffc5e4787c6caadfb12727c5fcff07d64642/Online/Online.s#L376-L378
        const OSB_APP_STATE: u32 = 0;
        const OSB_PLAYER_NAME: u32 = OSB_APP_STATE + 1;
        const OSB_CONNECT_CODE: u32 = OSB_PLAYER_NAME + 31;
    }
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
                if self.last_payload != $req {
                    let _ = discord_send.blocking_send($req);
                    self.last_payload = $req;
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

            CONFIG.with_ref(|c| {
                // self.get_game_variant();
                let gamemode_opt = self.get_melee_scene();
                // println!(c)
                if gamemode_opt.is_some() {
                    let gamemode = gamemode_opt.unwrap();

                    // Check if we are queueing a game
                    if gamemode == MeleeScene::SlippiCss && c.slippi.show_queueing {
                        match self.matchmaking_type() {
                            Some(MatchmakingMode::Initializing) | Some(MatchmakingMode::Matchmaking) => {
                                let port_op = self.get_player_port();
                                if !port_op.is_none() {
                                    let port = port_op.unwrap();
                                    let character = self.get_character_selection(port);
                                    let request = DiscordClientRequest::queue(
                                        self.slippi_online_scene(),
                                        character
                                    );
                                    send_discord_msg!(request.clone());
                                }
                            }
                            Some(_) => {
                                send_discord_msg!(DiscordClientRequest::clear());
                            }, // sometimes it's none, probably because the pointer indirection changes during the asynchronous memory requests
                            _ => {}
                        }
                    // Else, we want to see if the current game mode is enabled in the config
                    } else if gamemode != MeleeScene::SlippiCss && match gamemode {
                        MeleeScene::SlippiOnline => {
                            self.slippi_online_scene().and_then(|s| Some(match s {
                                SlippiMenuScene::Ranked => c.slippi.ranked.enabled,
                                SlippiMenuScene::Unranked => c.slippi.unranked.enabled,
                                SlippiMenuScene::Direct => c.slippi.direct.enabled,
                                SlippiMenuScene::Teams => c.slippi.teams.enabled,
                            })).unwrap_or(true)
                        },
                        MeleeScene::UnclePunch => c.uncle_punch.enabled,
                        MeleeScene::TrainingMode => c.training_mode.enabled,
                        MeleeScene::VsMode => c.vs_mode.enabled,
                        _ => true
                    } {
                        let game_time = self.game_time();
                        let timestamp = if c.global.show_in_game_time {
                            DiscordClientRequestTimestamp {
                                mode: match self.timer_mode() {
                                    TimerMode::Countdown => DiscordClientRequestTimestampMode::End,
                                    TimerMode::Frozen => DiscordClientRequestTimestampMode::Static,
                                    _ => DiscordClientRequestTimestampMode::Start
                                },
                                timestamp: if self.timer_mode() == TimerMode::Countdown { current_unix_time() + game_time } else { current_unix_time() - game_time }
                            }
                        } else {
                            DiscordClientRequestTimestamp::none()
                        };
                        let player_index = match gamemode {
                            MeleeScene::VsMode => self.get_player_port(),
                            MeleeScene::SlippiOnline => self.get_slippi_player_port(),
                            _ => Some(0u8) // default to port 1, mostly the case in single player modes like training mode/unclepunch
                        }.unwrap_or(0u8);
                        let request = DiscordClientRequest::game(
                            self.get_stage(),
                            if c.global.show_in_game_character { self.get_character(player_index) } else { None },
                            gamemode,
                            timestamp
                        );
                        
                        send_discord_msg!(request.clone());
                    } else {
                        send_discord_msg!(DiscordClientRequest::clear());
                    }
                } else if self.last_payload.req_type != DiscordClientRequestType::Clear {
                    send_discord_msg!(DiscordClientRequest::clear());
                }
            });

            sleep(1000);
        }
    }
}