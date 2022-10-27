use crate::app::Audio;
use crate::utils::misc::{View, split_leftright_by_percent, split_updown_by_percent};
use crate::utils::statelist::{StatefulList, TextItem};
use crate::widgets::button::draw_button;
use crate::widgets::textinput::Field;
use crate::widgets::topics::TopicList;
use crate::MyKey;
use crate::{NavDir, SpekiPaths};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::utils::card::{CardTypeData, PendingInfo};
use crate::utils::{aliases::*, card};
use crate::MyType;
use anyhow::Result;
use rusqlite::Connection;
use tui::layout::Rect;
use std::fs;
use tui::widgets::ListState;
use tui::{
    layout::{
        Constraint,
        Direction::{Horizontal, Vertical},
        Layout,
    },
    style::Style,
    text::Spans,
    widgets::{Block, Borders, List, ListItem},
};

use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct ImportProgress {
    pub curr_index: usize,
    pub total: usize,
}

pub enum UnzipStatus {
    Ongoing(String),
    Done,
}

#[derive(Clone, Debug)]
pub enum LoadState {
    OnGoing,
    Finished,
    Importing, // e.g.  70/500 cards imported...
}

#[derive(Clone, Debug)]
enum Ftype {
    Text,
    Audio,
    Image,
}

impl CardField {
    fn new(string: String) -> CardField {
        CardField {
            text: string,
            image: None,
            audio: None,
        }
    }
}

#[derive(Clone, Debug)]
struct Kort {
    note_id: NoteID,
    template_ord: usize,
    reps: u32,
}
#[derive(Clone, Debug)]
struct Note {
    model_id: ModelID,
    fields: Vec<CardField>,
}
#[derive(Default, Clone, Debug)]
struct Model {
    is_cloze: bool,
    fields: Vec<String>,
    name: String,
    templates: Vec<Temple>,
}
#[derive(Clone, Debug)]
struct CardField {
    text: String,
    image: Option<PathBuf>,
    audio: Option<PathBuf>,
}

// perhaps instead of having things like {{Word}} in the string i should rather
// save the index of where different fields should be in the template
#[derive(Default, Clone, Debug)]
struct Temple {
    name: String,
    qfmt: String,
    afmt: String,
}

use regex::Regex;

fn cloze_format(trd: &mut String, ord: u32) {
    let mut pattern = r"\{\{c".to_string();
    pattern.push_str(&ord.to_string());
    pattern.push_str(r"::(.*?)\}\}");
    let re = Regex::new(&pattern).unwrap();
    *trd = re.replace_all(trd, "[...]").to_string();
}
fn hide_close(trd: &mut String) {
    let pattern = r"\{\{c\d*::(?P<inside_cloze>(.*?))\}\}";
    let re = Regex::new(pattern).unwrap();
    *trd = re.replace_all(trd, "$inside_cloze").to_string();
}
fn strip_cloze(trd: &mut String) {
    let pattern = r"\{\{cloze\d*:(?P<inside_cloze>(.*?))\}\}";
    let re = Regex::new(pattern).unwrap();
    *trd = re.replace_all(trd, "{{$inside_cloze}}").to_string();
}
fn remove_useless_formatting(trd: &mut String) {
    let pattern = r"(<br />|<br>|<br/>)".to_string();
    let re = Regex::new(&pattern).unwrap();
    *trd = re.replace_all(trd, "\n").to_string();

    let pattern = r"(<.*?>)".to_string();
    let re = Regex::new(&pattern).unwrap();
    *trd = re.replace_all(trd, "").to_string();

    let pattern = r"&nbsp;".to_string();
    let re = Regex::new(&pattern).unwrap();
    *trd = re.replace_all(trd, "").to_string();

    let pattern = r"&quot;".to_string();
    let re = Regex::new(&pattern).unwrap();
    *trd = re.replace_all(trd, "").to_string();
}

fn extract_image(trd: &mut String, deckname: &String, paths: &SpekiPaths) -> Option<PathBuf> {
    let pattern = "<img src=\"(.*?)\" />".to_string();
    let re = Regex::new(&pattern).unwrap();

    let res = match re.captures(trd)?.get(1) {
        Some(res) => {
            let mut imagepath = paths.media.clone();
            imagepath.push(format!("{}/{}", deckname, res.as_str()));
            Some(imagepath)
        }
        None => None,
    };
    *trd = re.replace_all(trd, "").to_string();
    return res;
}

