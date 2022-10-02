use crate::Direction;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{UNIX_EPOCH, SystemTime};
use std::{fs::File, io::Read};
use crate::MyKey;
use crate::tabs::Widget;
use crate::utils::widgets::button::draw_button;
use crate::utils::widgets::message_box::draw_message;
use crate::utils::widgets::textinput::Field;
use crate::utils::widgets::topics::TopicList;
use crate::utils::{
    card::{Card, Review, Status},
    sql::insert::save_card,

};
use tui::widgets::ListState;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction::{Vertical, Horizontal}, Layout, Rect},
    style::{Color, Style, Modifier},
    widgets::{Block, Borders, ListItem, List},
    text::Spans,
    Frame,
};
use std::sync::{Arc, Mutex};
use crate::utils::aliases::*;
use rusqlite::Connection;
use csv::StringRecord;
use anyhow::Result;
use crate::utils::card::CardType;
use crate::MyType;
use reqwest;
use std::fs;
use crate::utils::widgets::ankimporter::{Ankimporter, ShouldQuit};
use crate::utils::widgets::load_cards::{Template, LoadState};


use sanitize_filename;

use crate::utils::widgets::filepicker::{FilePicker, PickState};
use regex::Regex;
use reqwest::header;
use std::io::prelude::*;


fn download_the_deck(url: String) -> String{
        std::fs::remove_dir_all("./temp").unwrap();
        std::fs::create_dir("./temp").unwrap();

        let downloc = "./temp/ankitemp.apkg";
        let body = reqwest::blocking::get(url).unwrap();
        let path = std::path::Path::new(downloc);
        let mut file = match File::create(&path) {
            Err(why) => panic!("couldn't create {}", why),
            Ok(file) => file,
        };
        file.write_all(&(body.bytes().unwrap())).unwrap();
        downloc.to_string()
    }



fn extract_download_link(trd: &String) -> String{
    let pattern = r"(?P<link>(https:.*));".to_string();
    let re = Regex::new(&pattern).unwrap();
    let foo = re.captures(&trd).unwrap();
    foo.get(1).unwrap().as_str().to_string()
}

fn get_k_value(pagesource: &String) -> String{
    let pattern = "k\" value=\"(.*)\"".to_string();
    let re = Regex::new(&pattern).unwrap();
    let foo = re.captures(&pagesource).unwrap();
    foo.get(1).unwrap().as_str().to_string()
}

fn get_download_link(pagesource: &String, deck: u32)->String{
    let k = get_k_value(pagesource);
    let main = format!("https://ankiweb.net/shared/downloadDeck/{}", deck);
    let body = format!("k={}&submit=Download", k);
    let mut headers = header::HeaderMap::new();
    headers.insert("Content-Type", "application/x-www-form-urlencoded".parse().unwrap());
    let client = reqwest::blocking::Client::builder()
    .redirect(reqwest::redirect::Policy::none())
    .build()
    .unwrap();
    let result = client.post(main)
        .headers(headers)
        .body(body)
        .send().unwrap()
        .text().unwrap();
    let link = extract_download_link(&result);
    return link;
}


pub fn download_deck(deckid: u32) -> String{
    let url = format!("https://ankiweb.net/shared/info/{}", deckid);
    let body = reqwest::blocking::get(url).unwrap().text().unwrap();
    let download_link = get_download_link(&body, deckid);
    let download_path = download_the_deck(download_link);
    download_path
}


#[derive(PartialEq)]
enum Selection{
    Local,
    Anki,
}


struct Selected{
    local: bool,
    anki: bool,
}

impl Selected{
    fn from(selected: &Selection)-> Self{
        use Selection::*;
        let mut foo = Selected{
            local: false,
            anki: false,
        };
        match selected{
            Local => foo.local = true,
            Anki => foo.anki = true,
        };
        foo
    }
}



enum DeckSource{
    id(u32),
    path(PathBuf),
}



enum Menu{
    Main,
    Anki(Ankimporter),
    Local(FilePicker),
    LoadCards(Template),
    
}

//use crate::utils::widgets::filepicker::FilePicker;

pub struct Importer{
    topics: TopicList,
    selection: Selection,
    menu: Menu,
}

