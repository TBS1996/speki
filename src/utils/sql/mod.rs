pub mod fetch;
pub mod insert;
pub mod update;




use rusqlite::{Connection, Result};


pub fn init_db() -> Result<()>{
    let conn = Connection::open("dbflash.db")?;

    conn.execute(
        "create table if not exists cards (
            id           integer primary key,
            question     text not null,
            answer       text not null,
            strength     real not null,
            stability    real not null,
            topic        integer not null,
            initiated    bool not null,
            complete     bool not null,
            resolved     bool not null,
            suspended    bool not null,
            gain         real not null

    )",
        [],
        )?;



    conn.execute(
        "create table if not exists topics ( 
        id     integer not null,
        name   text not null,
        parent text not null
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


    Ok(())
}


