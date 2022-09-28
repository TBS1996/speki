
use crate::Direction;
use crate::ui::review::draw_progress_bar;
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
use crate::utils::aliases::*;
use rusqlite::Connection;
use csv::StringRecord;
use anyhow::Result;
use crate::utils::card::CardType;
use crate::MyType;
use reqwest;
use std::fs;
use std::io;
use crate::utils::widgets::ankimporter::{Ankimporter, ShouldQuit};
use crate::utils::widgets::list::list_widget;



use crate::utils::widgets::filepicker::{FilePicker, PickState};

use super::progress_bar;



#[derive(Clone, Debug)]
pub struct ImportProgress{
    question: String,
    answer: String,
    curr_index: u32,
    total: u32,
}

impl ImportProgress{
    fn new(front: String, back: String, curr: u32, tot: u32)-> Self{
        ImportProgress{
            question: front,
            answer: back,
            curr_index: curr,
            total: tot,
        }

    }
}


#[derive(Clone, Debug)]
pub enum LoadState{
    OnGoing,
    Finished,
    Importing(ImportProgress), // e.g.  70/500 cards imported...
}

#[derive(Clone, Debug)]
enum Selected{
    Front,
    Back,
    Topics,
    Import,
    Preview,
}


struct IsSelected{
    front: bool,
    back: bool,
    topics: bool,
    import: bool,
    preview: bool,
}

impl IsSelected{
    fn new(selected: &Selected) -> Self{

        let mut foo = IsSelected{
            front: false,
            back: false,
            topics: false,
            import: false,
            preview: false,
        };

        match selected{
            Selected::Front => foo.front = true,
            Selected::Back => foo.back = true,
            Selected::Topics => foo.topics = true,
            Selected::Import => foo.import = true,
            Selected::Preview => foo.preview = true,
        };
        foo

    }
}


#[derive(Clone, Debug)] 
enum Ftype{ 
    Text,
    Audio,
    Image,
}

impl std::fmt::Display for CardField {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let emoji = match self.ftype{
            Ftype::Text =>  '📖',
            Ftype::Audio => '🔊',
            Ftype::Image => '📷',
        };
        write!(f, "{} {}", emoji, self.name)
    }
}


impl CardField{
    fn new(string: String)-> CardField{
        CardField{
            name: string,
            ftype: Ftype::Text,
        }
    }
    fn change_ftype(&mut self, ftype: Ftype){
        self.ftype = ftype;
    }
}



#[derive(Clone, Debug)]
struct CardField{
    name: String,
    ftype: Ftype,
}
struct Model{
    is_cloze: bool,
    fields: Vec<CardField>,
    name: String,
    templates: Vec<Temple>,
}
struct Temple{
    name: String,
    qfmt: String,
    afmt: String,
}
struct Note{
    model: Model,
    fields: Vec<String>,
}
struct Kort{
    note: Note,
    template_ord: usize,
}
struct Models{
    inner: Vec<Model>,
}

// {{c1::ose}}


use lazy_static::lazy_static;
use regex::Regex;


// {{c\d*::.+?(?=}})}}

fn cloze_format(trd: String, ord: u32)-> String{
    let mut pattern = r"\{\{c".to_string();
    pattern.push_str(&ord.to_string());
    pattern.push_str(&r"::.*\}\}");
    let re = Regex::new(&pattern).unwrap();
    let trd = re.replace_all(&trd, "[...]");

    let pattern = r"\{\{c\d*::(?P<inside_cloze>.*)\}\}";
    let re = Regex::new(pattern).unwrap();
    let trd = re.replace_all(&trd, "$inside_cloze");
    trd.to_string()
}







#[derive(Clone)]
pub struct Template{
    name: String,
    fields: Vec<CardField>,
    notes: Vec<Vec<String>>,
    viewpos: usize,
    front_template: Field,
    back_template:  Field,
    front_view: Field,
    back_view: Field,
    topics: TopicList,
    selected: Selected,
    pub state: LoadState,
}

impl Template{
    pub fn new(conn: &Connection, path: PathBuf)-> Template{
        let mut temp = Template{
            name:   String::new(),
            fields: Vec::new(),
            notes:  Vec::new(),
            viewpos: 0,
            front_template: Field::new(),
            back_template: Field::new(),
            front_view: Field::new(),
            back_view: Field::new(),
            topics: TopicList::new(conn),
            selected: Selected::Preview,
            state: LoadState::OnGoing,
        };
        temp.get_anki_notes(path);
        temp.fill_views();
        temp
    }

