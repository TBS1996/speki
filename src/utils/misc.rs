use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Write},
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{app::Audio, tabs::review::logic::ReviewMode, widgets::cardlist::CardItem, NavDir, MyType};
use rusqlite::Connection;
use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Color, Frame,
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
    if let Ok(file) = std::fs::File::open(path) {
        if let Some(audio) = audio {
            let beep1 = audio.handle.play_once(BufReader::new(file)).unwrap();
            beep1.set_volume(audio.volume);
            beep1.detach();
        }
    }
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

// sometimes splitting would leave a small gap, so this function fills the gaps
fn fill_areas(areas: &mut Vec<Rect>, direction: Direction) {
    match direction {
        Direction::Horizontal => {
            for i in 0..areas.len() - 1 {
                areas[i].width = areas[i + 1].x - areas[i].x;
            }
        }
        Direction::Vertical => {
            for i in 0..areas.len() - 1 {
                areas[i].height = areas[i + 1].y - areas[i].y;
            }
        }
    }
}

pub fn split_updown_by_percent<C: Into<Vec<u16>>>(constraints: C, area: Rect) -> Vec<Rect> {
    let mut constraintvec: Vec<Constraint> = vec![];
    for c in constraints.into() {
        constraintvec.push(Constraint::Percentage(c));
    }
    split(constraintvec, area, Direction::Vertical)
}

pub fn split_leftright_by_percent<C: Into<Vec<u16>>>(constraints: C, area: Rect) -> Vec<Rect> {
    let mut constraintvec: Vec<Constraint> = vec![];
    for c in constraints.into() {
        constraintvec.push(Constraint::Percentage(c));
    }
    split(constraintvec, area, Direction::Horizontal)
}

pub fn split_updown<C: Into<Vec<Constraint>>>(constraints: C, area: Rect) -> Vec<Rect> {
    split(constraints.into(), area, Direction::Vertical)
}

pub fn split_leftright<C: Into<Vec<Constraint>>>(constraints: C, area: Rect) -> Vec<Rect> {
    split(constraints.into(), area, Direction::Horizontal)
}

fn split(constraints: Vec<Constraint>, area: Rect, direction: Direction) -> Vec<Rect> {
    let mut areas = Layout::default()
        .direction(direction.clone())
        .constraints(constraints)
        .split(area);
    fill_areas(&mut areas, direction);
    areas
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

use super::{aliases::CardID, sql::fetch::fetch_card, statelist::StatefulList};

// stole code from here: https://github.com/zahidkhawaja/rusty
#[tokio::main]
pub async fn get_gpt3_response(api_key: &str, user_input: &str) -> String {
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
            std::process::exit(1);
        }
    };

    json.choices[0]
        .text
        .split('\n')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
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
    StatefulList::with_items(depvec)
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
    StatefulList::with_items(depvec)
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
}

