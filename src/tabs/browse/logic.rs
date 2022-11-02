use rusqlite::Connection;
use std::collections::HashSet;
use std::fmt::Display;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::widgets::Clear;

use crate::app::{AppData, PopUp, Widget};
use crate::utils::aliases::*;
use crate::utils::card::CardItem;
use crate::utils::card::{Card, CardType, CardView};
use crate::utils::misc::{centered_rect, split_leftright, split_updown_by_percent, View};
use crate::utils::sql::fetch::{get_highest_pos, is_pending, CardQuery};
use crate::utils::sql::update::{set_suspended, update_position};
use crate::utils::statelist::KeyHandler;
use crate::widgets::checkbox::CheckBoxItem;
use crate::widgets::find_card::{CardPurpose, FindCardWidget};
use crate::widgets::newchild::{AddChildWidget, Purpose};
use crate::widgets::numeric_input::NumItem;
use crate::widgets::optional_bool_filter::{FilterSetting, OptItem};
use crate::{app::Tab, utils::statelist::StatefulList};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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

pub enum FilterItem {
    Checkboxitem(CheckBoxItem),
    Optitem(OptItem),
    Numitem(NumItem),
    Search(InputItem),
}

pub struct InputItem {
    prompt: String,
    searchfield: String,
    writing: bool,
    max: Option<u32>,
}

impl Display for InputItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match (self.writing, self.searchfield.is_empty()) {
            (false, false) => self.searchfield.clone(),
            (false, true) => "~".to_string(),
            (true, _) => {
                let mut field = self.searchfield.clone();
                field.push('_');
                field
            }
        };
        write!(f, "{}: {}", self.prompt, val)
    }
}

impl KeyHandler for InputItem {
    fn keyhandler(&mut self, _appdata: &AppData, key: MyKey) -> bool {
        use MyKey::*;
        if self.writing {
            match key {
                Char(c) => {
                    if self.max.is_none() || self.max.unwrap() > self.searchfield.len() as u32 {
                        self.searchfield.push(c);
                    }
                }
                Enter => self.writing = false,
                Esc => {
                    self.searchfield.clear();
                    self.writing = false;
                }
                Backspace => {
                    self.searchfield.pop();
                }
                _ => {}
            }
            true
        } else {
            if let MyKey::Enter = key {
                self.writing = true;
                return true;
            }
            false
        }
    }
}

impl InputItem {
    pub fn new(prompt: String, max: Option<u32>) -> Self {
        Self {
            prompt,
            searchfield: String::new(),
            writing: false,
            max,
        }
    }
}

impl Display for FilterItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match self {
            Self::Checkboxitem(val) => format!("{}", val),
            Self::Optitem(val) => format!("{}", val),
            Self::Numitem(val) => format!("{}", val),
            Self::Search(val) => format!("{}", val),
        };
        write!(f, "{}", val)
    }
}

impl KeyHandler for FilterItem {
    fn keyhandler(&mut self, appdata: &AppData, key: MyKey) -> bool {
        match self {
            Self::Checkboxitem(val) => val.keyhandler(appdata, key),
            Self::Optitem(val) => val.keyhandler(appdata, key),
            Self::Numitem(val) => val.keyhandler(appdata, key),
            Self::Search(val) => val.keyhandler(appdata, key),
        }
    }
}

pub struct Browse {
    cardlimit: u32,
    filters: StatefulList<FilterItem>,
    filtered: StatefulList<CardItem>,
    selected: StatefulList<CardItem>,
    selected_ids: HashSet<CardID>,
    filteractions: StatefulList<ActionItem>,
    popup: Option<Box<dyn PopUp>>,
    cards: HashMap<CardID, Card>,
    view: View,
    cardview: CardView,
}

