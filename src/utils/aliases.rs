use std::sync::{Arc, MutexGuard};

use rusqlite::Connection;

pub type TopicID = u32;
pub type CardID = u32;
pub type IncID = u32;

pub type ModelID = u64;
pub type NoteID = u64;

//pub type Conn<'a> = Arc<MutexGuard<'a, Connection>>;
