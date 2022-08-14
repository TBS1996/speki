use crate::card::Status;
//use std::ops::Range

/*
search
status
strength
topic
*/



pub struct CardFilter{
    searchterm: Option<String>,
  //  status: Status::from(0b1111),
    min_strength: f32,
    max_strength: f32,
    topics: Vec<u32>,
}
