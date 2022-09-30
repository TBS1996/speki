use std::time::{SystemTime, UNIX_EPOCH};
use rusqlite::Connection;
use crate::utils::sql::{
    fetch::{fetch_card, get_skiptime},
};








#[derive(Debug, Clone)]
pub enum RecallGrade{
    None,
    Failed,
    Decent,
    Easy,
}

impl RecallGrade{
    pub fn from(num: u32) -> Option<Self>{
        match num{
            0 => Some(RecallGrade::None),
            1 => Some(RecallGrade::Failed),
            2 => Some(RecallGrade::Decent),
            3 => Some(RecallGrade::Easy),
            _ => None

        }
    }
}



#[derive(Clone, Debug)]
pub struct Status {
    pub initiated: bool,
    pub complete:  bool,
    pub resolved:  bool,
    pub suspended: bool,
}

impl Status{
    pub fn isactive(&self)->bool{
        if self.complete && self.resolved && self.initiated && !self.suspended{
            return true
        }
        false
    }

    pub fn new_ready(&self)->bool{
        if !self.initiated && self.complete && self.resolved && !self.suspended{
            return true
        }
        false
    }
    pub fn from(num: u32)->Status{
        // not tested
        Status{
            initiated: num & 1 << 3 != 0,
            complete:  num & 1 << 2 != 0,
            resolved:  num & 1 << 1 != 0,
            suspended: num & 1 << 0 != 0,
        } 
    }
    pub fn new_complete() -> Status{
        Status {
            initiated: true,
            complete:  true,
            resolved:  true,
            suspended: false,
        }
    }

    pub fn new_incomplete() -> Status {
        Status {
            initiated: true,
            complete:  false,
            resolved:  true,
            suspended: false,
        }
    }
}


#[derive(Debug,Clone)]
pub struct Review{
    pub grade: RecallGrade,
    pub date: u32,
    pub answertime: f32,
}

impl Review{
    pub fn from(grade: &RecallGrade) -> Review {
        let unix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32;

        Review{
            grade: grade.clone(),
            date: unix,
            answertime: -1 as f32,
        }
    }
}



#[derive(Debug, Clone)]
pub enum CardType{
    Pending,
    Unfinished,
    Finished,


}

#[derive(Clone, Debug)]
pub struct Card{
    pub id: u32,
    pub question: String,
    pub answer: String,
    pub frontaudio: Option<String>,
    pub backaudio: Option<String>,
    pub frontimage: Option<String>,
    pub backimage: Option<String>,
    pub cardtype: CardType,
    pub suspended: bool,
    pub resolved:  bool,
    pub dependencies: Vec<IncID>,
    pub dependents: Vec<IncID>,
    pub topic: TopicID,
    pub source: IncID,
}


impl Card {
    ///checks if the passed card should be resolved or not based on the completeness of its
    ///dependencies. If its status changed, it will recursively check all its dependents (and so
    ///on...)

    pub fn new() -> Card{
        Card{
            id: 0,
            question: String::new(),
            answer: String::new(),
            frontaudio: None,
            backaudio: None,
            frontimage: None,
            backimage: None,
            cardtype: CardType::Finished,
            suspended: false,
            resolved: false,
            dependencies: vec![],
            dependents: vec![],
            topic: 0,
            source: 0,
        }
    }

    pub fn question(mut self, question: String)-> Self{
        self.question = question;
        self
    }
    pub fn answer(mut self, answer: String)-> Self{
        self.answer = answer;
        self
    }
    pub fn cardtype(mut self, cardtype: CardType)-> Self{
        self.cardtype = cardtype;
        self
    }
    pub fn source(mut self, source: IncID) -> Self{
        self.source = source;
        self
    }
    pub fn topic(mut self, topic: TopicID) -> Self{
        self.topic = topic;
        self
    }
    pub fn frontaudio(mut self, audiopath: Option<String>) -> Self{
        self.frontaudio = audiopath;
        self
    }
    pub fn backaudio(mut self, audiopath: Option<String>) -> Self{
        self.backaudio = audiopath;
        self
    }
    pub fn frontimage(mut self, imagepath: Option<String>) -> Self{
        self.frontimage = imagepath;
        self
    }
    pub fn backimage(mut self, imagepath: Option<String>) -> Self{
        self.backimage = imagepath;
        self
    }


