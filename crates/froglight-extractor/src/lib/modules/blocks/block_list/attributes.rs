#![allow(dead_code)]

use serde::Serialize;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub(crate) enum BlockType {
    #[default]
    Block,
    Leaves,
    Bed,
    ShulkerBox,
    NetherStem,
    Log,
    Piston,
    StainedGlass,
    WoodenButton,
    StoneButton,
    Bamboo,
    FlowerPot,
    Candle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PistonBehavior {
    Normal,
    Destroy,
    Block,
    Ignore,
    PushOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SoundGroup {
    Stone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Instrument {
    Harp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum MapColor {
    Clear,
    PaleGreen,
    PaleYellow,
    WhiteGray,
    BrightRed,
    PalePurple,
    IronGray,
    DarkGreen,
    White,
    LightBlueGray,
    DirtBrown,
    StoneGray,
    WaterBlue,
    OakTan,
    OffWhite,
    Orange,
    Magenta,
    LightBlue,
    Yellow,
    Lime,
    Pink,
    Gray,
    LightGray,
    Cyan,
    Purple,
    Blue,
    Brown,
    Green,
    Red,
    Black,
    Gold,
    DiamondBlue,
    LapisBlue,
    EmeraldGreen,
    SpruceBrown,
    DarkRed,
    TerracottaWhite,
    TerracottaOrange,
    TerracottaMagenta,
    TerracottaLightBlue,
    TerracottaYellow,
    TerracottaLime,
    TerracottaPink,
    TerracottaGray,
    TerracottaLightGray,
    TerracottaCyan,
    TerracottaPurple,
    TerracottaBlue,
    TerracottaBrown,
    TerracottaGreen,
    TerracottaRed,
    TerracottaBlack,
    DullRed,
    DullPink,
    DarkCrimson,
    Teal,
    DarkAqua,
    DarkDullPink,
    BrightTeal,
    DeepslateGray,
    RawIronPink,
    LichenGreen,
}

#[allow(clippy::unreadable_literal)]
impl MapColor {
    fn value(self) -> (u32, u32) {
        match self {
            MapColor::Clear => (0, 0),
            MapColor::PaleGreen => (1, 8368696),
            MapColor::PaleYellow => (2, 16247203),
            MapColor::WhiteGray => (3, 0xC7C7C7),
            MapColor::BrightRed => (4, 0xFF0000),
            MapColor::PalePurple => (5, 0xA0A0FF),
            MapColor::IronGray => (6, 0xA7A7A7),
            MapColor::DarkGreen => (7, 31744),
            MapColor::White => (8, 0xFFFFFF),
            MapColor::LightBlueGray => (9, 10791096),
            MapColor::DirtBrown => (10, 9923917),
            MapColor::StoneGray => (11, 0x707070),
            MapColor::WaterBlue => (12, 0x4040FF),
            MapColor::OakTan => (13, 9402184),
            MapColor::OffWhite => (14, 0xFFFCF5),
            MapColor::Orange => (15, 14188339),
            MapColor::Magenta => (16, 11685080),
            MapColor::LightBlue => (17, 6724056),
            MapColor::Yellow => (18, 0xE5E533),
            MapColor::Lime => (19, 8375321),
            MapColor::Pink => (20, 15892389),
            MapColor::Gray => (21, 0x4C4C4C),
            MapColor::LightGray => (22, 0x999999),
            MapColor::Cyan => (23, 5013401),
            MapColor::Purple => (24, 8339378),
            MapColor::Blue => (25, 3361970),
            MapColor::Brown => (26, 6704179),
            MapColor::Green => (27, 6717235),
            MapColor::Red => (28, 0x993333),
            MapColor::Black => (29, 0x191919),
            MapColor::Gold => (30, 16445005),
            MapColor::DiamondBlue => (31, 6085589),
            MapColor::LapisBlue => (32, 4882687),
            MapColor::EmeraldGreen => (33, 55610),
            MapColor::SpruceBrown => (34, 8476209),
            MapColor::DarkRed => (35, 0x700200),
            MapColor::TerracottaWhite => (36, 13742497),
            MapColor::TerracottaOrange => (37, 10441252),
            MapColor::TerracottaMagenta => (38, 9787244),
            MapColor::TerracottaLightBlue => (39, 7367818),
            MapColor::TerracottaYellow => (40, 12223780),
            MapColor::TerracottaLime => (41, 6780213),
            MapColor::TerracottaPink => (42, 10505550),
            MapColor::TerracottaGray => (43, 0x392923),
            MapColor::TerracottaLightGray => (44, 8874850),
            MapColor::TerracottaCyan => (45, 0x575C5C),
            MapColor::TerracottaPurple => (46, 8014168),
            MapColor::TerracottaBlue => (47, 4996700),
            MapColor::TerracottaBrown => (48, 4993571),
            MapColor::TerracottaGreen => (49, 5001770),
            MapColor::TerracottaRed => (50, 9321518),
            MapColor::TerracottaBlack => (51, 2430480),
            MapColor::DullRed => (52, 12398641),
            MapColor::DullPink => (53, 9715553),
            MapColor::DarkCrimson => (54, 6035741),
            MapColor::Teal => (55, 1474182),
            MapColor::DarkAqua => (56, 3837580),
            MapColor::DarkDullPink => (57, 5647422),
            MapColor::BrightTeal => (58, 1356933),
            MapColor::DeepslateGray => (59, 0x646464),
            MapColor::RawIronPink => (60, 14200723),
            MapColor::LichenGreen => (61, 8365974),
        }
    }
}
