use std::sync::{Arc, Mutex};

use crate::{tabs::review::logic::ReviewMode, widgets::cardlist::CardItem};
use rusqlite::Connection;
use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Color,
};

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

pub fn split_updown<C>(constraints: C, area: Rect) -> Vec<Rect>
where
    C: Into<Vec<u16>>,
{
    split(constraints, area, Direction::Vertical)
}

pub fn split_leftright<C>(constraints: C, area: Rect) -> Vec<Rect>
where
    C: Into<Vec<u16>>,
{
    split(constraints, area, Direction::Horizontal)
}

fn split<C>(constraints: C, area: Rect, direction: Direction) -> Vec<Rect>
where
    C: Into<Vec<u16>>,
{
    let mut constraintvec: Vec<Constraint> = vec![];
    let constraints = constraints.into();
    for c in constraints {
        constraintvec.push(Constraint::Percentage(c));
    }

    Layout::default()
        .direction(direction)
        .constraints(constraintvec.as_ref())
        .split(area)
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

#[tokio::main]
pub async fn get_gpt3_response(api_key: &str, user_input: &str) -> String {
    // Check for environment variable OPENAI_KEY

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
            println!("Error calling OpenAI. Check environment variable OPENAI_KEY");
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
