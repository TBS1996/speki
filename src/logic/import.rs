use crate::Direction;
use std::time::{UNIX_EPOCH, SystemTime};
use std::{fs::File, io::Read};
use crate::MyKey;
use crate::tabs::Widget;
use crate::utils::widgets::button::draw_button;
use crate::utils::widgets::filepicker::FilePicker;
use crate::utils::widgets::message_box::draw_message;
use crate::utils::widgets::textinput::Field;
use crate::utils::widgets::topics::TopicList;
use crate::utils::{
    card::{Card, Review, Status},
    sql::insert::save_card,

};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction::{Vertical, Horizontal}, Layout, Rect},
    style::{Color, Style, Modifier},
    widgets::{Block, Borders, ListItem, List},
    text::Spans,
    Frame,
};
use crate::utils::aliases::*;
use rusqlite::Connection;
use csv::StringRecord;
use anyhow::Result;
use crate::utils::card::CardType;
use crate::MyType;
use reqwest;


enum FileSource{
    None,
    Url(Field),
    Local(FilePicker),
}

enum Selection{
    Topics,
    Notes,
    Nonefile,
    Urlfile,
    Localfile,
    Preview,
}


struct Selected{
    topics: bool,
    notes: bool,
    nonefile: bool,
    urlfile: bool,
    localfile: bool,
    preview: bool,
}

impl Selected{
    fn from(selected: &Selection)-> Self{
        use Selection::*;
        let mut foo = Selected{
            topics: false,
            notes: false,
            nonefile: false,
            urlfile: false,
            localfile: false,
            preview: false,
        };
        match selected{
            Topics => foo.topics = true,
            Notes => foo.notes = true,
            Nonefile => foo.nonefile = true,
            Urlfile => foo.urlfile = true,
            Localfile => foo.localfile = true,
            Preview => foo.preview = true,
        };
        foo
    }
}


pub struct Importer{
    ankon: Connection,
    topics: TopicList,
    notes: Option<Vec<Vec<String>>>,
    filesource: FileSource,
    question: Field,
    answer: Field,
    selection: Selection,
}

use crate::utils::widgets::list::list_widget;

impl Importer{
    pub fn new(conn: &Connection) -> Importer{
        let ankon = Connection::open("collection.anki2").unwrap();
        let notes = None;
        let topics = TopicList::new(conn);
        //let filesource = FileSource::Url(Field::new());
        let filesource = FileSource::None;
        let question = Field::new();
        let answer = Field::new();
        let selection = Selection::Nonefile;
        Importer::download_deck().unwrap();

        Importer{
            ankon,
            topics,
            notes,
            filesource,
            question,
            answer,
            selection,
        }
    }

    fn import_cards(&mut self, conn: &Connection){
        if let Some(notes) = &self.notes{
            for note in notes{
                let question = note[0].clone();
                let answer = note[1].clone();
                Card::new()
                    .question(question)
                    .answer(answer)
                    .cardtype(CardType::Pending)
                    .save_card(conn);
            }
        }
    }
    
    pub fn render(&mut self, f: &mut tui::Frame<MyType>, area: tui::layout::Rect) {
        let selection = Selected::from(&self.selection);

        let leftright = Layout::default()
            .direction(Horizontal)
            .constraints(
                [
                Constraint::Ratio(2, 3),
                Constraint::Ratio(1, 3),
                ]
                .as_ref(),
                )
            .split(area);
        let (left, right) = (leftright[0], leftright[1]);



        let leftcol = Layout::default()
            .direction(Vertical)
            .constraints(
                [
                Constraint::Ratio(1, 4),
                Constraint::Ratio(1, 4),
                Constraint::Ratio(1, 4),
                Constraint::Ratio(1, 4),
                ]
                .as_ref(),
                )
            .split(left);



        match &mut self.filesource{
            FileSource::Url(url) => {
                url.render(f, leftcol[0], selection.urlfile)
            },
            FileSource::Local(loc) => {
                loc.render(f, leftcol[0], selection.localfile)
            },
            FileSource::None => {
                draw_button(f, leftcol[0], "U -> url.  L -> local", selection.nonefile);
            }
        }
        draw_button(f, leftcol[1], "preview cards", selection.preview);
        self.question.render(f, leftcol[2], false);
        self.answer.render(f, leftcol[3], false);
        list_widget(f, &self.topics, right, selection.topics);
    }



    fn navigate(&mut self, dir: Direction){
        use Direction::*;
        use Selection::*;

        match (self.selection, dir){

        }

    }


    pub fn keyhandler(&mut self, conn: &Connection, key: MyKey){
        use MyKey::*;
        match key {
            Char('i') => self.import_cards(conn),
            _ => {},
        }
    }


    fn download_deck() -> Result<()>{
        if !std::path::Path::new("ankidecks/").exists(){
            std::fs::create_dir("ankidecks/").unwrap();
        }
        let target = "https://dl13.ankiweb.net/shared/downloadDeck2/399999380?k=WzM5OTk5OTM4MCwgMjM4MjAsIDE1MzcwMTQzMzld.wOZQe9zANTuefBwIGildi2HWUDe6V34-7T1eM-1jb7g";
        let body = reqwest::blocking::get(target)?;
        let path = std::path::Path::new("./download.zip");
        let mut file = match File::create(&path) {
            Err(why) => panic!("couldn't create {}", why),
            Ok(file) => file,
        };
        file.write_all(&(body.bytes().unwrap()))?;
        Ok(())


    }
}


use std::io::prelude::*;





pub fn load_notes(conn: &Connection) -> Result<Vec<Vec<String>>>{
    let mut stmt = conn.prepare("SELECT flds FROM notes")?;
    let inc_iter = stmt.query_map([], |row| {
                                  let myvec: Vec<String> = row
                                      .get::<usize, String>(0)
                                      .unwrap()
                                      .split('')
                                      .map(|x| x.to_string())
                                      .collect();

                                  Ok(myvec)
                                   })
    ?;


    let mut outervec = Vec::new();
    for inc in inc_iter {
        outervec.push(inc.unwrap().clone());}
    Ok(outervec)
}


