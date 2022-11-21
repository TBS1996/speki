use super::fetch::*;
use crate::utils::aliases::*;
use crate::utils::card::CardType;
use crate::widgets::textinput::CursorPos;
use rand::prelude::*;
use rusqlite::{params, Connection, Result, ToSql};
use std::fmt::Display;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn update_card<T, U, V, IDVec>(
    conn: &Arc<Mutex<Connection>>,
    table: T,
    column: U,
    value: V,
    idvec: IDVec,
) -> Result<()>
where
    T: Display,
    U: Display,
    V: ToSql,
    IDVec: Into<Vec<CardID>>,
{
    let idvec = idvec.into();
    let conn = conn.lock().unwrap();
    for id in idvec {
        conn.prepare(&format!(
            "UPDATE {} SET {} = ?1 WHERE id = {}",
            table, column, id
        ))?
        .execute(params![value])?;
    }
    Ok(())
}

pub fn set_suspended<IDVec: Into<Vec<CardID>>>(
    conn: &Arc<Mutex<Connection>>,
    id: IDVec,
    suspended: bool,
) {
    update_card(conn, "cards", "suspended", suspended as u8, id).unwrap()
}

pub fn update_card_question(conn: &Arc<Mutex<Connection>>, id: CardID, question: String) {
    update_card(conn, "cards", "question", question, [id]).unwrap()
}

pub fn update_card_answer(conn: &Arc<Mutex<Connection>>, id: CardID, answer: String) {
    update_card(conn, "cards", "answer", answer, [id]).unwrap()
}

pub fn update_strength(conn: &Arc<Mutex<Connection>>, id: CardID, strength: f32) {
    update_card(conn, "finished_cards", "strength", strength, [id]).unwrap()
}

pub fn set_stability(conn: &Arc<Mutex<Connection>>, id: CardID, stability: Duration) {
    update_card(
        conn,
        "finished_cards",
        "stability",
        stability.as_secs_f64() / 86400.,
        [id],
    )
    .unwrap()
}

pub fn update_position(conn: &Arc<Mutex<Connection>>, id: CardID, position: u32) {
    update_card(conn, "pending_cards", "position", position, [id]).unwrap()
}

pub fn update_topic(conn: &Arc<Mutex<Connection>>, id: CardID, topic_id: TopicID) {
    update_card(conn, "cards", "topic", topic_id, [id]).unwrap()
}

pub fn set_cardtype(conn: &Arc<Mutex<Connection>>, id: CardID, cardtype: CardType) {
    let cardtype = match cardtype {
        CardType::Pending => 0,
        CardType::Unfinished => 1,
        CardType::Finished => 2,
    };
    update_card(conn, "cards", "cardtype", cardtype, [id]).unwrap()
}

pub fn set_resolved(conn: &Arc<Mutex<Connection>>, id: CardID, resolved: bool) {
    update_card(conn, "cards", "resolved", resolved as u8, [id]).unwrap()
}

pub fn update_topic_name(conn: &Arc<Mutex<Connection>>, id: u32, name: String) {
    update_card(conn, "topics", "name", name, [id]).unwrap()
}

pub fn update_topic_relpos(conn: &Arc<Mutex<Connection>>, id: u32, relpos: u32) {
    update_card(conn, "topics", "relpos", relpos, [id]).unwrap()
}

pub fn update_card_source(conn: &Arc<Mutex<Connection>>, cardid: CardID, incid: IncID) {
    update_card(conn, "cards", "source", incid, [cardid]).unwrap()
}
pub fn update_card_topic(conn: &Arc<Mutex<Connection>>, old_topic: TopicID, new_topic: TopicID) {
    conn.lock()
        .unwrap()
        .prepare("UPDATE cards SET topic = ? WHERE topic = ?")
        .unwrap()
        .execute(params![new_topic, old_topic])
        .unwrap();
}

pub fn update_topic_parent(conn: &Arc<Mutex<Connection>>, id: u32, parent: u32) {
    update_card(conn, "topics", "parent", parent, [id]).unwrap()
}

pub fn update_inc_text(
    conn: &Arc<Mutex<Connection>>,
    source: String,
    id: IncID,
    cursor: &CursorPos,
) -> Result<()> {
    let row = cursor.row;
    let column = cursor.column;

    conn.lock()
        .unwrap()
        .prepare("UPDATE incread SET source = ?1, row = ?2, column = ?3 WHERE id = ?4")
        .unwrap()
        .execute(params![source, row, column, id])?;
    Ok(())
}

pub fn update_inc_active(conn: &Arc<Mutex<Connection>>, id: IncID, active: bool) -> Result<()> {
    conn.lock()
        .unwrap()
        .prepare("UPDATE incread SET active = ? WHERE id = ?")
        .unwrap()
        .execute(params![active, id])?;
    Ok(())
}

pub fn double_skip_duration(conn: &Arc<Mutex<Connection>>, id: CardID) {
    update_skiptime(conn, id);
    let mut rng = rand::thread_rng();
    let mut y: f64 = rng.gen();
    y += 0.5; // y is now between 0.5 and 1.5
    let skipduration = get_skipduration(conn, id).unwrap();
    let new_skipduration = std::cmp::max((skipduration as f64 * y * 2.0) as u32, 2);
    update_card(
        conn,
        "unfinished_cards",
        "skipduration",
        new_skipduration,
        [id],
    )
    .unwrap();
}

pub fn update_skiptime(conn: &Arc<Mutex<Connection>>, id: CardID) {
    let unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;
    update_card(conn, "unfinished_cards", "skiptime", unix, [id]).unwrap();
}

pub fn double_inc_skip_duration(conn: &Arc<Mutex<Connection>>, id: IncID) -> Result<()> {
    update_inc_skiptime(conn, id).unwrap();
    let mut rng = rand::thread_rng();
    let mut y: f64 = rng.gen();
    y += 0.5; // y is now between 0.5 and 1.5
    let skipduration = get_inc_skipduration(conn, id).unwrap();
    let new_skipduration = std::cmp::max((skipduration as f64 * y * 2.0) as u32, 2);
    conn.lock()
        .unwrap()
        .prepare("UPDATE incread SET skipduration = ? WHERE id = ?")
        .unwrap()
        .execute(params![new_skipduration, id])?;
    Ok(())
}

pub fn update_inc_skiptime(conn: &Arc<Mutex<Connection>>, id: IncID) -> Result<()> {
    let unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;
    conn.lock()
        .unwrap()
        .prepare("UPDATE incread SET skiptime = ? WHERE id = ?")
        .unwrap()
        .execute(params![unix, id])?;
    Ok(())
}
