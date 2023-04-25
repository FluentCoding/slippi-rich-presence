use num_enum::TryFromPrimitive;
use strum_macros::Display;

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