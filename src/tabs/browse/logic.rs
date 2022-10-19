use rusqlite::Connection;
use tui::layout::{Direction, Constraint, Layout};
use tui::style::Style;
use std::collections::HashSet;

use crate::utils::aliases::*;
use crate::utils::card::CardType;
use crate::utils::misc::{split_leftright, split_updown};
use crate::utils::sql::fetch::CardQuery;
use crate::widgets::cardlist::CardItem;
use crate::widgets::checkbox::CheckBox;
use crate::widgets::numeric_input::NumPut;
use crate::widgets::optional_bool_filter::{OptCheckBox, FilterSetting};
use crate::{app::Tab, utils::statelist::StatefulList};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

enum Selection {
    CardType,
    CardStatus,
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
    cardstatus: OptCheckBox,
    numfilters: NumPut,
    cardlimit: u32,
    filtered: StatefulList<CardItem>,
    selected: StatefulList<CardItem>,
    selected_ids: HashSet<CardID>,
}

impl Browse {
    pub fn new() -> Self {
        let cardlimit = 10000;
        let cardtypes = CheckBox::new(
            "Card types".to_string(),
            [
                "Finished".to_string(),
                "Unfinished".to_string(),
                "Pending".to_string(),
            ],
            false,
        );
        let selection = Selection::Filtered;
        let cardstatus = OptCheckBox::new("Status".to_string(), ["Resolved".to_string(), "Suspended".to_string()]);
        let numfilters = NumPut::new(String::from("My title"), ["test1".to_string(), "test2.to_string".to_string()]);
        let selected_ids = HashSet::new();
        

        Self {
            selection,
            cardtypes,
            cardstatus,
            numfilters,
            cardlimit,
            filtered: StatefulList::new(),
            selected: StatefulList::new(),
            selected_ids,
        }
    }

    fn apply_filter(&mut self, conn: &Arc<Mutex<Connection>>) {
        let mut typevec = vec![];

        if self.cardtypes.items.items[0].filter {
            typevec.push(CardType::Finished);
        }
        if self.cardtypes.items.items[1].filter {
            typevec.push(CardType::Unfinished);
        }
        if self.cardtypes.items.items[2].filter {
            typevec.push(CardType::Pending);
        }
        let mut query = CardQuery::default().cardtype(typevec);
        
        match self.cardstatus.items.items[0].filter {
            FilterSetting::TruePass => query = query.resolved(true),
            FilterSetting::FalsePass => query = query.resolved(false),
            _ => {},
        }
        match self.cardstatus.items.items[1].filter {
            FilterSetting::TruePass => query = query.suspended(true),
            FilterSetting::FalsePass => query = query.suspended(false),
            _ => {},
        }

        let items = query.fetch_carditems(conn).into_iter().filter(|card| !self.selected_ids.contains(&card.id)).collect();

        self.filtered = StatefulList::with_items(items);
    }

    fn navigate(&mut self, dir: NavDir) {
        use NavDir::*;
        use Selection::*;
        match (&self.selection, dir) {
            (CardType, Right) => self.selection = Selection::Filtered,
            (CardType, Down) => self.selection = Selection::CardStatus,
            (Filtered, Right) => self.selection = Selection::Selected,
            (Filtered, Left) => self.selection = Selection::CardType,
            (Selected, Left) => self.selection = Selection::Filtered,
            (CardStatus, Right) => self.selection = Selection::Filtered,
            (CardStatus, Up) => self.selection = Selection::CardType,
            _ => {}
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
        if let Nav(dir) = key {
            self.navigate(dir);
            return;
        }
        match (&self.selection, key) {
            (CardType, key) => {
                self.cardtypes.keyhandler(key.clone());
                if key == Enter || key == Char(' '){
                    self.apply_filter(&appdata.conn);
                }
            },
            (CardStatus, key) => {
                self.cardstatus.keyhandler(key.clone());
                if key == Left || key == Right || key == Char('l') || key == Char('h'){
                    self.apply_filter(&appdata.conn);
                }
            },
            (Filtered, Enter) | (Filtered, Char(' ')) => {
                if let Some(item) = self.filtered.take_selected_item(){
                    if !self.selected_ids.contains(&item.id){
                        self.selected_ids.insert(item.id);
                        self.selected.items.push(item);
                    }
                }
            },
            (Filtered, key) => self.filtered.keyhandler(key),
            
            (Selected, Enter) | (Selected, Char(' ')) => {
                if let Some(item) = self.selected.take_selected_item(){
                    self.selected_ids.remove(&item.id);
                    self.filtered.items.push(item);
                }
            },
            (Selected, key) => self.selected.keyhandler(key),
        }
    }

    fn render(
        &mut self,
        f: &mut tui::Frame<crate::MyType>,
        _appdata: &crate::app::AppData,
        area: tui::layout::Rect,
    ) {
        let chunks = split_leftright([20, 50, 50], area);
        let filters = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                         Constraint::Max(5),
                         Constraint::Max(4),
                         Constraint::Length(20),
            ].as_ref())
            .split(chunks[0]);

        self.cardtypes.items.render(
            f,
            filters[0],
            matches!(&self.selection, Selection::CardType),
            &self.cardtypes.title,
            Style::default(),
        );
        self.cardstatus.items.render(f, filters[1], matches!(&self.selection, Selection::CardStatus), &self.cardstatus.title, Style::default());
        self.filtered.render(
            f,
            chunks[1],
            matches!(&self.selection, Selection::Filtered),
            "Filtered",
            Style::default(),
        );
        self.selected.render(
            f,
            chunks[2],
            matches!(&self.selection, Selection::Selected),
            "Selected",
            Style::default(),
        );
    }
}

use crate::Direction as NavDir;
use crate::MyKey;
