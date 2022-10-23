use rusqlite::Connection;
use std::collections::HashSet;
use std::fmt::Display;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::Style;
use tui::widgets::Clear;

use crate::app::{AppData, PopUp, Widget};
use crate::utils::aliases::*;
use crate::utils::card::CardType;
use crate::utils::misc::{
    centered_rect, split_leftright, split_leftright_by_percent, split_updown_by_percent,
};
use crate::utils::sql::fetch::{get_highest_pos, get_pending, is_pending, CardQuery};
use crate::utils::sql::update::{set_cardtype, set_suspended, update_position};
use crate::utils::statelist::KeyHandler;
use crate::widgets::cardlist::CardItem;
use crate::widgets::checkbox::{CheckBox, CheckBoxItem};
use crate::widgets::find_card::{CardPurpose, FindCardWidget};
use crate::widgets::newchild::{AddChildWidget, Purpose};
use crate::widgets::numeric_input::{NumItem, NumPut};
use crate::widgets::optional_bool_filter::{FilterSetting, OptCheckBox, OptItem};
use crate::{app::Tab, utils::statelist::StatefulList};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

enum Selection {
    Filters,
    Filtered,
    Selected,
    Actions,
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

pub enum FilterItem {
    checkboxitem(CheckBoxItem),
    optitem(OptItem),
    numitem(NumItem),
}

impl Display for FilterItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match self {
            Self::checkboxitem(val) => format!("{}", val),
            Self::optitem(val) => format!("{}", val),
            Self::numitem(val) => format!("{}", val),
        };
        write!(f, "{}", val)
    }
}

impl KeyHandler for FilterItem {
    fn keyhandler(&mut self, key: MyKey) -> bool {
        match self {
            Self::checkboxitem(val) => val.keyhandler(key),
            Self::optitem(val) => val.keyhandler(key),
            Self::numitem(val) => val.keyhandler(key),
        }
    }
}

pub struct Browse {
    selection: Selection,
    cardlimit: u32,
    filters: StatefulList<FilterItem>,
    filtered: StatefulList<CardItem>,
    selected: StatefulList<CardItem>,
    selected_ids: HashSet<CardID>,
    filteractions: StatefulList<ActionItem>,
    popup: Option<Box<dyn PopUp>>,
}

impl Browse {
    pub fn new(conn: &Arc<Mutex<Connection>>) -> Self {
        let cardlimit = 10000;

        let filters = StatefulList::with_items(vec![
            FilterItem::checkboxitem(CheckBoxItem::new("Finished".to_string(), false)),
            FilterItem::checkboxitem(CheckBoxItem::new("Unfinished".to_string(), false)),
            FilterItem::checkboxitem(CheckBoxItem::new("Pending".to_string(), false)),
            FilterItem::optitem(OptItem::new("Resolved".to_string())),
            FilterItem::optitem(OptItem::new("Suspended".to_string())),
            FilterItem::numitem(NumItem::new("Max stability:".to_string(), None)),
            FilterItem::numitem(NumItem::new("Min stability:".to_string(), None)),
            FilterItem::numitem(NumItem::new("Max strength:".to_string(), Some(100))),
            FilterItem::numitem(NumItem::new("Min strength:".to_string(), None)),
        ]);
        let selected_ids = HashSet::new();
        let selection = Selection::Filtered;
        let filteractions = StatefulList::<ActionItem>::default();
        let popup = None;

        let mut myself = Self {
            selection,
            cardlimit,
            filters,
            filtered: StatefulList::new(),
            selected: StatefulList::new(),
            selected_ids,
            filteractions,
            popup,
        };
        myself.apply_filter(conn);
        myself
    }

