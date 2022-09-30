
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
use crate::utils::{aliases::*, card};
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


impl CardField{
    fn new(string: String)-> CardField{
        CardField{
            text: string,
            image: None,
            audio: None,
        }
    }
}






#[derive(Clone, Debug)]
struct Kort{
    note_id: NoteID,
    template_ord: usize,
}
#[derive(Clone, Debug)]
struct Note{
    model_id: ModelID,
    fields: Vec<CardField>,
}
#[derive(Default, Clone, Debug)]
struct Model{
    is_cloze: bool,
    fields: Vec<String>,
    name: String,
    templates: Vec<Temple>,
}
#[derive(Clone, Debug)]
struct CardField{
    text: String,
    image: Option<String>,
    audio: Option<String>,
}


// perhaps instead of having things like {{Word}} in the string i should rather 
// save the index of where different fields should be in the template
#[derive(Default, Clone, Debug)]
struct Temple{
    name: String,
    qfmt: String,
    afmt: String,
}


use regex::Regex;

fn cloze_format(trd: &mut String, ord: u32){
    let mut pattern = r"\{\{c".to_string();
    pattern.push_str(&ord.to_string());
    pattern.push_str(&r"::(.*?)\}\}"); 
    let re = Regex::new(&pattern).unwrap();
    *trd = re.replace_all(&trd, "[...]").to_string();
}
fn hide_close(trd: &mut String){
    let pattern = r"\{\{c\d*::(?P<inside_cloze>(.*?))\}\}";
    let re = Regex::new(pattern).unwrap();
    *trd = re.replace_all(&trd, "$inside_cloze").to_string();
}
fn strip_cloze(trd: &mut String){
    let pattern = r"\{\{cloze\d*:(?P<inside_cloze>(.*?))\}\}";
    let re = Regex::new(pattern).unwrap();
    *trd = re.replace_all(&trd, "{{$inside_cloze}}").to_string();
}
fn remove_useless_formatting(trd: &mut String){
    let pattern = r"(<br />|<br>|<br/>)".to_string();
    let re = Regex::new(&pattern).unwrap();
    *trd = re.replace_all(&trd, "\n").to_string();

    let pattern = r"(<.*?>)".to_string();
    let re = Regex::new(&pattern).unwrap();
    *trd = re.replace_all(&trd, "").to_string();
    
    let pattern = r"&nbsp;".to_string();
    let re = Regex::new(&pattern).unwrap();
    *trd = re.replace_all(&trd, "").to_string();
    
    let pattern = r"&quot;".to_string();
    let re = Regex::new(&pattern).unwrap();
    *trd = re.replace_all(&trd, "").to_string();
}




fn extract_image(trd: &mut String, folderpath: &String) -> Option<String>{
    let pattern = "<img src=\"(.*?)\"".to_string();
    let re = Regex::new(&pattern).unwrap();
    let foo = re.captures(&trd)?;
  
    let res = match foo.get(1) {
        Some(res) => Some(format!("media/{}/{}", &folderpath, res.as_str().to_string())),
        None => None,
    };
    *trd = re.replace_all(&trd, "").to_string();
    return res;
}


fn extract_audio(trd: &mut String, folderpath: &String) -> Option<String>{
    let pattern = r"\[sound:(.*?)\]".to_string();
    let re = Regex::new(&pattern).unwrap();
    let foo = re.captures(&trd)?;
  
    let res = match foo.get(1) {
        Some(res) => Some(format!("media/{}/{}", &folderpath, res.as_str().to_string())),
        None => None,
    };
    *trd = re.replace_all(&trd, "").to_string();
    return res;
}
#[derive(Default)]
struct MediaContents{
    frontaudio: Option<String>,
    backaudio:  Option<String>,
    frontimage: Option<String>,
    backimage:  Option<String>,
}



#[derive(Clone)]
pub struct Template{
    cards: Vec<Kort>,
    notes: HashMap<NoteID, Note>,
    models: HashMap<ModelID, Model>,
    viewpos: usize,
    front_template: Field,
    back_template: Field,
    front_view: Field,
    back_view: Field,
    topics: TopicList,
    selected: Selected,
    pub state: LoadState,
    folderpath: String,
}