fn extract_audio(trd: &mut String, deckname: &String, paths: &SpekiPaths) -> Option<PathBuf> {
    let pattern = r"\[sound:(.*?)\]".to_string();
    let re = Regex::new(&pattern).unwrap();
    let foo = re.captures(trd)?;

    let res = match foo.get(1) {
        Some(res) => {
            let mut audiopath = paths.media.clone();
            audiopath.push(format!("{}/{}", deckname, res.as_str().to_string()));
            Some(audiopath)
        }
        None => None,
    };
    *trd = re.replace_all(&trd, "").to_string();
    return res;
}
#[derive(Default)]
pub struct MediaContents {
    pub frontaudio: Option<PathBuf>,
    pub backaudio: Option<PathBuf>,
    pub frontimage: Option<PathBuf>,
    pub backimage: Option<PathBuf>,
}

#[derive(Clone)]
pub struct Template {
    cards: Vec<Kort>,
    notes: HashMap<NoteID, Note>,
    models: HashMap<ModelID, Model>,
    viewpos: usize,
    front_template: Field,
    back_template: Field,
    front_view: Field,
    back_view: Field,
    topics: TopicList,
    pub state: LoadState,
    view: View,
    fields: StatefulList<TextItem>,
}

impl Template {
    pub fn new(conn: &Arc<Mutex<Connection>>, deckname: String, paths: &SpekiPaths) -> Template {
        let cards = vec![];
        let notes: HashMap<NoteID, Note> = HashMap::new();
        let models: HashMap<ModelID, Model> = HashMap::new();
        let view = View::default();
        let fields = StatefulList::new();

        let mut temp = Template {
            cards,
            notes,
            models,
            viewpos: 0,
            front_view: Field::new(),
            back_view: Field::new(),
            front_template: Field::new(),
            back_template: Field::new(),
            topics: TopicList::new(conn),
            state: LoadState::OnGoing,
            view,
            fields,
        };
        temp.init(&deckname, paths);
        temp.front_view.stickytitle = true;
        temp.back_view.stickytitle = true;
        temp
    }

    fn play_front_audio(&self, audio: &Option<Audio>) {
        let media = self.get_media(self.viewpos);
        if let Some(path) = media.frontaudio {
            crate::utils::misc::play_audio(audio, path);
        }
    }
    fn play_back_audio(&self, audio: &Option<Audio>) {
        let media = self.get_media(self.viewpos);
        if let Some(path) = media.backaudio {
            crate::utils::misc::play_audio(audio, path);
        }
    }

    fn get_media(&self, idx: usize) -> MediaContents {
        let mut media = MediaContents::default();
        let mut frontaudiovec = Vec::<(usize, PathBuf)>::new();
        let mut backaudiovec = Vec::<(usize, PathBuf)>::new();
        let mut frontimagevec = Vec::<(usize, PathBuf)>::new();
        let mut backimagevec = Vec::<(usize, PathBuf)>::new();

        let front_template = self.get_front_template(idx);
        let back_template = self.get_back_template(idx);

        let note = self.note_from_card_index(idx);
        let model = self.model_from_card_index(idx);

        for (i, field) in note.fields.iter().enumerate() {
            let fieldname = Self::with_braces(model.fields[i].clone());
            if let Some(path) = &field.audio {
                front_template.match_indices(&fieldname).for_each(|foo| {
                    frontaudiovec.push((foo.0, path.to_owned()));
                });
                back_template.match_indices(&fieldname).for_each(|foo| {
                    backaudiovec.push((foo.0, path.to_owned()));
                });
            }
            if let Some(path) = &field.image {
                front_template.match_indices(&fieldname).for_each(|foo| {
                    frontimagevec.push((foo.0, path.to_owned()));
                });
                back_template.match_indices(&fieldname).for_each(|foo| {
                    backimagevec.push((foo.0, path.to_owned()));
                });
            }
        }

        frontaudiovec.sort_by_key(|el| el.0);
        backaudiovec.sort_by_key(|el| el.0);
        frontimagevec.sort_by_key(|el| el.0);
        backimagevec.sort_by_key(|el| el.0);

        if !frontaudiovec.is_empty() {
            media.frontaudio = Some(frontaudiovec[0].1.clone());
        }
        if !backaudiovec.is_empty() {
            media.backaudio = Some(backaudiovec[0].1.clone());
        }
        if !frontimagevec.is_empty() {
            media.frontimage = Some(frontimagevec[0].1.clone());
        }
        if !backimagevec.is_empty() {
            media.backimage = Some(backimagevec[0].1.clone());
        }
        media
    }

