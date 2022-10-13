use std::collections::HashMap;

use crate::utils::statelist::StatefulList;
use crate::widgets::textinput::Field;
use crate::{MyType, SpekiPaths};
use reqwest;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use tui::{
    layout::{Constraint, Direction::Vertical, Layout},
    widgets::ListState,
};

use super::message_box::draw_message;
use crate::MyKey;
use tui::layout::Direction::Horizontal;

use regex::*;

use tui::{
    style::{Color, Modifier, Style},
    text::Spans,
    widgets::{Block, Borders},
};

use tui::widgets::List;

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

impl Ankimporter {
    pub fn new() -> Self {
        let list = StatefulList::<Deck>::new();
        let searchterm = Field::new();
        let description = Field::new();
        let menu = Menu::Main;

        Ankimporter {
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

    pub fn keyhandler(&mut self, key: MyKey, _conn: &Arc<Mutex<Connection>>, paths: &SpekiPaths) {
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
                            let threadpaths = paths.clone();
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
                        self.searchterm.keyhandler(key);
                        self.list.state.select(None);
                    }
                }
            }
            Menu::Downloading(_) => {}
        }
    }

    pub fn render(
        &mut self,
        _conn: &Arc<Mutex<Connection>>,
        f: &mut tui::Frame<MyType>,
        mut area: tui::layout::Rect,
    ) {
        if let Menu::Downloading(deck) = &self.menu {
            if let Ok(prog) = deck.rx.recv() {
                let (current, max) = prog;
                let percent = ((current as f32 / max as f32) * 100.0) as u32;
                area.height = std::cmp::min(area.height, 7);
                crate::widgets::progress_bar::progress_bar(
                    f,
                    percent,
                    100,
                    Color::Blue,
                    area,
                    "Downloading deck...",
                );
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

        let items = {
            let items: Vec<ListItem> = self
                .list
                .items
                .iter()
                .map(|item| {
                    let lines = vec![Spans::from((*item).title.clone())];
                    ListItem::new(lines).style(Style::default().fg(Color::Black).bg(Color::Red))
                })
                .collect();

            let items =
                List::new(items).block(Block::default().borders(Borders::ALL).title("Decks"));
            let items = items.highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            );
            items
        };

        draw_message(f, prompt, "Select an anki deck!");
        self.searchterm.render(f, searchfield, true);
        f.render_stateful_widget(items, results, &mut self.list.state);

        if let Some(idx) = self.list.state.selected() {
            let id = self.list.items[idx].id;
            let mut newfield = Field::new();
            let text = match self.descmap.get(&id) {
                Some(desc) => desc.clone(),
                None => "Enter to load description ".to_string(),
            };
            newfield.replace_text(text);
            newfield.render(f, desc, false);
        }
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

enum Stringstatus {
    Onstring,
    Onint,
    Beforestring,
    Beforeint,
    Beforenewarray,
}

use tui::widgets::ListItem;
