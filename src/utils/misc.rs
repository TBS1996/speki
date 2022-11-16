use std::{
    fs::File,
    io::{BufReader, Write},
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{app::Audio, tabs::review::logic::ReviewMode, utils::card::CardItem, MyType, NavDir};
use rusqlite::Connection;
use tui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::Spans,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/*


   this is where i put stuff that doesn't fit neatly into a certain module


*/

pub fn modecolor(mode: &ReviewMode) -> Color {
    match mode {
        ReviewMode::Review(_) => Color::Red,
        ReviewMode::Unfinished(_) => Color::Yellow,
        ReviewMode::Pending(_) => Color::Cyan,
        ReviewMode::IncRead(_) => Color::Green,
        ReviewMode::Done => Color::Blue,
    }
}

#[derive(Clone)]
pub enum PopUpStatus {
    OnGoing,
    Finished,
}

pub fn play_audio(audio: &Option<Audio>, path: PathBuf) {
    if let Ok(file) = std::fs::File::open(&path) {
        if let Some(audio) = audio {
            match audio.handle.play_once(BufReader::new(file)) {
                Ok(beep) => {
                    beep.set_volume(audio.volume);
                    beep.detach();
                }
                Err(_err) => {}
            }
        }
    }
}

#[derive(Deserialize, Debug)]
struct OpenAIChoices {
    text: String,
}

#[derive(Deserialize, Debug)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoices>,
}

#[derive(Serialize, Debug)]
struct OpenAIRequest {
    model: String,
    prompt: String,
    max_tokens: u32,
    stop: String,
}

use hyper::{body::Buf, header, Body, Client, Request};
use hyper_tls::HttpsConnector;
use serde_derive::{Deserialize, Serialize};

use super::{
    aliases::{CardID, Pos},
    sql::fetch::fetch_card,
    statelist::StatefulList,
};

