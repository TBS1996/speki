use rusqlite::Connection;
use tui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, Tabs},
    Frame,
};

use crate::{
    popups::menu::Menu,
    tabs::add_card::NewCard,
    utils::{
        aliases::Pos,
        misc::{draw_paragraph, split_updown, View},
    },
    NavDir,
};

use crate::{
    tabs::{browse::Browse, incread::MainInc, review::logic::MainReview},
    utils::misc::split_leftright_by_percent,
    widgets::textinput::Field,
    MyType, SpekiPaths,
};

use serde_derive::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub gptkey: Option<String>,
}

use toml;

impl Config {
    pub fn new(paths: &SpekiPaths) -> Self {
        let contents = std::fs::read_to_string(&paths.config).expect("Error reading file");
        let config: Config = toml::from_str(&contents).expect("invalid config file");
        config
    }
}

pub struct Audio {
    pub source: rodio::OutputStream,
    pub handle: rodio::OutputStreamHandle,
    pub volume: f32,
}

impl Audio {
    pub fn new() -> Option<Self> {
        let (source, handle) = rodio::OutputStream::try_default().unwrap();
        let volume = 0.2;
        Some(Audio {
            source,
            handle,
            volume,
        })
    }
}

pub struct AppData {
    pub conn: Arc<Mutex<Connection>>,
    pub audio: Option<Audio>,
    pub paths: SpekiPaths,
    pub config: Config,
}

pub struct TabsState {
    pub tabs: Vec<Box<dyn Tab>>,
    pub index: usize,
    titlepositions: Vec<usize>,
}

impl TabsState {
    pub fn new(appdata: &AppData) -> TabsState {
        let mut tabs: Vec<Box<dyn Tab>> = vec![];
        let revlist = MainReview::new(appdata);
        let addcards = NewCard::new(appdata);
        let browse = Browse::new(appdata);
        let incread = MainInc::new(&appdata.conn);
        let importer = Menu::new_import_tab();

        tabs.push(Box::new(revlist));
        tabs.push(Box::new(addcards));
        tabs.push(Box::new(browse));
        tabs.push(Box::new(incread));
        tabs.push(Box::new(importer));

        let mut tabs = TabsState {
            tabs,
            index: 0,
            titlepositions: vec![],
        };
        tabs.calibrate_startpos();
        tabs
    }

    fn calibrate_startpos(&mut self) {
        self.titlepositions = vec![0];
        let mut xpos = 1;
        let padlen = 3;
        for (_, tab) in self.tabs.iter_mut().enumerate() {
            let title = tab.get_title();
            xpos += title.len() + padlen;
            self.titlepositions.push(xpos);
        }
        self.titlepositions.pop();
    }

    fn get_tab_index(&self, pos: Pos) -> usize {
        for idx in (0..self.titlepositions.len()).rev() {
            if self.titlepositions[idx] < pos.x as usize {
                return idx;
            }
        }
        0
    }
    pub fn next(&mut self) {
        if self.index < self.tabs.len() - 1 {
            self.index += 1;
        }
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        }
    }

    fn swap_left(&mut self) {
        if self.index == 0 {
            return;
        }
        self.tabs.swap(self.index, self.index - 1);
        self.previous();
    }
    fn swap_right(&mut self) {
        if self.index == self.tabs.len() - 1 {
            return;
        }
        self.tabs.swap(self.index, self.index + 1);
        self.next();
    }

    fn keyhandler(&mut self, appdata: &AppData, key: MyKey) {
        match key {
            //MyKey::Nav(dir) => self.tabs[self.index].navigate(dir),
            key => self.tabs[self.index].main_keyhandler(appdata, key),
        }
    }
    fn render(&mut self, f: &mut Frame<MyType>, appdata: &AppData, area: Rect) {
        let mut navbar = vec![];
        self.tabs[self.index].main_render(f, appdata, area, &mut navbar);
    }
}

use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

pub struct App {
    pub tabs: TabsState,
    pub should_quit: bool,
    pub display_help: bool,
    pub appdata: AppData,
}

impl App {
    pub fn new(display_help: bool, paths: SpekiPaths) -> App {
        let conn = Arc::new(Mutex::new(
            Connection::open(&paths.database).expect("Failed to connect to database."),
        ));
        let config = Config::new(&paths);
        let audio = Audio::new();
        let appdata = AppData {
            conn,
            audio,
            config,
            paths,
        };
        let tabs = TabsState::new(&appdata);

        App {
            tabs,
            display_help,
            should_quit: false,
            appdata,
        }
    }

    fn drag_tab(&mut self, _pos: Pos) {
        /*
        let current_index = self.tabs.index;
        let new_index = self.tabs.get_tab_index(pos);
        if new_index < current_index {
            let new_title_len = self.tabs.tabs[new_index].get_title().len();
        }
        self.tabs.tabs.swap(current_index, new_index);
        self.tabs.index = new_index;
        */
    }