use crate::utils::widgets::list::list_widget;

impl Importer{
    pub fn new(conn: &Arc<Mutex<Connection>>) -> Importer{
        let topics = TopicList::new(conn);
        let selection = Selection::Local;
        let aim = Ankimporter::new();
        let menu = Menu::Anki(aim);

        Importer{
            topics,
            selection,
            menu,
        }
    }
    pub fn render(&mut self, f: &mut tui::Frame<MyType>, area: tui::layout::Rect) {

        match &mut self.menu{
            Menu::Main => self.render_main(f, area),
            Menu::Local(filesource) => {
                filesource.render(f, area);
            },
            Menu::Anki(ankimporter) => {
                match &ankimporter.should_quit{
                    ShouldQuit::No => ankimporter.render(f, area),
                    ShouldQuit::Yeah => self.render_main(f, area),
                    ShouldQuit::Takethis(tmp) => self.menu = Menu::LoadCards(tmp.clone()),
                    };
                }
            
            Menu::LoadCards(tmpl) => {
                tmpl.render(f, area);
                if let LoadState::Finished = tmpl.state{
                    let aim = Ankimporter::new();
                    self.menu = Menu::Anki(aim);
                }
            },
        }
    }

    fn render_main(&mut self, f: &mut tui::Frame<MyType>, area: tui::layout::Rect) {
        let buttons = Layout::default()
            .direction(Vertical)
            .constraints(
                [
                Constraint::Ratio(1, 2),
                Constraint::Ratio(1, 2)
                ]
                .as_ref(),
                )
            .split(area);
        draw_button(f, buttons[0], "Local file", self.selection == Selection::Local);
        draw_button(f, buttons[1], "Anki decks", self.selection == Selection::Anki);
    }





    pub fn keyhandler(&mut self, conn: &Arc<Mutex<Connection>>, key: MyKey, handle: &rodio::OutputStreamHandle){
        use MyKey::*;
        use Selection::*;
        
        match &mut self.menu{
            Menu::Main => self.main_keyhandler(conn, key),
            Menu::Anki(ankimporter) => {
                use crate::utils::widgets::ankimporter::{ShouldQuit, Deck};
                match &ankimporter.should_quit{
                    ShouldQuit::No => ankimporter.keyhandler(key, conn),
                    ShouldQuit::Yeah => {
                        self.menu = Menu::Main;
                    },
                    ShouldQuit::Takethis(deck) => {
                    }
                }
            },

            Menu::Local(loc) => {
                match &loc.state{
                    PickState::Ongoing => loc.keyhandler(key),
                    PickState::ExitEarly => {
                        self.menu = Menu::Main;
                    },
                    PickState::Fetch(path) => {
                        let mut foldername = path.to_str().to_owned().unwrap().to_string();
                        foldername.pop();
                        foldername.pop();
                        foldername.pop();
                        foldername.pop();
                        let foldername = foldername.rsplit_once('/').unwrap().1.to_string();
                        let template = Template::new(conn, path.to_path_buf(), foldername);
                        self.menu = Menu::LoadCards(template);
                        },
                    };
                },
            Menu::LoadCards(tmpl) => {
                match tmpl.state{
                    LoadState::OnGoing => tmpl.keyhandler(conn, key, handle),
                    LoadState::Importing(_) => {},
                    LoadState::Finished => self.menu = Menu::Main,
                }
            }
        }
    }

    fn main_keyhandler(&mut self, _conn: &Arc<Mutex<Connection>>, key: MyKey){
        use MyKey::*;
        use Selection::*;
        
        match (&self.selection, key){
            (Local, Enter) | (Local, Char(' ')) => {
                let fp = FilePicker::new(["apkg".to_string()]);
                self.menu = Menu::Local(fp);
            }, 
            (Anki,  Enter) | (Anki, Char(' ')) => {
                let aim = Ankimporter::new();
                self.menu = Menu::Anki(aim);
            }, 
            (Local, Nav(Direction::Down)) => self.selection = Selection::Anki,
            (Anki,  Nav(Direction::Up))    => self.selection = Selection::Local,
            (_,_) => {},

        }
    }



}
  





