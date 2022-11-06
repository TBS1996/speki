use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::{
    app::{AppData, Audio},
    popups::message_popup::Msg,
    utils::aliases::*,
};

#[derive(Clone, Debug)]
pub struct ImportProgress {
    pub curr_index: usize,
    pub max: usize,
}

#[derive(Default)]
pub struct MediaContents {
    pub frontaudio: Option<PathBuf>,
    pub backaudio: Option<PathBuf>,
    pub frontimage: Option<PathBuf>,
    pub backimage: Option<PathBuf>,
}

#[derive(Clone, Debug)]
pub struct Kort {
    note_id: NoteID,
    pub template_ord: usize,
    reps: u32,
}

#[derive(Clone, Debug)]
pub struct Note {
    model_id: ModelID,
    fields: Vec<CardField>,
}
#[derive(Default, Clone, Debug)]
pub struct Model {
    pub is_cloze: bool,
    pub fields: Vec<String>,
    name: String,
    pub templates: Vec<Temple>,
}
#[derive(Clone, Debug)]
struct CardField {
    text: String,
    image: Option<PathBuf>,
    audio: Option<PathBuf>,
}

#[derive(Default, Clone, Debug)]
pub struct Temple {
    name: String,
    pub qfmt: String,
    pub afmt: String,
}

#[derive(Clone)]
pub struct Template {
    pub cards: Vec<Kort>,
    pub notes: HashMap<NoteID, Note>,
    pub models: HashMap<ModelID, Model>,
}

impl Template {
    // basically emulates anki template thing for a CSV file cause im lazy but hey it works.. soon
    pub fn new_csv(path: PathBuf) -> Self {
        let mut notes = HashMap::new();
        let mut cards = vec![];
        let mut models = HashMap::new();

        //        let path = String::from("importing.csv");
        //let mut rdr = Reader::from_path(path).unwrap();
        let mut rdr = ReaderBuilder::new()
            .delimiter(b'\t')
            .from_path(path)
            .unwrap();

        let mut index = 0;
        for result in rdr.records() {
            let record = result.unwrap();
            let mut cardfields = vec![];
            for field in record.iter() {
                cardfields.push(CardField {
                    text: field.to_string(),
                    image: None,
                    audio: None,
                });
            }
            let note = Note {
                model_id: 0,
                fields: cardfields,
            };
            notes.insert(index, note);
            let kort = Kort {
                note_id: index,
                template_ord: 0,
                reps: 0,
            };
            cards.push(kort);
            index += 1;
        }
        let mut afmt = String::new();

        let headers = rdr.headers().unwrap();
        let mut fields = vec![];
        for header in headers.iter() {
            let hdr = header.trim().to_owned();
            fields.push(hdr);
        }

        for i in 1..fields.len() {
            let mut field = fields[i].trim().to_owned();
            field.push('}');
            field.push('}');
            field.insert(0, '{');
            field.insert(0, '{');
            afmt.push_str(&field);
        }
        let qfmt = {
            let mut field = fields[0].trim().to_owned();
            field.push('}');
            field.push('}');
            field.insert(0, '{');
            field.insert(0, '{');
            field
        };

        let thetemple = Temple {
            name: String::from("tsv_template"),
            qfmt,
            afmt,
        };

        let model = Model {
            is_cloze: false,
            fields,
            name: "tsv_model".to_string(),
            templates: vec![thetemple],
        };

        models.insert(0, model);
        Template {
            cards,
            notes,
            models,
        }
    }

    pub fn new(appdata: &AppData, deckname: String) -> Self {
        let cards = vec![];
        let notes: HashMap<NoteID, Note> = HashMap::new();
        let models: HashMap<ModelID, Model> = HashMap::new();

        let mut temp = Self {
            cards,
            notes,
            models,
        };
        temp.init(&deckname, &appdata.paths);
        temp
    }

    fn init(&mut self, deckname: &String, paths: &SpekiPaths) {
        let mut deckdb = paths.media.clone();
        deckdb.push(format!("{}/collection.anki2", &deckname));
        let ankon = Arc::new(Mutex::new(Connection::open(deckdb).unwrap()));
        self.load_models(&ankon);
        self.load_notes(&ankon, deckname, paths).unwrap();
        self.load_cards(&ankon).unwrap();
    }

    pub fn fill_front_view(&self, template: String, idx: usize) -> String {
        let mut text = self.fill_view(template, idx);
        remove_useless_formatting(&mut text);
        let model = &self.models.get(&self.model_id_from_card_idx(idx)).unwrap();
        if model.is_cloze {
            cloze_format(&mut text, self.cards[idx].template_ord as u32 + 1);
            hide_close(&mut text);
        }
        text
    }
    pub fn fill_back_view(&self, template: String, idx: usize) -> String {
        let mut text = self.fill_view(template, idx);
        remove_useless_formatting(&mut text);
        let model = &self.models.get(&self.model_id_from_card_idx(idx)).unwrap();
        if model.is_cloze {
            hide_close(&mut text);
        }
        text
    }

