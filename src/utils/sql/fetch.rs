use rusqlite::{Connection,Row, Result};
use crate::utils::card::{Card, RecallGrade, Review, Status}; //, Topic, Review}
use crate::utils::widgets::topics::Topic;
use crate::utils::aliases::*;

#[derive(Clone)]
pub struct DepPair{
    dependent: u32,
    dependency: u32,

}



pub fn prev_id(conn: &Connection) -> Result<u32>{
    Ok(conn.last_insert_rowid() as u32)

}



pub fn fetch_card(conn: &Connection, cid: u32) -> Card {
    conn.query_row(
        "SELECT * FROM cards WHERE id=?",
        [cid],
        |row| row2card(conn, &row),
    ).unwrap()
}

pub fn highest_id(conn: &Connection) -> Result<u32> {
    let mut stmt = conn.prepare("SELECT * FROM cards")?;

    let card_iter = stmt.query_map([], |row| {
        Ok(row.get(0)?)
        
    })?;
    
    let mut maxid = 0;

    let mut temp;
    for card in card_iter {
        temp = card.unwrap();
            if temp > maxid{
                maxid = temp;
            }
        }
    
    Ok(maxid)
}

pub fn get_topics(conn: &Connection) -> Result<Vec<Topic>>{
    let mut stmt = conn.prepare("SELECT * FROM topics")?;
    let rows = stmt.query_map([], |row| {
        Ok(
            Topic{
                id: row.get(0)?,
                name: row.get(1)?,
                parent: row.get(2)?,
                children: Vec::<u32>::new(),
                ancestors: 0,
                relpos: row.get(3)? 
            }
        )
    })?;
    
    let mut vecoftops = Vec::<Topic>::new();

    for row in rows{
        vecoftops.push(row.unwrap().clone());
    }


    Ok(vecoftops)
}



pub fn get_history(conn: &Connection, id: u32) -> Result<Vec<Review>>{
    let mut stmt = conn.prepare("SELECT * FROM revlog WHERE cid = ?")?;
    let rows = stmt.query_map([id], |row| { Ok(
            Review{
                grade: RecallGrade::from(row.get(2)?).unwrap(),
                date: row.get(0)?,
                answertime: row.get(3)?,
            }
        )
    })?;
    
    let mut vecofrows = Vec::<Review>::new();

    for row in rows{
        vecofrows.push(row.unwrap().clone());
    }


    Ok(vecofrows)
}



pub fn load_cards(conn: &Connection) -> Result<Vec<Card>> {
    let mut cardvec = Vec::<Card>::new();
    let mut stmt = conn.prepare("SELECT * FROM cards")?;

    

    let card_iter = stmt.query_map([], |row| row2card(conn, &row))?;

    for card in card_iter {
        cardvec.push(card.unwrap().clone());}
    
    Ok(cardvec)
}




pub fn get_dependencies(conn: &Connection, dependent: u32) -> Result<Vec<u32>>{
    let mut stmt = conn.prepare("SELECT * FROM dependencies")?;

    let dep_iter = stmt.query_map([], |row| {
        Ok(
            DepPair{
                dependent:  row.get(0)?,
                dependency: row.get(1)?,
            }
        )
    })?;
        
    let mut dependencies = Vec::<u32>::new();
    let mut wtf = Vec::new();
    for dep in dep_iter{
        wtf.push(dep.unwrap().clone());
    }

    for lol in wtf{
        if lol.dependent == dependent{
            dependencies.push(lol.dependency);
        }
    }

    Ok(dependencies)
    
}





pub fn get_dependents(conn: &Connection, dependency: u32) -> Result<Vec<u32>>{

    let mut stmt = conn.prepare("SELECT * FROM dependencies")?;

    

    let dep_iter = stmt.query_map([], |row| {
        Ok(
            DepPair{
                dependent:  row.get(0)?,
                dependency: row.get(1)?,
            }
        )
    })?;
        
    let mut dependents = Vec::<u32>::new();
    let mut wtf = Vec::new();
    for dep in dep_iter{
        wtf.push(dep.unwrap().clone());
    }

    for lol in wtf{
        if lol.dependency == dependency{
            dependents.push(lol.dependent);
        }
    }

    Ok(dependents)
    
}



