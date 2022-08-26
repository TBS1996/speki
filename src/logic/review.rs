use crossterm::event::KeyCode;
use crate::utils::{
    card::{Card, Review, RecallGrade},
    sql::{
        fetch::load_cards,
        insert::revlog_new, update::update_status,
    },
    interval, widgets::find_card::FindCardWidget,
};

use rusqlite::Connection;
use crate::utils::widgets::textinput::Field;

#[derive(PartialEq)]
pub enum CardPurpose{
    Dependency,
    Dependent,
}

#[derive(PartialEq)]
pub struct SelectCard{
    pub cardfinder: FindCardWidget,
    pub purpose: CardPurpose,
}

impl SelectCard{
    pub fn new(conn: &Connection, prompt: String, purpose: CardPurpose) -> Self{
       SelectCard{
           cardfinder: FindCardWidget::new(conn, prompt),
           purpose,
       } 
    }
}

#[derive(PartialEq, Debug)]
pub enum ReviewMode{
    Review(bool),
    Unfinished,
    Pending(bool),
    Done,
}


#[derive(PartialEq)]
pub enum ReviewSelection{
    Question(bool),
    Answer(bool),
    Stats,
    Dependents,
    Dependencies,
    SelectCard(SelectCard),
    Skip,
    Finish,
}

pub struct ReviewList{
    pub title: String,
    pub question: Field,
    pub answer: Field,
    pub review_cards: Vec<u32>,
    pub unfinished_cards: Vec<u32>,
    pub pending_cards: Vec<u32>,
    pub mode: ReviewMode,
    pub start_qty: u16,
    pub unf_qty: u16,
    pub pending_qty: u16,
    pub selection: ReviewSelection,
}


use crate::utils::sql::fetch::fetch_card;


impl ReviewList {
    pub fn new(conn: &Connection)->ReviewList{
        interval::calc_strength(conn);
        let thecards = load_cards(conn).unwrap();

        let mut review_cards     = Vec::<u32>::new();
        let mut unfinished_cards = Vec::<u32>::new();
        let mut pending_cards    = Vec::<u32>::new();

        for card in thecards{
            if card.status.isactive(){
                if card.strength < 0.9{
                    review_cards.push(card.card_id);
                }
            } else if !card.status.initiated{
                pending_cards.push(card.card_id);
            } else if !card.status.complete{
                unfinished_cards.push(card.card_id);
            }
        }

        let qty = *(&review_cards.len()) as u16;
        let unf_qty = *(&unfinished_cards.len()) as u16;
        let pending_qty = *(&pending_cards.len()) as u16;

        let mut mode = ReviewMode::Done;

        if review_cards.len() > 0{
            mode = ReviewMode::Review(false);
        } else if unfinished_cards.len() > 0 {
            mode = ReviewMode::Unfinished;
        } else if pending_cards.len() > 0 {
            mode = ReviewMode::Pending(false);
        }
        let mut myself = ReviewList{
            title: String::from("reviewww"),
            question: Field::new(),
            answer:   Field::new(),
            review_cards,
            unfinished_cards,
            pending_cards,
            mode,
            start_qty: qty,
            unf_qty,
            pending_qty,
            selection: ReviewSelection::Question(false),
        };

        myself.fill_fields(conn);
        myself


        }

    pub fn new_review(&mut self, conn: &Connection, grade: RecallGrade){
        let mut card = fetch_card(conn, self.get_id().unwrap());
        let review = Review::from(&grade);
        card.history.push(review.clone());
        revlog_new(conn, card.card_id, review).unwrap();
        interval::calc_stability(conn, &mut card);
        card.status.initiated = true;
        update_status(conn, &card).unwrap();
        self.remove_card();
        self.next_mode();
        self.fill_fields(conn);
    }


    pub fn skip_unf(&mut self, conn: &Connection){
        self.unfinished_cards.remove(0);
        self.fill_fields(conn);
        self.next_mode();
        if !(self.mode == ReviewMode::Done){
            self.fill_fields(conn);
        }
    }

    pub fn complete_card(&mut self, conn: &Connection){
        let card = self.get_current_card(conn).unwrap();
        Card::toggle_complete(card, conn);
        self.unfinished_cards.remove(0);
        self.next_mode();
        if !(self.mode == ReviewMode::Done){
            self.fill_fields(conn);
        }
    }

    pub fn get_current_card(&mut self, conn: &Connection) -> Option<Card>{
        let id = self.get_id();
        if let None = id{return None};
        let card = fetch_card(conn, id.unwrap());
        Some(card)
    }

    pub fn fill_fields(&mut self, conn: &Connection){
        if let ReviewMode::Done = self.mode{return}
        let id = self.get_id();
        let card = fetch_card(conn, id.unwrap());
        self.question.replace_text(card.question);
        self.answer.replace_text(card.answer);
    }

    pub fn select_card(&mut self, conn: &Connection, prompt: String, purpose: CardPurpose) {
        self.selection = ReviewSelection::SelectCard(SelectCard::new(conn, prompt, purpose));
    }


