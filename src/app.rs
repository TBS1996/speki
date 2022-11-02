use rusqlite::Connection;
use tui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, Tabs},
    Frame,
};

use crate::{
    tabs::add_card::logic::NewCard,
    utils::misc::{centered_rect, View},
    NavDir,
};

use crate::{
    tabs::{browse::logic::Browse, incread::logic::MainInc, review::logic::MainReview},
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
    fn new(paths: &SpekiPaths) -> Self {
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
    fn new() -> Option<Self> {
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
}

impl TabsState {
    pub fn new(appdata: &AppData) -> TabsState {
        let mut tabs: Vec<Box<dyn Tab>> = vec![];
        let revlist = MainReview::new(appdata);
        let addcards = NewCard::new(&appdata.conn);
        let incread = MainInc::new(&appdata.conn);
        let importer = Importer::new(&appdata.conn);
        let browse = Browse::new(&appdata.conn);

        tabs.push(Box::new(revlist));
        tabs.push(Box::new(addcards));
        tabs.push(Box::new(browse));
        tabs.push(Box::new(incread));
        tabs.push(Box::new(importer));

        TabsState { tabs, index: 0 }
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
            MyKey::Nav(dir) => self.tabs[self.index].navigate(dir),
            key => self.tabs[self.index].main_keyhandler(appdata, key),
        }
    }
    fn render(&mut self, f: &mut Frame<MyType>, appdata: &AppData, area: Rect) {
        self.tabs[self.index].main_render(f, appdata, area);
    }
}

use crate::tabs::import::logic::Importer;
use std::sync::{Arc, Mutex};

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

    pub fn keyhandler(&mut self, key: MyKey) {
        match key {
            MyKey::KeyPress(pos) if pos.1 < 2 => self.press_tab(pos),
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

    fn press_tab(&mut self, pos: (u16, u16)) {
        let mut xpos = 1;
        let padlen = 3;
        for (index, tab) in self.tabs.tabs.iter().enumerate() {
            let title = tab.get_title();
            xpos += title.len() + padlen;
            if xpos > pos.0 as usize {
                self.tabs.index = index;
                return;
            }
        }
    }

    fn render_tab_menu(&self, f: &mut Frame<MyType>, area: Rect) -> Rect {
        let chunks = Layout::default()
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(area);

        let titles = self
            .tabs
            .tabs
            .iter()
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
        field.render(f, &self.appdata, &(0, 0));
        chunks[0]
    }
}

use crate::MyKey;

pub trait PopUp: Tab {
    fn should_quit(&self) -> bool;
    fn render_popup(&mut self, f: &mut Frame<MyType>, appdata: &AppData, mut area: Rect) {
        if area.height > 10 && area.width > 10 {
            area = centered_rect(80, 70, area);
            f.render_widget(Clear, area); //this clears out the background
            area.x += 2;
            area.y += 2;
            area.height -= 4;
            area.width -= 4;
        }
        self.main_render(f, appdata, area);
    }
}

pub trait Widget {
    fn keyhandler(&mut self, appdata: &AppData, key: MyKey);
    fn render(&mut self, f: &mut Frame<MyType>, appdata: &AppData, cursor: &(u16, u16));
    fn get_area(&self) -> Rect;
    fn set_area(&mut self, area: Rect);

    fn is_selected(&self, cursor: &(u16, u16)) -> bool {
        View::isitselected(self.get_area(), cursor)
    }
}

pub trait Tab {
    fn get_title(&self) -> String;
    fn get_manual(&self) -> String {
        String::new()
    }
    fn set_selection(&mut self, area: Rect);

    fn get_cursor(&mut self) -> (u16, u16) {
        self.get_view().clone().cursor
    }
    fn navigate(&mut self, dir: NavDir) {
        if let Some(popup) = self.get_popup() {
            popup.navigate(dir);
        } else {
            self.get_view().navigate(dir);
        }
    }

    fn get_view(&mut self) -> &mut View;

    fn main_keyhandler(&mut self, appdata: &AppData, key: MyKey) {
        if let Some(popup) = self.get_popup() {
            popup.main_keyhandler(appdata, key);
            return;
        }
        if let MyKey::KeyPress(pos) = key.clone() {
            self.get_view().cursor = pos;
        }
        match key {
            MyKey::Nav(dir) => self.navigate(dir),
            key => self.keyhandler(appdata, key),
        }
    }
    fn keyhandler(&mut self, appdata: &AppData, key: MyKey);

    fn main_render(&mut self, f: &mut Frame<MyType>, appdata: &AppData, area: Rect) {
        self.set_selection(area);
        self.render(f, appdata, area);
        if let Some(popup) = self.get_popup() {
            if popup.should_quit() {
                self.exit_popup(appdata);
                return;
            }
            popup.render_popup(f, appdata, area);
        }
    }

    fn render(&mut self, f: &mut Frame<MyType>, appdata: &AppData, area: Rect);

    fn get_popup(&mut self) -> Option<&mut Box<dyn PopUp>> {
        None
    }
    fn exit_popup(&mut self, appdata: &AppData) {
        let _ = appdata;
        // if there's a way to statically enforce this requirement, make an issue or PR about it <3
        panic!("Overriding the get_popup() method requires you to also override the exit_popup() method")
    }
}