impl Template{
    pub fn new(conn: &Connection, path: PathBuf)-> Template{
        let cards = vec![];
        let notes:  HashMap<NoteID, Note> = HashMap::new();
        let models: HashMap<ModelID, Model> = HashMap::new();
        let mut temp = Template{
            cards,
            notes,
            models,
            viewpos: 0,
            front_view: Field::new(),
            back_view: Field::new(),
            front_template: Field::new(),
            back_template: Field::new(),
            topics: TopicList::new(conn),
            selected: Selected::Preview,
            state: LoadState::OnGoing,
            folderpath: String::new(),
        };
        temp.init(path);
        temp
    }





/*




#[derive(Clone, Debug)]
struct Kort{
    note_id: NoteID,
    template_ord: usize,
}
#[derive(Clone, Debug)]
struct Note{
    model_id: ModelID,
    fields: Vec<CardField>,
}
#[derive(Default, Clone, Debug)]
struct Model{
    is_cloze: bool,
    fields: Vec<String>,
    name: String,
    templates: Vec<Temple>,
}
#[derive(Clone, Debug)]
struct CardField{
    text: String,
    image: Option<String>,
    audio: Option<String>,
}
#[derive(Default, Clone, Debug)]
struct Temple{
    name: String,
    qfmt: String,
    afmt: String,
}


find all the fields in question template in order 
iterate over the CardFields, when you find one that has audio, return it 
else return None



*/

    fn get_media(&self, idx: usize) -> MediaContents{
        let mut media = MediaContents::default();
        let mut frontaudiovec = Vec::<(usize, &str)>::new();
        let mut backaudiovec  = Vec::<(usize, &str)>::new();
        let mut frontimagevec = Vec::<(usize, &str)>::new();
        let mut backimagevec  = Vec::<(usize, &str)>::new();

        let front_template = self.get_front_template(idx);
        let back_template  = self.get_back_template (idx);

        let note = self.note_from_card_index(idx);
        let model = self.model_from_card_index(idx);

        for (i, field) in note.fields.iter().enumerate(){
            if let Some(path) = &field.audio{
                front_template.match_indices(&model.fields[i].clone()).for_each(|foo| {
                    frontaudiovec.push((foo.0, path));
                });
                back_template.match_indices(&model.fields[i].clone()).for_each(|foo| {
                    backaudiovec.push((foo.0, path));
                });
            }
            if let Some(path) = &field.image{
                front_template.match_indices(&model.fields[i].clone()).for_each(|foo| {
                    frontimagevec.push((foo.0, path));
                });
                back_template.match_indices(&model.fields[i].clone()).for_each(|foo| {
                    backimagevec.push((foo.0, path));
                });
            }
        }

        frontaudiovec.sort_by_key(|el| el.0);
        backaudiovec .sort_by_key(|el| el.0);
        frontimagevec.sort_by_key(|el| el.0);
        backimagevec .sort_by_key(|el| el.0);

        if frontaudiovec.len() > 0 {
            media.frontaudio = Some(frontaudiovec[0].1.to_string());
        }
        if backaudiovec.len() > 0 {
            media.backaudio =  Some(backaudiovec[0].1.to_string());
        }
        if frontimagevec.len() > 0 {
            media.frontimage = Some(frontimagevec[0].1.to_string());
        }
        if backimagevec.len() > 0 {
            media.backimage =  Some(backimagevec[0].1.to_string());
        }
        media
    }

    fn refresh_template_and_view(&mut self){
        self.front_template.replace_text(self.get_front_template(self.viewpos));
        self.back_template .replace_text(self.get_back_template (self.viewpos));

        self.front_view.replace_text(self.fill_front_view(self.front_template.return_text(), self.viewpos));
        self.back_view .replace_text(self.fill_back_view (self.back_template .return_text(), self.viewpos));
    }

    fn rename_media(&self)-> Result<()>{
        let mappath = format!("media/{}/media", self.folderpath);
        let contents = fs::read_to_string(mappath).unwrap();
        let jsonmodels: serde_json::Value = serde_json::from_str(&contents).unwrap();

        if let serde_json::Value::Object(ob) = jsonmodels{
            for (key, val) in ob{
                let mut val = val.to_string();
                val.pop();
                val.remove(0);
                let keypath = format!("media/{}/{}", self.folderpath, key.to_string());
                let valpath = format!("media/{}/{}", self.folderpath, val);
                
                std::fs::rename(keypath, valpath)?;
            }
        }
        Ok(())
    }



