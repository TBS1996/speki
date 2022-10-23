use crate::utils::sql::fetch::fetch_card;
use rusqlite::Connection;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::watch::channel;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub enum RecallGrade {
    None,
    Failed,
    Decent,
    Easy,
}

impl RecallGrade {
    pub fn from(num: u32) -> Option<Self> {
        match num {
            0 => Some(RecallGrade::None),
            1 => Some(RecallGrade::Failed),
            2 => Some(RecallGrade::Decent),
            3 => Some(RecallGrade::Easy),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Status {
    pub initiated: bool,
    pub complete: bool,
    pub resolved: bool,
    pub suspended: bool,
}

impl Status {
    pub fn isactive(&self) -> bool {
        if self.complete && self.resolved && self.initiated && !self.suspended {
            return true;
        }
        false
    }

    pub fn new_ready(&self) -> bool {
        if !self.initiated && self.complete && self.resolved && !self.suspended {
            return true;
        }
        false
    }
    pub fn from(num: u32) -> Status {
        // not tested
        Status {
            initiated: num & 1 << 3 != 0,
            complete: num & 1 << 2 != 0,
            resolved: num & 1 << 1 != 0,
            suspended: num & 1 << 0 != 0,
        }
    }
    pub fn new_complete() -> Status {
        Status {
            initiated: true,
            complete: true,
            resolved: true,
            suspended: false,
        }
    }

    pub fn new_incomplete() -> Status {
        Status {
            initiated: true,
            complete: false,
            resolved: true,
            suspended: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Review {
    pub grade: RecallGrade,
    pub date: u32,
    pub answertime: f32,
}

impl Review {
    pub fn from(grade: &RecallGrade) -> Review {
        let unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;

        Review {
            grade: grade.clone(),
            date: unix,
            answertime: -1.,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum CardType {
    Pending,
    Unfinished,
    Finished,
}

#[derive(Clone, Debug)]
pub struct Card {
    pub id: CardID,
    pub question: String,
    pub answer: String,
    pub frontaudio: Option<PathBuf>,
    pub backaudio: Option<PathBuf>,
    pub frontimage: Option<PathBuf>,
    pub backimage: Option<PathBuf>,
    pub cardtype: CardTypeData,
    pub suspended: bool,
    pub resolved: bool,
    pub dependencies: Vec<IncID>,
    pub dependents: Vec<IncID>,
    pub history: Vec<Review>,
    pub topic: TopicID,
    pub source: IncID,
}

#[derive(Clone, Debug)]
pub enum CardTypeData {
    Finished(FinishedInfo),
    Unfinished(UnfinishedInfo),
    Pending(PendingInfo),
}

#[derive(Clone, Debug)]
pub struct FinishedInfo {
    pub strength: f32,
    pub stability: f32,
}

impl Default for FinishedInfo {
    fn default() -> Self {
        Self {
            strength: 1.0,
            stability: 1.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct UnfinishedInfo {
    pub skiptime: u32,
    pub skipduration: u32,
}

impl Default for UnfinishedInfo {
    fn default() -> Self {
        Self {
            skiptime: get_current_unix(),
            skipduration: 1,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct PendingInfo {
    pub pos: u32,
}

impl Card {
    ///checks if the passed card should be resolved or not based on the completeness of its
    ///dependencies. If its status changed, it will recursively check all its dependents (and so
    ///on...)

    pub fn new(cardtype: CardTypeData) -> Card {
        Card {
            id: 0,
            question: String::new(),
            answer: String::new(),
            frontaudio: None,
            backaudio: None,
            frontimage: None,
            backimage: None,
            cardtype,
            suspended: false,
            resolved: false,
            dependencies: vec![],
            dependents: vec![],
            history: vec![],
            topic: 0,
            source: 0,
        }
    }

    pub fn question(mut self, question: String) -> Self {
        self.question = question;
        self
    }
    pub fn answer(mut self, answer: String) -> Self {
        self.answer = answer;
        self
    }
    pub fn source(mut self, source: IncID) -> Self {
        self.source = source;
        self
    }
    pub fn topic(mut self, topic: TopicID) -> Self {
        self.topic = topic;
        self
    }
    pub fn frontaudio(mut self, audiopath: Option<PathBuf>) -> Self {
        self.frontaudio = audiopath;
        self
    }
    pub fn backaudio(mut self, audiopath: Option<PathBuf>) -> Self {
        self.backaudio = audiopath;
        self
    }
    pub fn frontimage(mut self, imagepath: Option<PathBuf>) -> Self {
        self.frontimage = imagepath;
        self
    }
    pub fn backimage(mut self, imagepath: Option<PathBuf>) -> Self {
        self.backimage = imagepath;
        self
    }

    pub fn dependencies<IDVec: Into<Vec<CardID>>>(mut self, dependencies: IDVec) -> Self {
        for dependency in dependencies.into() {
            self.dependencies.push(dependency);
        }
        self
    }
    pub fn dependents<IDVec: Into<Vec<CardID>>>(mut self, dependents: IDVec) -> Self {
        for dependent in dependents.into() {
            self.dependents.push(dependent);
        }
        self
    }

    pub fn is_complete(&self) -> bool {
        if let CardTypeData::Finished(_) = self.cardtype {
            return true;
        }
        false
    }
    pub fn is_pending(&self) -> bool {
        if let CardTypeData::Pending(_) = self.cardtype {
            return true;
        }
        false
    }
    pub fn is_unfinished(&self) -> bool {
        if let CardTypeData::Unfinished(_) = self.cardtype {
            return true;
        }
        false
    }

    pub fn save_card(self, conn: &Arc<Mutex<Connection>>) -> CardID {
        let dependencies = self.dependencies.clone();
        let dependents = self.dependents.clone();
        let finished = self.is_complete();
        let card_id = save_card(conn, self);

        if finished {
            revlog_new(conn, card_id, Review::from(&RecallGrade::Decent)).unwrap();
        }

        for dependency in dependencies {
            update_both(conn, card_id, dependency).unwrap();
        }
        for dependent in dependents {
            update_both(conn, dependent, card_id).unwrap();
            Self::check_resolved(dependent, conn);
        }

        Self::check_resolved(card_id, conn);
        card_id
    }

    pub fn check_resolved(id: u32, conn: &Arc<Mutex<Connection>>) -> bool {
        let mut change_detected = false;
        let mut card = fetch_card(conn, id);
        let mut is_resolved = true;

        for dependency in &card.dependencies {
            let dep_card = fetch_card(conn, *dependency);
            if !dep_card.resolved || !dep_card.is_complete() {
                is_resolved = false;
                break;
            };
        }
        if card.resolved != is_resolved {
            change_detected = true;
            card.resolved = is_resolved;
            set_resolved(conn, card.id, card.resolved);

            for dependent in card.dependents {
                Card::check_resolved(dependent, conn);
            }
        }
        change_detected
    }

    pub fn new_review(conn: &Arc<Mutex<Connection>>, id: CardID, review: RecallGrade) {
        revlog_new(conn, id, Review::from(&review)).unwrap();
        super::interval::calc_stability(conn, id);
    }
    pub fn complete_card(conn: &Arc<Mutex<Connection>>, id: CardID) {
        let card = fetch_card(conn, id);
        remove_unfinished(conn, id).unwrap();
        new_finished(conn, id).unwrap();
        revlog_new(conn, id, Review::from(&RecallGrade::Decent)).unwrap();
        for dependent in card.dependents {
            Card::check_resolved(dependent, conn);
        }
    }

    pub fn activate_card(conn: &Arc<Mutex<Connection>>, id: CardID) {
        remove_pending(conn, id).unwrap();
        new_finished(conn, id).unwrap();
    }

    pub fn play_frontaudio(conn: &Arc<Mutex<Connection>>, id: CardID, audio: &Option<Audio>) {
        let card = fetch_card(conn, id);
        if let Some(path) = card.frontaudio {
            crate::utils::misc::play_audio(audio, path);
        }
    }
    pub fn play_backaudio(conn: &Arc<Mutex<Connection>>, id: CardID, audio: &Option<Audio>) {
        let card = fetch_card(conn, id);
        if let Some(path) = card.backaudio {
            crate::utils::misc::play_audio(audio, path);
        }
    }
}

use super::misc::get_current_unix;
use super::sql::delete::{remove_pending, remove_unfinished};
use super::sql::insert::new_finished;
use super::sql::insert::revlog_new;
use super::sql::{
    insert::{save_card, update_both},
    update::set_resolved,
};
use crate::app::Audio;
use crate::utils::aliases::*;

pub struct CardInfo {
    id: CardID,
    frontside: String,
    suspended: bool,
    resolved: bool,
    cardtype: CardType,
    strength: f32,
    stability: f32,
}
