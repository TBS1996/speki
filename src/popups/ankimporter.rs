use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

use crate::utils::ankitemplate::{ImportProgress, Template};
use crate::utils::libextensions::MyListState;
use crate::utils::misc::SpekiPaths;
use crate::utils::statelist::{KeyHandler, StatefulList};
use crate::widgets::textinput::Field;
use crate::MyType;
use hyper::header;
use reqwest;
//use tokio::sync::mpsc;
use tui::layout::{Constraint, Direction::Vertical, Layout};

use crate::widgets::button::Button;
use crate::MyKey;
use tui::layout::Direction::Horizontal;

use regex::*;

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

use std::sync::mpsc::{self, Receiver};
use std::thread;

pub struct Ankimporter {
    prompt: Button,
    searchterm: Field,
    description: Field,
    list: StatefulList<Deck>,
    descmap: HashMap<u32, String>,
    state: State,
    tabdata: TabData,
}

#[derive(Clone)]
enum State {
    Main,
    Downloading(String),
    Unzipping(String),
    Renaming(String),
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
use crate::app::{AppData, Tab, TabData, Widget};
impl Ankimporter {
    pub fn new() -> Self {
        let mut list = StatefulList::<Deck>::new("".to_string());
        list.persistent_highlight = true;
        let searchterm = Field::default();
        let description = Field::default();

        Ankimporter {
            prompt: Button::new("Choose anki deck".to_string()),
            searchterm,
            description,
            list,
            descmap: HashMap::new(),
            state: State::Main,
            tabdata: TabData::default(),
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
        self.list.state = MyListState::default();
    }
}

impl Tab for Ankimporter {
    fn get_title(&self) -> String {
        "ankimporter".to_string()
    }
    fn navigate(&mut self, _dir: crate::NavDir) {}

    fn get_tabdata(&mut self) -> &mut TabData {
        &mut self.tabdata
    }

    fn exit_popup(&mut self, appdata: &AppData) {
        if self.tabdata.popup.is_some() {
            match self.state.clone() {
                State::Downloading(name) => {
                    let deckname = name.clone();
                    let (tx, rx): (mpsc::Sender<Msg>, Receiver<Msg>) = mpsc::channel();
                    let threadpaths = appdata.paths.clone();
                    let deckname = deckname;
                    thread::spawn(move || {
                        Template::unzip_deck(threadpaths, deckname, tx);
                    });
                    let msg = MsgPopup::new(rx, "Unzipping".to_string());
                    self.set_popup(Box::new(msg));
                    self.state = State::Unzipping(name.clone());
                }
                State::Unzipping(name) => {
                    let (tx, rx): (mpsc::SyncSender<ImportProgress>, Receiver<ImportProgress>) =
                        mpsc::sync_channel(2);
                    let deckname = name.clone();
                    let paths = appdata.paths.clone();
                    thread::spawn(move || {
                        Template::rename_media(deckname, paths, tx).unwrap();
                    });
                    let prog = Progress::new(rx, "Preparing media files".to_string());
                    self.set_popup(Box::new(prog));
                    self.state = State::Renaming(name);
                }
                State::Renaming(name) => {
                    let ldc = LoadCards::new(appdata, name.clone());
                    self.set_popup(Box::new(ldc));
                    self.state = State::Main;
                }
                State::Main => self.tabdata.popup = None,
            }
        }
    }

