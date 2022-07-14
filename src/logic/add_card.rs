#![allow(non_camel_case_types)]

use rusqlite::Connection;
use crate::utils::{
    sql::{
        fetch::highest_id,
        insert::{update_both, save_card, revlog_new},
    },
    card::{Status, RecallGrade, Review, Card},
    textinput::Field,
};


pub struct NewCards{
    pub cards: Vec<CardEdit>,
    pub card: CardEdit,
}



impl NewCards{
    pub fn new() -> NewCards{
        NewCards {
            cards: Vec::<CardEdit>::new(),
            card: CardEdit::new(),
        }
    }
    pub fn add_dependency(&mut self){
        let newcard = CardEdit::new();

        self.card = newcard;
        self.card.prompt = String::from("Adding new dependency");
        self.card.page = Page::editing;
        self.card.state = DepState::dependency;
        //self.cards.push(self.card.clone());
    }

    pub fn add_dependent(&mut self){
        let newcard = CardEdit::new();

        self.card = newcard;
        self.card.prompt = String::from("Adding new dependent");
        self.card.page = Page::editing;
        self.card.state = DepState::dependent;
        //self.cards.push(self.card.clone());
    }




    pub fn submit_card(&mut self, conn: &Connection) {

        match self.card.selection{
            TextSelect::submit_finished   => self.card.submit_cardedit(conn, Status::new_complete(),   self.card.dependencies.clone(), self.card.dependents.clone(), self.card.topic),
            TextSelect::submit_unfinished => self.card.submit_cardedit(conn, Status::new_incomplete(), self.card.dependencies.clone(), self.card.dependents.clone(), self.card.topic),
            _  => panic!("wtf"),
        }


        let last_id: u32 = highest_id(conn).unwrap();

        self.card.id   = Some(last_id);
//        panic!("wtf");
        match &self.card.id{
            None => panic!("wtf"),
            _=>{},
        }
        self.card.page = Page::confirming;
        self.card.selection = TextSelect::new_card;
        self.cards.push(self.card.clone());
    }


    pub fn done(&mut self, conn: &Connection){
        let cardlen = &self.cards.len();
        match &self.card.state{
            DepState::base => {},
            DepState::dependency => {
                let ency_id = self.card.id.unwrap();
                let ent_id  = self.cards[cardlen - 2 as usize].id.unwrap();

                update_both(conn, ent_id, ency_id).unwrap();
            },
            DepState::dependent  => {
                let ency_id = self.cards[cardlen - 2 as usize].id.unwrap();
                let ent_id  = self.card.id.unwrap(); 

                update_both(conn, ent_id, ency_id).unwrap();
            },
        }
        
        
        self.cards.pop();
        
        if self.cards.is_empty(){
            self.card = CardEdit::new();
            self.card.page = Page::editing;
        }
        else {
            self.card = self.cards.last().unwrap().clone();
        }

    }



    pub fn enterkey(&mut self, conn: &Connection){
        if self.card.istextselected(){
            match self.card.selection{
                TextSelect::question(_) => self.card.selection = TextSelect::answer(true),
                TextSelect::answer(_)   => self.card.selection = TextSelect::submit_finished,
            _ => {},
            }
        }
        else {
            match self.card.selection {
                TextSelect::question(_) => self.card.selection = TextSelect::question(true),
                TextSelect::answer(_)   => self.card.selection = TextSelect::answer(true),
                TextSelect::submit_finished   => self.submit_card(conn),
                TextSelect::submit_unfinished => self.submit_card(conn),
                TextSelect::add_dependency => self.add_dependency(),
                TextSelect::add_dependent  => self.add_dependent(),
                TextSelect::new_card => self.done(conn),
            }
        }
}
/*
    pub fn get_max_id(app: &App)-> u32{
        let thecards = load_cards(&app.conn).unwrap();
        let mut maxid = 0 as u32;
        for card in thecards{
            if card.card_id > maxid{
                maxid = card.card_id;
            }
        }
        maxid
    }*/
}










#[derive(Clone)]
pub enum Page{
    editing,
    confirming,
}


#[derive(Clone)]
pub enum DepState{
    base,
    dependency,
    dependent,
}

#[derive(Clone, PartialEq)]
pub enum TextSelect{
    question(bool), // Bool indicates if youre in text-editing mode
    answer(bool),
    submit_finished,
    submit_unfinished,
    add_dependency,
    add_dependent,
    new_card,
}

#[derive(Clone)]
pub struct CardEdit{
    pub prompt: String,
    pub selection: TextSelect,
    pub question:  Field,
    pub answer:    Field,
    pub dependencies: Vec<u32>,
    pub dependents: Vec<u32>,
    pub topic: u32,
    pub page: Page,
    pub state: DepState,
    pub id: Option<u32>,

}


