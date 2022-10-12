pub mod aliases;
pub mod card;
pub mod incread;
pub mod interval;
pub mod misc;
pub mod sql;
pub mod statelist;

#[derive(Clone, PartialEq)]
pub struct CardInList {
    pub question: String,
    pub id: u32,
}