    fn keyhandler(&mut self, appdata: &AppData, key: MyKey, _cursor: &(u16, u16)) {
        match key {
            MyKey::Enter => match self.list.state.selected() {
                None => self.fetch(),
                Some(_) if !self.is_desc_loaded() => self.update_desc(),
                Some(idx) => {
                    let deck = self.list.items[idx].clone();
                    let name = sanitize_filename::sanitize(deck.title.clone());
                    let download_link = get_download_link(deck.id);
                    let (tx, rx): (mpsc::SyncSender<ImportProgress>, Receiver<ImportProgress>) =
                        mpsc::sync_channel(2);
                    let threadpaths = appdata.paths.clone();
                    thread::spawn(move || {
                        download_deck(download_link, tx, threadpaths);
                    });
                    let prog = Progress::new(rx, "Downloading deck".to_string());
                    self.set_popup(Box::new(prog));
                    self.state = State::Downloading(name);
                }
            },
            MyKey::Down => {
                self.list.next();
            }
            MyKey::Up => {
                self.list.previous();
            }
            MyKey::KeyPress(pos) => self.list.keypress(appdata, pos),

            key => {
                self.searchterm.keyhandler(appdata, key);
                self.list.state.select(None);
            }
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

    fn render(&mut self, f: &mut tui::Frame<MyType>, appdata: &AppData, _cursor: &(u16, u16)) {
        let cursor = &(0, 0);
        self.prompt.render(f, appdata, cursor);
        self.searchterm.render(f, appdata, cursor);
        self.list.render(f, appdata, cursor);

        if let Some(idx) = self.list.state.selected() {
            let id = self.list.items[idx].id;
            self.description.replace_text(match self.descmap.get(&id) {
                Some(desc) => desc.clone(),
                None => "Enter to load description ".to_string(),
            });
            self.description.render(f, appdata, cursor);
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

use futures_util::StreamExt;

use super::load_cards::LoadCards;
use super::message_popup::{Msg, MsgPopup};
use super::progress_popup::Progress;
#[tokio::main]
pub async fn download_deck(
    url: String,
    transmitter: std::sync::mpsc::SyncSender<ImportProgress>,
    paths: SpekiPaths,
) {
    if !std::path::Path::new(&paths.tempfolder).exists() {
        std::fs::create_dir(&paths.tempfolder).unwrap();
    } else {
        std::fs::remove_dir_all(&paths.tempfolder).unwrap();
        std::fs::create_dir(&paths.tempfolder).unwrap();
    }

    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .send()
        .await
        .or(Err(format!("Failed to GET from '{}'", &url)))
        .unwrap();

    let total_size = res
        .content_length()
        .ok_or(format!("Failed to get content length from '{}'", &url))
        .unwrap();

    // download chunks
    let mut file = File::create(&paths.downloc).unwrap();
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item
            .or(Err(format!("Error while downloading file")))
            .unwrap();
        file.write_all(&chunk)
            .or(Err(format!("Error while writing to file")))
            .unwrap();
        let new = std::cmp::min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        let _ = transmitter.try_send(ImportProgress {
            curr_index: new as usize,
            max: total_size as usize,
        });
    }
}

fn extract_download_link(trd: &String) -> String {
    let pattern = r"(?P<link>(https:.*));".to_string();
    let re = Regex::new(&pattern).unwrap();
    let foo = re.captures(&trd).expect(&format!(
        "Couldnt find pattern on following string: {}@@",
        trd
    ));
    foo.get(1).unwrap().as_str().to_string()
}

fn get_k_value(pagesource: &String) -> String {
    let pattern = "k\" value=\"(.*)\"".to_string();
    let re = Regex::new(&pattern).unwrap();
    let foo = re.captures(&pagesource).unwrap();
    foo.get(1).unwrap().as_str().to_string()
}

pub fn get_download_link(deckid: u32) -> String {
    let url = format!("https://ankiweb.net/shared/info/{}", deckid);
    let pagesource = reqwest::blocking::get(url).unwrap().text().unwrap();
    let k = get_k_value(&pagesource);
    let main = format!("https://ankiweb.net/shared/downloadDeck/{}", deckid);
    let body = format!("k={}&submit=Download", k);
    let mut headers = header::HeaderMap::new();
    headers.insert(
        "Content-Type",
        "application/x-www-form-urlencoded".parse().unwrap(),
    );
    let client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let result = client
        .post(main)
        .headers(headers)
        .body(body)
        .send()
        .unwrap()
        .text()
        .unwrap();
    let link = extract_download_link(&result);
    return link;
}
