pub mod aliases;
pub mod ankitemplate;
pub mod card;
pub mod epub;
pub mod incread;
pub mod interval;
pub mod libextensions;
pub mod misc;
pub mod sql;
pub mod statelist;

#[derive(Clone, PartialEq)]
pub struct CardInList {
    pub question: String,
    pub id: u32,
}

use std::fmt;
impl fmt::Display for CardInList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.question)
    }
}
