use crate::app::Tab;
use crate::utils::aliases::*;
use crate::utils::misc::split_leftright;
use crate::widgets::checkbox::CheckBox;
use std::time::{SystemTime, UNIX_EPOCH};
/*

maybe and/or logic :O

e.g.

suspended and strength less than 0.5 or resolved and stability over 50
   */

enum Filter {
    Suspended(bool),
    Resolved(bool),
    Finished(bool),
    Unfinished(bool),
    Reviewqty((u32, u32)),
    StrengthRange((f32, f32)),
    Minstability(u32),
    Maxstability(u32),
    Contains(String),
}

struct Browse {
    cardtypes: CheckBox,
    cardlimit: u32,
    all: Vec<CardID>,
    filtered: Vec<CardID>,
    selected: Vec<CardID>,
}

impl Browse {
    pub fn new() -> Self{
        let cardlimit = 1000;
        let cardtypes = CheckBox::new("Card types".to_string(), ["Finished".to_string(), "Unfinished".to_string(), "Pending".to_string()], false);
    }
}

impl Tab for Browse {
    fn get_title(&self) -> String {
        "Browse".to_string()
    }

    fn keyhandler(&mut self, appdata: &crate::app::AppData, key: crate::MyKey) {}

    fn render(
        &mut self,
        f: &mut tui::Frame<crate::MyType>,
        appdata: &crate::app::AppData,
        area: tui::layout::Rect,
    ) {
        let chunks = split_leftright([50, 50], area);
    }
}