    fn fill_view(&self, mut template: String, viewpos: usize) -> String {
        if template.is_empty() {
            return "".to_string();
        }
        let model = self.model_from_card_index(viewpos);

        for (val, key) in model.fields.iter().enumerate() {
            template.insert(0, ' '); // it wouldn't match fields if they were in beginning or end
            template.push(' ');

            let key = Template::with_braces(key.clone());
            let split_by_field: Vec<&str> = template.split_terminator(&key).collect();

            let foo = split_by_field[0];
            if foo.is_empty() {
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

    pub fn import_cards(
        &mut self,
        conn: Arc<Mutex<Connection>>,
        transmitter: std::sync::mpsc::SyncSender<ImportProgress>,
        topic: TopicID,
    ) {
        let cardlen = self.cards.len();

        for idx in 0..cardlen {
            let front_template = self.get_front_template(idx);
            let back_template = self.get_back_template(idx);
            let frontside = self.fill_front_view(front_template, idx);
            let backside = self.fill_back_view(back_template, idx);
            let media = self.get_media(idx);
            if idx % 1 == 0 {
                let _ = transmitter.try_send(ImportProgress {
                    curr_index: idx,
                    max: cardlen,
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

    pub fn play_front_audio(&self, audio: &Option<Audio>, viewpos: usize) {
        let media = self.get_media(viewpos);
        if let Some(path) = media.frontaudio {
            crate::utils::misc::play_audio(audio, path);
        }
    }
    pub fn play_back_audio(&self, audio: &Option<Audio>, viewpos: usize) {
        let media = self.get_media(viewpos);
        if let Some(path) = media.backaudio {
            crate::utils::misc::play_audio(audio, path);
        }
    }

    pub fn get_media(&self, idx: usize) -> MediaContents {
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

    pub fn rename_media(
        deckname: String,
        paths: SpekiPaths,
        transmitter: std::sync::mpsc::SyncSender<ImportProgress>,
    ) -> Result<()> {
        let mut mediapath = paths.media.clone();
        mediapath.push(format!("{}/", deckname));
        let mut medianames = mediapath.clone();
        medianames.push("media");
        let contents = fs::read_to_string(&medianames).unwrap();
        let jsonmodels: serde_json::Value = serde_json::from_str(&contents).unwrap();
        if let serde_json::Value::Object(ob) = jsonmodels {
            let mut index = 0;
            let tot = ob.len();
            for (key, val) in ob {
                let _ = transmitter.try_send(ImportProgress {
                    curr_index: index,
                    max: tot,
                });

                let mut val = val.to_string();
                val.pop();
                val.remove(0);

                let mut keypath = paths.media.clone();
                let mut valpath = paths.media.clone();
                keypath.push(format!("{}/{}", &deckname, key));
                valpath.push(format!("{}/{}", &deckname, val));

                std::fs::rename(keypath, valpath)?;
                index += 1;
            }
        } else {
            panic!();
        }
        Ok(())
    }

    pub fn unzip_deck(
        paths: SpekiPaths,
        deckname: String,
        transmitter: std::sync::mpsc::Sender<Msg>,
    ) {
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

        let _ = transmitter.send(Msg::Ok("Opening zip file".to_string()));
        let file = fs::File::open(&paths.downloc).unwrap();
        let _ = transmitter.send(Msg::Ok("Opening zip file".to_string()));
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let _ = transmitter.send(Msg::Ok("Extracting files...".to_string()));
        archive.extract(folderpath).unwrap();
        let _ = transmitter.send(Msg::Ok("Preparing files...".to_string()));

        let _ = transmitter.send(Msg::Done);
    }

    pub fn selected_model(&mut self, viewpos: usize) -> &mut Model {
        let key = self.selected_model_id(viewpos);
        let model = self.models.get_mut(&key).unwrap();
        model
    }
    pub fn get_front_template(&self, idx: usize) -> String {
        let card = &self.cards[idx];
        let mut temp_ord = card.template_ord;
        let note = &self.notes[&card.note_id];
        let model = &self.models[&note.model_id];
        if model.is_cloze {
            temp_ord = 0;
        }
        model.templates[temp_ord].qfmt.clone()
    }
    pub fn get_back_template(&self, idx: usize) -> String {
        let card = &self.cards[idx];
        let mut temp_ord = card.template_ord;
        let note = &self.notes[&card.note_id];
        let model = &self.models[&note.model_id];
        if model.is_cloze {
            temp_ord = 0;
        }
        model.templates[temp_ord].afmt.clone()
    }

    fn selected_note_id(&self, viewpos: usize) -> NoteID {
        self.cards[viewpos].note_id
    }
    fn selected_note(&self, viewpos: usize) -> &Note {
        &self.notes[&self.selected_note_id(viewpos)]
    }
    pub fn selected_model_id(&self, viewpos: usize) -> ModelID {
        self.selected_note(viewpos).model_id
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

use anyhow::Result;
use csv::ReaderBuilder;
use regex::Regex;
use rusqlite::Connection;

use super::{
    card::{self, CardTypeData, PendingInfo},
    misc::SpekiPaths,
};

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
    res
}

fn extract_audio(trd: &mut String, deckname: &String, paths: &SpekiPaths) -> Option<PathBuf> {
    let pattern = r"\[sound:(.*?)\]".to_string();
    let re = Regex::new(&pattern).unwrap();
    let foo = re.captures(trd)?;

    let res = match foo.get(1) {
        Some(res) => {
            let mut audiopath = paths.media.clone();
            audiopath.push(format!("{}/{}", deckname, res.as_str()));
            Some(audiopath)
        }
        None => None,
    };
    *trd = re.replace_all(trd, "").to_string();
    res
}
