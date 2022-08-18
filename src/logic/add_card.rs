use rusqlite::Connection;
use crossterm::event::KeyCode;
use crate::utils::{
    sql::{
        fetch::{highest_id, get_topics, fetch_card},
        insert::{update_both, save_card, revlog_new},
    },
    card::{Status, RecallGrade, Review, Card},
    textinput::Field,
    structs::StatefulList,
    topics::Topic,
};



#[derive(Clone)]
pub enum DepState{
    None,
    HasDependency(u32),
    HasDependent(u32),
}

#[derive(Clone, PartialEq)]
pub enum TextSelect{
    Question(bool), // Bool indicates if youre in text-editing mode
    Answer(bool),
    SubmitFinished,
    SubmitUnfinished,
    Topic(Option<NewTopic>),
}


#[derive(Clone, PartialEq)]
pub struct NewTopic{
    pub name: Field,
    pub id: u32,
}

impl NewTopic{
    pub fn new(id: u32) -> NewTopic{
        NewTopic{
            name: Field::new(None),
            id,
        }
    }
}




#[derive(Clone)]
pub struct NewCard{
    pub prompt: String,
    pub question:  Field,
    pub answer:    Field,
    pub state: DepState,
    pub topics: StatefulList<Topic>,
    pub selection: TextSelect,
}



impl NewCard{
    pub fn new(conn: &Connection, state: DepState) -> NewCard{
        let mut topics = StatefulList::with_items(get_topics(conn).unwrap()); 
        topics.add_kids();
        topics.sort_topics();
        
        NewCard {
            prompt: NewCard::make_prompt(state.clone(),conn),
            question:  Field::new(Some('^')),
            answer:    Field::new(Some('^')),
            state,
            topics,
            selection: TextSelect::Question(false),


        }
    }

    pub fn delete_topic(&mut self){

    }

    pub fn up_topic(&mut self){
        
    }

    pub fn navigate(&mut self, key: KeyCode){

        match (&self.selection, key){
            (TextSelect::Topic(None), KeyCode::Left) => self.selection = TextSelect::Question(false),
            _ => {},
        }
    }


    pub fn reload_topics(&mut self, conn: &Connection) {
        self.topics = StatefulList::with_items(get_topics(conn).unwrap());
        self.topics.add_kids();
        self.topics.sort_topics();


    }

    fn make_prompt(state: DepState, conn: &Connection) -> String{
        let mut prompt = String::new();
        match state{
            DepState::None =>  {
                prompt.push_str("Add new card");
                prompt
            },
            DepState::HasDependent(idx) => {
                prompt.push_str("Add new dependency for ");
                let card = fetch_card(conn, idx);
                prompt.push_str(&card.question);
                prompt

            },
            DepState::HasDependency(idx) => {
                prompt.push_str("Add new dependent of: ");
                let card = fetch_card(conn, idx);
                prompt.push_str(&card.question);
                prompt
            }
        }
    }


    pub fn submit_card(&mut self, conn: &Connection) {
        let topic: u32; 
        match self.topics.get_selected_id(){
            None => topic = 0,
            Some(num) => topic = num,
        }

        let mut question = self.question.text.clone(); 
        let mut answer   = self.answer.text.clone();
        question.pop();
        answer.pop();

        let status = match self.selection{
            TextSelect::SubmitFinished => Status::new_complete(),
            TextSelect::SubmitUnfinished => Status::new_incomplete(),
            _  => panic!("wtf"),
        };

        let newcard = Card{
            question,
            answer, 
            status,
            strength: 1f32,
            stability: 1f32,
            dependencies: Vec::<u32>::new(),
            dependents: Vec::<u32>::new(),
            history: vec![Review::from(&RecallGrade::Decent)] ,
            topic,
            future: String::from("[]"),
            integrated: 1f32,
            card_id: 0u32,

        };

        save_card(conn, newcard).unwrap();
        revlog_new(conn, highest_id(conn).unwrap(), Review::from(&RecallGrade::Decent)).unwrap();

        let last_id: u32 = highest_id(conn).unwrap();
        match self.state{
            DepState::None => {},
            DepState::HasDependent(id) => {
                update_both(conn, id, last_id).unwrap();
                Card::check_resolved(id, conn);
            },  
            DepState::HasDependency(id) => {update_both(conn, last_id, id).unwrap();},  
        }

        self.reset(DepState::None, conn);
    }


    pub fn reset(&mut self, state: DepState, conn: &Connection){
        self.prompt = NewCard::make_prompt(state.clone(), conn);
        self.question = Field::new(Some('^'));
        self.answer = Field::new(Some('^'));
        self.state = state;
        self.selection = TextSelect::Question(false);
    }





    pub fn enterkey(&mut self, conn: &Connection){
        match self.selection{
            TextSelect::Question(_) => self.selection = TextSelect::Question(true),
            TextSelect::Answer(_)   => self.selection = TextSelect::Answer(true),
            TextSelect::SubmitFinished | TextSelect::SubmitUnfinished => self.submit_card(conn),
            _ => {}, 
        }
    }




    pub fn istextselected(&self)->bool{
        (self.selection == TextSelect::Question(true)) || (self.selection == TextSelect::Answer(true))
    }

    pub fn deselect(&mut self){
        match self.selection{
            TextSelect::Answer(_) => self.selection = TextSelect::Answer(false),
            TextSelect::Question(_) => self.selection = TextSelect::Question(false),
            _ => {},
        }
    }
    pub fn uprow(&mut self){}
    pub fn downrow(&mut self){}
    pub fn home(&mut self){}
    pub fn end(&mut self){}
    pub fn pageup(&mut self){
        match self.selection{
            TextSelect::Question(_) => self.question.cursor = 0,
            TextSelect::Answer(_) => self.answer.cursor = 0,
            _ => {},
        }
    }
    pub fn pagedown(&mut self){
        match self.selection{
            TextSelect::Question(_) => self.question.cursor = self.question.text.len() - 2,
            TextSelect::Answer(_)   => self.answer.cursor   = self.answer.text.len()   - 2,
            _ => {},
        }
    }
    pub fn tab(&mut self){

        match self.selection{
            TextSelect::Question(_) => self.selection = TextSelect::Answer(false),
            TextSelect::Answer(_)   => {},
            _ => {},
        }
    }
    pub fn backtab(&mut self){
        match self.selection{
            TextSelect::Question(_) => {},
            TextSelect::Answer(_)   => self.selection = TextSelect::Question(false),
            _ => {},
        }
    }

    pub fn rightkey(&mut self){
        self.selection = TextSelect::Topic(None);
    }

    pub fn leftkey(&mut self){
        self.selection = TextSelect::Question(false);
    }

    pub fn upkey(&mut self){
        match self.selection{
            TextSelect::Answer(_)         => self.selection = TextSelect::Question(false),
            TextSelect::SubmitFinished   => self.selection = TextSelect::Answer(false),
            TextSelect::SubmitUnfinished => self.selection = TextSelect::SubmitFinished,
            _ => {},

            }
    }
    pub fn downkey(&mut self){
        match self.selection{
            TextSelect::Question(_)       => self.selection = TextSelect::Answer(false),
            TextSelect::Answer(_)         => self.selection = TextSelect::SubmitFinished,
            TextSelect::SubmitFinished   => self.selection = TextSelect::SubmitUnfinished,
            _ => {},
        }
    }
}