    fn selected_model(&mut self) -> &mut Model{
        let key = self.selected_model_id();
        let model = self.models.get_mut(&key).unwrap();
        model
    }

    fn update_template(&mut self){
        let ord = {
            let model = &self.models[&self.selected_model_id()];
            if model.is_cloze{
                0
            } else {
                self.cards[self.viewpos].template_ord
            }
        };

        self.models
            .get_mut(&self.selected_model_id())
            .unwrap()
            .templates[ord]
            .qfmt = self.front_template.return_text();

        self.models
            .get_mut(&self.selected_model_id())
            .unwrap()
            .templates[ord]
            .afmt = self.back_template.return_text();
    }


    fn init(&mut self, path: PathBuf){
        let loc = self.unzip_deck(path).unwrap();
        let ankon = Connection::open(loc).unwrap();
        self.load_models(&ankon);
        self.load_notes(&ankon).unwrap();
        self.load_cards(&ankon).unwrap();
    }

    fn selected_note_id(&self) -> NoteID{
        self.cards[self.viewpos].note_id
    }
    fn selected_note(&self) -> &Note{
        &self.notes[&self.selected_note_id()]
    }
    fn selected_model_id(&self) -> ModelID{
        self.selected_note().model_id
    }
    fn note_id_from_card_id(&self, id: CardID) -> NoteID{
        self.cards[id as usize].note_id
    }
    fn note_id_from_card_index(&self, idx: usize) -> NoteID{
        self.cards[idx].note_id
    }
    fn note_from_card_index(&self, idx: usize) -> &Note{
        &self.notes[&self.note_id_from_card_index(idx)]
    }
    fn model_from_card_index(&self, idx: usize) -> &Model{
        &self.models[&self.note_from_card_index(idx).model_id]
    }
    fn model_id_from_card_idx(&self, idx: usize) -> ModelID{
        self.note_from_card_index(idx).model_id
    }




    fn fill_front_view(&self, template: String, idx: usize)-> String{
        let mut text = self.fill_view(template, idx);
        remove_useless_formatting(&mut text);
        let model = &self.models.get(&self.model_id_from_card_idx(idx)).unwrap();
        if model.is_cloze{
            cloze_format(&mut text, self.cards[idx].template_ord as u32 + 1);
            hide_close(&mut text);
        }
        text
    }
    fn fill_back_view(&self, template: String, idx: usize)-> String{
        let mut text = self.fill_view(template, idx);
        remove_useless_formatting(&mut text);
        let model = &self.models.get(&self.model_id_from_card_idx(idx)).unwrap();
        if model.is_cloze{
            hide_close(&mut text);
        }
        text
    }




    fn fill_view(&self, mut template: String, viewpos: usize) -> String{
        if template.len() == 0 {return "".to_string()}
        let mut ord = self.cards[viewpos].template_ord;
        let model = self.model_from_card_index(viewpos);
        if model.is_cloze{ord = 0}

        for (val, key) in model.fields.iter().enumerate(){

            template.insert(0, ' '); // it wouldn't match fields if they were in beginning or end
            template.push(' ');

            let key = Self::with_braces(key.clone());
            let split_by_field: Vec<&str> = template.split_terminator(&key).collect();


            let foo = split_by_field[0];
            if foo.len() == 0{
                dbg!(&key, &template,);
                continue
            }
            let mut tempstring = foo.to_string(); 


            for i in 0..split_by_field.len(){
                if i != 0{
                    let replaced = self.note_from_card_index(viewpos).fields[val].clone();
                    let right = split_by_field[i].clone();
                    tempstring.push_str(&replaced.text);
                    tempstring.push_str(&right);
                }
            }
            template = tempstring.clone();
        }
        template
    }










