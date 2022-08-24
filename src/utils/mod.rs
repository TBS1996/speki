pub mod sql;
pub mod card;
pub mod interval;
pub mod statelist;
pub mod widgets;


#[derive(Clone, PartialEq)]
pub struct CardInList{
    pub question: String,
    pub id: u32,
}
