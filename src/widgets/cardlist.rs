use std::fmt;
use std::fmt::Display;

use crate::utils::aliases::*;

#[derive(Debug, Clone)]
pub struct CardItem {
    pub question: String,
    pub id: CardID,
}

impl Display for CardItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.question)
    }
}