    // this removes all the bullshit tags and only keeps the on related to fields  
    // although side effect is raw text will dissapear too, which could be nice to have
    // i suppose
    // TODO: remove all the weird formatting stuff while also keeping raw text and such 
    fn fix_format(&mut self, frmt: String, fields: &Vec<String>) -> String{
        let linebreak = "<br/>";
        let mut myvec:Vec<Vec<(usize, &str)>> = Vec::new(); 
        let mut flattened: Vec<(usize, &str)> = Vec::new();
        for field in fields{
            let foo = Self::with_braces(field.clone());
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


    fn get_front_template(&self, idx: usize) -> String{
        let card = &self.cards[idx];
        let mut temp_ord = card.template_ord;
        let note = &self.notes[&card.note_id];
        let model = &self.models[&note.model_id];
        if model.is_cloze{
            temp_ord = 0;
        }
        model.templates[temp_ord].qfmt.clone()
    }
    fn get_back_template(&self, idx: usize) -> String{
        let card = &self.cards[idx];
        let mut temp_ord = card.template_ord;
        let note = &self.notes[&card.note_id];
        let model = &self.models[&note.model_id];
        if model.is_cloze{
            temp_ord = 0;
        }
        model.templates[temp_ord].afmt.clone()
    }
    





    fn load_cards(&mut self, conn: &Connection) -> Result<()>{
        let mut stmt = conn.prepare("SELECT nid, ord FROM cards").unwrap();
        let foo = stmt.query_map([], |row| {
                                      let note_id: NoteID = row
                                          .get::<usize, NoteID>(0)
                                          .unwrap();
                                      let template_ord: usize = row
                                          .get::<usize, usize>(1)
                                          .unwrap();
                                      Ok(
                                          Kort { 
                                              note_id,
                                              template_ord,
                                          }
                                          )

        })?;
        for x in foo{
            self.cards.push(x.unwrap());
        }
        Ok(())
    }
    fn load_notes(&mut self, conn: &Connection) -> Result<()>{
        let mut stmt = conn.prepare("SELECT id, mid, flds FROM notes")?;
        let foo = stmt.query_map([], |row| {
                                  let id: NoteID = row
                                      .get::<usize, NoteID>(0)
                                      .unwrap();
                                  let model_id: ModelID = row
                                      .get::<usize, ModelID>(1)
                                      .unwrap();
                                  let fields: Vec<CardField> = row
                                      .get::<usize, String>(2)
                                      .unwrap()
                                      .split('')
                                      .map(|x| {
                                        let mut text = x.to_string();
                                        let audio = extract_audio(&mut text, &self.folderpath);
                                        let image = extract_image(&mut text, &self.folderpath);
                                        CardField{
                                            text,
                                            audio,
                                            image,
                                           }
                                        }
                                      )
                                      .collect();
                                  Ok(
                                      (id, Note {
                                          model_id,
                                          fields,
                                      })
                                      )
                                  
                                   })
        ?;

        for x in foo{
            let y = x.unwrap();
            let (id, note) = y;
            self.notes.insert(id, note.clone());
        }

        Ok(())
    }


    /*



    is_cloze: bool,
    fields: Vec<CardField>,
    name: String,
    templates: Vec<Temple>,

       */



    fn load_models(&mut self, conn: &Connection){
        let rawmodel: String = conn.query_row(
            "select models from col",
            [],
            |row| row.get(0),
            ).unwrap();

        let jsonmodels: serde_json::Value = serde_json::from_str(&rawmodel).unwrap();
        let mut models = Vec::<Model>::new();
        let mut model_ids = Vec::<ModelID>::new();


        if let serde_json::Value::Object(ob) = jsonmodels{
            for (_, val) in ob {
                let mut model = Model::default();
                model.name = val["name"].to_string();
                model.is_cloze = if val["type"].to_string() == "0".to_string() {false} else {true};
                model_ids.push(
                    {
                    let raw = val["id"].to_string();
                    let mut new = String::new();
                    for c in raw.chars(){
                        if c.is_ascii_digit(){
                            new.push(c);
                        }
                    }
                    let formatted = new.parse::<ModelID>().unwrap();
                    formatted
                    }
                ); 

                let mut fieldvec: Vec<String> = Vec::new();
                if let serde_json::Value::Array(fields) = &val["flds"]{
                    for field in fields{
                        if let serde_json::Value::Object(fld) = field{
                            let mut text = fld["name"].to_string();
                            text.pop(); // removing quotes 
                            text.remove(0);
                            fieldvec.push(text);
                        } else {panic!()}
                    }
                } else {panic!()}

                let mut tmplvec: Vec<Temple> = vec![];
                if let serde_json::Value::Array(templates) = &val["tmpls"]{
                    for template in templates{
                        let mut temp = Temple::default();
                        if let serde_json::Value::Object(tmpl) = template{
                            temp.name = tmpl["name"].to_string();

                            let mut qfmt = tmpl["qfmt"].to_string();
                            let mut afmt = tmpl["afmt"].to_string();

                            strip_cloze(&mut qfmt);
                            strip_cloze(&mut afmt);

                            temp.qfmt = self.fix_format(qfmt, &fieldvec);
                            temp.afmt = self.fix_format(afmt, &fieldvec);
                            tmplvec.push(temp);
                        } else {panic!()}
                    }
                } else {panic!()}
                model.fields = fieldvec;
                model.templates = tmplvec;
                models.push(model);
            }
        } else {panic!()}

        assert!(models.len() == model_ids.len());
        for (idx, id) in model_ids.iter().enumerate(){
            self.models.insert(*id, models[idx].clone());
        }
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


    fn unzip_deck(&mut self, downloc: PathBuf) -> Result<String> {
        let mut foldername = downloc.clone().to_string_lossy().to_string();
        foldername.pop();
        foldername.pop();
        foldername.pop();
        foldername.pop();
        let foldername = foldername.rsplit_once('/').unwrap().1.to_string();
        let folderpath = format!("media/{}", &foldername);
        if !std::path::Path::new(&folderpath).exists(){
            std::fs::create_dir(&folderpath).unwrap();
        }

        let db_path = format!("media/{}/collection.anki2", foldername);
        let file = fs::File::open(downloc).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        archive.extract(folderpath).unwrap();
        self.folderpath = foldername;
        self.rename_media().unwrap();
        Ok(db_path)
    }

    fn with_cloze_braces(string:  String)->String{
        let mut formatted = "{{cloze:".to_string();
        formatted.push_str(&string);
        formatted.push_str(&"}}");
        formatted
    }

    fn with_braces(mut string:  String)->String{
        string.insert(0, '{');
        string.insert(0, '{');
        string.push('}');
        string.push('}');
        string
    }





    fn import_cards(&mut self, conn: &Connection){
        let cardlen = self.cards.len();
        let topic = self.topics.get_selected_id().unwrap();

        for idx in 0..cardlen{
            let front_template = self.get_front_template(idx);
            let back_template  = self.get_back_template(idx);
            let frontside = self.fill_front_view(front_template, idx);
            let backside  = self.fill_back_view(back_template, idx);
            let media = self.get_media(idx);


            card::Card::new()
                .question(frontside)
                .answer(backside)
                .topic(topic)
                .frontimage(media.frontimage)
                .backimage(media.backimage)
                .frontaudio(media.frontaudio)
                .backaudio(media.backaudio)
                .cardtype(CardType::Pending)
                .save_card(conn);
        }
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
        

        let flds: Vec<ListItem> = {
            let model = self.selected_model();
            let lines = model.fields.iter().map(|field| {
                let lines = vec![Spans::from(format!("{}",field))];
                ListItem::new(lines).style(Style::default())
            }).collect();
                lines



        };         let fieldlist = List::new(flds).block(Block::default().borders(Borders::ALL).title("Available fields"));
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
        };
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
            },
            (Preview, Char('l')) => {
                if self.viewpos < self.notes.len() - 1{
                    self.viewpos += 1;
                    self.refresh_template_and_view();
                }
            },
            (Preview, Char('h')) => {
                if self.viewpos > 0{
                    self.viewpos -= 1;
                    self.refresh_template_and_view();
                }
            }
            (Import, Enter) => self.import_cards(conn),
            (_, Esc) => self.state = LoadState::Finished,
            (Front,  key) => {
                self.front_template.keyhandler(key);
                self.update_template();
                self.refresh_template_and_view();
            },
            (Back,   key) => {
                self.back_template.keyhandler(key);
                self.update_template();
                self.refresh_template_and_view();
            },
            (Topics, key) => self.topics.keyhandler(key, conn),
            (_, _) => {},
        }
    }
}












/*

    fn set_cardtype(&mut self){
        let notes = &self.notes[0];

        for i in 0..notes.len(){
            if notes[i].starts_with("<img src="){
                self.fields[i].change_ftype(Ftype::Image);
            } else if notes[i].starts_with("[sound:") {
                self.fields[i].change_ftype(Ftype::Audio);
            }
        }
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



    /* 
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
*/