// stole code from here: https://github.com/zahidkhawaja/rusty
#[tokio::main]
pub async fn get_gpt3_response(api_key: &str, user_input: &str) -> Option<String> {
    let https = HttpsConnector::new();
    let client = Client::builder().build(https);
    let uri = "https://api.openai.com/v1/completions";

    let model = String::from("text-davinci-002");
    let stop = String::from("Text");

    let auth_header_val = format!("Bearer {}", api_key);

    let openai_request = OpenAIRequest {
        model,
        prompt: format!("{}\n\n", user_input),
        max_tokens: 200,
        stop,
    };

    let body = Body::from(serde_json::to_vec(&openai_request).unwrap());

    let req = Request::post(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("Authorization", &auth_header_val)
        .body(body)
        .unwrap();

    let res = client.request(req).await.unwrap();

    let body = hyper::body::aggregate(res).await.unwrap();

    let json: OpenAIResponse = match serde_json::from_reader(body.reader()) {
        Ok(response) => response,
        Err(_) => {
            return None;
        }
    };

    Some(
        json.choices[0]
            .text
            .split('\n')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

pub fn get_dependencies(conn: &Arc<Mutex<Connection>>, id: CardID) -> StatefulList<CardItem> {
    let thecard = fetch_card(conn, id);
    let dep_ids = &thecard.dependencies;
    let mut depvec: Vec<CardItem> = vec![];

    for dep in dep_ids {
        let card = fetch_card(conn, *dep);
        depvec.push(CardItem {
            question: card.question,
            id: *dep,
        });
    }
    StatefulList::with_items("Dependencies".to_string(), depvec)
}

pub fn get_dependents(conn: &Arc<Mutex<Connection>>, id: CardID) -> StatefulList<CardItem> {
    let thecard = fetch_card(conn, id);
    let dep_ids = &thecard.dependents;
    let mut depvec: Vec<CardItem> = vec![];

    for dep in dep_ids {
        let card = fetch_card(conn, *dep);
        depvec.push(CardItem {
            question: card.question,
            id: *dep,
        });
    }
    StatefulList::with_items("Dependents".to_string(), depvec)
}

#[derive(Clone)]
pub struct SpekiPaths {
    pub base: PathBuf,
    pub database: PathBuf,
    pub media: PathBuf,
    pub tempfolder: PathBuf,
    pub downloc: PathBuf,
    pub backups: PathBuf,
    pub config: PathBuf,
    pub anki: PathBuf,
}

impl SpekiPaths {
    const DEFAULTCONFIG: &'static str = r#"
#gptkey = ""
        "#;
    pub fn new(home: &PathBuf) -> Self {
        let mut home = home.clone();
        let mut configpath = home.clone();
        let mut anki = home.clone();
        if cfg!(windows) {
            home.push(".speki/");
            anki.push("AppData/Roaming/Anki2/");
            configpath.push("config.toml");
            std::fs::create_dir_all(&anki.parent().unwrap()).unwrap();
            std::fs::create_dir_all(&home.parent().unwrap()).unwrap();
            if !std::path::Path::new(&configpath).exists() {
                let mut file = File::create(&configpath).unwrap();
                file.write_all(Self::DEFAULTCONFIG.as_bytes()).unwrap();
            }
        } else {
            home.push(".local/share/speki/");
            anki.push(".local/share/Anki2");
            std::fs::create_dir_all(&home).unwrap();
            std::fs::create_dir_all(&anki).unwrap();
            configpath.push(".config/speki/config.toml");
            std::fs::create_dir_all(&configpath.parent().unwrap()).unwrap();
            if !std::path::Path::new(&configpath).exists() {
                let mut file = File::create(&configpath).unwrap();
                file.write_all(Self::DEFAULTCONFIG.as_bytes()).unwrap();
            }
        }
        let mut database = home.clone();
        let mut media = home.clone();
        let mut tempfolder = home.clone();
        let mut backups = home.clone();

        database.push("dbflash.db");
        media.push("media/");
        tempfolder.push("temp/");
        backups.push("backups/");

        let mut downloc = tempfolder.clone();
        downloc.push("ankitemp.apkg");

        Self {
            base: home.clone(),
            database,
            media,
            tempfolder,
            downloc,
            backups,
            config: configpath,
            anki,
        }
    }
}

/*

*/

pub fn get_current_unix() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32
}

pub fn get_current_unix_millis() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u32
}

pub fn new_mod(num: i64) -> u32 {
    let mut val = (num % 510) - 254;
    if val < 0 {
        val *= -1;
    }
    val as u32
}

pub fn get_rgb(val: u32) -> (u8, u8, u8) {
    let unix = get_current_unix() as i64;
    let val = val as i64;
    let r = new_mod(val | unix) as u8;
    let g = new_mod(val ^ unix) as u8;
    let b = new_mod(val & unix) as u8;
    (r, g, b)
}

#[derive(Clone, Default)]
pub struct View {
    pub areas: Vec<Rect>,
    pub cursor: Pos,
}

impl View {
    pub fn debug_show_cursor(&self, f: &mut Frame<MyType>) {
        f.set_cursor(self.cursor.x, self.cursor.y);
    }

    pub fn validate_pos(&mut self) {
        for area in self.areas.iter() {
            if self.is_selected(*area) {
                return;
            }
        }
        if !self.areas.is_empty() {
            self.move_to_area(self.areas[0]);
        }
    }

    pub fn move_to_area(&mut self, area: Rect) {
        let x = area.x + area.width / 2;
        let y = area.y + area.height / 2;
        self.cursor = Pos::new(x, y);
    }

    pub fn is_selected(&self, area: Rect) -> bool {
        Self::isitselected(area, &self.cursor)
    }

    pub fn isitselected(area: Rect, cursor: &Pos) -> bool {
        cursor.x >= area.x
            && cursor.x < area.x + area.width
            && cursor.y >= area.y
            && cursor.y < area.y + area.height
    }

    fn move_cursor<F>(&mut self, mut func: F)
    where
        F: FnMut(&mut Pos, &Rect),
    {
        let mut currentarea = self.areas[0];
        let mut new_pos = self.cursor;
        for area in &self.areas {
            if Self::isitselected(*area, &self.cursor) {
                currentarea = *area;
            }
        }
        func(&mut new_pos, &currentarea);
        for area in &self.areas {
            // validating that new pos is in an area
            if Self::isitselected(*area, &new_pos) {
                self.cursor = new_pos;
                return;
            }
        }
    }

    pub fn move_right(&mut self) {
        let closure = |new_pos: &mut Pos, currentarea: &Rect| {
            new_pos.x = currentarea.x + currentarea.width;
        };
        self.move_cursor(closure);
    }
    pub fn move_left(&mut self) {
        let closure = |new_pos: &mut Pos, currentarea: &Rect| {
            new_pos.x = std::cmp::max(currentarea.x, 1) - 1;
        };
        self.move_cursor(closure);
    }
    pub fn move_up(&mut self) {
        let closure = |new_pos: &mut Pos, currentarea: &Rect| {
            new_pos.y = std::cmp::max(currentarea.y, 1) - 1;
        };
        self.move_cursor(closure);
    }
    pub fn move_down(&mut self) {
        let closure = |new_pos: &mut Pos, currentarea: &Rect| {
            new_pos.y = currentarea.y + currentarea.height;
        };
        self.move_cursor(closure);
    }

    pub fn navigate(&mut self, direction: NavDir) {
        match direction {
            NavDir::Up => self.move_up(),
            NavDir::Down => self.move_down(),
            NavDir::Left => self.move_left(),
            NavDir::Right => self.move_right(),
        }
    }
}

pub fn draw_paragraph(
    f: &mut Frame<MyType>,
    area: Rect,
    text: Vec<Spans>,
    borderstyle: Style,
    alignment: Alignment,
    borders: Borders,
) {
    let block = Block::default().borders(borders).border_style(borderstyle);

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(alignment)
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}
