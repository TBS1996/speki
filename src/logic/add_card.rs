use crate::utils::card::CardType;
use crate::utils::{aliases::*, sql::fetch::load_inc_title};
use rusqlite::Connection;
use crate::utils::{
    card::Card,
    widgets::textinput::Field,
    widgets::find_card::FindCardWidget,
    sql::fetch::fetch_card,
};



#[derive(Clone)]
pub enum DepState{
    None,
    NewDependent(CardID),
    NewDependency(CardID),
    NewChild(IncID),
}

//#[derive(Clone)]
pub enum TextSelect{
    Question, // Bool indicates if youre in text-editing mode
    Answer,
    Topic,
    ChooseCard(FindCardWidget),
}

use crate::utils::widgets::topics::TopicList;


//#[derive(Clone)]
pub struct NewCard{
    pub prompt: String,
    pub question:  Field,
    pub answer:    Field,
    pub state: DepState,
    pub topics: TopicList,
    pub selection: TextSelect,
}


use std::sync::{Arc, Mutex};

impl NewCard{
    pub fn new(conn: &Arc<Mutex<Connection>>, state: DepState) -> NewCard{
        let mut topics = TopicList::new(conn);
        topics.next();
        
        NewCard {
            prompt: NewCard::make_prompt(&state,conn),
            question:  Field::new(),
            answer:    Field::new(),
            state,
            topics,
            selection: TextSelect::Question,
        }
    }


    pub fn navigate(&mut self, dir: crate::Direction){
        use TextSelect::*;
        use crate::Direction::*;
        match (&self.selection, dir){
            (Question, Right) => self.selection = Topic,
            (Question, Down) => self.selection = Answer,
            (Answer, Up) => self.selection = Question,
            (Answer, Right) => self.selection = Topic,
            (Topic, Left) => self.selection = Question,
            (_,_) => {},
        }
    }



    fn make_prompt(state: &DepState, conn: &Arc<Mutex<Connection>>) -> String{
        let mut prompt = String::new();
        match state{
            DepState::None =>  {
                prompt.push_str("Add new card");
                prompt
            },
            DepState::NewDependency(idx) => {
                prompt.push_str("Add new dependency for ");
                let card = fetch_card(&conn, *idx);
                prompt.push_str(&card.question);
                prompt

            },
            DepState::NewDependent(idx) => {
                prompt.push_str("Add new dependent of: ");
                let card = fetch_card(&conn, *idx);
                prompt.push_str(&card.question);
                prompt
            }
            DepState::NewChild(id) => {
                prompt.push_str("Add new child of source: ");
                let title = load_inc_title(&conn, *id, 15).unwrap();
                prompt.push_str(&title);
                prompt
            }
        }
    }

    pub fn submit_card(&mut self, conn: &Arc<Mutex<Connection>>, iscompleted: bool){
        let question = self.question.return_text();
        let answer = self.answer.return_text();
        let topic = self.topics.get_selected_id().unwrap();
        let source = if let DepState::NewChild(incid) = self.state{
            incid
        } else {
            0
        };
        let status = if iscompleted{
            CardType::Finished
        } else {
            CardType::Unfinished
        };


     //(conn, question, answer, topic, source, iscompleted);
        let mut card = Card::new()
            .question(question)
            .answer(answer)
            .topic(topic)
            .source(source)
            .cardtype(status);

     //   revlog_new(conn, highest_id(conn).unwrap(), Review::from(&RecallGrade::Decent)).unwrap();

        match self.state{
            DepState::None => {},
            DepState::NewDependency(id) => {
                card.dependency(id);
            },  
            DepState::NewDependent(id) => {
                card.dependent(id);
            },  
            DepState::NewChild(_id) => {
            }
        }

        card.save_card(conn);
        self.reset(DepState::None, conn);
    }


    pub fn reset(&mut self, state: DepState, conn: &Arc<Mutex<Connection>>){
        self.prompt = NewCard::make_prompt(&state, conn);
        self.question = Field::new();
        self.answer = Field::new();
        self.state = state;
        self.selection = TextSelect::Question;
    }

    pub fn uprow(&mut self){}
    pub fn downrow(&mut self){}
    pub fn home(&mut self){}
    pub fn end(&mut self){}
}