    pub fn keyhandler(&mut self, key: MyKey) {
        match key {
            MyKey::KeyPress(pos) if pos.y < 2 => self.press_tab(pos),
            MyKey::Drag(pos) if pos.y < 2 => self.drag_tab(pos),
            MyKey::Tab => self.tabs.next(),
            MyKey::BackTab => self.tabs.previous(),
            MyKey::SwapTab => self.tabs.swap_right(),
            MyKey::BackSwapTab => self.tabs.swap_left(),
            MyKey::F(1) => self.display_help = !self.display_help,
            MyKey::Alt('q') | MyKey::Alt('Q') => self.should_quit = true,
            MyKey::Alt('m') => {
                if self.appdata.audio.is_some() {
                    self.appdata.audio = None;
                } else {
                    self.appdata.audio = Audio::new();
                }
            }
            key => self.tabs.keyhandler(&self.appdata, key),
        };
    }

    pub fn render(&mut self, f: &mut Frame<MyType>) {
        let mut area = f.size();
        area = self.render_help(f, area);
        area = self.render_tab_menu(f, area);
        self.tabs.render(f, &self.appdata, area);
    }

    fn press_tab(&mut self, pos: Pos) {
        self.tabs.index = self.tabs.get_tab_index(pos);
    }

    fn render_tab_menu(&mut self, f: &mut Frame<MyType>, area: Rect) -> Rect {
        let chunks = Layout::default()
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(area);

        let titles = self
            .tabs
            .tabs
            .iter_mut()
            .map(|t| {
                Spans::from(Span::styled(
                    t.get_title(),
                    Style::default().fg(Color::Green),
                ))
            })
            .collect();

        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL))
            .highlight_style(Style::default().fg(Color::Yellow))
            .select(self.tabs.index);

        f.render_widget(tabs, chunks[0]);
        chunks[1]
    }

    fn render_help(&mut self, f: &mut Frame<MyType>, area: Rect) -> Rect {
        if !self.display_help {
            return area;
        }
        let mut msg = r#"@@@@@@@@@@@@@@@@@@@@@@@@
@F1 TO TOGGLE HELP MENU@
@@@@@@@@@@@@@@@@@@@@@@@@
(if your terminal blocks F1, try shift+F1)

next tab: Tab,
previous tab: Shift+Tab,
move between widgets: Alt + arrow-keys (or vim-keys)
quit: Alt+q

"#
        .to_string();

        let help_msg = self.tabs.tabs[self.tabs.index].get_manual();
        msg.push_str(&help_msg);
        let mut field = Field::new_with_text(msg, 0, 0);
        let chunks = split_leftright_by_percent([66, 33], area);
        field.set_area(chunks[1]);
        field.render(f, &self.appdata, &Pos::default());
        chunks[0]
    }
}

use crate::MyKey;

pub enum PopupValue {
    Path(PathBuf),
    None,
    Err,
    Ok,
}

impl Default for PopupValue {
    fn default() -> Self {
        Self::None
    }
}

pub enum PopUpState {
    Continue,
    Exit,
    Switch(Box<dyn Tab>),
}

impl Default for PopUpState {
    fn default() -> Self {
        Self::Continue
    }
}

pub trait Widget {
    fn keyhandler(&mut self, appdata: &AppData, key: MyKey);
    fn render(&mut self, f: &mut Frame<MyType>, appdata: &AppData, cursor: &Pos);
    fn get_area(&self) -> Rect;
    fn set_area(&mut self, area: Rect);
    fn refresh(&mut self) {}

    fn is_selected(&self, cursor: &Pos) -> bool {
        View::isitselected(self.get_area(), cursor)
    }
}

pub trait Tab {
    fn keyhandler(&mut self, appdata: &AppData, key: MyKey, cursor: &Pos);
    fn render(&mut self, f: &mut Frame<MyType>, appdata: &AppData, cursor: &Pos);

    fn refresh(&mut self, _appdata: &AppData) {}
    fn get_manual(&self) -> String {
        String::new()
    }
    fn set_selection(&mut self, area: Rect);

    fn get_cursor(&mut self) -> &Pos {
        &self.get_view().cursor
    }
    fn navigate(&mut self, dir: NavDir) {
        if let Some(popup) = self.get_popup() {
            popup.navigate(dir);
        } else {
            self.get_view().navigate(dir);
        }
    }

    fn get_view(&mut self) -> &mut View {
        &mut self.get_tabdata().view
    }

    fn get_tabdata(&mut self) -> &mut TabData;

