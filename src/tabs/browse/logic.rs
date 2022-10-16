use tui::style::Style;

use crate::widgets::cardlist::CardItem;
use crate::{app::Tab, utils::statelist::StatefulList};
use crate::utils::aliases::*;
use crate::utils::misc::split_leftright;
use crate::widgets::checkbox::CheckBox;
use std::time::{SystemTime, UNIX_EPOCH};



enum Selection{
    Filter,
    Filtered,
    Selected,
}

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

pub struct Browse {
    selection: Selection,
    cardtypes: CheckBox,
    cardlimit: u32,
    filtered: StatefulList<CardItem>,
    selected: StatefulList<CardItem>,
}

impl Browse {
    pub fn new() -> Self{
        let cardlimit = 1000;
        let cardtypes = CheckBox::new("Card types".to_string(), ["Finished".to_string(), "Unfinished".to_string(), "Pending".to_string()], false);
        let selection = Selection::Filter;

        Self {
            selection,
            cardtypes,
            cardlimit,
            filtered: StatefulList::new(),
            selected: StatefulList::new(),

        }
    }


    fn is_selected(&self, widget: Selection) -> bool{
        let result = matches!(&self.selection, widget);
        result
    }


    fn navigate(&mut self, dir: Direction){
        use Selection::*;
        use Direction::*;
        match (&self.selection, dir){
            (Filter, Right) => self.selection = Selection::Filtered,
            (Filtered, Right) => self.selection = Selection::Selected,
            (Filtered, Left) => self.selection = Selection::Filter,
            (Selected, Left) => self.selection = Selection::Filtered,
            _ => {},
        }
    }



}






impl Tab for Browse {
    fn get_title(&self) -> String {
        "Browse".to_string()
    }

    fn keyhandler(&mut self, appdata: &crate::app::AppData, key: MyKey) {
        use MyKey::*;
        use Selection::*;
        if let Nav(dir) = key{
            self.navigate(dir);
            return;
        }
        match (&self.selection, key){
            (Filter, key) => self.cardtypes.keyhandler(key),
            (Filtered, key) => self.filtered.keyhandler(key),
            (Selected, key) => self.selected.keyhandler(key),
        }
    }

    fn render(
        &mut self,
        f: &mut tui::Frame<crate::MyType>,
        appdata: &crate::app::AppData,
        area: tui::layout::Rect,
    ) {
        let chunks = split_leftright([20, 50, 50], area);
        self.cardtypes.items.render(f, chunks[0], matches!(&self.selection, Selection::Filter) , &self.cardtypes.title, Style::default());
        self.filtered.render(f, chunks[1], matches!(&self.selection, Selection::Filtered), "Filtered", Style::default());
        self.selected.render(f, chunks[2], matches!(&self.selection, Selection::Selected) , "Selected", Style::default());
    }
}

use crate::MyKey;
use crate::Direction;