    fn refresh_template_and_view(&mut self) {
        self.front_template
            .replace_text(self.get_front_template(self.viewpos));
        self.back_template
            .replace_text(self.get_back_template(self.viewpos));

        self.front_view
            .replace_text(self.fill_front_view(self.front_template.return_text(), self.viewpos));
        self.back_view
            .replace_text(self.fill_back_view(self.back_template.return_text(), self.viewpos));

        self.fields.items = {
            let model = self.selected_model();
            model.fields.iter().map(|item| {TextItem::new(item.clone())}).collect()
        };

    }

    fn refresh_views(&mut self) {
        self.front_view
            .replace_text(self.fill_front_view(self.front_template.return_text(), self.viewpos));
        self.back_view
            .replace_text(self.fill_back_view(self.back_template.return_text(), self.viewpos));
    }

    fn rename_media(deckname: &String, paths: &SpekiPaths) -> Result<()> {
        let mut mediapath = paths.media.clone();
        mediapath.push(format!("{}/", deckname));
        let mut medianames = mediapath.clone();
        medianames.push("media");
        let contents = fs::read_to_string(&medianames).unwrap();
        let jsonmodels: serde_json::Value = serde_json::from_str(&contents).unwrap();
        if let serde_json::Value::Object(ob) = jsonmodels {
            for (key, val) in ob {
                let mut val = val.to_string();
                val.pop();
                val.remove(0);

                let mut keypath = paths.media.clone();
                let mut valpath = paths.media.clone();
                keypath.push(format!("{}/{}", &deckname, key));
                valpath.push(format!("{}/{}", &deckname, val));

                std::fs::rename(keypath, valpath)?;
            }
        } else {
            panic!();
        }
        Ok(())
    }

    pub fn unzip_deck(
        paths: SpekiPaths,
        deckname: String,
        transmitter: std::sync::mpsc::Sender<UnzipStatus>,
    ) -> PathBuf {
        let mut folderpath = paths.media.clone();
        folderpath.push(format!("{}/", &deckname));

        if !std::path::Path::new(&paths.media).exists() {
            std::fs::create_dir(&paths.media).unwrap();
        }

        if !std::path::Path::new(&folderpath).exists() {
            std::fs::create_dir(&folderpath).unwrap();
        }

        let mut unzipped_db = paths.media.clone();
        unzipped_db.push(format!("{}/collection.anki2", &deckname));

        let _ = transmitter.send(UnzipStatus::Ongoing("Opening zip file".to_string()));
        let file = fs::File::open(&paths.downloc).expect(&format!(
            "couldnt open file: {}",
            &paths.downloc.to_str().unwrap()
        ));
        let _ = transmitter.send(UnzipStatus::Ongoing(
            "Loading zip file to memory".to_string(),
        ));
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let _ = transmitter.send(UnzipStatus::Ongoing("Extracting files...".to_string()));
        archive.extract(folderpath).unwrap();
        let _ = transmitter.send(UnzipStatus::Ongoing("Preparing files...".to_string()));
        Self::rename_media(&deckname, &paths).unwrap();
        unzipped_db
    }

    fn selected_model(&mut self) -> &mut Model {
        let key = self.selected_model_id();
        let model = self.models.get_mut(&key).unwrap();
        model
    }

