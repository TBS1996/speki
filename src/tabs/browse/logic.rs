use rusqlite::Connection;
use std::collections::HashSet;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::Style;

use crate::app::AppData;
use crate::utils::aliases::*;
use crate::utils::card::CardType;
use crate::utils::misc::{split_leftright, split_leftright_by_percent, split_updown_by_percent};
use crate::utils::sql::fetch::CardQuery;
use crate::widgets::cardlist::CardItem;
use crate::widgets::checkbox::CheckBox;
use crate::widgets::numeric_input::NumPut;
use crate::widgets::optional_bool_filter::{FilterSetting, OptCheckBox};
use crate::{app::Tab, utils::statelist::StatefulList};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

enum Selection {
    CardType,
    CardStatus,
    Numfilters,
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
    pub fn new(conn: &Arc<Mutex<Connection>>) -> Self {
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
        let cardstatus = OptCheckBox::new(
            "Status".to_string(),
            ["Resolved".to_string(), "Suspended".to_string()],
        );
        let numfilters = NumPut::new(
            String::from("Numeric filters"),
            [
                ("Max stability".to_string(), None),
                ("Min stability".to_string(), None),
                ("Max strength".to_string(), Some(100)),
                ("Min strength".to_string(), None),
            ],
        );
        let selected_ids = HashSet::new();

        let mut myself = Self {
            selection,
            cardtypes,
            cardstatus,
            numfilters,
            cardlimit,
            filtered: StatefulList::new(),
            selected: StatefulList::new(),
            selected_ids,
        };
        myself.apply_filter(conn);
        myself
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
            _ => {}
        }
        match self.cardstatus.items.items[1].filter {
            FilterSetting::TruePass => query = query.suspended(true),
            FilterSetting::FalsePass => query = query.suspended(false),
            _ => {}
        }

        if let Some(val) = self.numfilters.items.items[0].input.get_value() {
            query = query.max_stability(val);
        }
        if let Some(val) = self.numfilters.items.items[1].input.get_value() {
            query = query.minimum_stability(val);
        }
        if let Some(val) = self.numfilters.items.items[2].input.get_value() {
            query = query.max_strength(val as f32 / 100.);
        }
        if let Some(val) = self.numfilters.items.items[3].input.get_value() {
            query = query.minimum_strength(val as f32 / 100.);
        }

        let items = query
            .fetch_carditems(conn)
            .into_iter()
            .filter(|card| !self.selected_ids.contains(&card.id))
            .collect();

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
            (CardStatus, Down) => self.selection = Selection::Numfilters,
            (Numfilters, Right) => self.selection = Selection::Filtered,
            (Numfilters, Up) => self.selection = Selection::CardStatus,
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
                self.cardtypes.items.keyhandler(key.clone());
                if key == Enter || key == Char(' ') {
                    self.apply_filter(&appdata.conn);
                }
            }
            (CardStatus, key) => {
                self.cardstatus.items.keyhandler(key.clone());
                if key == Left || key == Right || key == Char('l') || key == Char('h') {
                    self.apply_filter(&appdata.conn);
                }
            }
            (Filtered, Enter) | (Filtered, Char(' ')) => {
                if let Some(item) = self.filtered.take_selected_item() {
                    if !self.selected_ids.contains(&item.id) {
                        self.selected_ids.insert(item.id);
                        self.selected.items.push(item);
                    }
                }
            }
            (Filtered, key) => self.filtered.keyhandler(key),

            (Selected, Enter) | (Selected, Char(' ')) => {
                if let Some(item) = self.selected.take_selected_item() {
                    self.selected_ids.remove(&item.id);
                    self.filtered.items.push(item);
                }
            }
            (Selected, key) => self.selected.keyhandler(key),
            (Numfilters, key) => {
                self.numfilters.items.keyhandler(key);
                self.apply_filter(&appdata.conn);
            }
        }
    }

    fn render(
        &mut self,
        f: &mut tui::Frame<crate::MyType>,
        _appdata: &crate::app::AppData,
        area: tui::layout::Rect,
    ) {
        let chunks = split_leftright(
            [
                Constraint::Length(27),
                Constraint::Percentage(40),
                Constraint::Percentage(40),
            ],
            area,
        );
        let filters = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Max(5),
                    Constraint::Max(4),
                    Constraint::Length(20),
                ]
                .as_ref(),
            )
            .split(chunks[0]);

        self.cardtypes.items.render(
            f,
            filters[0],
            matches!(&self.selection, Selection::CardType),
            &self.cardtypes.title,
            Style::default(),
        );
        self.cardstatus.items.render(
            f,
            filters[1],
            matches!(&self.selection, Selection::CardStatus),
            &self.cardstatus.title,
            Style::default(),
        );
        self.numfilters.render(
            f,
            filters[2],
            matches!(&self.selection, Selection::Numfilters),
        );
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
