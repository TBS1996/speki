use std::time::{SystemTime, UNIX_EPOCH};
use serde_derive::{Deserialize, Serialize};

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



#[derive(Serialize, Deserialize, Clone)]
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


#[derive(Clone)]
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




#[derive(Clone)]
pub struct Topic{
    pub topic_id: u32,
    pub name: String,
    pub children: Vec<u32>,
}


