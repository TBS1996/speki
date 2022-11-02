use std::collections::HashMap;

use crate::utils::statelist::{KeyHandler, StatefulList};
use crate::widgets::textinput::Field;
use crate::MyType;
use reqwest;
use tui::{
    layout::{Constraint, Direction::Vertical, Layout},
    widgets::ListState,
};

use super::button::Button;
use crate::MyKey;
use tui::layout::Direction::Horizontal;

use regex::*;

use tui::style::Color;

pub enum ShouldQuit {
    No,
    Yeah,
    Takethis(String),
}

fn get_description(pagesource: &String, id: &u32) -> String {
    let pattern = "<div class=\"shared-item-description pb-3\">((.|\n)*)<h2>Sample".to_string();
    let re = Regex::new(&pattern).unwrap();
    let captures = match re.captures(pagesource) {
        Some(x) => x,
        None => panic!("{}, @@{}", pagesource, id),
    };
    captures.get(1).unwrap().as_str().to_string()
}

use std::sync::mpsc;
use std::thread;
enum Menu {
    Main,
    Downloading(DeckDownload),
}

struct DeckDownload {
    name: String,
    rx: mpsc::Receiver<(u64, u64)>,
}

pub struct Ankimporter {
    prompt: Button,
    searchterm: Field,
    description: Field,
    list: StatefulList<Deck>,
    descmap: HashMap<u32, String>,
    menu: Menu,
    pub should_quit: ShouldQuit,
}

#[derive(Clone, PartialEq)]
pub struct Deck {
    pub title: String,
    pub id: u32,
}

impl KeyHandler for Deck {}

use std::fmt;
impl fmt::Display for Deck {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.title)
    }
}
use crate::app::{AppData, Tab, Widget};
impl Ankimporter {
    pub fn new() -> Self {
        let list = StatefulList::<Deck>::new("".to_string());
        let searchterm = Field::default();
        let description = Field::default();
        let menu = Menu::Main;

        Ankimporter {
            prompt: Button::new("Choose anki deck".to_string()),
            searchterm,
            description,
            list,
            descmap: HashMap::new(),
            menu,
            should_quit: ShouldQuit::No,
        }
    }

    fn update_desc(&mut self) {
        if let Some(idx) = self.list.state.selected() {
            let id = self.list.items[idx].id;
            if !self.descmap.contains_key(&id) {
                let url = format!("https://ankiweb.net/shared/info/{}", id);
                let body = reqwest::blocking::get(url).unwrap().text().unwrap();
                let desc = get_description(&body, &id);
                self.descmap.insert(id, desc);
            }
        }
    }

    fn is_desc_loaded(&self) -> bool {
        if let Some(idx) = self.list.state.selected() {
            let id = self.list.items[idx].id;
            return self.descmap.contains_key(&id);
        }
        false
    }

    fn fetch(&mut self) {
        let searchtext = self.searchterm.return_text();
        let searchtext = str::replace(&searchtext, " ", "%20");
        let url = format!("https://ankiweb.net/shared/decks/{}", searchtext);
        let body = reqwest::blocking::get(url).unwrap().text().unwrap();

        let splitter: Vec<&str> = body.split("const shared = new anki.SharedList(").collect();
        let foo = splitter[1].to_string();

        let mut myvec = Vec::<Deck>::new();
        let mut stringstatus = Stringstatus::Beforeint;
        let mut title = String::new();
        let mut intrep = String::new();
        for c in foo.chars() {
            if c == ';' {
                break;
            }

            match stringstatus {
                Stringstatus::Beforeint => {
                    if c.is_ascii_digit() {
                        intrep.push(c);
                        stringstatus = Stringstatus::Onint;
                    }
                }
                Stringstatus::Onint => {
                    if c.is_ascii_digit() {
                        intrep.push(c);
                    } else {
                        stringstatus = Stringstatus::Beforestring;
                    }
                }
                Stringstatus::Beforestring => {
                    if c == '\"' {
                        stringstatus = Stringstatus::Onstring;
                    }
                }
                Stringstatus::Onstring => {
                    if c == '"' {
                        stringstatus = Stringstatus::Beforenewarray;
                        let num = intrep.parse::<u32>().unwrap();
                        myvec.push(Deck {
                            title: title.clone(),
                            id: num,
                        });
                        title.clear();
                        intrep.clear();
                    } else {
                        title.push(c);
                    }
                }
                Stringstatus::Beforenewarray => {
                    if c == ']' {
                        stringstatus = Stringstatus::Beforeint;
                    }
                }
            }
        }

        for deck in &myvec {
            if !self.descmap.contains_key(&deck.id) {
                let url = format!("https://ankiweb.net/shared/info/{}", deck.id);
                let body = reqwest::blocking::get(url).unwrap().text().unwrap();
                let desc = get_description(&body, &deck.id);
                self.descmap.insert(deck.id, desc);
                break;
            }
        }

        self.list.items = myvec;
        self.list.state = ListState::default();
    }
}

