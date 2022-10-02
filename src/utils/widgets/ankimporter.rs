use std::collections::HashMap;

use crate::MyType;
use reqwest;
use crate::utils::{aliases::*, sql::insert::update_both, card::Card};
use rusqlite::Connection;
use crate::utils::sql::fetch::load_cards;
use crate::utils::statelist::StatefulList;
use crate::utils::widgets::textinput::Field;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction::Vertical, Layout, Rect},
    Frame, widgets::ListState,
};
use std::sync::{Arc, Mutex};

use crate::utils::widgets::list::list_widget;
use super::{message_box::draw_message, list::StraitList};
use crate::MyKey;
use crate::utils::misc::PopUpStatus;
use tui::layout::Direction::Horizontal;

use regex::*;
use reqwest::header;

use tui::{
    style::{Color, Modifier, Style},
    text::Spans,
    widgets::{
        Block, Borders},
};

use tui::widgets::List;

pub enum ShouldQuit{
    No,
    Yeah,
    Takethis(super::load_cards::Template),
}


fn get_description(pagesource: &String, id: &u32) -> String{
    let pattern = "<div class=\"shared-item-description pb-3\">((.|\n)*)<h2>Sample".to_string();
    let re = Regex::new(&pattern).unwrap();
    let foo = match re.captures(&pagesource){
        Some(x) => x,
        None => panic!("{}, @@{}", pagesource, id),
    };
    foo.get(1).unwrap().as_str().to_string()
}




pub struct Ankimporter{
    searchterm: Field,
    description: Field,
    list: StatefulList<Deck>,
    descmap: HashMap<u32, String>,
    pub should_quit: ShouldQuit,
}


#[derive(Clone, PartialEq)]
pub struct Deck{
    pub title: String,
    pub id: u32,
}

impl Ankimporter{
    pub fn new() -> Self{
        let list = StatefulList::<Deck>::new();
        let searchterm = Field::new();
        let description = Field::new();

        Ankimporter {
            searchterm,
            description,
            list,
            descmap: HashMap::new(),
            should_quit: ShouldQuit::No,

        }
    }
    
    fn update_desc(&mut self){
        if let Some(idx) = self.list.state.selected(){
            let id = self.list.items[idx].id;
            if !self.descmap.contains_key(&id){
                let url = format!("https://ankiweb.net/shared/info/{}", id);
                let body = reqwest::blocking::get(url).unwrap().text().unwrap();
                let desc = get_description(&body, &id);
                self.descmap.insert(id, desc);
            }
        }
    }

    fn is_desc_loaded(&self) -> bool {
        if let Some(idx) = self.list.state.selected(){
            let id = self.list.items[idx].id;
            return self.descmap.contains_key(&id)
        } 
        false
    }

    pub fn keyhandler(&mut self, key: MyKey, conn: &Arc<Mutex<Connection>>){
        match key {
            MyKey::Enter => {
                match self.list.state.selected(){
                    None => self.fetch(),
                    Some(_) if !self.is_desc_loaded() => self.update_desc(),
                    Some(idx) => self.choose(idx, conn),
                }
            }
            MyKey::Esc => self.should_quit = ShouldQuit::Yeah,
            MyKey::Down => {
                self.list.next();

            },
            MyKey::Up => {
                self.list.previous();
            },
            MyKey::Right => self.update_desc(),
            key => {
                self.searchterm.keyhandler(key);
                self.list.state.select(None);
            },
            
        }
    }

            

    pub fn render(&mut self, f: &mut tui::Frame<MyType>, area: tui::layout::Rect) {
        let chunks = Layout::default()
            .direction(Horizontal)
            .constraints([
                         Constraint::Ratio(1, 2),
                         Constraint::Ratio(1, 2)
            ]
            .as_ref(),)
            .split(area);
    
        let (left, desc) = (chunks[0], chunks[1]);

        let chunks = Layout::default()
            .direction(Vertical)
            .constraints([
                         Constraint::Max(5),
                         Constraint::Max(3),
                         Constraint::Ratio(1, 10),
            ]
            .as_ref(),)
            .split(left);

        let (prompt, searchfield, results) = (chunks[0], chunks[1], chunks[2]);



        let items = {
            let items: Vec<ListItem> = self.list.items.iter()
            .map(|item| {
                let lines = vec![Spans::from((*item).title.clone())];
                ListItem::new(lines).style(Style::default().fg(Color::Black).bg(Color::Red))
            })
            .collect();
    
            let items = List::new(items).block(Block::default().borders(Borders::ALL).title("Decks"));
            let items = items
                .highlight_style(
                    Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            );
            items
        };


        draw_message(f, prompt, "Select an anki deck!");
        self.searchterm.render(f, searchfield, true);
        f.render_stateful_widget(items, results, &mut self.list.state);

        if let Some(idx) = self.list.state.selected(){
            let id = self.list.items[idx].id;
            let mut newfield = Field::new();
            let text = match self.descmap.get(&id){
                Some(desc) => desc.clone(),
                None => "Enter to load description ".to_string(),
            };
            newfield.replace_text(text);
            newfield.render(f, desc, false);
        }
    }


    fn choose(&mut self, idx: usize, conn: &Arc<Mutex<Connection>>) {
        use std::path::PathBuf;
        let deck = self.list.items[idx].clone();
        let downpath = crate::logic::import::download_deck(deck.id);
        let name = sanitize_filename::sanitize(deck.title.clone());
        let tmp = super::load_cards::Template::new(conn, PathBuf::from(downpath), name);
        self.should_quit = ShouldQuit::Takethis(tmp);
    }

    

    fn fetch(&mut self){
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
        for c in foo.chars(){
            if c == ';' {break}

            match stringstatus{
                Stringstatus::Beforeint => {
                    if c.is_ascii_digit(){
                        intrep.push(c);
                        stringstatus = Stringstatus::Onint;
                    }                 },
                Stringstatus::Onint => {
                    if c.is_ascii_digit(){
                        intrep.push(c);
                    } else {
                        stringstatus = Stringstatus::Beforestring;
                    }
                },
                Stringstatus::Beforestring => {
                    if c == '\"'{
                        stringstatus = Stringstatus::Onstring;
                    }
                },
                Stringstatus::Onstring => {
                    if c == '"'{
                        stringstatus = Stringstatus::Beforenewarray;
                        let num = intrep.parse::<u32>().unwrap();
                        myvec.push(
                            Deck{
                                title: title.clone(),
                                id: num,
                            }
                            );
                        title.clear();
                        intrep.clear();
                    } else {
                        title.push(c);
                    }
                },
                Stringstatus::Beforenewarray => {
                    if c == ']' {
                        stringstatus = Stringstatus::Beforeint;
                    }

                },

            }
        }



        for deck in &myvec{
            if !self.descmap.contains_key(&deck.id){
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

enum Stringstatus{
    Onstring,
    Onint,
    Beforestring,
    Beforeint,
    Beforenewarray,
}

use tui::widgets::ListItem;

