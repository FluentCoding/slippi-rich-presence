use std::fmt::Display;

use num_enum::TryFromPrimitive;

#[derive(Debug, PartialEq, Copy, Clone, TryFromPrimitive)]
#[repr(u8)]
pub enum MeleeStage {
    // Dummy, (unused)
    // Test, (unused)
    Castle = 2,
    Rcruise,
    Kongo,
    Garden,
    Greatbay,
    Shrine,
    Zebes,
    Kraid,
    Story,
    Yoster,
    Izumi,
    Greens,
    Corneria,
    Venom,
    PStad,
    Pura,
    MuteCity,
    BigBlue,
    Onett,
    Fourside,
    IceMt,
    // IceTop, (unused)
    Mk1 = 24,
    Mk2,
    Akaneia,
    FlatZone,
    OldPu,
    OldStory,
    OldKongo,
    // AdvKraid, (unused)
    // AdvShrine, (unused)
    // AdvZr, (unused)
    // AdvBr, (unused)
    // AdvTe, (unused)
    Battle = 36,
    FD
}

impl Display for MeleeStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            // Self::Dummy => write!(f, "Dummy"),
            // Self::Test => write!(f, "Test"),
            Self::Castle => write!(f, "Princess Peach's Castle"),
            Self::Rcruise => write!(f, "Rainbow Cruise"),
            Self::Kongo => write!(f, "Kongo Jungle"),
            Self::Garden => write!(f, "Jungle Japes"),
            Self::Greatbay => write!(f, "Great Bay"),
            Self::Shrine => write!(f, "Temple"),
            Self::Zebes => write!(f, "Brinstar"),
            Self::Kraid => write!(f, "Brinstar Depths"),
            Self::Story => write!(f, "Yoshi's Story"),
            Self::Yoster => write!(f, "Yoshi's Island"),
            Self::Izumi => write!(f, "Fountain of Dreams"),
            Self::Greens => write!(f, "Green Greens"),
            Self::Corneria => write!(f, "Corneria"),
            Self::Venom => write!(f, "Venom"),
            Self::PStad => write!(f, "Pokemon Stadium"),
            Self::Pura => write!(f, "Poke Floats"),
            Self::MuteCity => write!(f, "Mute City"),
            Self::BigBlue => write!(f, "Big Blue"),
            Self::Onett => write!(f, "Onett"),
            Self::Fourside => write!(f, "Fourside"),
            Self::IceMt => write!(f, "IcicleMountain"),
            // Self::IceTop => write!(f, "Icetop"),
            Self::Mk1 => write!(f, "Mushroom Kingdom"),
            Self::Mk2 => write!(f, "Mushroom Kingdom II"),
            Self::Akaneia => write!(f, "Akaneia"),
            Self::FlatZone => write!(f, "Flat Zone"),
            Self::OldPu => write!(f, "Dream Land"),
            Self::OldStory => write!(f, "Yoshi's Island (N64)"),
            Self::OldKongo => write!(f, "Kongo Jungle (N64)"),
            Self::Battle => write!(f, "Battlefield"),
            Self::FD => write!(f, "Final Destination")
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct OptionalMeleeStage(pub Option<MeleeStage>);
impl OptionalMeleeStage {
	pub fn as_discord_resource(&self) -> String {
		self.0.as_ref().and_then(|c| Some(format!("stage{}", (*c) as u8))).unwrap_or("questionmark".to_string())
	}
}
impl Display for OptionalMeleeStage {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match &(*self).0 {
			Some(v) => write!(f, "{}", v),
			_ => write!(f, "Unknown stage")
		}
	}
}