    fn apply_filter(&mut self, conn: &Arc<Mutex<Connection>>) {
        let mut typevec = vec![];

        if let FilterItem::checkboxitem(val) = &self.filters.items[0] {
            if val.filter {
                typevec.push(CardType::Finished);
            }
        }
        if let FilterItem::checkboxitem(val) = &self.filters.items[1] {
            if val.filter {
                typevec.push(CardType::Unfinished);
            }
        }
        if let FilterItem::checkboxitem(val) = &self.filters.items[2] {
            if val.filter {
                typevec.push(CardType::Pending);
            }
        }

        let mut query = CardQuery::default().cardtype(typevec);

        if let FilterItem::optitem(val) = &self.filters.items[3] {
            match val.filter {
                FilterSetting::TruePass => query = query.resolved(true),
                FilterSetting::FalsePass => query = query.resolved(false),
                _ => {}
            }
        }
        if let FilterItem::optitem(val) = &self.filters.items[4] {
            match val.filter {
                FilterSetting::TruePass => query = query.suspended(true),
                FilterSetting::FalsePass => query = query.suspended(false),
                _ => {}
            }
        }

        if let FilterItem::numitem(val) = &self.filters.items[5] {
            if let Some(num) = val.input.get_value() {
                query = query.max_stability(num);
            }
        }

        if let FilterItem::numitem(val) = &self.filters.items[6] {
            if let Some(num) = val.input.get_value() {
                query = query.minimum_stability(num);
            }
        }

        if let FilterItem::numitem(val) = &self.filters.items[7] {
            if let Some(num) = val.input.get_value() {
                query = query.max_strength(num as f32 / 100.);
            }
        }

        if let FilterItem::numitem(val) = &self.filters.items[8] {
            if let Some(num) = val.input.get_value() {
                query = query.minimum_strength(num as f32 / 100.);
            }
        }

        let items = query
            .fetch_carditems(conn)
            .into_iter()
            .filter(|card| !self.selected_ids.contains(&card.id))
            .collect();

        self.filtered = StatefulList::with_items(items);
    }

    fn clear_selected(&mut self, conn: &Arc<Mutex<Connection>>) {
        self.selected.items.clear();
        self.selected.state.select(None);
        self.selected_ids.clear();
        self.apply_filter(conn)
    }

    fn save_pending_queue(&mut self, conn: &Arc<Mutex<Connection>>) {
        let highest_pos = get_highest_pos(conn);

        for i in 0..self.selected.items.len() {
            let id = self.selected.items[i].id;
            let pos = highest_pos + self.selected.items.len() as u32 - i as u32;
            if is_pending(conn, id) {
                update_position(conn, id, pos);
            }
        }
    }

    fn navigate(&mut self, dir: NavDir) {
        use NavDir::*;
        use Selection::*;
        self.selection = match (&self.selection, dir) {
            (Filters, Right) => Filtered,
            (Filters, Down) => Actions,

            (Filtered, Left) => Filters,
            (Filtered, Right) => Selected,

            (Selected, Left) => Filtered,
            (Actions, Up) => Filters,
            (Actions, Right) => Filtered,

            _ => return,
        }
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
}

impl Widget for Browse {
    fn keyhandler(&mut self, appdata: &crate::app::AppData, key: MyKey) {
        use MyKey::*;
        use Selection::*;

        if let Some(popup) = &mut self.popup {
            popup.keyhandler(appdata, key);
            return;
        }

        if let Nav(dir) = key {
            self.navigate(dir);
            return;
        }
        match (&self.selection, key) {
            (Filters, key) => {
                self.filters.keyhandler(key);
                self.apply_filter(&appdata.conn);
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
            (Actions, Enter) | (Actions, Char(' ')) => self.do_action(appdata),
            (Actions, key) => self.filteractions.keyhandler(key),
        }
    }

    fn render(
        &mut self,
        f: &mut tui::Frame<crate::MyType>,
        appdata: &crate::app::AppData,
        mut area: tui::layout::Rect,
    ) {
        let chunks = split_leftright(
            [
                Constraint::Length(27),
                Constraint::Percentage(40),
                Constraint::Percentage(40),
                Constraint::Percentage(10),
            ],
            area,
        );
        let filters = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(11), Constraint::Min(1)].as_ref())
            .split(chunks[0]);

        self.filters.render(
            f,
            filters[0],
            matches!(&self.selection, Selection::Filters),
            "Filters",
            Style::default(),
        );

        self.filteractions.render(
            f,
            filters[1],
            matches!(&self.selection, Selection::Actions),
            "Actions",
            Style::default(),
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

        if let Some(popup) = &mut self.popup {
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
}

use crate::Direction as NavDir;
use crate::MyKey;

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
            "Add existing dependency".to_string(),
            "Add new dependent".to_string(),
            "Add existing dependent".to_string(),
            "Add to pending queue".to_string(),
        ];
        let items = actions
            .into_iter()
            .map(|action| ActionItem::new(action))
            .collect();
        StatefulList::with_items(items)
    }
}