    fn update_template(&mut self) {
        let ord = {
            let model = &self.models[&self.selected_model_id()];
            if model.is_cloze {
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

    fn init(&mut self, deckname: &String, paths: &SpekiPaths) {
        let mut deckdb = paths.media.clone();
        deckdb.push(format!("{}/collection.anki2", &deckname));
        let ankon = Arc::new(Mutex::new(Connection::open(deckdb).unwrap()));
        self.load_models(&ankon);
        self.load_notes(&ankon, deckname, paths).unwrap();
        self.load_cards(&ankon).unwrap();
        self.refresh_template_and_view();
    }

    fn selected_note_id(&self) -> NoteID {
        self.cards[self.viewpos].note_id
    }
    fn selected_note(&self) -> &Note {
        &self.notes[&self.selected_note_id()]
    }
    fn selected_model_id(&self) -> ModelID {
        self.selected_note().model_id
    }
    fn note_id_from_card_id(&self, id: CardID) -> NoteID {
        self.cards[id as usize].note_id
    }
    fn note_id_from_card_index(&self, idx: usize) -> NoteID {
        self.cards[idx].note_id
    }
    fn note_from_card_index(&self, idx: usize) -> &Note {
        &self.notes[&self.note_id_from_card_index(idx)]
    }
    fn model_from_card_index(&self, idx: usize) -> &Model {
        &self.models[&self.note_from_card_index(idx).model_id]
    }
    fn model_id_from_card_idx(&self, idx: usize) -> ModelID {
        self.note_from_card_index(idx).model_id
    }

    fn fill_front_view(&self, template: String, idx: usize) -> String {
        let mut text = self.fill_view(template, idx);
        remove_useless_formatting(&mut text);
        let model = &self.models.get(&self.model_id_from_card_idx(idx)).unwrap();
        if model.is_cloze {
            cloze_format(&mut text, self.cards[idx].template_ord as u32 + 1);
            hide_close(&mut text);
        }
        text
    }
    fn fill_back_view(&self, template: String, idx: usize) -> String {
        let mut text = self.fill_view(template, idx);
        remove_useless_formatting(&mut text);
        let model = &self.models.get(&self.model_id_from_card_idx(idx)).unwrap();
        if model.is_cloze {
            hide_close(&mut text);
        }
        text
    }

    fn fill_view(&self, mut template: String, viewpos: usize) -> String {
        if template.len() == 0 {
            return "".to_string();
        }
        let model = self.model_from_card_index(viewpos);

        for (val, key) in model.fields.iter().enumerate() {
            template.insert(0, ' '); // it wouldn't match fields if they were in beginning or end
            template.push(' ');

            let key = Self::with_braces(key.clone());
            let split_by_field: Vec<&str> = template.split_terminator(&key).collect();

            let foo = split_by_field[0];
            if foo.len() == 0 {
                continue;
            }
            let mut tempstring = foo.to_string();

            for i in 0..split_by_field.len() {
                if i != 0 {
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
    fn fix_format(&mut self, frmt: String, fields: &Vec<String>) -> String {
        let linebreak = "<br/>";
        let mut myvec: Vec<Vec<(usize, &str)>> = Vec::new();
        let mut flattened: Vec<(usize, &str)> = Vec::new();
        for field in fields {
            let foo = Self::with_braces(field.clone());
            myvec.push(frmt.match_indices(&foo).collect());
        }
        myvec.push(frmt.match_indices(linebreak).collect());
        for innervec in myvec {
            for elm in innervec {
                flattened.push(elm);
            }
        }
        flattened.sort_by_key(|x| x.0);
        for i in 0..flattened.len() {
            if flattened[i].1 == linebreak {
                flattened[i] = (flattened[i].0, "\n");
            }
        }
        let mut formatted_text = String::new();
        for tup in flattened {
            formatted_text.push_str(tup.1);
        }
        formatted_text
    }

    fn get_front_template(&self, idx: usize) -> String {
        let card = &self.cards[idx];
        let mut temp_ord = card.template_ord;
        let note = &self.notes[&card.note_id];
        let model = &self.models[&note.model_id];
        if model.is_cloze {
            temp_ord = 0;
        }
        model.templates[temp_ord].qfmt.clone()
    }
    fn get_back_template(&self, idx: usize) -> String {
        let card = &self.cards[idx];
        let mut temp_ord = card.template_ord;
        let note = &self.notes[&card.note_id];
        let model = &self.models[&note.model_id];
        if model.is_cloze {
            temp_ord = 0;
        }
        model.templates[temp_ord].afmt.clone()
    }

    fn load_cards(&mut self, conn: &Arc<Mutex<Connection>>) -> Result<()> {
        let guard = conn.lock().unwrap();
        let mut stmt = guard.prepare("SELECT nid, ord, reps FROM cards").unwrap();
        stmt.query_map([], |row| {
            let note_id: NoteID = row.get::<usize, NoteID>(0).unwrap();
            let template_ord: usize = row.get::<usize, usize>(1).unwrap();
            let reps: u32 = row.get::<usize, u32>(2).unwrap();
            self.cards.push(Kort {
                note_id,
                template_ord,
                reps,
            });
            Ok(())
        })?
        .for_each(|_| {});
        Ok(())
    }
    fn load_notes(
        &mut self,
        conn: &Arc<Mutex<Connection>>,
        deckname: &String,
        paths: &SpekiPaths,
    ) -> Result<()> {
        let guard = conn.lock().unwrap();
        let mut stmt = guard.prepare("SELECT id, mid, flds FROM notes")?;
        stmt.query_map([], |row| {
            let id: NoteID = row.get::<usize, NoteID>(0).unwrap();
            let model_id: ModelID = row.get::<usize, ModelID>(1).unwrap();
            let fields: Vec<CardField> = row
                .get::<usize, String>(2)
                .unwrap()
                .split('')
                .map(|x| {
                    let mut text = x.to_string();
                    let audio = extract_audio(&mut text, deckname, paths);
                    let image = extract_image(&mut text, deckname, paths);
                    CardField { text, audio, image }
                })
                .collect();
            self.notes.insert(id, Note { model_id, fields });
            Ok(())
        })?
        .for_each(|_| {});

        Ok(())
    }

    fn load_models(&mut self, conn: &Arc<Mutex<Connection>>) {
        let guard = conn.lock().unwrap();
        let rawmodel: String = guard
            .query_row("select models from col", [], |row| row.get(0))
            .unwrap();

        let jsonmodels: serde_json::Value = serde_json::from_str(&rawmodel).unwrap();
        let mut models = Vec::<Model>::new();
        let mut model_ids = Vec::<ModelID>::new();

        if let serde_json::Value::Object(ob) = jsonmodels {
            for (_, val) in ob {
                let mut model = Model::default();
                model.name = val["name"].to_string();
                model.is_cloze = if val["type"].to_string() == "0".to_string() {
                    false
                } else {
                    true
                };
                model_ids.push({
                    let raw = val["id"].to_string();
                    let mut new = String::new();
                    for c in raw.chars() {
                        if c.is_ascii_digit() {
                            new.push(c);
                        }
                    }
                    let formatted = new.parse::<ModelID>().unwrap();
                    formatted
                });

                let mut fieldvec: Vec<String> = Vec::new();
                if let serde_json::Value::Array(fields) = &val["flds"] {
                    for field in fields {
                        if let serde_json::Value::Object(fld) = field {
                            let mut text = fld["name"].to_string();
                            text.pop(); // removing quotes
                            text.remove(0);
                            fieldvec.push(text);
                        } else {
                            panic!()
                        }
                    }
                } else {
                    panic!()
                }

                let mut tmplvec: Vec<Temple> = vec![];
                if let serde_json::Value::Array(templates) = &val["tmpls"] {
                    for template in templates {
                        let mut temp = Temple::default();
                        if let serde_json::Value::Object(tmpl) = template {
                            temp.name = tmpl["name"].to_string();

                            let mut qfmt = tmpl["qfmt"].to_string();
                            let mut afmt = tmpl["afmt"].to_string();

                            strip_cloze(&mut qfmt);
                            strip_cloze(&mut afmt);

                            temp.qfmt = self.fix_format(qfmt, &fieldvec);
                            temp.afmt = self.fix_format(afmt, &fieldvec);
                            tmplvec.push(temp);
                        } else {
                            panic!()
                        }
                    }
                } else {
                    panic!()
                }
                model.fields = fieldvec;
                model.templates = tmplvec;
                models.push(model);
            }
        } else {
            panic!()
        }

        assert!(models.len() == model_ids.len());
        for (idx, id) in model_ids.iter().enumerate() {
            self.models.insert(*id, models[idx].clone());
        }
    }

    fn with_braces(mut string: String) -> String {
        string.insert(0, '{');
        string.insert(0, '{');
        string.push('}');
        string.push('}');
        string
    }

    pub fn import_cards(
        &mut self,
        conn: Arc<Mutex<Connection>>,
        transmitter: std::sync::mpsc::SyncSender<ImportProgress>,
    ) {
        let cardlen = self.cards.len();
        let topic = self.topics.get_selected_id().unwrap();

        for idx in 0..cardlen {
            let front_template = self.get_front_template(idx);
            let back_template = self.get_back_template(idx);
            let frontside = self.fill_front_view(front_template, idx);
            let backside = self.fill_back_view(back_template, idx);
            let media = self.get_media(idx);

            if idx % 10 == 0 {
                let _ = transmitter.try_send(ImportProgress {
                    curr_index: idx,
                    total: cardlen,
                });
            };

            card::Card::new(CardTypeData::Pending(PendingInfo::default()))
                .question(frontside)
                .answer(backside)
                .topic(topic)
                .frontimage(media.frontimage)
                .backimage(media.backimage)
                .frontaudio(media.frontaudio)
                .backaudio(media.backaudio)
                .save_card(&conn);
        }
    }

    fn set_sorted(&mut self, area: Rect){
        let leftright = split_leftright_by_percent([66, 33], area);

        let (left, right) = (leftright[0], leftright[1]);
        let rightcol = split_updown_by_percent([50, 50], right);
        let (thetopics, thefields) = (rightcol[0], rightcol[1]);
        let updown = split_updown_by_percent([20, 30, 30, 20], left);
        let (preview, up, down, button) = (updown[0], updown[1], updown[2], updown[3]);

        let toprow = split_leftright_by_percent([50, 50], up);
        let bottomrow = split_leftright_by_percent([50, 50], down);

        let (topleft, topright) = (toprow[0], toprow[1]);
        let (bottomleft, bottomright) = (bottomrow[0], bottomrow[1]);

        self.front_template.set_dimensions(topleft);
        self.back_template.set_dimensions(bottomleft);

        self.view.areas.insert("topics", thetopics);
        self.view.areas.insert("preview", preview);
        self.view.areas.insert("front_template", topleft);
        self.view.areas.insert("front_view", topright);
        self.view.areas.insert("back_template", bottomleft);
        self.view.areas.insert("back_view", bottomright);
        self.view.areas.insert("import", button);
        self.view.areas.insert("fields", thefields);

    }

    pub fn render(&mut self, f: &mut tui::Frame<MyType>, area: tui::layout::Rect) {
        self.set_sorted(area);

       
        let media = self.get_media(self.viewpos);
        let mut frontstring = "Front side ".to_string();
        let mut backstring = "Back side ".to_string();
        if media.frontaudio.is_some() {
            frontstring.push('🔊')
        }
        if media.backaudio.is_some() {
            backstring.push('🔊')
        }
        if media.frontimage.is_some() {
            frontstring.push('📷')
        }
        if media.backimage.is_some() {
            backstring.push('📷')
        }
        self.front_view.title = frontstring;
        self.back_view.title = backstring;

        draw_button(
            f,
            self.view.get_area("preview"),
            &format!(
                "Previewing card {} out of {}",
                self.viewpos + 1,
                self.cards.len()
            ),
            self.view.name_selected("preview"),
        );

        self.fields.render(f, self.view.get_area("fields"), self.view.name_selected("fields"), "Fields", Style::default());
        self.topics
            .render(f, self.view.get_area("topics"), self.view.name_selected("topics"), "Topics", Style::default());
        self.front_template.render(f, self.view.get_area("front_template"), self.view.name_selected("front_template"));
        self.back_template.render(f, self.view.get_area("back_template"), self.view.name_selected("back_template"));
        self.front_view.render(f, self.view.get_area("front_view"), self.view.name_selected("front_view"));
        self.back_view.render(f, self.view.get_area("back_view"), self.view.name_selected("back_view"));
        draw_button(f, self.view.get_area("import"), "Import cards!", self.view.name_selected("import"));
    }

    pub fn keyhandler(&mut self, conn: &Arc<Mutex<Connection>>, key: MyKey, audio: &Option<Audio>) {
        use MyKey::*;

        if let MyKey::Nav(dir) = key {
            self.view.navigate(dir);
            return;
        }

        match key {
            KeyPress(pos) => self.view.cursor = pos,
            Alt('s') => {
                let front_text = self.front_template.return_text();
                let back_text = self.back_template.return_text();
                self.front_template.replace_text(back_text);
                self.back_template.replace_text(front_text);
                self.update_template();
                self.refresh_template_and_view();
            }
            Char('l') | Right if self.view.name_selected("preview")=> {
                if self.viewpos < self.notes.len() - 1 {
                    self.viewpos += 1;
                    self.refresh_template_and_view();
                    self.play_front_audio(audio);
                    self.play_back_audio(audio);
                }
            }
            Char('h') | Left if self.view.name_selected("preview")=> {
                if self.viewpos > 0 {
                    self.viewpos -= 1;
                    self.refresh_template_and_view();
                    self.play_front_audio(audio);
                    self.play_back_audio(audio);
                }
            }
            Enter if self.view.name_selected("import")=> self.state = LoadState::Importing,
            Esc => self.state = LoadState::Finished,
            key if self.view.name_selected("front_template")=> {
                self.front_template.keyhandler(key);
                self.update_template();
                self.refresh_views();
            }
            key if self.view.name_selected("back_template")=> {
                self.back_template.keyhandler(key);
                self.update_template();
                self.refresh_views();
            }
            key if self.view.name_selected("topics")=> self.topics.keyhandler(key, conn),
            _ => {}
        }
    }
}
