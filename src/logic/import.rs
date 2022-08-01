use std::{fs::File, io::Read};
use crate::utils::{
    card::{Card, Review, Status},
    sql::insert::save_card,

};


use rusqlite::Connection;
use csv::{StringRecord};




pub fn import_cards(conn: &Connection){
    // Get the file path
    let file_path = "newcards.csv";

    // Get the file
    let mut file = File::open(file_path).expect("File not found");

    // Turn the data into a string
    let mut raw_data = String::new();

    file.read_to_string(&mut raw_data).expect("Error while reading file");

    // Serialize the raw data
    let data = read_from_raw_data(&raw_data);


    for record in data {

        let thestat = Status{
            initiated: false,
            complete: true,
            resolved: true,
            suspended: false,
        };

        let card = Card{
            question: record[0].to_string(),
            answer:  record[1].to_string(),
            status: thestat,
            strength: 1.0,
            stability : 1.0,
            dependencies: Vec::<u32>::new(),
            dependents:  Vec::<u32>::new(),
            history: Vec::<Review>::new(),
            topic: 0u32,
            future: String::new(),
            integrated: 1.0,
            card_id: 0u32,
        };
        save_card(conn, card);
        
    }

}


fn read_from_raw_data(data :&str) -> Vec<StringRecord> {
    // Create a new csv reader from path
    let mut reader = csv::Reader::from_reader(data.as_bytes());

    // Return a vector of StringRecords
    let mut data: Vec<StringRecord> = Vec::new();
    // reader.records() returns an interator for the records it got from the csv
    for result in reader.records() {
        let record = result.unwrap();
        
        data.push(record);
    }

    return data;
}
