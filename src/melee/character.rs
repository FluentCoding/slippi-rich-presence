use std::fmt::Display;

use num_enum::TryFromPrimitive;

#[derive(TryFromPrimitive)]
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

impl Display for MeleeCharacter {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match *self {
			Self::DrMario => write!(f, "Dr. Mario"),
			Self::Mario => write!(f, "Mario"),
			Self::Luigi => write!(f, "Luigi"),
			Self::Bowser => write!(f, "Bowser"),
			Self::Peach => write!(f, "Peach"),
			Self::Yoshi => write!(f, "Yoshi"),
			Self::DonkeyKong => write!(f, "Donkey Kong"),
			Self::CaptainFalcon => write!(f, "Captain Falcon"),
			Self::Ganondorf => write!(f, "Ganondorf"),
			Self::Falco => write!(f, "Falco"),
			Self::Fox => write!(f, "Fox"),
			Self::Ness => write!(f, "Ness"),
			Self::IceClimbers => write!(f, "Ice Climbers"),
			Self::Kirby => write!(f, "Kirby"),
			Self::Samus => write!(f, "Samus"),
			Self::Zelda => write!(f, "Zelda"),
			Self::Sheik => write!(f, "Sheik"),
			Self::Link => write!(f, "Link"),
			Self::YoungLink => write!(f, "Young Link"),
			Self::Pichu => write!(f, "Pichu"),
			Self::Pikachu => write!(f, "Pikachu"),
			Self::Jigglypuff => write!(f, "Jigglypuff"),
			Self::Mewtwo => write!(f, "Mewtwo"),
			Self::MrGameAndWatch => write!(f, "Mr. Game & Watch"),
			Self::Marth => write!(f, "Marth"),
			Self::Roy => write!(f, "Roy"),
		}
	}
}