    fn main_keyhandler(&mut self, appdata: &AppData, key: MyKey) {
        if let Some(popup) = self.get_popup() {
            match key {
                MyKey::Esc if popup.get_popup().is_none() => {
                    popup.save_state(appdata);
                    self.exit_popup(appdata);
                }
                key => popup.main_keyhandler(appdata, key),
            }
            return;
        }
        if let MyKey::KeyPress(pos) = key.clone() {
            self.get_view().cursor = pos;
        }
        let cursor = self.get_cursor().clone();
        match key {
            MyKey::Nav(dir) => self.navigate(dir),
            key => self.keyhandler(appdata, key, &cursor),
        }
    }

    fn main_render(
        &mut self,
        f: &mut Frame<MyType>,
        appdata: &AppData,
        mut area: Rect,
        navbar: &mut Vec<Span>,
    ) {
        let bg = Color::Reset;
        let arrcol = Style::default().fg(Color::Rgb(255, 255, 255)).bg(bg);
        let hicol = Style::default().fg(Color::Rgb(150, 255, 150)).bg(bg);
        let titcol = Style::default().fg(Color::Rgb(200, 255, 200)).bg(bg);
        if let Some(popup) = self.get_popup() {
            let title = popup.get_title().clone();
            navbar.push(Span::styled(" ❱❱", arrcol));
            let state = popup.get_state();
            match std::mem::take(state) {
                PopUpState::Continue => {
                    if popup.get_popup().is_some() {
                        navbar.push(Span::styled(title, titcol));
                    } else {
                        navbar.push(Span::styled(title, hicol));
                    }
                    popup.main_render(f, appdata, area, navbar);
                }
                PopUpState::Exit => self.exit_popup(appdata),
                PopUpState::Switch(tab) => {
                    navbar.push(Span::styled(title, arrcol));
                    *popup = tab;
                    popup.main_render(f, appdata, area, navbar);
                }
            }
        } else {
            if !navbar.is_empty() {
                f.render_widget(Clear, area); //this clears out the background
                let chunks = split_updown([Constraint::Length(1), Constraint::Min(1)], area);
                draw_paragraph(
                    f,
                    chunks[0],
                    vec![Spans::from(navbar.clone())],
                    Style::default(),
                    Alignment::Left,
                    Borders::NONE,
                );
                area = chunks[1];
                self.transform_area(&mut area);
            }
            let cursor = self.get_cursor().clone();
            self.set_selection(area);
            self.get_tabdata().view.validate_pos(); // ensures cursor is on a widget;
            self.render(f, appdata, &cursor);
        }
    }

    fn transform_area(&self, area: &mut Rect) {
        return;
        area.x += 2;
        area.y += 2;
        area.height -= 4;
        area.width -= 4;
    }

    fn get_popup(&mut self) -> Option<&mut Box<dyn Tab>> {
        if let Some(popup) = &mut self.get_tabdata().popup {
            Some(popup)
        } else {
            None
        }
    }
    fn save_state(&mut self, _appdata: &AppData) {}
    fn exit_popup(&mut self, appdata: &AppData) {
        self.get_tabdata().popup = None;
        self.refresh(appdata);
    }

    fn switch_popup(&mut self, _obj: Box<dyn Tab>) {}

    fn set_popup(&mut self, popup: Box<dyn Tab>) {
        self.get_tabdata().popup = Some(popup);
    }

    fn get_state(&mut self) -> &mut PopUpState {
        &mut self.get_tabdata().state
    }

    fn get_popup_value(&mut self) -> &PopupValue {
        &self.get_tabdata().value
    }
    fn set_next_tab(&mut self, next_tab: Option<Box<dyn Tab>>) {
        self.get_tabdata().next_tab = next_tab;
    }
    fn get_title(&mut self) -> &String {
        &self.get_tabdata().title
    }
}

pub struct TabData {
    pub title: String,
    pub view: View,
    pub popup: Option<Box<dyn Tab>>,
    pub value: PopupValue,
    pub state: PopUpState,
    pub next_tab: Option<Box<dyn Tab>>,
}

impl TabData {
    pub fn new(title: String) -> Self {
        Self {
            title,
            view: View::default(),
            popup: None,
            value: PopupValue::default(),
            state: PopUpState::default(),
            next_tab: None,
        }
    }
}

impl Default for Box<dyn Tab> {
    fn default() -> Self {
        Box::new(Dummy)
    }
}

struct Dummy;
impl Tab for Dummy {
    fn set_selection(&mut self, _area: Rect) {}
    fn render(&mut self, _f: &mut Frame<MyType>, _appdata: &AppData, _cursor: &Pos) {}
    fn keyhandler(&mut self, _appdata: &AppData, _key: MyKey, _cursor: &Pos) {}
    fn get_title(&mut self) -> &String {
        todo!()
    }
    fn get_tabdata(&mut self) -> &mut TabData {
        todo!()
    }
}
