pub mod sql;
pub mod card;
pub mod interval;
pub mod statelist;
pub mod widgets;
pub mod aliases;
pub mod incread;
pub mod misc;


#[derive(Clone, PartialEq)]
pub struct CardInList{
    pub question: String,
    pub id: u32,
}