impl Browse {
    pub fn new(conn: &Arc<Mutex<Connection>>) -> Self {
        let cardlimit = 10000;

        let filters = StatefulList::with_items(
            "Filters".to_string(),
            vec![
                FilterItem::Checkboxitem(CheckBoxItem::new("Finished".to_string(), true)),
                FilterItem::Checkboxitem(CheckBoxItem::new("Unfinished".to_string(), true)),
                FilterItem::Checkboxitem(CheckBoxItem::new("Pending".to_string(), true)),
                FilterItem::Optitem(OptItem::new("Resolved".to_string())),
                FilterItem::Optitem(OptItem::new("Suspended".to_string())),
                FilterItem::Search(InputItem::new("Search".to_string(), Some(20))),
                FilterItem::Numitem(NumItem::new("Max stability".to_string(), None)),
                FilterItem::Numitem(NumItem::new("Min stability".to_string(), None)),
                FilterItem::Numitem(NumItem::new("Max strength".to_string(), Some(100))),
                FilterItem::Numitem(NumItem::new("Min strength".to_string(), Some(100))),
            ],
        );
        let selected_ids = HashSet::new();
        let filteractions = StatefulList::<ActionItem>::default();
        let popup = None;
        let cards = HashMap::new();
        let view = View::default();
        let cardview = CardView::default();

        let mut myself = Self {
            cardlimit,
            filters,
            filtered: StatefulList::new("Filtered".to_string()),
            selected: StatefulList::new("Selected".to_string()),
            selected_ids,
            filteractions,
            popup,
            cards,
            view,
            cardview,
        };

        myself.apply_filter(conn);
        myself
    }

    fn apply_filter(&mut self, conn: &Arc<Mutex<Connection>>) {
        let mut typevec = vec![];

        if let FilterItem::Checkboxitem(val) = &self.filters.items[0] {
            if val.filter {
                typevec.push(CardType::Finished);
            }
        }
        if let FilterItem::Checkboxitem(val) = &self.filters.items[1] {
            if val.filter {
                typevec.push(CardType::Unfinished);
            }
        }
        if let FilterItem::Checkboxitem(val) = &self.filters.items[2] {
            if val.filter {
                typevec.push(CardType::Pending);
            }
        }

        let mut query = CardQuery::default().cardtype(typevec);

        if let FilterItem::Optitem(val) = &self.filters.items[3] {
            match val.filter {
                FilterSetting::TruePass => query = query.resolved(true),
                FilterSetting::FalsePass => query = query.resolved(false),
                _ => {}
            }
        }
        if let FilterItem::Optitem(val) = &self.filters.items[4] {
            match val.filter {
                FilterSetting::TruePass => query = query.suspended(true),
                FilterSetting::FalsePass => query = query.suspended(false),
                _ => {}
            }
        }

        if let FilterItem::Search(val) = &self.filters.items[5] {
            if !val.searchfield.is_empty() {
                query = query.contains(val.searchfield.clone());
            }
        }

        if let FilterItem::Numitem(val) = &self.filters.items[6] {
            if let Some(num) = val.input.get_value() {
                query = query.max_stability(num);
            }
        }

        if let FilterItem::Numitem(val) = &self.filters.items[7] {
            if let Some(num) = val.input.get_value() {
                query = query.minimum_stability(num);
            }
        }

        if let FilterItem::Numitem(val) = &self.filters.items[8] {
            if let Some(num) = val.input.get_value() {
                query = query.max_strength(num as f32 / 100.);
            }
        }

        if let FilterItem::Numitem(val) = &self.filters.items[9] {
            if let Some(num) = val.input.get_value() {
                query = query.minimum_strength(num as f32 / 100.);
            }
        }

        let items = query
            .fetch_carditems(conn)
            .into_iter()
            .filter(|card| !self.selected_ids.contains(&card.id))
            .collect();

        self.filtered = StatefulList::with_items("Filtered".to_string(), items);
    }

    fn clear_selected(&mut self, conn: &Arc<Mutex<Connection>>) {
        self.selected.items.clear();
        self.selected.state.select(None);
        self.selected_ids.clear();
        self.apply_filter(conn)
    }

