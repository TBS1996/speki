use rusqlite::Connection;
use tui::widgets::Clear;
use std::collections::HashSet;
use std::fmt::Display;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::Style;

use crate::app::{AppData, Widget, PopUp};
use crate::utils::aliases::*;
use crate::utils::card::CardType;
use crate::utils::misc::{split_leftright, split_leftright_by_percent, split_updown_by_percent, centered_rect};
use crate::utils::sql::fetch::CardQuery;
use crate::utils::sql::update::{set_cardtype, set_suspended};
use crate::utils::statelist::KeyHandler;
use crate::widgets::cardlist::CardItem;
use crate::widgets::checkbox::CheckBox;
use crate::widgets::find_card::{CardPurpose, FindCardWidget};
use crate::widgets::newchild::{AddChildWidget, Purpose};
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

pub struct Browse {
    selection: Selection,
    cardtypes: CheckBox,
    cardstatus: OptCheckBox,
    numfilters: NumPut,
    cardlimit: u32,
    filtered: StatefulList<CardItem>,
    selected: StatefulList<CardItem>,
    selected_ids: HashSet<CardID>,
    filteractions: StatefulList<ActionItem>,
    popup: Option<Box<dyn PopUp>>,
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
        let filteractions = StatefulList::<ActionItem>::default();
        let popup = None;

        let mut myself = Self {
            selection,
            cardtypes,
            cardstatus,
            numfilters,
            cardlimit,
            filtered: StatefulList::new(),
            selected: StatefulList::new(),
            selected_ids,
            filteractions,
            popup,
        };
        myself.apply_filter(conn);
        myself
    }

    fn clear_selected(&mut self, conn: &Arc<Mutex<Connection>>) {
        self.selected.items.clear();
        self.selected.state.select(None);
        self.selected_ids.clear();
        self.apply_filter(conn)
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
        self.selection = match (&self.selection, dir) {
            (CardType, Right) => Selection::Filtered,
            (CardType, Down) => Selection::CardStatus,
            (Filtered, Right) => Selection::Selected,
            (Filtered, Left) => Selection::CardType,
            (Selected, Left) => Selection::Filtered,
            (CardStatus, Right) => Selection::Filtered,
            (CardStatus, Up) => Selection::CardType,
            (CardStatus, Down) => Selection::Numfilters,
            (Numfilters, Right) => Selection::Filtered,
            (Numfilters, Up) => Selection::CardStatus,
            (Numfilters, Down) => Selection::Actions,
            (Actions, Up) => Selection::Numfilters,
            (Actions, Right) => Selection::Filtered,
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
                    let dependencies = self.selected_ids.clone().into_iter().collect::<Vec<CardID>>();
                    if dependencies.is_empty() {return}
                    let addchild = AddChildWidget::new(&appdata.conn, Purpose::Dependent(dependencies));
                    self.popup = Some(Box::new(addchild));
                },
                4 => {
                    let dependencies = self.selected_ids.clone().into_iter().collect::<Vec<CardID>>();
                    if dependencies.is_empty() {return}
                    let purpose = CardPurpose::NewDependency(dependencies);
                    let cardfinder = FindCardWidget::new(&appdata.conn, purpose);
                    self.popup = Some(Box::new(cardfinder));
                },
                5 => {
                    let dependents = self.selected_ids.clone().into_iter().collect::<Vec<CardID>>();
                    if dependents.is_empty() {return}
                    let addchild = AddChildWidget::new(&appdata.conn, Purpose::Dependency(dependents));
                    self.popup = Some(Box::new(addchild));
                },
                6 => {
                    let dependents = self.selected_ids.clone().into_iter().collect::<Vec<CardID>>();
                    if dependents.is_empty() {return}
                    let purpose = CardPurpose::NewDependent(dependents);
                    let cardfinder = FindCardWidget::new(&appdata.conn, purpose);
                    self.popup = Some(Box::new(cardfinder));
                },
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

        if let Some(popup) = &mut self.popup{
            popup.keyhandler(appdata, key);
            return;
        }



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
            (Actions, Enter) => self.do_action(appdata),
            (Actions, key) => self.filteractions.keyhandler(key),
            (Numfilters, key) => {
                self.numfilters.items.keyhandler(key);
                self.apply_filter(&appdata.conn);
            }
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
            .constraints(
                [
                    Constraint::Max(5),
                    Constraint::Max(4),
                    Constraint::Max(6),
                    Constraint::Max(9),
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
        self.filteractions.render(
            f,
            filters[3],
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


        if let Some(popup) = &mut self.popup{

            if popup.should_quit(){
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
