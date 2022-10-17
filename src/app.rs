use rusqlite::Connection;
use tui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Tabs},
    Frame,
};

use crate::tabs::add_card::logic::{DepState, NewCard};

use crate::{
    tabs::{
        incread::logic::MainInc,
        review::logic::MainReview,
        //browse::logic::Browse,
    },
    utils::misc::split_leftright,
    widgets::textinput::Field,
    MyType, SpekiPaths,
};



use serde_derive::Deserialize;


#[derive(Deserialize)]
pub struct Config{
    pub gptkey: Option<String>,
}

use toml;

impl Config{
    fn new(paths: &SpekiPaths) -> Self{
        let contents = std::fs::read_to_string(&paths.config).expect("Error reading file");
        let config: Config = toml::from_str(&contents).expect("invalid config file");
        config
    }
}

pub struct Audio{
    pub source: rodio::OutputStream,
    pub handle: rodio::OutputStreamHandle,
}

impl Audio{
    fn new() -> Option<Self>{
        let (source, handle) = rodio::OutputStream::try_default().unwrap();
        Some(Audio{
            source,
            handle
        })
    }
}


pub struct AppData{
    pub conn: Arc<Mutex<Connection>>,
    pub audio: Option<Audio>,
    pub paths: SpekiPaths,
    pub config: Config
}

pub struct TabsState {
    pub tabs: Vec<Box<dyn Tab>>,
    pub index: usize,
}

impl TabsState {
    pub fn new(
        conn: &Arc<Mutex<Connection>>,
        audio: &Option<Audio>,
    ) -> TabsState {
        let mut tabs: Vec<Box<dyn Tab>> = vec![];
        let revlist  = MainReview::new(conn, audio);
        let addcards = NewCard::new(conn, DepState::None);
        let incread  = MainInc::new(conn);
        let importer = Importer::new(conn);
        //let browse = Browse::new();

        tabs.push(Box::new(revlist));
        tabs.push(Box::new(addcards));
        tabs.push(Box::new(incread));
        tabs.push(Box::new(importer));
        //tabs.push(Box::new(browse));

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
        self.tabs[self.index].keyhandler(appdata, key);
    }
    fn render(&mut self, f: &mut Frame<MyType>, appdata: &AppData, area: Rect) {
        self.tabs[self.index].render(f, appdata, area);
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
        let tabs = TabsState::new(&conn, &audio);
        let appdata = AppData {
            conn,
            audio,
            config,
            paths,
        };

        App {
            tabs,
            display_help,
            should_quit: false,
            appdata,
        }
    }

    pub fn keyhandler(&mut self, key: MyKey) {
        match key {
            MyKey::Tab => self.tabs.next(),
            MyKey::BackTab => self.tabs.previous(),
            MyKey::SwapTab => self.tabs.swap_right(),
            MyKey::BackSwapTab => self.tabs.swap_left(),
            MyKey::F(1) => self.display_help = !self.display_help,
            MyKey::Alt('q') => self.should_quit = true,
            MyKey::Alt('m') => {
                if self.appdata.audio.is_some(){
                    self.appdata.audio = None;
                } else {
                    self.appdata.audio = Audio::new();
                }
            },
            key => self.tabs.keyhandler(&self.appdata, key),
        };
    }

    pub fn render(&mut self, f: &mut Frame<MyType>) {
        let mut area = f.size();
        area = self.render_help(f, area);
        area = self.render_tab_menu(f, area);
        self.tabs.render(f, &self.appdata, area);
    }

    fn render_tab_menu(&self, f: &mut Frame<MyType>, area: Rect) -> Rect {
        let chunks = Layout::default()
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(area);

        let block = Block::default().style(Style::default().bg(Color::Rgb(20, 31, 31)));
        f.render_widget(block, f.size());

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
        let chunks = split_leftright([66, 33], area);
        field.render(f, chunks[1], false);
        chunks[0]
    }
}

use crate::MyKey;

pub trait Tab {
    fn keyhandler(
        &mut self,
        appdata: &AppData,
        key: MyKey,
    );
    fn render(
        &mut self,
        f: &mut Frame<MyType>,
        appdata: &AppData,
        area: Rect,
    );
    fn get_title(&self) -> String;
    fn get_manual(&self) -> String {
        String::new()
    }
}
