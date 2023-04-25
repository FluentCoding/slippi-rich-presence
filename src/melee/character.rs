use num_enum::TryFromPrimitive;
use strum_macros::Display;

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