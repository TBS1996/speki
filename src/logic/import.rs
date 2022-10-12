use crate::app::Tab;
use crate::utils::widgets::button::draw_button;
use crate::utils::widgets::message_box::draw_message;
use crate::utils::widgets::topics::TopicList;
use crate::MyKey;
use crate::{utils::misc::split_updown, Direction, SpekiPaths};
use std::fs::File;

use crate::utils::widgets::ankimporter::Ankimporter;
use crate::utils::widgets::load_cards::{ImportProgress, LoadState, Template};
use crate::MyType;
use reqwest;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use tui::{
    layout::{Constraint, Direction::Vertical, Layout},
    style::Color,
};

use crate::utils::widgets::ankimporter::ShouldQuit;
use crate::utils::widgets::filepicker::{FilePicker, PickState};
use regex::Regex;
use reqwest::header;
use std::io::prelude::*;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;

#[tokio::main]
pub async fn foo(
    url: String,
    transmitter: std::sync::mpsc::SyncSender<(u64, u64)>,
    paths: SpekiPaths,
) {
    download_deck(url, transmitter, paths).await;
}

use futures_util::StreamExt;
pub async fn download_deck(
    url: String,
    transmitter: std::sync::mpsc::SyncSender<(u64, u64)>,
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
        let _ = transmitter.try_send((new, total_size));
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

#[derive(PartialEq)]
enum Selection {
    Local,
    Anki,
}

struct Selected {
    local: bool,
    anki: bool,
}

impl Selected {
    fn from(selected: &Selection) -> Self {
        use Selection::*;
        let mut foo = Selected {
            local: false,
            anki: false,
        };
        match selected {
            Local => foo.local = true,
            Anki => foo.anki = true,
        };
        foo
    }
}

struct Unzipper {
    rx: mpsc::Receiver<UnzipStatus>,
    name: String,
}

use crate::utils::widgets::load_cards::UnzipStatus;
enum Menu {
    Main,
    Anki(Ankimporter),
    Local(FilePicker),
    LoadCards(Template),
    ImportAnki(mpsc::Receiver<ImportProgress>),
    Unzipping(Unzipper),
}

pub struct Importer {
    topics: TopicList,
    selection: Selection,
    menu: Menu,
}

impl Importer {
    pub fn new(conn: &Arc<Mutex<Connection>>) -> Importer {
        let topics = TopicList::new(conn);
        let selection = Selection::Local;
        let aim = Ankimporter::new();
        let menu = Menu::Anki(aim);

        Importer {
            topics,
            selection,
            menu,
        }
    }

    fn render_main(&mut self, f: &mut tui::Frame<MyType>, area: tui::layout::Rect) {
        let buttons = split_updown([50, 50], area);
        draw_button(
            f,
            buttons[0],
            "Local file",
            self.selection == Selection::Local,
        );
        draw_button(
            f,
            buttons[1],
            "Anki decks",
            self.selection == Selection::Anki,
        );
    }

    fn main_keyhandler(&mut self, _conn: &Arc<Mutex<Connection>>, key: MyKey) {
        use MyKey::*;
        use Selection::*;

        match (&self.selection, key) {
            (Local, Enter) | (Local, Char(' ')) => {
                let fp = FilePicker::new(["apkg".to_string()]);
                self.menu = Menu::Local(fp);
            }
            (Anki, Enter) | (Anki, Char(' ')) => {
                self.menu = Menu::Anki(Ankimporter::new());
            }
            (Local, Nav(Direction::Down)) => self.selection = Selection::Anki,
            (Anki, Nav(Direction::Up)) => self.selection = Selection::Local,
            (_, _) => {}
        }
    }
}

impl Tab for Importer {
    fn get_title(&self) -> String {
        "Import cards".to_string()
    }

    fn get_manual(&self) -> String {
        r#"

Here you can import any anki decks you want! audio included, but not yet images. Press enter to view description about the deck, and then enter again to download

When inspecting the deck, you can edit the templates for the deck. The front/back view are how the cards will look like after you import them! 

If you don't want to import the selected deck, press escape!


        "#.to_string()
    }

    fn keyhandler(
        &mut self,
        conn: &Arc<Mutex<Connection>>,
        key: MyKey,
        handle: &rodio::OutputStreamHandle,
        paths: &SpekiPaths,
    ) {
        match &mut self.menu {
            Menu::Main => self.main_keyhandler(conn, key),
            Menu::Anki(ankimporter) => match &ankimporter.should_quit {
                ShouldQuit::No => ankimporter.keyhandler(key, conn, paths),
                ShouldQuit::Yeah => {
                    self.menu = Menu::Anki(Ankimporter::new());
                }
                ShouldQuit::Takethis(_) => {}
            },

            Menu::Local(loc) => {
                match &loc.state {
                    PickState::Ongoing => loc.keyhandler(key),
                    PickState::ExitEarly => {
                        self.menu = Menu::Main;
                    }
                    PickState::Fetch(path) => {
                        let mut foldername = path.to_str().to_owned().unwrap().to_string();
                        foldername.pop();
                        foldername.pop();
                        foldername.pop();
                        foldername.pop();
                        let foldername = foldername.rsplit_once('/').unwrap().1.to_string();
                        let template = Template::new(conn, foldername, paths);
                        self.menu = Menu::LoadCards(template);
                    }
                };
            }
            Menu::LoadCards(tmpl) => match tmpl.state {
                LoadState::OnGoing => {
                    tmpl.keyhandler(conn, key, handle);
                    if let LoadState::Importing = tmpl.state {
                        let mut tmpclone = tmpl.clone();
                        let (tx, rx): (mpsc::SyncSender<ImportProgress>, Receiver<ImportProgress>) =
                            mpsc::sync_channel(5);
                        let connclone = Arc::clone(conn);
                        thread::spawn(move || {
                            tmpclone.import_cards(connclone, tx);
                        });
                        self.menu = Menu::ImportAnki(rx);
                    }
                }
                LoadState::Importing => {}
                LoadState::Finished => self.menu = Menu::Anki(Ankimporter::new()),
            },
            Menu::ImportAnki(_) => {}
            Menu::Unzipping(_) => {}
        }
    }

    fn render(
        &mut self,
        f: &mut tui::Frame<MyType>,
        area: tui::layout::Rect,
        conn: &Arc<Mutex<Connection>>,
        paths: &SpekiPaths,
    ) {
        match &mut self.menu {
            Menu::Main => self.render_main(f, area),
            Menu::Local(filesource) => {
                filesource.render(f, area);
            }
            Menu::Anki(ankimporter) => {
                match &ankimporter.should_quit {
                    ShouldQuit::No => ankimporter.render(conn, f, area),
                    ShouldQuit::Yeah => self.render_main(f, area),
                    ShouldQuit::Takethis(deckname) => {
                        let (tx, rx): (mpsc::Sender<UnzipStatus>, Receiver<UnzipStatus>) =
                            mpsc::channel();
                        let threadpaths = paths.clone();
                        let deckname = deckname.to_string();
                        let anotherone = deckname.clone();
                        thread::spawn(move || {
                            Template::unzip_deck(threadpaths, deckname.clone(), tx);
                        });
                        self.menu = Menu::Unzipping(Unzipper {
                            rx,
                            name: anotherone,
                        });
                    }
                };
            }

            Menu::Unzipping(unzipper) => {
                if let Ok(unstat) = unzipper.rx.recv() {
                    if let UnzipStatus::Ongoing(msg) = unstat {
                        draw_message(f, area, &msg);
                    } else {
                        let tmpl = Template::new(conn, unzipper.name.clone(), paths);
                        self.menu = Menu::LoadCards(tmpl);
                    }
                } else {
                    let tmpl = Template::new(conn, unzipper.name.clone(), paths);
                    self.menu = Menu::LoadCards(tmpl);
                }
            }

            Menu::LoadCards(tmpl) => {
                tmpl.render(f, area);
                if let LoadState::Finished = tmpl.state {
                    let aim = Ankimporter::new();
                    self.menu = Menu::Anki(aim);
                }
            }
            Menu::ImportAnki(rx) => {
                if let Ok(prog) = rx.recv() {
                    let rightcol = Layout::default()
                        .direction(Vertical)
                        .constraints(
                            [
                                Constraint::Ratio(1, 3),
                                Constraint::Ratio(1, 3),
                                Constraint::Ratio(1, 3),
                            ]
                            .as_ref(),
                        )
                        .split(area);
                    let mut progbar = rightcol[0];

                    progbar.height = std::cmp::min(progbar.height, 5);
                    crate::utils::widgets::progress_bar::progress_bar(
                        f,
                        prog.curr_index as u32,
                        prog.total as u32,
                        Color::LightMagenta,
                        progbar,
                        "Importing cards..",
                    );

                    if prog.curr_index == prog.total - 1 {
                        self.menu = Menu::Anki(Ankimporter::new());
                    }
                } else {
                    self.menu = Menu::Anki(Ankimporter::new());
                }
            }
        }
    }
}