    fn save_pending_queue(&mut self, conn: &Arc<Mutex<Connection>>) {
        let highest_pos = match get_highest_pos(conn) {
            Some(pos) => pos,
            None => return, // if pending_cards is empty
        };

        for i in 0..self.selected.items.len() {
            let id = self.selected.items[i].id;
            let pos = highest_pos + self.selected.items.len() as u32 - i as u32;
            if is_pending(conn, id) {
                update_position(conn, id, pos);
            }
        }
    }

    fn navigate(&mut self, dir: NavDir) {
        self.view.navigate(dir);
    }

    fn apply_suspended(&self, appdata: &AppData, suspend: bool) {
        set_suspended(
            &appdata.conn,
            self.selected_ids.iter().cloned().collect::<Vec<CardID>>(),
            suspend,
        )
    }

    fn do_action(&mut self, appdata: &AppData) {
        if let Some(idx) = self.filteractions.state.selected() {
            match idx {
                0 => self.clear_selected(&appdata.conn),
                1 => self.apply_suspended(appdata, true),
                2 => self.apply_suspended(appdata, false),
                3 => {
                    let dependencies = self
                        .selected_ids
                        .clone()
                        .into_iter()
                        .collect::<Vec<CardID>>();
                    if dependencies.is_empty() {
                        return;
                    }
                    let addchild =
                        AddChildWidget::new(&appdata.conn, Purpose::Dependent(dependencies));
                    self.popup = Some(Box::new(addchild));
                }
                4 => {
                    let dependencies = self
                        .selected_ids
                        .clone()
                        .into_iter()
                        .collect::<Vec<CardID>>();
                    if dependencies.is_empty() {
                        return;
                    }
                    let purpose = CardPurpose::NewDependency(dependencies);
                    let cardfinder = FindCardWidget::new(&appdata.conn, purpose);
                    self.popup = Some(Box::new(cardfinder));
                }
                5 => {
                    let dependents = self
                        .selected_ids
                        .clone()
                        .into_iter()
                        .collect::<Vec<CardID>>();
                    if dependents.is_empty() {
                        return;
                    }
                    let addchild =
                        AddChildWidget::new(&appdata.conn, Purpose::Dependency(dependents));
                    self.popup = Some(Box::new(addchild));
                }
                6 => {
                    let dependents = self
                        .selected_ids
                        .clone()
                        .into_iter()
                        .collect::<Vec<CardID>>();
                    if dependents.is_empty() {
                        return;
                    }
                    let purpose = CardPurpose::NewDependent(dependents);
                    let cardfinder = FindCardWidget::new(&appdata.conn, purpose);
                    self.popup = Some(Box::new(cardfinder));
                }
                7 => self.save_pending_queue(&appdata.conn),
                _ => return,
            }
            self.apply_filter(&appdata.conn);
        }
    }
}

impl Tab for Browse {
    fn get_title(&self) -> String {
        "Browse".to_string()
    }
    fn get_cursor(&self) -> (u16, u16) {
        self.view.cursor
    }

    fn navigate(&mut self, dir: NavDir) {
        self.view.navigate(dir);
    }