impl CardEdit{
    pub fn new()->CardEdit{
        CardEdit{
            prompt: String::from("Add card"),
            selection: TextSelect::question(false),
            question:  Field::new(),
            answer:    Field::new(),
            dependencies: Vec::<u32>::new(),
            dependents:   Vec::<u32>::new(),
            topic: 0 as u32,
            page: Page::editing,
            id: None,  // the id of the card in the table is put here after its inserted, for use when adding dependencies directly
            state: DepState::base,
        }
    }


    pub fn submit_cardedit(&mut self, conn: &Connection, status: Status, dependencies: Vec<u32>, dependents: Vec<u32>, topic: u32){
        let mut question = self.question.text.clone();
        let mut answer   = self.answer.text.clone();
        question.pop();
        answer.pop();

        let newcard = Card{
            question: question, 
            answer:   answer,
            status: status,
            strength: 1f32,
            stability: 1f32,
            dependencies: dependencies,
            dependents: dependents,
            history: vec![Review::from(&RecallGrade::Decent)] ,
            topic: topic,
            future: String::from("[]"),
            integrated: 1f32,
            card_id: 0u32,

        };

        save_card(conn, newcard).unwrap();
        revlog_new(conn, highest_id(conn).unwrap(), Review::from(&RecallGrade::Decent)).unwrap();
    }

    pub fn addchar(&mut self, c: char){
        match self.selection{
            TextSelect::question(_) => self.question.addchar(c),
            TextSelect::answer(_)   => self.answer.addchar(c),
            _ => {}
        }
    }
    pub fn backspace(&mut self){
        match self.selection {
            TextSelect::question(_) => self.question.backspace(),
            TextSelect::answer(_)   => self.answer.backspace(),
            _ => {},
        }
    }

    pub fn next(&mut self) {
        match self.selection {
            TextSelect::question(_) => self.question.next(),
            TextSelect::answer(_)   => self.answer.next(),
            _ => {},
        }
    }


    pub fn prev(&mut self) {
        match self.selection {
            TextSelect::question(_) => self.question.prev(),
            TextSelect::answer(_)   => self.answer.prev(),
            _ => {},
        }
    }

    pub fn delete(&mut self){
        match self.selection{
            TextSelect::question(_) => self.question.delete(), 
            TextSelect::answer(_)   => self.answer.delete(),
            _ => {},
        }
    }

    pub fn istextselected(&self)->bool{
        (self.selection == TextSelect::question(true)) || (self.selection == TextSelect::answer(true))
    }

    pub fn deselect(&mut self){
        match self.selection{
            TextSelect::answer(_) => self.selection = TextSelect::answer(false),
            TextSelect::question(_) => self.selection = TextSelect::question(false),
            _ => {},
        }
    }
    pub fn uprow(&mut self){}
    pub fn downrow(&mut self){}
    pub fn home(&mut self){}
    pub fn end(&mut self){}
    pub fn pageup(&mut self){
        match self.selection{
            TextSelect::question(_) => self.question.cursor = 0,
            TextSelect::answer(_) => self.answer.cursor = 0,
            _ => {},
        }
    }
    pub fn pagedown(&mut self){
        match self.selection{
            TextSelect::question(_) => self.question.cursor = self.question.text.len() - 2,
            TextSelect::answer(_)   => self.answer.cursor   = self.answer.text.len()   - 2,
            _ => {},
        }
    }
    pub fn tab(&mut self){

        match self.selection{
            TextSelect::question(_) => self.selection = TextSelect::answer(false),
            TextSelect::answer(_)   => {},
            _ => {},
        }
    }
    pub fn backtab(&mut self){
        match self.selection{
            TextSelect::question(_) => {},
            TextSelect::answer(_)   => self.selection = TextSelect::question(false),
            _ => {},
        }
    }

    pub fn upkey(&mut self){
        match self.selection{
            TextSelect::answer(_)         => self.selection = TextSelect::question(false),
            TextSelect::submit_finished   => self.selection = TextSelect::answer(false),
            TextSelect::submit_unfinished => self.selection = TextSelect::submit_finished,
            TextSelect::add_dependency    => self.selection = TextSelect::new_card,
            TextSelect::add_dependent     => self.selection = TextSelect::add_dependency,
            _ => {},

            }
    }
    pub fn downkey(&mut self){
        match self.selection{
            TextSelect::question(_)       => self.selection = TextSelect::answer(false),
            TextSelect::answer(_)         => self.selection = TextSelect::submit_finished,
            TextSelect::submit_finished   => self.selection = TextSelect::submit_unfinished,
            TextSelect::new_card          => self.selection = TextSelect::add_dependency,
            TextSelect::add_dependency    => self.selection = TextSelect::add_dependent,
            _ => {},
        }
    }
}

