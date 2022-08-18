use std::time::{SystemTime, UNIX_EPOCH};
use serde_derive::{Deserialize, Serialize};
use rusqlite::Connection;
use crate::utils::sql::{
    fetch::fetch_card,
    update::update_status,
};


#[derive(Serialize,Debug, Deserialize, Clone)]
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



#[derive(Serialize, Deserialize, Clone, Debug)]
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


#[derive(Serialize, Deserialize, Debug,Clone)]
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
    pub dependencies: Vec<u32>,
    pub dependents: Vec<u32>,
    pub history: Vec<Review>,
    pub topic: u32,
    pub future: String,
    pub integrated: f32,
    pub card_id: u32,
}


impl Card {

    ///checks if the passed card should be resolved or not based on the completeness of its
    ///dependencies. If its status changed, it will recursively check all its dependents (and so
    ///on...)
    pub fn check_resolved(id: u32, conn: &Connection) -> bool {
        let mut change_detected = false;
        let mut card = fetch_card(conn, id);
        let mut check_resolved = true;
        for dependency in &card.dependencies{
           let dep_card = fetch_card(conn, *dependency);
           if !dep_card.status.complete || !dep_card.status.resolved{
               check_resolved = false;
               break;
           };
       } 
        if card.status.resolved != check_resolved{
            change_detected = true;
            card.status.resolved = check_resolved;
            update_status(conn, &card.clone()).unwrap();

            for dependent in card.dependents{
                Card::check_resolved(dependent, conn);
            }
            
        }
        change_detected
    }


    pub fn toggle_complete(mut card: Card, conn: &Connection) {
        //let card.status.complete = false;
        card.status.complete = !card.status.complete;
        update_status(conn, &card).unwrap();
        for dependent in card.dependents{
            Card::check_resolved(dependent, conn);
        }
    }

}



