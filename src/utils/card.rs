use std::time::{SystemTime, UNIX_EPOCH};
use rusqlite::Connection;
use crate::utils::sql::{
    fetch::fetch_card,
    update::{activate_card, update_status},
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


#[derive(Clone, Debug)]
pub struct Card{
    pub question: String,
    pub answer: String,
    pub status: Status,
    pub strength: f32,
    pub stability: f32,
    pub dependencies: Vec<IncID>,
    pub dependents: Vec<IncID>,
    pub history: Vec<Review>,
    pub topic: TopicID,
    pub future: String,
    pub integrated: f32,
    pub card_id: CardID,
    pub source: IncID,
    pub skiptime: u32,
    pub skipduration: u32,
}


impl Card {
    ///checks if the passed card should be resolved or not based on the completeness of its
    ///dependencies. If its status changed, it will recursively check all its dependents (and so
    ///on...)

    pub fn new() -> Card{
        Card{
            question: String::new(),
            answer: String::new(),
            status: Status::new_complete(),
            strength: 1.0,
            stability: 1.0,
            dependencies: vec![],
            dependents: vec![],
            history: Vec::<Review>::new(),
            topic: 0,
            future: String::new(),
            integrated: 1.0,
            card_id: 0,
            source: 0,
            skiptime: 0,
            skipduration: 0,
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
    pub fn status(mut self, status: Status)-> Self{
        self.status = status;
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


    // these don't take self because theyre dependent on conditional logic 
    // after the card has been created 
    pub fn dependency(&mut self, dependency: CardID){
        self.dependencies.push(dependency);
    }
    pub fn dependent(&mut self, dependent: CardID){
        self.dependents.push(dependent);
    }


    pub fn save_card(self, conn: &Connection) -> CardID{
        let dependencies = self.dependencies.clone();
        let dependents = self.dependents.clone();
        save_card(conn, self).unwrap();
        let card_id = conn.last_insert_rowid() as CardID;
        revlog_new(conn, card_id , Review::from(&RecallGrade::Decent)).unwrap();

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
           if  !dep_card.status.resolved  ||!dep_card.status.complete {
               is_resolved = false;
               break;
           };
       } 
        if card.status.resolved != is_resolved{
            change_detected = true;
            card.status.resolved = is_resolved;
            update_status(conn, &card.clone()).unwrap();

            for dependent in card.dependents{
                Card::check_resolved(dependent, conn);
            }
        }
        change_detected
    }




    pub fn new_review(conn: &Connection, id: CardID, review: RecallGrade){
        revlog_new(conn, id, Review::from(&review)).unwrap();
        super::interval::calc_stability(conn, id);
        activate_card(conn, id).unwrap();
    }


    pub fn toggle_complete(id: CardID, conn: &Connection) {
        //let card.status.complete = false;
        let mut card = fetch_card(conn, id);
        card.status.complete = !card.status.complete;
        update_status(conn, &card).unwrap();
        for dependent in card.dependents{
            Card::check_resolved(dependent, conn);
        }
    }
}


use crate::utils::aliases::*;
use super::sql::insert::{save_card, update_both};
use super::sql::insert::revlog_new;
