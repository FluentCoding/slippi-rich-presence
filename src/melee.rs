use num_enum::TryFromPrimitive;
use strum_macros::Display;
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;

use crate::{discord::{DiscordClientRequest, DiscordClientRequestTimestamp, DiscordClientRequestTimestampMode}, util::{current_unix_time, sleep}};

use self::dolphin_mem::DolphinMemory;
use std::{thread, time::Duration};

mod dolphin_mem;

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

#[derive(Display, TryFromPrimitive)]
#[strum(serialize_all = "lowercase")]
#[repr(u8)]
pub enum MeleeCharacter {
    DrMario = 0x16,
	Mario = 0x08,
	Luigi = 0x07,
	Bowser = 0x05,
	Peach = 0x0C,
	Yoshi = 0x11,
	DonkeyKong = 0x01,
	CaptainFalcon = 0x00,
	Ganondorf = 0x19,
	Falco = 0x14,
	Fox = 0x02,
	Ness = 0x0B,
	IceClimbers = 0x0E,
	Kirby = 0x04,
	Samus = 0x10,
	Zelda = 0x12,
    Sheik = 0x13,
	Link = 0x06,
	YoungLink = 0x15,
	Pichu = 0x18,
	Pikachu = 0x0D,
	Jigglypuff = 0x0F,
	Mewtwo = 0x0A,
	MrGameAndWatch = 0x03,
	Marth = 0x09,
	Roy = 0x17
}

#[derive(Display, TryFromPrimitive)]
#[strum(serialize_all = "lowercase")]
#[repr(u8)]
pub enum MeleeStage {
    /*FountainOfDreams = 2,
    PokemonStadium,
    PrincessPeachsCastle,
    KongoJungle,
    Brinstar,
    Corneria,
    YoshisStory,
    Onett,
    MuteCity,
    RainbowCruise,
    JungleJapes,
    GreatBay,
    HyruleTemple,
    BrinstarDepths,
    YoshisIsland,
    GreenGreens,
    Fourside,
    MushroomKingdomI,
    MushroomKingdomII,
    Venom = 22,
    PokeFloats,
    BigBlue,
    IcicleMountain,
    Icetop,
    FlatZone,
    DreamLandN64,
    YoshisIslandN64,
    KongoJungleN64,
    Battlefield,
    FinalDestination*/
    Battlefield = 0x24,
	YoshisStory = 0x0A,
	FountainOfDreams = 0x0C,
	Dreamland = 0x1C,
	FinalDestination = 0x25,
	PokemonStadium = 0x10
}

impl MeleeClient {
    pub fn new() -> Self {
        MeleeClient { mem: DolphinMemory::new(), last_payload: None }
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

            let gamemode = self.get_gamemode();
            if gamemode.is_some() {
                let game_time = self.mem.read::<u32>(0x8046B6C8).and_then(|v| Some(u32::from_be(v)));
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