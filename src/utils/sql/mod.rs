pub mod fetch;
pub mod insert;
pub mod update;
pub mod delete;




use rusqlite::{Connection, Result};
use crate::utils::sql::insert::new_topic;

use std::{sync::{Mutex, Arc}, path::PathBuf};

use self::insert::new_incread;

use super::card::Card;





pub fn init_db(dbpath: &PathBuf) -> Result<bool>{

    let mut new_db = false;
    if let Err(_) = std::fs::metadata(dbpath){
        new_db = true;
    }


    let conn = Connection::open(dbpath)?;

    conn.execute(
        "create table if not exists cards (
            id           integer primary key,
            question     text not null,
            answer       text not null,
            frontaudio   text,
            backaudio    text,
            frontimg     text,
            backimg      text,
            cardtype     integer not null,
            suspended    bool not null,
            resolved     bool not null,
            topic        integer not null,
            source       integer not null
            
    )",
        [],
        )?;

    conn.execute(
        "create table if not exists finished_cards (
            id           integer not null,
            strength     real not null,
            stability    real not null
    )",
        [],
        )?;


    conn.execute(
        "create table if not exists unfinished_cards (
            id           integer not null,
            skiptime     integer not null,
            skipduration integer not null
    )",
        [],
        )?;


    conn.execute(
        "create table if not exists pending_cards (
            id           integer not null,
            position     integer not null
    )",
        [],
        )?;

    conn.execute(
        "create table if not exists topics ( 
            id     integer primary key,
            name   text,
            parent integer not null,
            relpos integer not null
    )",
        [],
        )?;
    
    conn.execute(
        "create table if not exists revlog ( 
            unix   integer not null,
            cid    integer not null,
            grade  integernot null,
            qtime  real not null,
            atime  real not null
        )",
        [],
        )?;


    conn.execute(
        "create table if not exists dependencies ( 
            dependent integer not null,
            dependency integer not null

    )",
        [],
        )?;


    conn.execute(
        "create table if not exists incread ( 
            id integer primary key,
            parent integer not null,
            topic integer not null,
            source text not null,
            active integer not null,
            skiptime integer,
            skipduration integer,
            row integer,
            column integer

    )",
        [],
        )?;
    
    let conn = Arc::new(Mutex::new(conn));


    if new_db {
        new_topic(&conn, String::from("root"), 0, 0)?;
        Card::new()
            .question("How do you navigate between widgets?".to_string())
            .answer("Alt+(h/j/k/l) ... or alt+arrowkeys".to_string())
            .cardtype(super::card::CardType::Pending)
            .save_card(&conn);
        Card::new()
            .question("How do you suspend a card?".to_string())
            .answer("Alt+i".to_string())
            .cardtype(super::card::CardType::Pending)
            .save_card(&conn);
        Card::new()
            .question("What does it mean for a card to have another card as a dependency?".to_string())
            .answer("It means you cannot understand the question/answer unless you first understand the dependency card.".to_string())
            .cardtype(super::card::CardType::Pending)
            .save_card(&conn);
        Card::new()
            .question("What does it mean for a card to have another card as a dependent?".to_string())
            .answer("it means the other card has the current card as a dependency".to_string())
            .cardtype(super::card::CardType::Pending)
            .save_card(&conn);
        Card::new()
            .question("What does it mean for a card to be resolved?".to_string())
            .answer("It means it doesn't have any dependencies that are unfinished or also unresolved".to_string())
            .cardtype(super::card::CardType::Pending)
            .save_card(&conn);
        Card::new()
            .question("How do you add a new card that you have the answer to?".to_string())
            .answer("On the \"Add card\" tab, type Alt+f".to_string())
            .cardtype(super::card::CardType::Pending)
            .save_card(&conn);
        Card::new()
            .question("How do you add a new card that you DON'T have the answer to?".to_string())
            .answer("On the \"Add card\" tab, type Alt+u".to_string())
            .cardtype(super::card::CardType::Pending)
            .save_card(&conn);
        Card::new()
            .question("When you review an unfinished card, how do you skip it?".to_string())
            .answer("Alt+s".to_string())
            .cardtype(super::card::CardType::Pending)
            .save_card(&conn);
        Card::new()
            .question("When you review an unfinished card, how do you mark it as complete?".to_string())
            .answer("Alt+f".to_string())
            .cardtype(super::card::CardType::Pending)
            .save_card(&conn);
        Card::new()
            .question("When reviewing a card, how do you add a new card as a dependency to it?".to_string())
            .answer("Alt+Y (upper case)".to_string())
            .cardtype(super::card::CardType::Pending)
            .save_card(&conn);
        Card::new()
            .question("When reviewing a card, how do you add an existing card as a dependency to it?".to_string())
            .answer("Alt+y (lower case)".to_string())
            .cardtype(super::card::CardType::Pending)
            .save_card(&conn);
        Card::new()
            .question("When reviewing a card, how do you add a new card as a dependent to it?".to_string())
            .answer("Alt+T (upper case)".to_string())
            .cardtype(super::card::CardType::Pending)
            .save_card(&conn);
        Card::new()
            .question("When reviewing a card, how do you add an existing card as a dependent to it?".to_string())
            .answer("Alt+t (lower case)".to_string())
            .cardtype(super::card::CardType::Pending)
            .save_card(&conn);
        Card::new()
            .question("How do you exit speki?".to_string())
            .answer("Alt+q".to_string())
            .cardtype(super::card::CardType::Pending)
            .save_card(&conn);
        Card::new()
            .question("How do you take extracts in incremental reading?".to_string())
            .answer("Alt+x in visual mode".to_string())
            .cardtype(super::card::CardType::Pending)
            .save_card(&conn);
        Card::new()
            .question("How do you make cloze deletions in incremental reading?".to_string())
            .answer("Alt+z".to_string())
            .cardtype(super::card::CardType::Pending)
            .save_card(&conn);
        Card::new()
            .question("How do you skip an incremental reading text during review?".to_string())
            .answer("Alt+s".to_string())
            .cardtype(super::card::CardType::Pending)
            .save_card(&conn);
        Card::new()
            .question("How do you mark an incremental reading text as done during review?".to_string())
            .answer("Alt+d".to_string())
            .cardtype(super::card::CardType::Pending)
            .save_card(&conn);
        Card::new()
            .question("What happens when you mark an incremental reading text as done?".to_string())
            .answer("It won't show up in review again".to_string())
            .cardtype(super::card::CardType::Pending)
            .save_card(&conn);
        Card::new()
            .question("When should you mark an incremental reading text as done?".to_string())
            .answer("When you've made extracts or clozes of everything you want to remember in it".to_string())
            .cardtype(super::card::CardType::Pending)
            .save_card(&conn);
        Card::new()
            .question("How do you use AI to find answers to your questions?".to_string())
            .answer("Add your openai-key to the config file, and press alt+g when reviewing unfinished cards or adding new ones".to_string())
            .cardtype(super::card::CardType::Pending)
            .save_card(&conn);



        let inc_introduction = r#"!!!if you see this in review, you can skip it with Alt+s!!!
This tab is dedicated to incremental reading!

Incremental reading is a way of incrementally reading through (usually a lot of) texts, while first extracting out the important parts, then making cloze deletions of the facts that you want to remember. 

to take extracts or clozes, you first have to go to the visual  mode on text input mode.

first go to normal mode by pressing Ctrl+c
then from normal mode go to visual mode by pressing v 

in visual mode, move with your arrow keys to mark your selection. To make the selection into an extract, press Alt+x. To make the selection a cloze deletion, press Alt+z.

Oh, and to go from visual mode to normal mode, press Ctrl+c 
and from normal mode to insert mode, press i
This text-mode system is based on vim. 

If youre not familiar with incremental reading I reccommend googling it first! when you want to go ahead and try it, either press Alt+w and search for a wikipedia page, or paste the text you want to use.

In the future there will be many more ways of getting text in here.

            "#.to_string();
        new_incread(&conn, 0, 1, inc_introduction, true).unwrap();
    }
     
    



    Ok(new_db)
}