impl SpekiPaths {
    const DEFAULTCONFIG: &'static str = r#"
#gptkey = ""
        "#;
    pub fn new(mut home: PathBuf) -> Self {
        let mut configpath = home.clone();
        if cfg!(windows) {
            home.push(".speki/");
            if !std::path::Path::new(&home).exists() {
                std::fs::create_dir(&home).unwrap();
            }
            configpath.push("config.toml");
            if !std::path::Path::new(&configpath).exists() {
                let mut file = File::create(&configpath).unwrap();
                file.write_all(Self::DEFAULTCONFIG.as_bytes()).unwrap();
            }
        } else {
            home.push(".local/share/speki/");
            std::fs::create_dir_all(&home).unwrap();
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
            base: home,
            database,
            media,
            tempfolder,
            downloc,
            backups,
            config: configpath,
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

impl Default for View {
    fn default() -> Self {
        let areas = HashMap::new();
        let cursor = (0, 4);
        Self { areas, cursor }
    }
}

#[derive(Clone)]
pub struct View {
    pub areas: HashMap<&'static str, Rect>,
    pub cursor: (u16, u16),
}

impl View {

    pub fn debug_show_cursor(&self, f: &mut Frame<MyType>){
        f.set_cursor(self.cursor.0, self.cursor.1);
    }


    pub fn get_area(&self, name: &'static str) -> Rect {
        *self.areas.get(name).unwrap()
    }
    pub fn is_selected(&self, area: Rect) -> bool {
        Self::isitselected(area, &self.cursor)
    }

    pub fn name_selected(&self, name: &'static str) -> bool {
        self.is_selected(*self.areas.get(name).unwrap())
    }

    pub fn validate_pos(&mut self) {
        for (_, area) in self.areas.iter() {
            if self.is_selected(*area) {
                return;
            }
        }
        let area = self.areas.values().next().unwrap();
        self.cursor = (area.x, area.y);
    }

    pub fn isitselected(area: Rect, cursor: &(u16, u16)) -> bool {
        cursor.0 >= area.x
            && cursor.0 < area.x + area.width
            && cursor.1 >= area.y
            && cursor.1 < area.y + area.height
    }

    fn move_cursor<F>(&mut self, mut func: F)
    where
        F: FnMut(&mut (u16, u16), &Rect),
    {
        let areas: Vec<Rect> = self.areas.values().cloned().collect();
        let mut currentarea = areas[0];
        let mut new_pos = self.cursor;
        for area in &areas {
            if Self::isitselected(*area, &self.cursor) {
                currentarea = *area;
            }
        }
        func(&mut new_pos, &currentarea);
        for area in &areas {
            // validating that new pos is in an area
            if Self::isitselected(*area, &new_pos) {
                self.cursor = new_pos;
                return;
            }
        }
    }

    pub fn move_right(&mut self) {
        let closure = |new_pos: &mut (u16, u16), currentarea: &Rect| {
            new_pos.0 = currentarea.x + currentarea.width;
        };
        self.move_cursor(closure);
    }
    pub fn move_left(&mut self) {
        let closure = |new_pos: &mut (u16, u16), currentarea: &Rect| {
            new_pos.0 = std::cmp::max(currentarea.x, 1) - 1;
        };
        self.move_cursor(closure);
    }
    pub fn move_up(&mut self) {
        let closure = |new_pos: &mut (u16, u16), currentarea: &Rect| {
            new_pos.1 = std::cmp::max(currentarea.y, 1) - 1;
        };
        self.move_cursor(closure);
    }
    pub fn move_down(&mut self) {
        let closure = |new_pos: &mut (u16, u16), currentarea: &Rect| {
            new_pos.1 = currentarea.y + currentarea.height;
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

    /*
        pub fn move_up(&mut self) {
            let mut new_ypos = self.cursor.1.;
            let mut areamatch = false;
            for area in &self.areas {
                if area.x <= self.cursor.0
                    && area.x + area.width >= self.cursor.0
                    && area.y < self.cursor.1
                    && !self.is_selected(*area)
                {
                    if areamatch {
                        new_ypos = std::cmp::max(new_ypos, area.y + area.height - 1);
                    } else {
                        new_ypos = area.y + area.height / 2;
                    }
                    areamatch = true;
                }
            }
            self.cursor.1 = new_ypos;
        }
        pub fn move_down(&mut self) {
            let mut new_ypos = self.cursor.1;
            let mut areamatch = false;
            for area in &self.areas {
                if area.x < self.cursor.0
                    && area.x + area.width > self.cursor.0
                    && area.y > self.cursor.1
                    && !self.is_selected(*area)
                {
                    if areamatch {
                        new_ypos = std::cmp::min(new_ypos, area.y);
                    } else {
                        new_ypos = area.y;
                    }
                    areamatch = true;
                }
            }
            self.cursor.1 = new_ypos;
        }
    pub fn move_right(&mut self) {
            let mut new_xpos = self.cursor.0;
            let mut areamatch = false;
            for area in &self.areas {
                if area.y < self.cursor.1
                    && area.y + area.height > self.cursor.1
                    && area.x > self.cursor.0
                    && !self.is_selected(*area)
                {
                    if areamatch {
                        new_xpos = std::cmp::min(new_xpos, area.x);
                    } else {
                        new_xpos = area.x
                    }
                    areamatch = true;
                }
            }
            self.cursor.0 = new_xpos;
        }
    */
}