    fn get_anki_notes(&mut self, path: PathBuf){
        //let download_location = Self::download_deck(url).unwrap();
        let unzip_location = Self::unzip_deck(path).unwrap();
      //  let unzip_location = "./ankidecks/ankidb.db";
        let conn = Connection::open(unzip_location).unwrap();
        let mystr: String = conn.query_row(
            "select models from col",
            [],
            |row| row.get(0),
            ).unwrap();
        let v: serde_json::Value = serde_json::from_str(&mystr).unwrap();
        (self.name, self.fields) = Self::load_columns(&v).unwrap();
        self.notes = Self::load_notes(&conn).unwrap();
        self.autoformat(&v);
        self.set_cardtype();
        
    }
    
    fn get_url(deckid: u32){

    }

    fn download_deck(url: String) -> Result<String>{
        let downloc = "./ankidecks/download.zip";
        if !std::path::Path::new("ankidecks/").exists(){
            std::fs::create_dir("ankidecks/").unwrap();
        }
        let body = reqwest::blocking::get(url)?;
        let path = std::path::Path::new(downloc);
        let mut file = match File::create(&path) {
            Err(why) => panic!("couldn't create {}", why),
            Ok(file) => file,
        };
 //       file.write_all(&(body.bytes().unwrap()))?;
        Ok(downloc.to_string())
    }


    fn unzip_deck(downloc: PathBuf) -> Result<String> {
        let outpath = "./ankidecks/ankidb.db";
        let file = fs::File::open(downloc).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let mut file = archive.by_name("collection.anki2").unwrap();
        let mut outfile = fs::File::create(outpath).unwrap();
        io::copy(&mut file, &mut outfile).unwrap();
        Ok(outpath.to_string())
    }


    fn load_columns(v: &serde_json::Value) -> Result<(String, Vec<CardField>)> {
        let mut templates: Vec<(String, Vec<CardField>)> = Vec::new();
        if let serde_json::Value::Object(ob) = v{
            for (_, val) in ob {
                let name = val["name"].to_string();
                let mut myvec: Vec<CardField> = Vec::new();
                if let serde_json::Value::Array(fields) = &val["flds"]{
                    for field in fields{
                        if let serde_json::Value::Object(fld) = field{
                            let mut text = fld["name"].to_string();
                            text.pop();
                            text.remove(0);
                            let cardfield = CardField::new(text);
                            myvec.push(cardfield);
                        } else {panic!()}
                    }
                } else {panic!()}
                templates.push((name, myvec));
            }
        } else {panic!()}
        let mut template = 0;

        for i in 0..templates.len(){
            if templates[i].1.len() > templates[template].1.len(){
                template = i;
            }
        }


        let template = templates[template].clone();
        Ok(template)
    }