    // these don't take self because theyre dependent on conditional logic 
    // after the card has been created 
    pub fn dependency(&mut self, dependency: CardID){
        self.dependencies.push(dependency);
    }
    pub fn dependent(&mut self, dependent: CardID){
        self.dependents.push(dependent);
    }


    pub fn is_complete(&self) -> bool {
        if let CardType::Finished = self.cardtype {
            return true
        }
        false
    }
    pub fn is_pending(&self) -> bool {
        if let CardType::Pending = self.cardtype {
            return true
        }
        false
    }
    pub fn is_unfinished(&self) -> bool {
        if let CardType::Unfinished = self.cardtype {
            return true
        }
        false
    }


    pub fn skiptime(&self, conn: &Connection) -> u32{
        if let CardType::Unfinished = self.cardtype{
            return get_skiptime(conn, self.id).unwrap();
        } else {
            panic!();
        }
    }
  pub fn skipduration(&self, conn: &Connection) -> u32{
        if let CardType::Unfinished = self.cardtype{
            return get_skipduration(conn, self.id).unwrap();
        } else {
            panic!();
        }
    }

    pub fn save_card(self, conn: &Connection) -> CardID{
        let dependencies = self.dependencies.clone();
        let dependents = self.dependents.clone();
        let finished = self.is_complete();
        save_card(conn, self).unwrap();
        let card_id = conn.last_insert_rowid() as CardID;

        if finished{
            revlog_new(conn, card_id , Review::from(&RecallGrade::Decent)).unwrap();
        }

        for dependency in dependencies{
            update_both(conn, card_id, dependency).unwrap();
        }
        for dependent in dependents{
            update_both(conn, dependent, card_id).unwrap();
            Self::check_resolved(dependent, conn);
        }

        Self::check_resolved(card_id, conn);
        card_id
    }


    pub fn check_resolved(id: u32, conn: &Connection) -> bool {
        let mut change_detected = false;
        let mut card = fetch_card(conn, id);
        let mut is_resolved = true;

        for dependency in &card.dependencies{
           let dep_card = fetch_card(conn, *dependency);
           if  !dep_card.resolved  || !dep_card.is_complete() {
               is_resolved = false;
               break;
           };
       } 
        if card.resolved != is_resolved{
            change_detected = true;
            card.resolved = is_resolved;
            set_resolved(conn, card.id, card.resolved).unwrap();

            for dependent in card.dependents{
                Card::check_resolved(dependent, conn);
            }
        }
        change_detected
    }

    pub fn new_review(conn: &Connection, id: CardID, review: RecallGrade){
        revlog_new(conn, id, Review::from(&review)).unwrap();
        super::interval::calc_stability(conn, id);
    }
    pub fn complete_card(conn: &Connection, id: CardID){
        let card = fetch_card(conn, id);
        remove_unfinished(conn, id).unwrap();
        new_finished(conn, id).unwrap();
        revlog_new(conn, id, Review::from(&RecallGrade::Decent)).unwrap();
        for dependent in card.dependents{
            Card::check_resolved(dependent, conn);
        }
    }

    pub fn play_frontaudio(conn: &Connection, id: CardID, handle: &rodio::OutputStreamHandle) {
        let card = fetch_card(conn, id);
        if let Some(path) = card.frontaudio{
            Self::play_audio(handle, path);
        }
    }
    pub fn play_backaudio(conn: &Connection, id: CardID, handle: &rodio::OutputStreamHandle) {
        let card = fetch_card(conn, id);
        if let Some(path) = card.backaudio{
            Self::play_audio(handle, path);
        }
    }
    fn play_audio(handle: &rodio::OutputStreamHandle, path: String){
        let file = std::fs::File::open(path).unwrap();
        let beep1 = handle.play_once(BufReader::new(file)).unwrap();
        beep1.set_volume(0.2);
        beep1.detach();
    }

}


use crate::utils::aliases::*;
use super::sql::{insert::{save_card, update_both}, update::set_resolved, fetch::get_skipduration};
use super::sql::insert::revlog_new;
use super::sql::insert::new_finished;
use super::sql::delete::remove_unfinished;
use std::io::BufReader;
