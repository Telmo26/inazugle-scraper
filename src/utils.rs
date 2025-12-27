use std::{
    ops::Deref,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU8, AtomicU16, Ordering},
    },
};

use bitflags::bitflags;

pub const BASE_URL: &str = "https://zukan.inazuma.jp";
pub const SEARCH_URL: &str = "/en/chara_list/process_form";

pub const ELEMENT_LIST: [Element; 4] = [
    Element::FIRE,
    Element::FOREST,
    Element::MOUNTAIN,
    Element::WIND,
];

pub const POSITION_LIST: [Position; 4] = [
    Position::FW, 
    Position::MF, 
    Position::DF, 
    Position::GK
];

pub const GAME_LIST: [Game; 9] = [
    Game::IE1,
    Game::IE2,
    Game::IE3,
    Game::GO1,
    Game::GO2,
    Game::GO3,
    Game::ARES,
    Game::ORION,
    Game::VR,
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Element {
    MOUNTAIN,
    FIRE,
    FOREST,
    WIND,
    NONE,
}

impl Element {
    pub fn req_str(&self) -> &str {
        match self {
            Self::MOUNTAIN => "山",
            Self::FIRE => "火",
            Self::FOREST => "林",
            Self::WIND => "風",
            Self::NONE => "",
        }
    }

    pub fn db_str(&self) -> &str {
        match self {
            Self::MOUNTAIN => "Mountain",
            Self::FIRE => "Fire",
            Self::FOREST => "Forest",
            Self::WIND => "Wind",
            Self::NONE => "Unknown",
        }
    }

    pub fn flag(self) -> ElementFlags {
        match self {
            Element::FOREST => ElementFlags::FOREST,
            Element::FIRE => ElementFlags::FIRE,
            Element::WIND => ElementFlags::WIND,
            Element::MOUNTAIN => ElementFlags::MOUNTAIN,
            Element::NONE => ElementFlags::NONE,
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, Default)]
    pub struct ElementFlags: u8 {
        const FOREST    =   0b00001;
        const FIRE      =   0b00010;
        const WIND      =   0b00100;
        const MOUNTAIN  =   0b01000;
        const NONE      =   0b10000;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Position {
    GK,
    DF,
    MF,
    FW,
    NONE,
}

impl Position {
    pub fn to_str(&self) -> &str {
        match self {
            Self::GK => "GK",
            Self::DF => "DF",
            Self::MF => "MF",
            Self::FW => "FW",
            Self::NONE => "",
        }
    }

    pub fn flag(&self) -> PositionFlags {
        match self {
            Position::GK    => PositionFlags::GK,
            Position::DF    => PositionFlags::DF,
            Position::MF    => PositionFlags::MF,
            Position::FW    => PositionFlags::FW,
            Position::NONE  => PositionFlags::NONE,
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, Default)]
    pub struct PositionFlags: u8 {
        const GK    = 0b00001;
        const DF    = 0b00010;
        const MF    = 0b00100;
        const FW    = 0b01000;
        const NONE  = 0b10000;
    }
}

pub enum Game {
    IE1,
    IE2,
    IE3,
    GO1,
    GO2,
    GO3,
    ARES,
    ORION,
    VR, // Victory Road
}

impl Game {
    pub fn req_str(&self) -> &str {
        match self {
            Game::IE1   => "IE1",
            Game::IE2   => "IE2",
            Game::IE3   => "IE3",
            Game::GO1   => "GO1",
            Game::GO2   => "GO2",
            Game::GO3   => "GO3",
            Game::ARES  => "アレス",
            Game::ORION => "オリオン",
            Game::VR    => "Vロード",
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            Game::IE1   => "IE 1",
            Game::IE2   => "IE 2",
            Game::IE3   => "IE 3",
            Game::GO1   => "GO",
            Game::GO2   => "GO Chrono Stones",
            Game::GO3   => "GO Galaxy",
            Game::ARES  => "Ares",
            Game::ORION => "Orion",
            Game::VR    => "Victory Road",
        }
    }

    pub fn flag(&self) -> GameFlags {
        match self {
            Game::IE1   => GameFlags::IE1,
            Game::IE2   => GameFlags::IE2,
            Game::IE3   => GameFlags::IE3,
            Game::GO1   => GameFlags::GO1,
            Game::GO2   => GameFlags::GO2,
            Game::GO3   => GameFlags::GO3,
            Game::ARES  => GameFlags::ARES,
            Game::ORION => GameFlags::ORION,
            Game::VR    => GameFlags::VR,
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, Default)]
    pub struct GameFlags: u16 {
        const IE1   = 0b0000_0000_0000_0001;
        const IE2   = 0b0000_0000_0000_0010;
        const IE3   = 0b0000_0000_0000_0100;
        const GO1   = 0b0000_0000_0000_1000;
        const GO2   = 0b0000_0000_0001_0000;
        const GO3   = 0b0000_0000_0010_0000;
        const ARES  = 0b0000_0000_0100_0000;
        const ORION = 0b0000_0000_1000_0000;
        const VR    = 0b0000_0001_0000_0000;
    }
}

/// This structure stores the basic information about a character that is displayed
/// on the search results.
#[derive(Debug)]
pub struct Character {
    pub number: u16,
    pub name: String,
    pub nickname: String,
    pub element: Element,
    pub position: Position,
    pub stats: Option<Stats>,
    pub page_url: String,
}

#[derive(Debug)]
pub struct Stats {
    pub kick: u8,
    pub control: u8,
    pub technique: u8,
    pub pressure: u8,
    pub physical: u8,
    pub agility: u8,
    pub intelligence: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortColumn {
    ID,
    Name,
    Kick,
    Control,
    Technique,
    Pressure,
    Physical,
    Agility,
    Intelligence,
}

pub struct Progress {
    internal: Arc<InternalProgress>,
}

impl Progress {
    pub fn new() -> Progress {
        let internal = InternalProgress {
            page: (AtomicU8::new(0), AtomicU8::new(0)),
            characters: (AtomicU16::new(0), AtomicU16::new(0)),
            pages_fetched: AtomicBool::new(false),
            characters_fetched: AtomicBool::new(false),
        };
        Progress {
            internal: Arc::new(internal),
        }
    }
}

impl Deref for Progress {
    type Target = InternalProgress;

    fn deref(&self) -> &Self::Target {
        &self.internal
    }
}

pub struct InternalProgress {
    page: (AtomicU8, AtomicU8),
    characters: (AtomicU16, AtomicU16),
    pages_fetched: AtomicBool,
    characters_fetched: AtomicBool,
}

impl InternalProgress {
    pub fn set_page_total(&self, total: u8) {
        self.page.1.store(total, Ordering::Relaxed);
    }

    pub fn inc_page(&self) {
        let prev = self.page.0.fetch_add(1, Ordering::Relaxed);
        let total = self.page.1.load(Ordering::Relaxed);

        if prev + 1 == total {
            self.pages_fetched.store(true, Ordering::Relaxed);
        }
    }

    pub fn set_char_total(&self, total: u16) {
        self.characters.1.store(total, Ordering::Relaxed);
    }

    pub fn inc_char(&self) {
        let prev = self.characters.0.fetch_add(1, Ordering::Relaxed);
        let total = self.characters.1.load(Ordering::Relaxed);

        if prev + 1 == total {
            self.characters_fetched.store(true, Ordering::Relaxed);
        }
    }

    pub fn pages(&self) -> (u8, u8) {
        let fetched = self.page.0.load(Ordering::Relaxed);
        let total = self.page.1.load(Ordering::Relaxed);
        (fetched, total)
    }

    pub fn characters(&self) -> (u16, u16) {
        let fetched = self.characters.0.load(Ordering::Relaxed);
        let total = self.characters.1.load(Ordering::Relaxed);
        (fetched, total)
    }

    pub fn pages_done(&self) -> bool {
        self.pages_fetched.load(Ordering::Relaxed)
    }

    pub fn characters_done(&self) -> bool {
        self.characters_fetched.load(Ordering::Relaxed)
    }
}

impl Clone for Progress {
    fn clone(&self) -> Self {
        Progress {
            internal: self.internal.clone(),
        }
    }
}