    fn load_notes(conn: &Connection) -> Result<Vec<Vec<String>>>{
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

    fn set_cardtype(&mut self){
        let notes = &self.notes[0];

        for i in 0..notes.len(){
            if notes[i].starts_with("<img src="){
                self.fields[i].change_ftype(Ftype::Image);
            } else if notes[i].starts_with("[sound:") {
                self.fields[i].change_ftype(Ftype::Audio);
            }
        }
/*
        let (text, ftype) = if string.starts_with("<img src="){
            let mut trimmed = String::new();
            let strlen = string.len();
            for (idx, chr) in string.chars().enumerate(){
                if idx > 9 && idx < (strlen - 4){
                    trimmed.push(chr);
                }
            }
            (trimmed, Ftype::Image)
        } else if string.starts_with("[sound:]") {
            let (_, right) = string.split_at(6);
            let mut right = right.to_string();
            right.pop();
            (right, Ftype::Audio)
        } else{
            (string, Ftype::Text)
        };
        */

    }

    fn with_braces(mut string:  String)->String{
        string.insert(0, '{');
        string.insert(0, '{');
        string.push('}');
        string.push('}');
        string
    }


    fn autoformat(&mut self, obj: &serde_json::Value){
        if let serde_json::Value::Object(ob) = obj{
            for (_, val) in ob {
                if let serde_json::Value::Object(ob) = val{
                    if let serde_json::Value::Array(arr) = &ob["tmpls"]{
                        let qstring = self.fix_format(arr[0]["qfmt"].to_string());
                        self.front_template.replace_text(qstring);
                        let astring = self.fix_format(arr[0]["afmt"].to_string());
                        self.back_template.replace_text(astring);
                    } else {panic!()};
                } else {panic!()};
            }
        }
    }


    fn fix_format(&mut self, frmt: String) -> String{
        let linebreak = "<br/>";
        let mut myvec:Vec<Vec<(usize, &str)>> = Vec::new(); 
        let mut flattened: Vec<(usize, &str)> = Vec::new();
        for field in &self.fields{
            let foo = Self::with_braces(field.name.clone());
            myvec.push(frmt.match_indices(&foo).collect());
        }
        myvec.push(frmt.match_indices(linebreak).collect());

        for innervec in myvec{
            for elm in innervec{
                flattened.push(elm);
            }
        }
        flattened.sort_by_key(|x| x.0);
        
        for i in 0..flattened.len(){
            if flattened[i].1 == linebreak{
                flattened[i] = (flattened[i].0, "\n");
            }
        }

        let mut formatted_text = String::new();
        for tup in flattened{
            formatted_text.push_str(tup.1);
        }

        formatted_text
    }

    fn get_field(&self, fieldpos: usize) -> String{
        self.notes[self.viewpos][fieldpos].clone()
    }

    fn fill_views(&mut self){
        self.fill_front_view();
        self.fill_back_view();
    }

    fn fill_front_view(&mut self){
        self.front_view.replace_text(self.fill_view(self.front_template.return_text(), self.viewpos));
    }
    fn fill_back_view(&mut self){
        self.back_view.replace_text(self.fill_view(self.back_template.return_text(), self.viewpos));
    }


    fn fill_view(&self, mut template: String, viewpos: usize) -> String{
        if template.len() == 0 {return "".to_string()}
        for (val, key) in self.fields.iter().enumerate(){
            template.insert(0, ' ');
            template.push(' ');
            let mut isaudio = false;
            let mut isimg = false;
            let mut img_qty   = 0;
            let mut audio_qty = 0;
            if let Ftype::Image = key.ftype {
                isimg = true;
            } else if let Ftype::Audio = key.ftype {
                isaudio = true;
            }
            let key = Self::with_braces(key.name.clone());
            let split_by_field: Vec<&str> = template.split_terminator(&key).collect();


            let foo = split_by_field[0];
            if foo.len() == 0{
                dbg!(&key, &template,);
                continue
            }
            let mut tempstring = foo.to_string(); 


            for i in 0..split_by_field.len(){
                if i != 0{
                    if isaudio{
                        audio_qty += 1;
                    }
                    if isimg{
                        img_qty += 1;
                    }
                    let replaced = self.notes[viewpos][val].clone();
                 //   dbg!(&self.notes[viewpos]);
                    let right = split_by_field[i].clone();
                    if isaudio{
                        if audio_qty > 1{
                            tempstring.push_str(&"!ONLY ONE AUDIO PER FIELD!");
                        }
                    }
                    else if isimg{
                        if img_qty > 1{
                            tempstring.push_str(&"!ONLY ONE IMAGE PER FIELD!");
                        }
                    } else {
                        tempstring.push_str(&replaced);

                    }
                    tempstring.push_str(&right);
                }
            }
            template = tempstring.clone();
        }
        template
    }

    fn import_cards(&mut self, conn: &Connection){
        let totlen = self.notes.len() -1;
        let progress = ImportProgress{
            question: String::new(),
            answer: String::new(),
            curr_index: 0,
            total: self.notes.len() as u32,
        };
        self.state = LoadState::Importing(progress);
        let front_template = self.front_template.return_text();
        let back_template = self.back_template.return_text();
        for idx in 0..self.notes.len(){
            let question = self.fill_view(front_template.clone(), idx);
            let answer = self.fill_view(back_template.clone(), idx);
            let prog = ImportProgress::new(question.clone(), answer.clone(), idx as u32, totlen as u32);
            Card::new()
                .question(question)
                .answer(answer)
                .topic(self.topics.get_selected_id().unwrap())
                .cardtype(CardType::Pending)
                .save_card(conn);
            self.state = LoadState::Importing(prog);
            }
        self.state = LoadState::OnGoing;
    }



    pub fn render(&mut self, f: &mut tui::Frame<MyType>, area: tui::layout::Rect) {

        if let LoadState::Importing(prog) = &self.state{
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


            draw_message(f, rightcol[0], &prog.question);
            draw_message(f, rightcol[1], &prog.answer);
            progress_bar::progress_bar(f, prog.curr_index, prog.total, Color::Cyan, rightcol[2]);
            return;

        } 

        let selected = IsSelected::new(&self.selected);

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
        let rightcol = Layout::default()
            .direction(Vertical)
            .constraints(
                [
                Constraint::Ratio(1, 2),
                Constraint::Ratio(1, 2),
                ]
                .as_ref(),
                )
            .split(right);

        let (thetopics, thefields) = (rightcol[0], rightcol[1]);

        let updown = Layout::default()
            .direction(Vertical)
            .constraints(
                [

                Constraint::Ratio(1, 5),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 5),

                ]
                .as_ref(),
                )
            .split(left);

        let (preview, up, down, button) = (updown[0], updown[1], updown[2], updown[3]);

        let toprow = Layout::default()
            .direction(Horizontal)
            .constraints(
                [
                Constraint::Ratio(1, 2),
                Constraint::Ratio(1, 2),
                ]
                .as_ref(),
                )
            .split(up);


        let bottomrow = Layout::default()
            .direction(Horizontal)
            .constraints(
                [
                Constraint::Ratio(1, 2),
                Constraint::Ratio(1, 2),
                ]
                .as_ref(),
                )
            .split(down);


        let (topleft, topright) = (toprow[0], toprow[1]);
        let (bottomleft, bottomright) = (bottomrow[0], bottomrow[1]);
        

        let flds: Vec<ListItem> = self.fields.iter().map(|field| {
            let lines = vec![Spans::from(format!("{}",field))];
            ListItem::new(lines).style(Style::default())
        }).collect();
        let fieldlist = List::new(flds).block(Block::default().borders(Borders::ALL).title("Available fields"));
        f.render_stateful_widget(fieldlist, thefields, &mut ListState::default());


        draw_button(f, preview, &format!("Previewing card {} out of {}", self.viewpos + 1, self.notes.len()), selected.preview);
        list_widget(f, &self.topics, thetopics, selected.topics);
        self.front_template.render(f, topleft, selected.front);
        self.back_template.render(f, bottomleft, selected.back);
        self.front_view.render(f, topright, false);
        self.back_view.render(f, bottomright, false);
        draw_button(f, button, &format!("Import cards!"), selected.import);


        self.front_template.set_win_height(topleft.height);
        self.front_template.set_rowlen(topleft.width);
        self.back_template.set_win_height(bottomleft.height);
        self.back_template.set_rowlen(bottomleft.width);

    }
    pub fn keyhandler(&mut self, conn: &Connection, key: MyKey) {

        use MyKey::*;
        use Selected::*;

        if let MyKey::Nav(dir) = key{
            self.navigate(dir);
            return;
        }



        match (&self.selected, key) {
            (_, Alt('s')) => {
                let front_text = self.front_template.return_text();
                let back_text = self.back_template.return_text();
                self.front_template.replace_text(back_text);
                self.back_template.replace_text(front_text);
                self.fill_views();
            },
            (Preview, Char('l')) => {
                if self.viewpos < self.notes.len() - 1{
                    self.viewpos += 1;
                    self.fill_views();
                }
            },
            (Preview, Char('h')) => {
                if self.viewpos > 0{
                    self.viewpos -= 1;
                    self.fill_views();
                }
            }
            (Import, Enter) => self.import_cards(conn),
            (_, Esc) => self.state = LoadState::Finished,
            (Front,  key) => {
                self.front_template.keyhandler(key);
                self.fill_front_view();
            },
            (Back,   key) => {
                self.back_template.keyhandler(key);
                self.fill_back_view();
            },
            (Topics, key) => self.topics.keyhandler(key, conn),
            (_, _) => {},
        }
    }




    fn navigate(&mut self, dir: Direction){
        use Direction::*;
        use Selected::*;

        match (&self.selected, dir){
            (Preview, Down) => self.selected = Selected::Front,

            (Front, Down) => self.selected = Selected::Back,
            (Front, Up) => self.selected = Selected::Preview,

            (Back, Up) => self.selected = Selected::Front,
            (Back, Down) => self.selected = Selected::Import,

            (Import, Up) => self.selected = Selected::Back,

            (Preview, Right) => self.selected = Selected::Topics,
            (Topics, Left)   => self.selected = Selected::Preview,
            (_, _) => {},
        }
    }
}