    fn set_selection(&mut self, area: Rect) {
        let chunks = split_leftright(
            [
                Constraint::Length(20),
                Constraint::Percentage(50),
                Constraint::Percentage(25),
            ],
            area,
        );
        let filters = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Ratio(10, 20), Constraint::Ratio(10, 20)].as_ref())
            .split(chunks[0]);
        let filteredandselected = split_updown_by_percent([50, 50], chunks[1]);
        let cardview = split_updown_by_percent([25, 25, 25, 25], chunks[2]);

        self.view.areas.clear();
        self.view.areas.push(cardview[0]);
        self.view.areas.push(cardview[1]);
        self.view.areas.push(cardview[2]);
        self.view.areas.push(cardview[3]);
        self.view.areas.push(filters[0]);
        self.view.areas.push(filters[1]);
        self.view.areas.push(filteredandselected[0]);
        self.view.areas.push(filteredandselected[1]);

        self.cardview.dependents.set_area(cardview[0]);
        self.cardview.question.set_area(cardview[1]);
        self.cardview.answer.set_area(cardview[2]);
        self.cardview.dependencies.set_area(cardview[3]);
        self.filters.set_area(filters[0]);
        self.filteractions.set_area(filters[1]);
        self.filtered.set_area(filteredandselected[0]);
        self.selected.set_area(filteredandselected[1]);
    }

    fn render(
        &mut self,
        f: &mut tui::Frame<crate::MyType>,
        appdata: &crate::app::AppData,
        area: Rect,
    ) {
        self.set_selection(area);
        let cursor = &self.view.cursor;
        self.filters.render(f, appdata, cursor);

        self.filteractions.render(f, appdata, cursor);
        self.filtered.render(f, appdata, cursor);
        self.selected.render(f, appdata, cursor);
        self.cardview.render(f, appdata, cursor);

        if let Some(popup) = &mut self.popup {
            let mut area = f.size();
            if popup.should_quit() {
                self.popup = None;
                return;
            }
            if area.height > 10 && area.width > 10 {
                area = centered_rect(80, 70, area);
                f.render_widget(Clear, area); //this clears out the background
                area.x += 2;
                area.y += 2;
                area.height -= 4;
                area.width -= 4;
            }

            popup.render(f, appdata, area);
        }
    }

    fn keyhandler(&mut self, appdata: &crate::app::AppData, key: MyKey) {
        use MyKey::*;
        let cursor = &self.get_cursor();

        if let Some(popup) = &mut self.popup {
            popup.keyhandler(appdata, key);
            return;
        }

        if let Nav(dir) = key {
            self.navigate(dir);
            return;
        }
        match key {
            KeyPress(pos) => self.view.cursor = pos,
            Enter | Char(' ') if self.filtered.is_selected(cursor) => {
                if let Some(item) = self.filtered.take_selected_item() {
                    if !self.selected_ids.contains(&item.id) {
                        self.selected_ids.insert(item.id);
                        self.selected.push(item);
                    }
                }
            }

            Enter | Char(' ') if self.selected.is_selected(cursor) => {
                if let Some(item) = self.selected.take_selected_item() {
                    self.selected_ids.remove(&item.id);
                    self.filtered.push(item);
                }
            }
            Enter | Char(' ') if self.filteractions.is_selected(cursor) => self.do_action(appdata),

            key if self.filtered.is_selected(cursor) => {
                self.filtered.keyhandler(appdata, key);
                if let Some(idx) = self.filtered.state.selected() {
                    let id = self.filtered.items[idx].id;
                    self.cardview.change_card(&appdata.conn, id);
                }
            }
            key if self.selected.is_selected(cursor) => {
                self.selected.keyhandler(appdata, key);
                if let Some(idx) = self.selected.state.selected() {
                    let id = self.selected.items[idx].id;
                    self.cardview.change_card(&appdata.conn, id);
                }
            }
            key if self.filteractions.is_selected(cursor) => {
                self.filteractions.keyhandler(appdata, key)
            }
            key if self.cardview.is_selected(cursor) => {
                self.cardview.keyhandler(appdata, cursor, key)
            }

            key if self.filters.is_selected(cursor) => {
                self.filters.keyhandler(appdata, key);
                self.apply_filter(&appdata.conn);
            }
            _ => {}
        }
    }
}

use crate::{MyKey, NavDir};

struct ActionItem {
    text: String,
}

impl ActionItem {
    pub fn new(text: String) -> Self {
        Self { text }
    }
}

impl KeyHandler for ActionItem {}

impl Display for ActionItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl Default for StatefulList<ActionItem> {
    fn default() -> Self {
        let actions = vec![
            "Clear selected".to_string(),
            "Suspend".to_string(),
            "Unsuspend".to_string(),
            "Add new dependency".to_string(),
            "Add old dependency".to_string(),
            "Add new dependent".to_string(),
            "Add old dependent".to_string(),
            "Save to pending".to_string(),
        ];
        let items = actions.into_iter().map(ActionItem::new).collect();
        StatefulList::with_items("Actions".to_string(), items)
    }
}