    pub fn remove_card(&mut self){
        match self.mode{
            ReviewMode::Review(_)  => self.review_cards.remove(0 as usize),
            ReviewMode::Unfinished => self.unfinished_cards.remove(0 as usize),
            ReviewMode::Pending(_)    => self.pending_cards.remove(0 as usize),
            _ => panic!("remove_card shouldnt be called in mode Done"),
        };
    }


    pub fn next_mode(&mut self){
        if let ReviewMode::Review(_) = self.mode{
            self.mode = ReviewMode::Review(false);
            if self.review_cards.len() == 0{
                self.mode = ReviewMode::Unfinished;
            }
        }
        if let ReviewMode::Unfinished = self.mode{
            if self.unfinished_cards.len() == 0{
                self.mode = ReviewMode::Pending(false);
            }
        }
        if let ReviewMode::Pending(_) = self.mode{
            if self.pending_cards.len() == 0{
                self.mode = ReviewMode::Done;
            }
        }
    }


    // requires mode not to be set to empty vec to not panic
    pub fn get_id(&self)-> Option<u32>{
        match self.mode{
            ReviewMode::Review(_)  => Some(self.review_cards[0]),
            ReviewMode::Unfinished => Some(self.unfinished_cards[0]),
            ReviewMode::Pending(_)    => Some(self.pending_cards[0]),
            ReviewMode::Done       => None,
        }
    }

pub fn navigate_unfinished(&mut self, key: KeyCode){
    match (key, &self.selection){
        (KeyCode::Char('s'), ReviewSelection::Answer(false))  => {self.selection = ReviewSelection::Skip},
        (KeyCode::Char('a'), ReviewSelection::Finish)  => {self.selection = ReviewSelection::Skip},
        (KeyCode::Char('d'), ReviewSelection::Skip)  => {self.selection = ReviewSelection::Finish},
        (KeyCode::Char('w'), ReviewSelection::Skip)  => {self.selection = ReviewSelection::Answer(false)},
        (KeyCode::Char('w'), ReviewSelection::Finish)  => {self.selection = ReviewSelection::Answer(false)},
        (KeyCode::Char('d'), ReviewSelection::Question(false))     => {self.selection = ReviewSelection::Dependents},
        (KeyCode::Char('s'), ReviewSelection::Question(false))     => {self.selection = ReviewSelection::Answer(false)},
        (KeyCode::Char('d'), ReviewSelection::Answer(false))       => {self.selection = ReviewSelection::Dependencies},
        (KeyCode::Char('w'), ReviewSelection::Answer(false))       => {self.selection = ReviewSelection::Question(false)},
        (KeyCode::Char('s'), ReviewSelection::Dependents)   => {self.selection = ReviewSelection::Dependencies},
        (KeyCode::Char('a'), ReviewSelection::Dependents)   => {self.selection = ReviewSelection::Question(false)},
        (KeyCode::Char('a'), ReviewSelection::Stats)        => {self.selection = ReviewSelection::Answer(false)},
        (KeyCode::Char('w'), ReviewSelection::Stats)        => {self.selection = ReviewSelection::Answer(false)},
        (KeyCode::Char('w'), ReviewSelection::Dependencies) => {self.selection = ReviewSelection::Dependents},
        (KeyCode::Char('a'), ReviewSelection::Dependencies) => {self.selection = ReviewSelection::Answer(false)},
        _ => {},
}
}



    pub fn navigate(&mut self, key: KeyCode){
        if self.mode == ReviewMode::Unfinished{
            self.navigate_unfinished(key);
        } else {
            match (key, &self.selection){
                (KeyCode::Char('d'), ReviewSelection::Question(false))     => {self.selection = ReviewSelection::Dependents},
                (KeyCode::Char('s'),  ReviewSelection::Question(false))     => {self.selection = ReviewSelection::Answer(false)},
                (KeyCode::Char('d'), ReviewSelection::Answer(false))       => {self.selection = ReviewSelection::Dependencies},
                (KeyCode::Char('w'),    ReviewSelection::Answer(false))       => {self.selection = ReviewSelection::Question(false)},
                (KeyCode::Char('s'),  ReviewSelection::Answer(false))       => {self.selection = ReviewSelection::Stats},
                (KeyCode::Char('s'),  ReviewSelection::Dependents)   => {self.selection = ReviewSelection::Dependencies},
                (KeyCode::Char('a'),  ReviewSelection::Dependents)   => {self.selection = ReviewSelection::Question(false)},
                (KeyCode::Char('a'),  ReviewSelection::Stats)        => {self.selection = ReviewSelection::Answer(false)},
                (KeyCode::Char('w'),    ReviewSelection::Stats)        => {self.selection = ReviewSelection::Answer(false)},
                (KeyCode::Char('w'),    ReviewSelection::Dependencies) => {self.selection = ReviewSelection::Dependents},
                (KeyCode::Char('a'),  ReviewSelection::Dependencies) => {self.selection = ReviewSelection::Answer(false)},
                (KeyCode::Char('s'),  ReviewSelection::Dependencies) => {self.selection = ReviewSelection::Stats},
                _ => {},
        }
        }
    }
}
