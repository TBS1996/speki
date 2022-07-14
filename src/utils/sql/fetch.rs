
use rusqlite::{Connection,Row, Result};
use crate::utils::card::{Card, RecallGrade, Review, Status}; //, Topic, Review}

#[derive(Clone)]
pub struct DepPair{
    dependent: u32,
    dependency: u32,

}

pub fn prev_id(conn: &Connection) -> Result<u32>{
    Ok(conn.last_insert_rowid() as u32)

}



pub fn fetch_card(conn: &Connection, cid: u32) -> Result<Card> {
    conn.query_row(
        "SELECT * FROM cards WHERE id=?",
        [cid],
        |row| row2card(conn, &row),
    )
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

pub fn get_history(conn: &Connection, id: u32) -> Result<Vec<Review>>{
    let mut stmt = conn.prepare("SELECT * FROM revlog WHERE cid = ?")?;
    let rows = stmt.query_map([id], |row| {
        Ok(
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
            future: String::from("[]"),
            integrated: 1f32,
            card_id: row.get(0)?,
        })
}