impl Tab for Ankimporter {
    fn get_title(&self) -> String {
        "ankimporter".to_string()
    }
    fn navigate(&mut self, _dir: crate::NavDir) {}
    fn get_cursor(&self) -> (u16, u16) {
        (0, 0)
    }
    fn keyhandler(&mut self, appdata: &AppData, key: MyKey) {
        match self.menu {
            Menu::Main => {
                match key {
                    MyKey::Enter => match self.list.state.selected() {
                        None => self.fetch(),
                        Some(_) if !self.is_desc_loaded() => self.update_desc(),
                        Some(idx) => {
                            let deck = self.list.items[idx].clone();
                            let name = sanitize_filename::sanitize(deck.title.clone());
                            let download_link =
                                crate::tabs::import::logic::get_download_link(deck.id);
                            let (tx, rx) = mpsc::sync_channel(1);
                            let downdeck = DeckDownload { name, rx };
                            self.menu = Menu::Downloading(downdeck);
                            use crate::tabs::import::logic::download_deck;
                            let threadpaths = appdata.paths.clone();
                            thread::spawn(move || {
                                download_deck(download_link, tx, threadpaths);
                            });
                        }
                    },
                    //MyKey::Esc => self.should_quit = ShouldQuit::Yeah,
                    MyKey::Down => {
                        self.list.next();
                    }
                    MyKey::Up => {
                        self.list.previous();
                    }
                    key => {
                        self.searchterm.keyhandler(appdata, key);
                        self.list.state.select(None);
                    }
                }
            }
            Menu::Downloading(_) => {}
        }
    }

    fn set_selection(&mut self, area: tui::layout::Rect) {
        let chunks = Layout::default()
            .direction(Horizontal)
            .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)].as_ref())
            .split(area);

        let (left, desc) = (chunks[0], chunks[1]);

        let chunks = Layout::default()
            .direction(Vertical)
            .constraints(
                [
                    Constraint::Max(3),
                    Constraint::Max(3),
                    Constraint::Ratio(5, 10),
                ]
                .as_ref(),
            )
            .split(left);

        let (prompt, searchfield, results) = (chunks[0], chunks[1], chunks[2]);

        self.prompt.set_area(prompt);
        self.searchterm.set_area(searchfield);
        self.list.set_area(results);
        self.description.set_area(desc);
    }

    fn render(
        &mut self,
        f: &mut tui::Frame<MyType>,
        appdata: &AppData,
        mut area: tui::layout::Rect,
    ) {
        if let Menu::Downloading(deck) = &self.menu {
            if let Ok(prog) = deck.rx.recv() {
                let (current, max) = prog;
                let percent = ((current as f32 / max as f32) * 100.0) as u32;
                area.height = std::cmp::min(area.height, 7);
                let mut progbar = crate::widgets::progress_bar::ProgressBar::new_full(
                    percent,
                    100,
                    Color::Blue,
                    area,
                    "Downloading deck...".to_string(),
                );
                progbar.render(f, appdata, &(0, 0));
                if current == max {
                    let deckname = deck.name.clone();
                    self.should_quit = ShouldQuit::Takethis(deckname);
                }
                return;
            } else {
                let deckname = deck.name.clone();
                self.should_quit = ShouldQuit::Takethis(deckname);
                return;
            }
        }

        self.set_selection(area);
        let cursor = &self.get_cursor();

        self.prompt.render(f, appdata, cursor);
        self.searchterm.render(f, appdata, cursor);
        self.list.render(f, appdata, cursor);

        if let Some(idx) = self.list.state.selected() {
            let id = self.list.items[idx].id;
            let mut newfield = Field::default();
            let text = match self.descmap.get(&id) {
                Some(desc) => desc.clone(),
                None => "Enter to load description ".to_string(),
            };
            newfield.replace_text(text);
            newfield.render(f, appdata, cursor);
        }
    }
}

enum Stringstatus {
    Onstring,
    Onint,
    Beforestring,
    Beforeint,
    Beforenewarray,
}
