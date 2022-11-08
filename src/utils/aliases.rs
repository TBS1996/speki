pub type TopicID = u32;
pub type CardID = u32;
pub type IncID = u32;

pub type ModelID = u64;
pub type NoteID = u64;
pub type AnkiCID = u64;
pub type UnixTime = u64;

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct Pos {
    pub x: u16,
    pub y: u16,
}

impl Pos {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}
