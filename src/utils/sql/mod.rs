pub mod fetch;
pub mod insert;
pub mod update;
pub mod delete;




use rusqlite::{Connection, Result};
use crate::utils::sql::insert::new_topic;







pub fn init_db() -> Result<()>{

    let mut new_db = false;
    if let Err(_) = std::fs::metadata("dbflash.db"){
        new_db = true;
    }


    let conn = Connection::open("dbflash.db")?;

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
            active integer not null

    )",
        [],
        )?;
    
    if new_db {new_topic(&conn, String::from("root"), 0, 0)?;}
     
    



    Ok(())
}