pub fn row2card(conn: &Connection, row: &Row) -> Result<Card>{
        let stat = Status{
            initiated: row.get(6)?,
            complete: row.get(7)?,
            resolved: row.get(8)?,
            suspended: row.get(9)?,
        };



        Ok(Card {
            question:      row.get(1)?,
            answer:        row.get(2)?,
            status:        stat,
            strength:      row.get(3)?,
            stability:     row.get(4)?,
            dependencies:  get_dependencies(conn, row.get(0).unwrap()).unwrap(),
            dependents:    get_dependents(conn, row.get(0).unwrap()).unwrap(),
            history:       get_history(conn, row.get(0).unwrap()).unwrap(),  
            topic:         row.get(5)?,
            future:        String::from("[]"),
            integrated: 1f32,
            card_id: row.get(0)?,
            source: row.get(11)?,
            skiptime: row.get(12)?,
            skipduration: row.get(13)?,
        })
}

use crate::utils::incread::IncRead;
use crate::utils::widgets::textinput::Field;
use crate::utils::statelist::StatefulList;

struct IncTemp{
    parent: u32,
    topic: u32,
    source: String,
    isactive: bool,
}



// TODO fix this mess
pub fn get_incread(conn: &Connection, id: u32) -> Result<IncRead>{
    let mut stmt = conn.prepare("SELECT * FROM incread WHERE id = ?")?;

    let mut temp = stmt.query_map([id], |row| {
     
        Ok(
            IncTemp{
                parent: row.get(1)?,
                topic: row.get(2)?,
                source: row.get(3)?,
                isactive: row.get(4)?,
            }
            )
    
    })?;

    let temp = temp.next().unwrap().unwrap();


    let extracts = StatefulList::with_items(load_extracts(conn, id).unwrap());

    let clozes = crate::utils::statelist::StatefulList::with_items(load_cloze_cards(conn, id).unwrap());


    let mut foo = 
        IncRead{
            id,
            parent: temp.parent,
            topic: temp.topic,
            source: Field::new(),
            extracts,
            clozes,
            isactive: temp.isactive,
        };

    foo.source.replace_text(temp.source.to_string());

    Ok(foo)
}


use crate::utils::incread::IncListItem;

pub fn load_inc_items(conn: &Connection, topic: TopicID) -> Result<Vec<IncListItem>> {
    let mut incvec = Vec::<IncListItem>::new();
    let mut stmt = conn.prepare("SELECT * FROM incread where parent = 0 and topic = ?")?;

    

    let inc_iter = stmt.query_map([topic], |row| 
                                   Ok(
                                       IncListItem{
                                           text: row.get(3)?,
                                           id: row.get(0)?,
                                       }
                                       ))
                                   ?;

    for inc in inc_iter {
        incvec.push(inc.unwrap().clone());}
    
    Ok(incvec)
}




pub fn load_extracts(conn: &Connection, parent: IncID) -> Result<Vec<IncListItem>> {
    let mut incvec = Vec::<IncListItem>::new();
    let mut stmt = conn.prepare("SELECT * FROM incread where parent = ?")?;

    

    let inc_iter = stmt.query_map([parent], |row| 
                                   Ok(
                                       IncListItem{
                                           text: row.get(3)?,
                                           id: row.get(0)?,
                                       }
                                       ))
                                   ?;

    for inc in inc_iter {
        incvec.push(inc.unwrap().clone());}
    
    Ok(incvec)
}


pub fn load_active_inc(conn: &Connection) -> Result<Vec<IncID>>{
    let mut incvec = Vec::<IncID>::new();
    let mut stmt = conn.prepare("SELECT * FROM incread where active = 1")?;
    let inc_iter = stmt.query_map([], |row| Ok(row.get(0)?))?;

    for inc in inc_iter {
        incvec.push(inc.unwrap());}

    Ok(incvec)
}

use crate::utils::widgets::cardlist::CardItem;





pub fn load_cloze_cards(conn: &Connection, source: IncID) -> Result<Vec<CardItem>> {
    let mut clozevec = Vec::<CardItem>::new();
    let mut stmt = conn.prepare("SELECT * FROM cards where source = ?")?;

    

    let inc_iter = stmt.query_map([source], |row| 
                                   Ok(
                                       CardItem{
                                           question: row.get(1)?,
                                           id: row.get(0)?,
                                       }
                                       ))
                                   ?;

    for inc in inc_iter {
        clozevec.push(inc.unwrap().clone());}
    
    Ok(clozevec)
}






















